# Changelog

No public release has been made. The Cargo package version identifies the current
development project; it does not imply a supported or production-ready release.

## Unreleased

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

### Fixed during development

- Muxer timestamp normalization that shifted controlled VFR/audio fixtures.
- Startup accepting executable version banners without checking required timing
  behavior and FFprobe's rotation schema; incompatible tools now keep processing disabled.

### Known limitations

- The repair algorithm is FFmpeg rectangle interpolation; the original Python
  Telea/Navier–Stokes algorithms and six-preset comparison are not implemented yet.
- Automatic installation supports Windows/WinGet; Linux setup remains manual.
- Clean-account Windows installation and native GUI/accessibility validation remain pending.
- Explorer executable-icon resources, signed packaging, complete per-binary notices
  and the wider production roadmap remain open. See ROADMAP.md and VALIDATION.md.
