mod support;
use support::{EXPIRED, FUTURE, Fixture, number, require_failure, require_success};

#[tokio::test]
async fn process_restart_and_same_version_renewal() {
    let fixture = Fixture::new().await;
    let server = fixture.dir.path().join("repository");
    let state = fixture.dir.path().join("client-state");
    fixture.publish(&server, 1, FUTURE).await;
    let first = require_success(fixture.probe(&server, &state));
    // Each probe is a new OS process reading the same durable client state.
    assert_eq!(first, require_success(fixture.probe(&server, &state)));
    fixture.publish(&server, 2, FUTURE).await;
    assert_eq!(first, require_success(fixture.probe(&server, &state)));
    assert!(state.join("root.json").exists());
    assert!(state.join("timestamp.json").exists());
}

#[tokio::test]
async fn earlier_generation_is_rejected_after_process_restart() {
    let fixture = Fixture::new().await;
    let newer = fixture.dir.path().join("newer");
    let older = fixture.dir.path().join("older");
    let state = fixture.dir.path().join("client-state");
    fixture.publish(&newer, 2, FUTURE).await;
    fixture.publish(&older, 1, FUTURE).await;
    require_success(fixture.probe(&newer, &state));
    let message = require_failure(fixture.probe(&older, &state));
    assert!(message.contains("OlderMetadata"), "{message}");
}

#[tokio::test]
async fn expired_metadata_never_exposes_catalog() {
    let fixture = Fixture::new().await;
    let server = fixture.dir.path().join("expired");
    fixture.publish(&server, 1, EXPIRED).await;
    let message = require_failure(fixture.probe(&server, &fixture.dir.path().join("state")));
    assert!(message.contains("Expired"), "{message}");
}

#[tokio::test]
async fn modified_timestamp_signature_is_rejected() {
    let fixture = Fixture::new().await;
    let server = fixture.dir.path().join("modified");
    fixture.publish(&server, 1, FUTURE).await;
    let timestamp = server.join("metadata/timestamp.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&timestamp).unwrap()).unwrap();
    value["signed"]["version"] = 9.into();
    std::fs::write(timestamp, serde_json::to_vec(&value).unwrap()).unwrap();
    let message = require_failure(fixture.probe(&server, &fixture.dir.path().join("state")));
    assert!(message.contains("VerifyMetadata"), "{message}");
}

#[tokio::test]
async fn altered_or_truncated_catalog_fails_at_stream_completion() {
    for truncate in [false, true] {
        let fixture = Fixture::new().await;
        let server = fixture.dir.path().join("damaged");
        fixture.publish(&server, 1, FUTURE).await;
        let target = std::fs::read_dir(server.join("targets"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut bytes = std::fs::read(&target).unwrap();
        if truncate {
            bytes.truncate(bytes.len() / 2);
        } else {
            bytes[12] ^= 1;
        }
        std::fs::write(target, bytes).unwrap();
        let message = require_failure(fixture.probe(&server, &fixture.dir.path().join("state")));
        assert!(
            message.contains("HashMismatch") || message.contains("SizeMismatch"),
            "{message}"
        );
    }
}

#[tokio::test]
async fn rotated_root_is_persisted_and_required_on_the_next_process() {
    let original = Fixture::new().await;
    let mut rotated = Fixture::new().await;
    rotated.root.version = number(2);
    let server = original.dir.path().join("rotated");
    let state = original.dir.path().join("state");
    original.publish(&server, 1, FUTURE).await;
    require_success(original.probe(&server, &state));
    rotated.publish(&server, 2, FUTURE).await;
    rotated.sign_root(Some(&original)).await;
    std::fs::copy(
        rotated.dir.path().join("root.json"),
        server.join("metadata/2.root.json"),
    )
    .unwrap();
    require_success(original.probe(&server, &state));
    std::fs::remove_file(server.join("metadata/2.root.json")).unwrap();
    require_success(original.probe(&server, &state));
    // The same repository fails with the obsolete embedded root and a fresh datastore.
    require_failure(original.probe(&server, &original.dir.path().join("fresh-state")));
}

#[tokio::test]
async fn stalled_http_peer_is_cancelled_without_exposing_catalog() {
    use std::{
        io::Read,
        net::TcpListener,
        sync::mpsc,
        time::{Duration, Instant},
    };
    let fixture = Fixture::new().await;
    fixture.sign_root(None).await;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    let (stop, stopped) = mpsc::channel();
    let peer = std::thread::spawn(move || {
        let mut socket = accept_fixture_connection(&listener);
        socket
            .set_read_timeout(Some(Duration::from_secs(8)))
            .unwrap();
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).unwrap() > 0);
        // Keep the connection open without headers; release it after the probe exits.
        let _ = stopped.recv_timeout(Duration::from_secs(8));
    });
    let started = Instant::now();
    let output = fixture.probe_metadata_url(
        fixture.dir.path(),
        &fixture.dir.path().join("state"),
        endpoint,
    );
    let elapsed = started.elapsed();
    let _ = stop.send(());
    peer.join().unwrap();
    let message = require_failure(output);
    assert!(message.contains("Elapsed"), "{message}");
    // Tokio's timeout is cooperative: synchronous transport initialization can
    // delay it. Record observed latency rather than assert a false hard bound.
    eprintln!("stalled peer elapsed: {elapsed:?}");
    // A first-attempt network failure must not make a fresh client unrecoverable.
    let recovery = fixture.dir.path().join("recovery-repository");
    fixture.publish(&recovery, 1, FUTURE).await;
    require_success(fixture.probe(&recovery, &fixture.dir.path().join("state")));
}

#[tokio::test]
async fn interrupted_http_response_never_exposes_catalog() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        time::Duration,
    };
    let fixture = Fixture::new().await;
    fixture.sign_root(None).await;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    let peer = std::thread::spawn(move || {
        let mut socket = accept_fixture_connection(&listener);
        socket
            .set_read_timeout(Some(Duration::from_secs(8)))
            .unwrap();
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).unwrap() > 0);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1024\r\nConnection: close\r\n\r\n{")
            .unwrap();
        // Closing before Content-Length bytes tests a real interrupted HTTP body.
    });
    let output = fixture.probe_metadata_url(
        fixture.dir.path(),
        &fixture.dir.path().join("state"),
        endpoint,
    );
    peer.join().unwrap();
    let message = require_failure(output);
    // Default transport retries can defer failure until the outer timeout.
    eprintln!("interrupted response failure: {message}");
}

fn accept_fixture_connection(listener: &std::net::TcpListener) -> std::net::TcpStream {
    use std::{
        io::ErrorKind,
        time::{Duration, Instant},
    };
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match listener.accept() {
            Ok((socket, _)) => return socket,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "Probe never connected to fixture peer"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("Fixture accept failed: {error}"),
        }
    }
}

#[path = "support/publisher.rs"]
mod publisher;

#[tokio::test]
async fn package_binding_rejects_changed_short_and_long_bytes() {
    for damage in ["none", "changed", "short", "long"] {
        let fixture = Fixture::new().await;
        let server = fixture.dir.path().join("repository");
        let state = fixture.dir.path().join("state");
        fixture.publish(&server, 1, FUTURE).await;
        let path = fixture.dir.path().join("fixture-package.bin");
        let mut bytes = std::fs::read(&path).unwrap();
        match damage {
            "changed" => bytes[0] ^= 1,
            "short" => {
                bytes.pop();
            }
            "long" => bytes.push(0),
            "none" => {}
            _ => unreachable!(),
        }
        std::fs::write(path, bytes).unwrap();
        let output = fixture.probe_package(&server, &state);
        if damage == "none" {
            require_success(output);
        } else {
            let message = require_failure(output);
            assert!(
                message.contains("package hash mismatch")
                    || message.contains("package length mismatch"),
                "{message}"
            );
        }
    }
}

#[tokio::test]
async fn signed_invalid_package_limits_fail_before_opening_package() {
    for package in [
        serde_json::json!({"length": 0, "sha256": "a".repeat(64)}),
        serde_json::json!({"length": 268435457_u64, "sha256": "a".repeat(64)}),
        serde_json::json!({"length": 10, "sha256": "INVALID"}),
    ] {
        let fixture = Fixture::new().await;
        let server = fixture.dir.path().join("repository");
        let catalog =
            serde_json::to_vec(&serde_json::json!({"version": "0.1.0-beta.2", "package": package}))
                .unwrap();
        fixture.publish_catalog(&server, 1, FUTURE, &catalog).await;
        // Package file deliberately absent. Signed catalog alone must not bypass bounds.
        let message =
            require_failure(fixture.probe_package(&server, &fixture.dir.path().join("state")));
        assert!(
            message.contains("package length exceeds spike policy")
                || message.contains("invalid package SHA-256"),
            "{message}"
        );
    }
}

#[tokio::test]
async fn root_rotation_survives_later_metadata_failure() {
    let original = Fixture::new().await;
    let mut replacement = Fixture::new().await;
    replacement.root.version = number(2);
    let server = original.dir.path().join("repository");
    let state = original.dir.path().join("state");
    original.publish(&server, 1, FUTURE).await;
    let catalog = require_success(original.probe(&server, &state));
    replacement.publish(&server, 2, EXPIRED).await;
    replacement.sign_root(Some(&original)).await;
    std::fs::copy(
        replacement.dir.path().join("root.json"),
        server.join("metadata/2.root.json"),
    )
    .unwrap();
    let message = require_failure(original.probe(&server, &state));
    assert!(message.contains("Expired"), "{message}");
    let root: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("root.json")).unwrap()).unwrap();
    assert_eq!(root["signed"]["version"], 2);
    replacement.publish(&server, 3, FUTURE).await;
    std::fs::remove_file(server.join("metadata/2.root.json")).unwrap();
    assert_eq!(catalog, require_success(original.probe(&server, &state)));
    require_failure(original.probe(&server, &original.dir.path().join("obsolete-client")));
}

#[tokio::test]
async fn missing_or_corrupt_saved_root_never_falls_back_to_bootstrap() {
    for missing in [true, false] {
        let fixture = Fixture::new().await;
        let server = fixture.dir.path().join("repository");
        let state = fixture.dir.path().join("state");
        fixture.publish(&server, 1, FUTURE).await;
        require_success(fixture.probe(&server, &state));
        if missing {
            std::fs::remove_file(state.join("root.json")).unwrap();
        } else {
            std::fs::write(state.join("root.json"), b"{").unwrap();
        }
        let message = require_failure(fixture.probe(&server, &state));
        assert!(
            message.contains("missing trusted root") || message.contains("ParseTrustedMetadata"),
            "{message}"
        );
    }
}
