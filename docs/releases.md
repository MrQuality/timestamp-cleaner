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

Only the draft job has repository write permission. It resolves the newest unexpired
Windows package artifact ID within its own workflow run, verifies its checksum and
never executes the binary. Resolution uses read-only Actions access; partial retries
can reuse a successful producer's earlier-attempt artifact without guessing its name.
Actions are pinned by full revision; checkout does not retain Git credentials.
Existing releases/assets are not overwritten. A rerun with an existing draft
will fail at creation; inspect the existing draft rather than silently replacing it.

## Published beta.1

On 2026-10-10, the owner requested public beta availability. The verified existing
ZIP and checksum were published as [v0.1.0-beta.1](https://github.com/MrQuality/timestamp-cleaner/releases/tag/v0.1.0-beta.1),
retaining prerelease status and without marking it latest. The tag and assets were unchanged.
This owner-directed early beta publication precedes completion of the checklist below;
it does not establish production readiness. Exact-package desktop testing, clean-machine
recovery, executable branding and artifact-specific notice review remain outstanding
and are disclosed in the public release notes.

## Beta.1 preparation checklist and outstanding work

The package declares `0.1.0-beta.1` to prepare a draft candidate. Creating its tag
does not establish that the public binary release gates have passed.

1. Merge the small version/lockfile/inventory/changelog PR after CI passes.
2. Create an annotated `v0.1.0-beta.1` tag on that merged main commit and push it.
   Inspect the tag workflow's source validation, seven quality gates and draft job.
3. Download the draft's **actual attached ZIP** and checksum. Confirm its build-info
   commit and checksum. Earlier preview evidence does not prove this build passed.
4. Before public distribution, review the exact Windows dependency notices,
   including embedded fonts and supplemental upstream attributions. Packaging
   copies available declarations and fails on missing/changed notices; it does
   not establish licensing clearance.
5. Resolve executable branding/version metadata and review runtime DLL requirements.
   The runtime window icon exists; the Windows PE icon resource is still pending.
6. Test the actual package on the available Windows x64 desktop. Record native
   GUI, MKV/MP4 playback/audio, cancellation and collision results with commit/
   tool/OS versions in VALIDATION.md. The owner has no clean Windows host and
   explicitly deferred clean-machine installation/dependency recovery verification
   to beta testing. Label those scenarios unverified in the beta release notes;
   they remain production-readiness gates. See [manual-verification.md](manual-verification.md).
7. Keep the release in draft until its selected
   [binary-release gates](open-source-readiness.md#before-a-public-binary-release)
   pass. Replace candidate notes with verified scope and known limitations before
   publishing it as a prerelease. Do not mark it a stable/latest release.

Never move a release tag to incorporate later fixes. If branding, runtime or other
changes are needed after tagging beta.1, prepare beta.2 through another reviewed
version PR and tag, then repeat verification on that package. A draft may remain
unpublished; preserve the source/build evidence rather than silently replacing it.

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
