//! Explicit local publisher experiment; never supplied production keys.
use super::support::{FUTURE, Fixture, number, require_failure, require_success};
use std::{path::Path, process::Command};
use url::Url;

fn update(
    tool: &Path,
    fixture: &Fixture,
    source: &Path,
    destination: &Path,
    generation: u64,
) -> Command {
    let mut command = Command::new(tool);
    command
        .arg("update")
        .arg("--root")
        .arg(fixture.dir.path().join("root.json"))
        .arg("--metadata-url")
        .arg(
            Url::from_directory_path(source.join("metadata"))
                .unwrap()
                .as_str(),
        )
        .arg("--outdir")
        .arg(destination);
    for role in ["targets", "snapshot", "timestamp"] {
        command
            .arg(format!("--{role}-version"))
            .arg(generation.to_string())
            .arg(format!("--{role}-expires"))
            .arg(FUTURE);
    }
    command
}

fn copy_targets(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination.join("targets")).unwrap();
    for item in std::fs::read_dir(source.join("targets")).unwrap() {
        let item = item.unwrap();
        assert!(item.file_type().unwrap().is_file());
        std::fs::copy(
            item.path(),
            destination.join("targets").join(item.file_name()),
        )
        .unwrap();
    }
}

#[tokio::test]
#[ignore = "explicit publisher experiment: set TUFTOOL_BIN to pinned tuftool 0.17.0"]
async fn cli_renewal_rotation_and_timestamp_key_separation() {
    let tool =
        std::env::var_os("TUFTOOL_BIN").expect("TUFTOOL_BIN must name pinned test-only publisher");
    let tool = Path::new(&tool);
    let version = Command::new(tool).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert!(
        String::from_utf8_lossy(&version.stdout)
            .trim()
            .ends_with("0.17.0")
    );
    let original = Fixture::new().await;
    let initial = original.dir.path().join("initial");
    let renewed = original.dir.path().join("renewed");
    let state = original.dir.path().join("client-state");
    original.publish(&initial, 1, FUTURE).await;
    let catalog = require_success(original.probe(&initial, &state));

    let mut renewal = update(tool, &original, &initial, &renewed, 2);
    // Offline publishing uses targets + snapshot; the timestamp role is separate.
    for index in 1..=3 {
        renewal
            .arg("--key")
            .arg(original.dir.path().join(format!("test-role-{index}.der")));
    }
    let output = renewal.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    copy_targets(&initial, &renewed);
    assert_eq!(catalog, require_success(original.probe(&renewed, &state)));
    let before = std::fs::read(renewed.join("metadata/2.targets.json")).unwrap();

    // The generic CLI update is not a timestamp-only renewal API. Prove a sole
    // timestamp key cannot authorize a new targets/snapshot generation.
    let denied = original.dir.path().join("timestamp-only");
    let output = update(tool, &original, &renewed, &denied, 3)
        .arg("--key")
        .arg(original.dir.path().join("test-role-3.der"))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "Timestamp key unexpectedly signed other roles"
    );
    let message = String::from_utf8_lossy(&output.stderr);
    assert!(
        message.contains("threshold") || message.contains("signing keys"),
        "{message}"
    );
    assert!(!denied.join("metadata/timestamp.json").exists());
    assert_eq!(
        before,
        std::fs::read(renewed.join("metadata/2.targets.json")).unwrap()
    );

    // Use the upstream role signer for a true timestamp-only renewal. The
    // approved snapshot and targets remain byte-for-byte unchanged.
    use tough::{
        editor::signed::SignedRole,
        schema::{KeyHolder, Signed, Timestamp},
    };
    let timestamp_path = renewed.join("metadata/timestamp.json");
    let mut timestamp: Signed<Timestamp> =
        serde_json::from_slice(&std::fs::read(&timestamp_path).unwrap()).unwrap();
    timestamp.signed.version = number(3);
    timestamp.signed.expires = "2099-02-01T00:00:00Z".parse().unwrap();
    let snapshot_path = renewed.join("metadata/2.snapshot.json");
    let snapshot_before = std::fs::read(&snapshot_path).unwrap();
    let signature = SignedRole::new(
        timestamp.signed,
        &KeyHolder::Root(original.root.clone()),
        &original.keys[3..4],
        &aws_lc_rs::rand::SystemRandom::new(),
    )
    .await
    .unwrap();
    std::fs::write(timestamp_path, signature.buffer()).unwrap();
    assert_eq!(catalog, require_success(original.probe(&renewed, &state)));
    assert_eq!(snapshot_before, std::fs::read(snapshot_path).unwrap());
    assert_eq!(
        before,
        std::fs::read(renewed.join("metadata/2.targets.json")).unwrap()
    );

    // Optional benchmark export contains public metadata/target bytes only.
    // Never export the fixture directory: it contains temporary private keys.
    if let Some(export) = std::env::var_os("TUF_SPIKE_EXPORT") {
        let export = Path::new(&export);
        assert!(!export.exists(), "Benchmark destination must be fresh");
        std::fs::create_dir_all(export.join("metadata")).unwrap();
        for item in std::fs::read_dir(renewed.join("metadata")).unwrap() {
            let item = item.unwrap();
            assert!(item.file_type().unwrap().is_file());
            std::fs::copy(item.path(), export.join("metadata").join(item.file_name())).unwrap();
        }
        copy_targets(&renewed, export);
        std::fs::copy(
            original.dir.path().join("root.json"),
            export.join("bootstrap-root.json"),
        )
        .unwrap();
    }

    let mut replacement = Fixture::new().await;
    replacement.root.version = number(2);
    let rotated = original.dir.path().join("rotated");
    replacement.publish(&rotated, 4, FUTURE).await;
    let output = Command::new(tool)
        .args(["root", "sign"])
        .arg(replacement.dir.path().join("root.json"))
        .arg("--cross-sign")
        .arg(original.dir.path().join("root.json"))
        .arg("--key")
        .arg(original.dir.path().join("test-role-0.der"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let root = rotated.join("metadata/2.root.json");
    std::fs::copy(replacement.dir.path().join("root.json"), &root).unwrap();
    assert_eq!(catalog, require_success(original.probe(&rotated, &state)));
    std::fs::remove_file(root).unwrap();
    assert_eq!(catalog, require_success(original.probe(&rotated, &state)));
    require_failure(original.probe(&rotated, &original.dir.path().join("obsolete-client")));
}
