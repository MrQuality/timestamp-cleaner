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

## Application icon integration

On 2026-10-10, the generated icon was saved in assets with its prompt/provenance.
The 256-pixel PNG was visually inspected and FFprobe confirmed RGBA at 256 x 256;
the ICO contains a 256-pixel PNG image. Formatting, strict all-target/all-feature
Clippy and the release build passed after embedding the PNG in the native window.
No media processing code changed, so media regressions were not repeated for artwork.
Native taskbar, Explorer and high-DPI appearance remain unverified. The standalone
Windows executable does not yet embed a PE icon resource; the ICO is available for
that future packaging step.

## Publication preparation and Linux portability

Validated on 2026-10-10 at application commit 06daae7 and its unchanged locked graph.
Earlier statements about missing Linux local evidence describe the historical runs above;
Linux WSL verification has now been completed. Hosted GitHub CI remains unrun.

- Independent clean-checkout Windows tests on Rust 1.92.0: all 32 pass (20 library,
  3 headless desktop, 9 real-media integration). Windows strict Clippy/formatting and
  updated release build pass on the default Rust 1.93.0 toolchain.
  The updated independent-checkout release build also passes on Rust 1.92.0.
- Ubuntu 22.04 WSL, Rust 1.92.0, FFmpeg/FFprobe n8.1.3-17-gc389840894-20261010:
  all 32 tests, release build, strict all-target/all-feature Clippy and formatting pass.
  Modern tools were downloaded from an FFmpeg-linked provider and checksum-verified.
- The initial Linux suite failed seven integration tests using distribution FFmpeg/FFprobe
  4.4.2. Startup now verifies the repair/timing pipeline and required rotation schema;
  the shared diagnostic rejects these old tools and accepts the modern build. This was
  a compatibility finding, not a silently skipped fixture or successful old-tool result.
- An isolated inverted FFprobe schema guard fails its regression on Rust 1.92.0.
  Original checkout contents were restored; primary source was never mutated.
- Cargo-deny 0.20.2 license/source policy and the 376-package inventory pass. Inventory
  output is reproducible on both platforms; wrong hashes and stale inventory are rejected.
  Upstream documents for 46 missing-notice packages are preserved; two unused-target/feature
  gaps and per-binary notice review remain open. This is not a complete legal clearance.
- Gitleaks 8.30.1 finds no secrets in 16 reachable commits through 06daae7, with no custom
  suppression. An isolated deleted synthetic token is detected. Actionlint 1.7.12 passes.
- Refreshed cargo-audit 0.22.2 `audit --deny warnings` passes: 1296 advisories and
  377 locked packages scanned. Reachable commit-message bodies also pass secret/path review.
- Source-package file-list and historical-path reviews found no family footage or ignored
  tooling in the distributable source list. Author identity/ownership remains an owner decision.

See [docs/publication-audit.md](docs/publication-audit.md) for reproducible commands,
provenance and exact limitations. Clean-account Windows installation, interactive native GUI,
Explorer/PE icon packaging, signing and the remaining production roadmap are still pending.
