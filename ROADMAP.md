# Delivery plan and release gates

The acceptance target is a production desktop application. The first milestone establishes
a usable native GUI and a media pipeline; it is not the completed product.

## First milestone

- [x] Separate project folder, Git repository and Cargo lockfile.
- [x] GUI selection, source-coordinate rectangle editing and before/after frame previews.
- [x] Export one chosen repair with quality controls.
- [x] Worker cancellation, bounded subprocess I/O, explicit resource limits.
- [x] Audio-copy and VFR timing regression fixtures; failures never publish success.
- [x] Export collision protection and temporary output cleanup.
- [x] Windows/Linux CI definition and dependency audit gate.
- [ ] Manual GUI validation on a normal interactive Windows desktop.
- [x] User-reported smoke test: successful video generation with audio.

## Dependency management and output formats

- [x] Detect each required media tool at startup and offer installation with explicit consent (Windows/WinGet).
- [x] Keep media operations disabled when dependencies are missing or installation is declined.
- [x] Select MKV or MP4 with explicit audio compatibility behavior.
- [ ] Validate installation end to end on a clean Windows account, including one missing component and offline failure; follow [the pending manual checklist](docs/manual-verification.md).
- [x] Document manual package-manager/archive setup and recovery for missing components.

## Algorithm parity and mask editor

- Reproducible, pinned OpenCV/Clang/native FFmpeg development and runtime packages.
- Typed six-preset selector matching the original Python values exactly.
- Telea and Navier–Stokes implementation through OpenCV with independent radius and dilation.
- Source-coordinate brush, rectangle, eraser, undo/redo and binary mask import/export.
- Explicit mask-size mismatch handling; no silent stretching.
- Quality comparisons against Python reference outputs and synthetic overlaid timestamps.
- Short segment playback preview; optional six-preset comparison processes previews only.

## Production readiness

- Rotation, sample-aspect-ratio and color metadata handling; explicit HDR/alpha policy.
- Image workflow with EXIF orientation, ICC and supported bit-depth policies.
- Versioned projects, reusable timestamp masks and sequential batch queue.
- Additional output codecs and configurable audio conversion settings.
- Metadata privacy control; subtitle/additional video stream preservation policy.
- OS-enforced subprocess memory/CPU limits and strengthened executable provenance checks.
- Representative 1080p/4K performance baselines, cancellation latency and long-video soak tests.
- Crash recovery, disk-full, publication-race, malformed codec and fuzz/property test coverage.
- Accessibility, keyboard operation, high-DPI and manual end-to-end GUI tests.
- Signed Windows installer, dependency notices, update and support policy.
- No unresolved high-severity dependency advisories; reviewed release checklist.

Temporal reconstruction and moving-overlay tracking follow measurement of real footage quality.
They are separate restoration features, not assumed consequences of switching to Rust.
