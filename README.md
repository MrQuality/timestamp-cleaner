# Timestamp Cleaner

A Rust desktop application for repairing fixed camera timestamps in local family videos.

**Status: first working development milestone, not a production release.** The current repair
backend is FFmpeg rectangle interpolation. It is explicitly different from the original
Python Telea/Navier–Stokes algorithms. See [ROADMAP.md](ROADMAP.md) for the remaining work.

## Run

The application checks FFmpeg and FFprobe in the background at startup. Existing tools are
detected from PATH and known Windows package-manager locations, then checked with their version
commands. If either tool is unavailable, the application asks whether to install them. Choosing
**Continue disabled** keeps the application open with media operations disabled. You can install
or check again later.

On Windows, accepting installation runs Microsoft WinGet's `Gyan.FFmpeg` package for your user
account. The package includes both tools. WinGet retains its checksum and malware checks; the
prompt explains that installation accepts the package/source terms. Both tools are checked again
before the application enables media operations. Microsoft App Installer (WinGet) must be present;
if it is unavailable or installation fails, the application stays disabled with recovery instructions.
Linux currently requires installation through the system package manager.
See [manual installation and recovery](docs/media-tools.md) if automatic setup fails.
Clean-account installation is still unverified; the pending scenarios are listed in
[manual verification](docs/manual-verification.md).

For manual setup, the FFmpeg build must support `libx264`, `delogo`, and PNG encoding. Executables
in the current working directory are not discovered implicitly. Absolute paths can instead be
supplied through `TIMESTAMP_CLEANER_FFMPEG` and `TIMESTAMP_CLEANER_FFPROBE`; invalid overrides
must be corrected or removed, since installation does not silently replace explicit configuration.

To build from source, install Rust 1.92 or newer and run:

```powershell
cargo run --locked --release
```

Open a video, drag over its timestamp, render a preview, and toggle Original / Repaired.
Numeric controls allow precise source-pixel selection. Select export quality and export
one result. Changing the rectangle invalidates the previous repair preview. All media work
runs off the UI thread; Cancel and closing the app terminate the active media subprocess.

Select Matroska (`.mkv`) or MP4 (`.mp4`) before exporting. Both use H.264 and retain variable
frame timing through the native FFmpeg pipeline. MKV copies all audio tracks without re-encoding.
MP4 copies AAC/MP3 tracks and converts other audio codecs to AAC at 192 kbps per track for
playback compatibility. MP4 faststart places playback metadata before the video data.
The output extension must match the selected format. Existing outputs are never replaced.
Only the first non-attached-picture video stream is processed. Subtitle, data, and additional
video streams are not exported in this milestone. Source metadata and chapters are requested
from FFmpeg, but preservation of every container-specific field is not guaranteed.

Supported input: local MOV/MP4, Matroska/WebM, AVI, MPEG-TS, MPEG and Ogg containers with a
finite duration and an opaque, 8-bit SDR video stream. Dimensions must be even, at most 8192
per side and at most 32 megapixels. Rotated and non-square-pixel inputs are rejected explicitly.
Image editing, imported masks, arbitrary brush masks and temporal restoration are not yet implemented.

## Validate

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo build --locked --release
```

The `media-tests` feature enables real-media integration tests. These generate small synthetic
fixtures and require FFmpeg; missing tools fail rather than silently skip tests. For library-only
development, use `cargo test --locked --no-default-features`. CI runs Windows and Linux checks
and audits the locked dependency graph. Local Windows checks are recorded in [VALIDATION.md](VALIDATION.md).

On Linux the desktop build also needs the platform development packages listed in CI.

## Project structure

- `src/model.rs`: source-coordinate geometry and typed encoding quality.
- `src/process.rs`: cancellable, time-limited subprocess supervision and bounded pipe capture.
- `src/dependencies.rs`: independent local tool discovery and executable validation.
- `src/installer.rs`: consent-gated per-user Windows package installation.
- `src/media.rs`: input restrictions, probing and frame previews.
- `src/export.rs`: single-pass export, validation and publication without overwriting.
- `src/ui.rs`: desktop interface and one active background job.
- `tests/media_pipeline.rs`: audio, timing, corruption, cancellation and collision regressions.

See [SECURITY.md](SECURITY.md) for the threat model and [docs/architecture.md](docs/architecture.md)
for pipeline design and limitations. The earlier Python scripts remain untouched outside this repository.

FFmpeg is externally installed, not bundled. Distribution of a bundled build needs a separate
license and update review for its selected components. This repository does not yet grant a public
software license; choose one before publishing it.
