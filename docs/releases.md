# Windows ZIP builds and first beta

## Available automation

The Windows quality job creates a `windows-portable-<attempt>` Actions artifact after
formatting, packaging tests, strict Clippy, all media tests and a release build.
This runs for pull requests and main. Attempt suffixes allow rerunning jobs without
replacing an earlier attempt's immutable artifact. Artifacts expire after 14 days; they are
development previews, not public releases. GitHub wraps the already-built ZIP
and its checksum in an artifact download; extract that outer wrapper first.

The **Windows portable release** workflow has two entry points:

- Run manually from `main` to rebuild all seven quality gates and produce an
  artifact. It does not create a tag or a release.
- Push a tag such as `v0.1.0-beta.1`, exactly matching Cargo's package version,
  to rerun the same quality workflow and create a **draft prerelease** with the
  ZIP and checksum attached. The tagged commit must already be part of main.

Only the draft job has repository write permission. It downloads the artifact
from its own workflow run, verifies its checksum and never executes the binary.
Actions are pinned by full revision; checkout does not retain Git credentials.
Existing releases/assets are not overwritten. A rerun with an existing draft
will fail at creation; inspect the existing draft rather than silently replacing it.

## Preparing v0.1.0-beta.1

The project currently declares `0.1.0` for development. No beta tag is created by
this change. Before tagging:

1. In a small PR, set Cargo's version to `0.1.0-beta.1`, regenerate Cargo.lock and
   `docs/dependency-licenses.csv`, update CHANGELOG.md, and merge after CI passes.
2. Verify the exact Windows dependency notices, including embedded fonts and
   supplemental upstream attributions. Packaging copies available declarations
   and fails on missing/changed notices; it does not establish licensing clearance.
3. Resolve executable branding/version metadata and review runtime DLL requirements.
   The runtime window icon exists; the Windows PE icon resource is still pending.
4. On main, run `gh workflow run release.yml --ref main`; inspect all seven gates
  and download its `windows-portable-<attempt>` artifact. Test on a clean Windows x64 host.
5. Record native GUI, MKV/MP4 playback/audio, cancellation, collision, dependency
   installation/decline/offline/manual recovery results with commit/tool/OS versions
   in VALIDATION.md. See [manual-verification.md](manual-verification.md).
6. Merge the validation/release notes, then create an annotated `v0.1.0-beta.1`
   tag on the selected main commit and push that tag. Never move a published tag.
7. Inspect the new draft and test its **actual attached ZIP**. Confirm the build-info
   commit and checksum. Local/manual preview evidence does not prove this build passed.
8. Replace candidate notes with verified scope and limitations, complete the
   [binary-release checklist](open-source-readiness.md#before-a-public-binary-release),
   then publish the draft as a prerelease. Do not mark it a stable/latest release.

No installer, signing certificate or paid packaging service is required for the
ZIP pipeline. Builds are unsigned and that status must remain visible. Signing,
installers, Linux binary distribution and automatic updates are deferred.
Prerelease versions may change behavior; no backwards compatibility is promised.
Updates are manual, by extracting the new version into a separate directory.

## Local packaging

With Python 3.10+, Cargo and the Windows MSVC toolchain:

```powershell
cargo build --locked --release --target x86_64-pc-windows-msvc
python -m unittest discover -s scripts -p "test_*.py" -v
python scripts/package_windows.py
```

The output is under ignored `dist/`; choose `--output <empty-directory>` for another
build. The ZIP includes the executable, instructions, app license, third-party
notice files, target dependency CSV, compiler/commit/lock metadata and file hashes.
Normal/build dependencies are included conservatively, even when build tooling
does not ship in the executable. Development-only and other-target packages are
excluded. FFmpeg binaries, personal media, credentials and local tool caches are
not included. The container is deterministic for identical payload bytes; this
does not claim independently reproducible Rust binaries. Always rebuild before
packaging; the script cannot prove an arbitrary `--binary` came from the current tree.
