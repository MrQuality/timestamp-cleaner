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
    assert!(state.join("trusted-root.json").exists());
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
