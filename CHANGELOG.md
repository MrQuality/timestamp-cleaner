# Changelog

Public beta releases are available. They are development builds and do not imply
a supported or production-ready release. See [release instructions](docs/releases.md).

## Unreleased

## 0.1.0-beta.2

- Free per-user NSIS setup EXE alongside the portable ZIP, with versioned Start
  shortcuts/uninstall entries, side-by-side updates and preservation of user-added files.
- Verified ZIP-to-installer packaging, NSIS notices/provenance and automated Windows
  install/payload/collision/uninstall checks; setup links on the project website.
- Corrected download, release-state and metadata-privacy documentation; setup/update/removal
  instructions, FAQ and explicit remaining manual validation.

## 0.1.0-beta.1 — draft candidate

Published as the first Windows x64 portable ZIP beta at the owner's request.
Exact-package desktop testing, clean-host recovery, executable branding and
artifact-specific notice review remained outstanding and were disclosed in its release notes.

### Added

- Rust native desktop interface with video loading, source-coordinate rectangle
  selection, numeric controls, original/repaired frame comparison and one selected export.
- H.264 export to MKV or MP4 with variable-frame timing. MKV copies audio tracks;
  MP4 copies AAC/MP3 and converts other audio to AAC.
- Background FFmpeg/FFprobe discovery, capability checks, consent-gated Windows
  WinGet installation and disabled/recovery states when tools are unavailable.
- Cancellation, bounded subprocess diagnostics, input restrictions, export validation
  and publication without replacing an existing file.
- Application artwork and a native window icon; PNG/ICO assets and provenance.
- Apache-2.0 licensing, contribution guidance, issue/PR templates and local publication
  audits, plus Windows/Linux CI quality, minimum-Rust and security/license checks.
- Windows portable ZIP previews with dependency notices, build metadata, file
  hashes and SHA-256 checksums; matching beta tags prepare draft GitHub prereleases.

### Fixed during development

- Muxer timestamp normalization that shifted controlled VFR/audio fixtures.
- Startup accepting executable version banners without checking required timing
  behavior and FFprobe's rotation schema; incompatible tools now keep processing disabled.

### Known limitations

- The Windows ZIP requires Microsoft's external Visual C++ v14 x64 runtime;
  see the included START-HERE.md if Windows reports VCRUNTIME140.dll missing.

- The repair algorithm is FFmpeg rectangle interpolation; the original Python
  Telea/Navier–Stokes algorithms and six-preset comparison are not implemented yet.
- Automatic installation supports Windows/WinGet; Linux setup remains manual.
- Clean-account Windows installation and native GUI/accessibility validation remain pending.
- Explorer executable-icon resources, signed packaging, complete per-binary notices
  and the wider production roadmap remain open. See ROADMAP.md and VALIDATION.md.
