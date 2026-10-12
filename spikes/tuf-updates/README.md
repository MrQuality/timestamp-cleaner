# TUF update feasibility experiment

This standalone Cargo workspace probes `tough = 0.24.0` with its real HTTP/TLS feature
and an independent lockfile. It is not application code, a production publisher or
an installer launcher. The application's graph, version and binaries are unchanged.
The parent [architecture](../../docs/auto-update-architecture.md) is still gated on
the findings in this experiment and the [results and integration gates](RESULTS.md).

```powershell
cargo +1.92.0 test --locked --manifest-path spikes/tuf-updates/Cargo.toml
cargo +1.92.0 build --locked --release --manifest-path spikes/tuf-updates/Cargo.toml
```

Use `--target-dir .tools/tuf-target` to keep experimental output in the existing
ignored tools directory. Linux uses its own target directory. The optional GitHub
workflow runs isolated Windows/Linux Rust 1.92 checks and does not publish packages.

Fixtures generate four temporary Ed25519 role keys with AWS-LC and construct metadata
with the upstream TUF editor. No private key is committed or used for real releases.
Tests run the verifier as a fresh OS process, checking durable metadata/root state,
same-version renewal, rollback, expiry, altered signatures, modified/truncated catalog
bytes, stalled/interrupted local HTTP responses and a cross-signed root rotation. A stale-root negative control proves that the
successful restart depends on the explicitly persisted new root.

The CLI takes bootstrap root, metadata URL, targets URL and state directory. It
collects the complete bounded catalog stream before exposing JSON, and has a five-second
cooperative timeout; this is not a hard wall-clock bound. File URLs and arbitrary fixture endpoints are deliberate test
facilities. Production origin/redirect policy, atomic storage, cross-process locking,
catalog schema/version policy and user-facing cancellation are not implemented here.
The upstream datastore does not persist roots and uses non-atomic file writes; the
probe's additional root write is also non-atomic. Do not integrate this storage path
into executable updates without resolving those gates.

Inspect the spike separately from the app:

```powershell
cargo audit --file spikes/tuf-updates/Cargo.lock --deny warnings
cargo deny --manifest-path spikes/tuf-updates/Cargo.toml --config deny.toml --locked check licenses sources
```

The declaration/source check is expected to expose newly required licenses under the
unchanged project policy. Do not broaden policy or claim artifact-notice clearance
merely to obtain a green experiment. No spike binaries or dependency notices are
added to current release assets.

## Publisher ceremony experiment

The ordinary suite does not require the publisher CLI. Its explicitly ignored
publisher test is run separately; a skipped test is not publisher evidence. Install
`tuftool = 0.17.0` with `--locked --no-default-features` using an isolated Rust 1.94.1
toolchain (its AWS dependencies prevent building it on 1.92/1.93). Keep the app MSRV
unchanged. The dedicated read-only Linux CI job builds this tool and runs:

```powershell
$env:TUFTOOL_BIN = (Resolve-Path .tools/tuftool/bin/tuftool.exe).Path
cargo +1.92.0 test --locked --manifest-path spikes/tuf-updates/Cargo.toml --test verification publisher::cli_renewal_rotation_and_timestamp_key_separation -- --ignored --nocapture
```

The test generates disposable keys, runs CLI metadata renewal and old-root
cross-signing, verifies the results with fresh client processes, rejects CLI
multi-role signing with just the timestamp key, and uses the upstream role signer
for timestamp-only renewal without modifying snapshot/targets bytes. It does not
create a production repository. CLI initial `create`, release promotion, expiry
monitoring, version allocation and deployment races remain untested.

For measurements, optionally set `TUF_SPIKE_EXPORT` to a **fresh**, ignored path
before running the publisher experiment. It exports only the public bootstrap root,
metadata and catalog. Never copy the complete temporary fixture directory: it
contains private test keys. An existing export path fails rather than overwrites.

## Runtime measurements

With a release probe binary and the public fixture export above:

```powershell
python spikes/tuf-updates/measure_probe.py .tools/tuf-target/release/timestamp-cleaner-tuf-spike.exe .tools/tuf-public-benchmark
```

The standard-library helper runs five trials each for file/local HTTP and
fresh/persisted state. Each trial launches a separate process; the HTTP server binds
only loopback and serves the public export. Windows samples OS peak-working-set
counters; a short process can exit between samples, so these are lower bounds.
No private key, installer, real release or remote server is used. Output is JSON;
redirect to an ignored file with a UTF-8-aware shell if it will be parsed elsewhere.
It supports non-Windows timing but does not measure non-Windows memory. This does
not measure integrated app size growth, TLS/proxy latency or a reused worker.
