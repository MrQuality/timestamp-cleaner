# Timestamp Cleaner

<img src="assets/icon.png" alt="Timestamp Cleaner: a film frame, mountain landscape and restoration sparkle" width="128" />

**Keep the memories. Clean up the timestamp.** Timestamp Cleaner is a local Rust desktop
application for repairing camera timestamps burned into personal and family videos.

I created this project to remove timestamps from my own family videos recorded during trips.
It started as a Python script; this Rust application adds a native interface to select the
timestamp area, compare a repair preview and export one chosen result with audio.
Video processing happens on your computer, with no uploads or telemetry.

**[Project website and download instructions](https://mrquality.github.io/timestamp-cleaner/)**
· [Windows beta releases](https://github.com/MrQuality/timestamp-cleaner/releases)

[![Rust quality gates](https://github.com/MrQuality/timestamp-cleaner/actions/workflows/ci.yml/badge.svg)](https://github.com/MrQuality/timestamp-cleaner/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## What you can do today

- Select a timestamp rectangle visually and refine it with source-pixel controls.
- Compare original and repaired frame previews before processing the full video.
- Export one selected result as MKV or MP4, with quality controls and audio preservation.
- Cancel work, keep the original untouched and avoid overwriting existing exports.
- Detect FFmpeg/FFprobe at startup and optionally install them through WinGet on Windows.

**Status: first working development milestone, not a production release.** The current repair
backend is FFmpeg rectangle interpolation. It is explicitly different from the original
Python Telea/Navier–Stokes algorithms. See [ROADMAP.md](ROADMAP.md) for the remaining work.

## Download and run

The application checks FFmpeg and FFprobe in the background at startup. Existing tools are
detected from PATH and known Windows package-manager locations, then checked with their version
commands. FFmpeg also runs a bounded, one-frame repair/timing self-test, and FFprobe must
expose the rotation side-data schema. If either tool is unavailable or incompatible, the
application asks whether to install them. Choosing
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

For manual setup, use a current FFmpeg/FFprobe build supporting `libx264`, `delogo`, `lavfi`,
PNG encoding, `-fps_mode passthrough` and `-enc_time_base demux`. Ubuntu 22.04's FFmpeg 4.4
package is incompatible; install a current build instead. Executables
in the current working directory are not discovered implicitly. Absolute paths can instead be
supplied through `TIMESTAMP_CLEANER_FFMPEG` and `TIMESTAMP_CLEANER_FFPROBE`; invalid overrides
must be corrected or removed, since installation does not silently replace explicit configuration.

To build from source, install Rust 1.92 or newer and run:

```powershell
git clone https://github.com/MrQuality/timestamp-cleaner.git
cd timestamp-cleaner
cargo run --locked --release
```

For Windows downloads, visit [Releases](https://github.com/MrQuality/timestamp-cleaner/releases)
or the [project website](https://mrquality.github.io/timestamp-cleaner/). Beta.1 is ZIP-only;
beta.2 adds a free NSIS setup EXE. Choose setup for a per-user Start shortcut and
uninstall entry, or ZIP for a portable folder. Both are unsigned beta builds. Neither
requires Rust or Visual Studio to run; the external Microsoft Visual C++ v14 x64 runtime
and compatible FFmpeg/FFprobe are required. See [Windows setup, updates and removal](docs/windows-installer.md)
and [portable instructions](docs/portable-windows.md).

To compile on Windows, install the Rust MSVC toolchain and Visual Studio Build Tools
with the **Desktop development with C++** workload. Windows CI builds both download
formats; expiring Actions artifacts are development previews. See [builds and beta releases](docs/releases.md)
for packaging and publication instructions. NSIS is required only to build setup EXEs.
On Ubuntu, install the desktop build prerequisites before running Cargo:

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libxkbcommon-dev libwayland-dev libgtk-3-dev
```

Install compatible media tools using [the setup guide](docs/media-tools.md); older distro
packages may fail the startup capability checks. Windows and Linux are the current targets;
macOS is not validated.

The Linux file picker uses the system D-Bus library (`libdbus-1-3` on Ubuntu), a
session bus and an XDG desktop portal with your desktop's backend. Install these
through your distribution if they are missing. `zenity` provides a fallback when
portal dialogs are unavailable. Headless build/media tests do not verify these dialogs.

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
and audits the locked dependency graph. Local Windows/Linux evidence is recorded in [VALIDATION.md](VALIDATION.md).

CI also checks Rust 1.92, dependency license/source policy, inventory freshness,
workflow syntax and full-history secrets. Publication audit evidence and remaining
owner decisions are recorded in [docs/publication-audit.md](docs/publication-audit.md).

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

## License

Original project code and documentation are licensed under [Apache License 2.0](LICENSE).
The generated icon's provenance and licensing scope are recorded in [assets/README.md](assets/README.md).
Third-party dependencies retain their own licenses; see [THIRD_PARTY.md](THIRD_PARTY.md).
FFmpeg is externally installed, not bundled. A bundled build needs a separate license and
update review for its selected components.

## Documentation

Start with the [FAQ and troubleshooting](docs/faq.md), [media-tool recovery](docs/media-tools.md),
[release process](docs/releases.md), and [pending manual checks](docs/manual-verification.md).
[Architecture](docs/architecture.md), [security](SECURITY.md) and [validation evidence](VALIDATION.md)
explain the engineering scope and its current limits.
The [updater maintainer checklist](docs/updater-maintainer-onboarding.md) covers
free-service boundaries, later signing setup and ongoing maintenance.

Accepted [bug reporting and update requirements](docs/reporting-and-updates.md)
describe planned features; beta.2 still uses manual reports and updates.

## Join the project

Bug reports, feature proposals, documentation, accessibility feedback and focused pull
requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](ROADMAP.md). Share synthetic examples rather than personal recordings.

- [Issues](https://github.com/MrQuality/timestamp-cleaner/issues): reproducible bugs and scoped feature requests.
- [Discussions](https://github.com/MrQuality/timestamp-cleaner/discussions): questions, ideas and workflow feedback.
- [Private security reports](https://github.com/MrQuality/timestamp-cleaner/security/advisories/new): suspected vulnerabilities.
- [Support](.github/SUPPORT.md), [community conduct](CODE_OF_CONDUCT.md) and [maintainership](GOVERNANCE.md).

The project is maintained by [Nir Rozen (@MrQuality)](https://github.com/MrQuality).
Pull requests require passing CI and resolved conversations; no approving review is
required while the project has a single maintainer. Independent review is welcome.
Remaining release gates are tracked in [docs/open-source-readiness.md](docs/open-source-readiness.md).
Development changes and known limitations are summarized in [CHANGELOG.md](CHANGELOG.md).
