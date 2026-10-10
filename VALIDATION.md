# Validation record

This file records local evidence, separately from CI configuration. CI results must not be
inferred merely because a workflow exists.

Validated locally on Windows, 2026-10-10, with Rust 1.93.0 and FFmpeg
N-114971-g1b41631185-20240423. The application requires Rust 1.92 or newer.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed.
- `cargo test --locked --all-features`: 16 tests passed (9 core, 1 headless UI, 6 real-media integration).
- `cargo audit --deny warnings --no-fetch`: passed against 1296 advisories fetched in this session;
  377 locked dependencies scanned. The initial font-parser maintenance advisory was resolved by
  upgrading eframe from 0.33 to 0.35, not by suppressing the advisory.
- `cargo build --locked --release`: passed; native Windows executable produced at
  `target/release/timestamp-cleaner.exe` using eframe 0.35.
- Isolated mutation check: inverting the source-region boundary condition caused both geometry
  tests to fail. The original source was never changed by this experiment.

Integration checks include both source audio payload hashes, individual audio packet and video
frame timestamps (2 ms rounding tolerance), real variable frame intervals, unchanged source bytes,
non-ASCII and shell-metacharacter paths, corrupt/playlist rejection, failed tools, timeout,
midflight cancellation, existing destinations and publication collisions. No fixture is silently skipped.

The VFR test initially caught a 23 ms muxer timestamp shift. Disabling automatic negative-timestamp
normalization fixed it; both audio packet timing and video frame timing now pass the regression.
The Windows build also encountered generated read-only directories and stale PDB artifacts;
clearing generated directory attributes and regenerating the affected PDBs resolved those host issues.

Linux CI is configured but has not run in this local session. The headless layout test is not
a substitute for interactive Windows GUI, high-DPI, keyboard, accessibility or visual-quality testing.
No installer, signing, long-duration soak, real-footage benchmark or full OS codec sandbox is validated yet.

No family footage is included in the test fixtures. Tests synthesize short local video and audio.
Visual restoration quality on the user's recordings still needs manual assessment.
