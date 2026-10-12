//! Test-only package stream verification; never publishes or executes bytes.
use crate::Failure;
use aws_lc_rs::digest::{Context, SHA256};
use futures::TryStreamExt;
use tough::{DefaultTransport, Transport};
use url::Url;

pub async fn verify(url: Url, catalog: &serde_json::Value) -> Result<(), Failure> {
    let package = catalog.get("package").ok_or("missing package binding")?;
    let length = package
        .get("length")
        .and_then(|v| v.as_u64())
        .ok_or("invalid package length")?;
    if length == 0 || length > 256 * 1024 * 1024 {
        return Err("package length exceeds spike policy".into());
    }
    let hash = package
        .get("sha256")
        .and_then(|v| v.as_str())
        .ok_or("missing package SHA-256")?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid package SHA-256".into());
    }
    let mut stream = DefaultTransport::default().fetch(url).await?;
    let mut digest = Context::new(&SHA256);
    let mut received = 0_u64;
    while let Some(chunk) = stream.try_next().await? {
        received = received
            .checked_add(chunk.len() as u64)
            .ok_or("package length overflow")?;
        if received > length {
            return Err("package length mismatch".into());
        }
        digest.update(&chunk);
    }
    if received != length {
        return Err("package length mismatch".into());
    }
    let actual: String = digest
        .finish()
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != hash {
        return Err("package hash mismatch".into());
    }
    Ok(())
}
