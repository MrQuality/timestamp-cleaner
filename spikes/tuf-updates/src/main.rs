//! Disposable verifier probe; not application update code or an installer launcher.
mod package;
use futures::TryStreamExt;
use std::{error::Error, path::PathBuf, time::Duration};
use tough::{RepositoryLoader, TargetName};

type Failure = Box<dyn Error + Send + Sync>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err(
            "usage: tuf-spike ROOT METADATA_URL TARGETS_URL STATE_DIR [FIXTURE_PACKAGE_URL]".into(),
        );
    }
    let state = PathBuf::from(&args[3]);
    std::fs::create_dir_all(&state)?;
    let persisted_root = state.join("root.json");
    let root = if persisted_root.exists() {
        std::fs::read(persisted_root)?
    } else {
        if std::fs::read_dir(&state)?.next().is_some() {
            return Err("missing trusted root in nonempty state; manual recovery required".into());
        }
        let root = std::fs::read(PathBuf::from(&args[0]))?;
        let signed: tough::schema::Signed<tough::schema::Root> = serde_json::from_slice(&root)?;
        signed.signed.verify_role(&signed)?;
        // Seed the caller-trusted bootstrap before network work. Otherwise an
        // initial timeout leaves a clock file but no root and makes retry ambiguous.
        // This write is intentionally subject to the same production atomicity gate.
        std::fs::write(&persisted_root, &root)?;
        root
    };
    let work = async {
        let repo = RepositoryLoader::new(
            &root,
            args[1].to_str().ok_or("invalid metadata URL")?.parse()?,
            args[2].to_str().ok_or("invalid targets URL")?.parse()?,
        )
        .datastore(&state)
        .load()
        .await?;
        // tough persists root.json even if a later role fails. On the next run,
        // supply that root explicitly; the loader does not select it for us.
        // Atomic storage and process locking remain production integration gates.
        let name = TargetName::new("catalog-v1.json")?;
        if repo
            .targets()
            .signed
            .targets
            .get(&name)
            .ok_or("missing catalog")?
            .length
            > 256 * 1024
        {
            return Err("catalog exceeds spike limit".into());
        }
        let stream = repo.read_target(&name).await?.ok_or("missing catalog")?;
        let chunks: Vec<_> = stream.try_collect().await?;
        let bytes: Vec<_> = chunks.into_iter().flatten().collect();
        let catalog: serde_json::Value = serde_json::from_slice(&bytes)?;
        if let Some(package_url) = args.get(4) {
            package::verify(
                package_url.to_str().ok_or("invalid package URL")?.parse()?,
                &catalog,
            )
            .await?;
        }
        println!("{catalog}");
        Ok::<(), Failure>(())
    };
    tokio::time::timeout(Duration::from_secs(5), work).await??;
    Ok(())
}
