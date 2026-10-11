# Frequently asked questions

## What does Timestamp Cleaner do?

It repairs a rectangular area containing a camera timestamp burned into a local video.
Nir Rozen created it for family trip recordings. The native Rust app lets you select
an area, inspect an original/repaired frame and export a chosen result with audio.

## Does it recover the original image under the timestamp?

No. The timestamp replaced those pixels. The current FFmpeg rectangle interpolation
estimates a repair from surrounding pixels; results vary with the scene and motion.
It does not yet implement the original Python algorithms, brush masks, six-preset
comparison, batch processing or temporal reconstruction.

## Is my video uploaded?

Processing is local, with no uploads, telemetry or remote crash reporting. If you
accept FFmpeg installation, WinGet downloads packages separately. GitHub hosts the
website and downloads; visiting those services involves their normal network requests.
Use synthetic footage when reporting problems. Exported metadata may still contain
recording dates/location; metadata stripping is not implemented yet.

## Which Windows download should I choose?

Use the **setup EXE** for a per-user Start shortcut and uninstall entry. Use the
**portable ZIP** to extract and run from your own folder. Both use the same app payload.
Beta.1 offers only ZIP; later releases can offer both. Neither is an MSI, and neither
requires Rust or Visual Studio to run. Read [setup](windows-installer.md),
[portable instructions](portable-windows.md) and [release notes](https://github.com/MrQuality/timestamp-cleaner/releases).

## Why does Windows warn, or report VCRUNTIME140.dll missing?

The builds are unsigned, so a reputation warning is possible. Use official project
downloads and compare their SHA-256 checksums; do not disable security protections.
For a missing runtime, install the official Microsoft Visual C++ v14 x64 Redistributable
linked in the setup guide. Do not obtain individual DLLs from unrelated websites.

## Why are media operations disabled?

FFmpeg and FFprobe are missing, explicitly misconfigured, or lack required capabilities.
Accept the offered WinGet installation or follow [manual recovery](media-tools.md).
Check both programs and use **Check again**. Correct invalid explicit environment
overrides first; automatic installation does not silently replace them.

## What is preserved in MKV and MP4?

Both export H.264 video with source variable-frame timing. MKV copies audio tracks;
MP4 copies AAC/MP3 and converts other audio to AAC. Only the first eligible video
stream is processed. Additional video, subtitle and data streams are omitted.
Metadata/chapters are requested, but every container-specific field is not guaranteed.
Keep the source recording and inspect picture, audio and timing after export.

## Why is my input rejected?

Current support is finite-duration, opaque 8-bit SDR video with even dimensions,
at most 8192 pixels per side and 32 megapixels. Rotated, non-square-pixel, HDR and
alpha inputs are rejected. Container and codec restrictions are in the README.
An input restriction is not necessarily file corruption; include the error and
redacted tool/version information in a report.

## How do updates and uninstall work?

Updates are manual. Setup versions install side by side, with separate uninstall
entries; remove old versions after testing the new one. Uninstall removes owned
packaged filenames and preserves files you added. ZIP users extract a new version
into a separate folder and remove the old folder when ready. External prerequisites
are managed separately. See the setup guide for partial-install and locked-file recovery.

## Is this production-ready? Where can I get help?

It is an early beta. Passing automated tests does not establish clean-host startup,
interactive setup, accessibility or long-video performance. See [manual verification](manual-verification.md),
[validation evidence](../VALIDATION.md) and [roadmap](../ROADMAP.md).
Ask in [Discussions](https://github.com/MrQuality/timestamp-cleaner/discussions),
report reproducible bugs in [Issues](https://github.com/MrQuality/timestamp-cleaner/issues),
and use [private vulnerability reporting](../SECURITY.md#reporting-vulnerabilities)
for suspected security problems. There is no supported stable release or guaranteed SLA.
