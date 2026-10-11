# Windows downloads, builds and beta releases

Official downloads are in [Releases](https://github.com/MrQuality/timestamp-cleaner/releases)
and linked from the [website](https://mrquality.github.io/timestamp-cleaner/).
Public beta.1 was published on 2026-10-10 as ZIP-only, with outstanding checks disclosed.
The next beta adds a free NSIS setup EXE. Builds remain unsigned development releases,
not a production-readiness claim. No paid installer tool or service is required.

## Download choices

| Format | Use | Update/remove |
| --- | --- | --- |
| Portable ZIP | Extract the entire ZIP and run the app; keep notices together | Extract updates separately; delete the old folder when ready |
| NSIS setup EXE | Per-user, fixed versioned folder; Start shortcut and uninstall entry | Install new versions alongside old ones; remove through Settings > Apps |

See [portable instructions](portable-windows.md) and [setup/recovery](windows-installer.md).
Both use the same verified application payload. FFmpeg/FFprobe and Microsoft's Visual C++
v14 x64 runtime are external. Setup downloads no prerequisites and requests no elevation.
MSI, signing, automatic updates and Linux binary packages remain deferred.

Download `SHA256SUMS.txt` from the same release and compare the chosen file with:

```powershell
Get-FileHash .\<downloaded-file> -Algorithm SHA256
```

The checksum file covers both formats when present. Checksums detect changed bytes;
they do not authenticate the publisher. Use the official repository and keep originals.

## CI previews

The Windows quality job runs formatting, Python packaging/policy tests, strict Clippy,
Rust/media tests and a release build. It packages the ZIP, installs pinned NSIS 3.12.0,
builds setup from that ZIP, then tests silent setup, payload hashes, destination/collision
guards, relocated-uninstaller refusal, uninstall and added-file preservation.

The `windows-portable-<attempt>` Actions artifact contains ZIP, setup EXE and checksums.
The historical artifact name is retained for compatibility. These previews expire after
14 days; they are not public releases. Extract GitHub's outer artifact wrapper first.
Attempt suffixes preserve earlier immutable artifacts when jobs are rerun.
Linux checks packaging logic but does not execute the Windows installer.

## Prepare and publish a beta

1. Prepare a small version PR. Cargo's package version, lockfile and dependency inventory
   fingerprint must agree. Document changes, known limits and actual validation.
2. Review and merge through protected main with all seven required checks passing.
3. Create an annotated tag, for example `v0.1.0-beta.2`, on merged main and push it.
   Never move a published tag or silently replace verified packages.
4. Inspect **Windows release packages**. It validates tag/version and main ancestry,
   reuses the full quality workflow, then creates a draft prerelease with both packages.
   A manual main dispatch builds previews only; it creates no tag or release.
5. Download actual draft assets. Verify both checksums and the ZIP's version/commit/lock
   metadata. Setup's additional build metadata records the exact source ZIP hash,
   installer source commit and NSIS version/notice hash. Build-runner smoke results
   must pass; an earlier preview does not establish this tagged package's results.
6. Review the selected artifact's dependency/font notices, setup runtime and branding.
   Run interactive app/setup smoke tests and record date, version/commit, OS/tool
   versions and results in VALIDATION.md. Do not substitute compile success for them.
7. Replace generated candidate notes with verified scope and explicit remaining limits.
   Publish an authorized early beta as a prerelease, not stable/latest. The owner
   deferred clean-host installation/recovery testing to beta; keep it unverified until
   evidence exists. Any other incomplete release-readiness item must remain disclosed.

Public beta.1 preceded completion of desktop, branding and artifact-notice review;
publication did not close those gates. See [readiness](open-source-readiness.md),
[manual checklist](manual-verification.md) and [roadmap](../ROADMAP.md).

Only the draft job has repository write access. It resolves the newest unexpired
Windows artifact from its own run, verifies all listed checksums, and does not execute
downloaded application files. Partial retries can reuse a successful producer's earlier
attempt. Existing release/assets are never overwritten; an existing draft causes creation
to fail safely. Inspect it instead of retrying blindly. Actions use immutable revisions
and checkout does not retain credentials. Later fixes require a new version and tag.

## Local packaging

Python 3.10+, Cargo/Rust MSVC prerequisites and NSIS 3.12 are required on Windows:

```powershell
cargo build --locked --release --target x86_64-pc-windows-msvc
python -m unittest discover -s scripts -p "test_*.py" -v
python scripts/package_windows.py
python scripts/package_installer.py dist/timestamp-cleaner-<version>-windows-x64.zip --compiler 'C:\Program Files (x86)\NSIS\makensis.exe'
```

Use an empty ignored output directory. The ZIP includes app license/changelog, verified
normal/build dependency notices, target inventory, compiler/commit/lock metadata and file
hashes. Development-only/other-target packages, FFmpeg, private media, secrets and caches
are excluded. Its container is deterministic for identical payload bytes; this does not
claim independently reproducible Rust or NSIS binaries. Rebuild before packaging; the
ZIP script cannot prove an arbitrary binary was compiled from the current tree.

NSIS compilation uses no local configuration and treats warnings as errors. A completed
setup is copied without replacing an existing EXE and its hash is appended to the shared
checksum file. A failed build may need a fresh output directory; never edit an existing
release's files as recovery. Run installation tests only on disposable Windows runners.
