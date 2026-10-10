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

## User-reported smoke test

On 2026-10-10, the user reported successful video generation with audio using the application.
This confirms their export/playback smoke test. The input properties, selected settings and
output container were not supplied, so this report does not establish a broader format,
timing, accessibility or restoration-quality result. It is separate from automated test evidence.

## Dependency management and selectable output update

Validated locally on Windows, 2026-10-10, with the same Rust and FFmpeg versions above.

- Formatting and strict Clippy across all targets/features passed.
- `cargo test --locked --all-features`: 29 tests passed (17 library, 3 headless desktop,
  9 real-media integration); no ignored or silently skipped fixtures.
- `cargo build --locked --release`: passed; the native executable includes both new features.
- Dependency audit with `--deny warnings --no-fetch`: passed against the locally fetched
  1296-advisory database, scanning 377 locked dependencies.
- MP4 regressions cover VFR frame and audio-packet timing, copied AAC payload hashes,
  faststart ordering, two-track FLAC-to-AAC conversion and silent input. MKV checks still pass.
- Discovery tests cover one missing component, explicit overrides, excluded working-directory
  candidates and package-manager links visible without refreshing the process PATH.
- Installer tests use injected runners; decline and cancellation never run the package manager,
  failure never reports readiness, and a successful exit still requires both usable tools.
- Headless desktop checks cover both output choices and missing/declined/failed/recovered
  dependency states. Rendering a consent prompt does not grant consent.
- Mutation experiment: reversing the private installer consent guard caused three tests to fail,
  including the decline test's forbidden-runner assertion. Source was restored byte-for-byte;
  all four installer tests then passed with all features. Windows held the default-feature test
  executable open after the experiment, preventing its relink; the all-feature test artifact
  was unaffected. No real installer was invoked by the experiment.

A clean-account WinGet installation, offline failure and interactive native save-dialog/prompt
checks remain unvalidated. This host already has both media tools; no actual download or
installation was performed. Linux CI and production-readiness gates remain open as recorded
in ROADMAP.md. The user-reported audio smoke test above predates this update.

The outstanding native/installation scenarios and evidence requirements are tracked in
[docs/manual-verification.md](docs/manual-verification.md). Manual fallback instructions are
available in [docs/media-tools.md](docs/media-tools.md). These documents do not change test results.
