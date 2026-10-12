//! Ephemeral Ed25519 fixtures generated with upstream crypto and TUF editors.
use aws_lc_rs::{rand::SystemRandom, signature::Ed25519KeyPair};
use jiff::Timestamp;
use std::{collections::HashMap, num::NonZeroU64, path::Path, process::Output};
use tempfile::TempDir;
use tough::{
    editor::{
        RepositoryEditor,
        signed::{PathExists, SignedRole},
    },
    key_source::{KeySource, LocalKeySource},
    schema::{KeyHolder, RoleKeys, RoleType, Root},
};
use url::Url;

pub struct Fixture {
    pub dir: TempDir,
    pub keys: Vec<Box<dyn KeySource>>,
    pub root: Root,
}

pub fn number(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

impl Fixture {
    pub async fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let mut keys: Vec<Box<dyn KeySource>> = Vec::new();
        let mut public = HashMap::new();
        let mut roles = HashMap::new();
        for (index, role) in [
            RoleType::Root,
            RoleType::Targets,
            RoleType::Snapshot,
            RoleType::Timestamp,
        ]
        .into_iter()
        .enumerate()
        {
            let key = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
            let path = dir.path().join(format!("test-role-{index}.der"));
            std::fs::write(&path, key.as_ref()).unwrap();
            let source = LocalKeySource { path };
            let key = source.as_sign().await.unwrap().tuf_key();
            let id = key.key_id().unwrap();
            public.insert(id.clone(), key);
            roles.insert(
                role,
                RoleKeys {
                    keyids: vec![id],
                    threshold: number(1),
                    _extra: HashMap::new(),
                },
            );
            keys.push(Box::new(source));
        }
        Self {
            dir,
            keys,
            root: Root {
                spec_version: "1.0.0".into(),
                consistent_snapshot: true,
                version: number(1),
                expires: "2099-01-01T00:00:00Z".parse().unwrap(),
                keys: public,
                roles,
                _extra: HashMap::new(),
            },
        }
    }

    pub async fn sign_root(&self, old: Option<&Self>) {
        let mut authority = self.root.clone();
        let mut keys: Vec<Box<dyn KeySource>> = vec![Box::new(LocalKeySource {
            path: self.dir.path().join("test-role-0.der"),
        })];
        if let Some(old) = old {
            authority.keys.extend(old.root.keys.clone());
            authority
                .roles
                .get_mut(&RoleType::Root)
                .unwrap()
                .keyids
                .extend(old.root.roles[&RoleType::Root].keyids.clone());
            keys.push(Box::new(LocalKeySource {
                path: old.dir.path().join("test-role-0.der"),
            }));
        }
        let signed = SignedRole::new(
            self.root.clone(),
            &KeyHolder::Root(authority),
            &keys,
            &SystemRandom::new(),
        )
        .await
        .unwrap();
        std::fs::write(self.dir.path().join("root.json"), signed.buffer()).unwrap();
    }

    pub async fn publish(&self, destination: &Path, generation: u64, expires: &str) {
        let bytes = b"test-only package bytes; never execute";
        std::fs::write(self.dir.path().join("fixture-package.bin"), bytes).unwrap();
        let hash = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, bytes);
        let hex: String = hash
            .as_ref()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let catalog = serde_json::json!({"version": "0.1.0-beta.2", "package": {"length": bytes.len(), "sha256": hex}});
        self.publish_catalog(
            destination,
            generation,
            expires,
            &serde_json::to_vec(&catalog).unwrap(),
        )
        .await;
    }

    pub async fn publish_catalog(
        &self,
        destination: &Path,
        generation: u64,
        expires: &str,
        bytes: &[u8],
    ) {
        self.sign_root(None).await;
        let root_path = self.dir.path().join("root.json");
        let target = self.dir.path().join("catalog-v1.json");
        std::fs::write(&target, bytes).unwrap();
        let expiration: Timestamp = expires.parse().unwrap();
        let mut editor = RepositoryEditor::new(&root_path).await.unwrap();
        editor
            .targets_version(number(generation))
            .unwrap()
            .targets_expires(expiration)
            .unwrap()
            .snapshot_version(number(generation))
            .snapshot_expires(expiration)
            .timestamp_version(number(generation))
            .timestamp_expires(expiration);
        editor.add_target_path(&target).await.unwrap();
        let signed = editor.sign(&self.keys).await.unwrap();
        signed.write(destination.join("metadata")).await.unwrap();
        std::fs::create_dir_all(destination.join("targets")).unwrap();
        signed
            .copy_target(
                &target,
                &destination.join("targets"),
                PathExists::Replace,
                None,
            )
            .await
            .unwrap();
    }

    pub fn probe(&self, destination: &Path, state: &Path) -> Output {
        self.probe_metadata_url(
            destination,
            state,
            Url::from_directory_path(destination.join("metadata")).unwrap(),
        )
    }

    pub fn probe_metadata_url(&self, destination: &Path, state: &Path, metadata: Url) -> Output {
        self.command(destination, state, metadata).output().unwrap()
    }

    fn command(&self, destination: &Path, state: &Path, metadata: Url) -> std::process::Command {
        let mut command =
            std::process::Command::new(env!("CARGO_BIN_EXE_timestamp-cleaner-tuf-spike"));
        command
            .arg(self.dir.path().join("root.json"))
            .arg(metadata.as_str())
            .arg(
                Url::from_directory_path(destination.join("targets"))
                    .unwrap()
                    .as_str(),
            )
            .arg(state);
        command
    }

    pub fn probe_package(&self, destination: &Path, state: &Path) -> Output {
        self.command(
            destination,
            state,
            Url::from_directory_path(destination.join("metadata")).unwrap(),
        )
        .arg(
            Url::from_file_path(self.dir.path().join("fixture-package.bin"))
                .unwrap()
                .as_str(),
        )
        .output()
        .unwrap()
    }
}

pub const FUTURE: &str = "2099-01-01T00:00:00Z";
pub const EXPIRED: &str = "2000-01-01T00:00:00Z";

pub fn require_success(output: Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub fn require_failure(output: Output) -> String {
    assert!(!output.status.success(), "Probe accepted invalid fixture");
    assert!(
        output.stdout.is_empty(),
        "Failed verification exposed catalog data"
    );
    String::from_utf8(output.stderr).unwrap()
}
