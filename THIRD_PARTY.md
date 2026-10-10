# Third-party licensing and distribution

Apache-2.0 applies to original Timestamp Cleaner material. It does not replace the
licenses of Rust crates, fonts, native libraries, codecs or external tools.

## Rust desktop dependencies

Cargo.toml and Cargo.lock record the direct and resolved dependencies. The desktop
uses eframe/egui, including its default fonts, rfd and platform libraries, plus the
image and media-support crates. A vulnerability audit is **not** a license audit.

A complete transitive license/notice inventory has **not yet been completed**.
Before distributing binaries, enumerate the exact locked graph for each release
target and feature set, inspect the included license and NOTICE files (including
font licenses), resolve ambiguous/missing declarations, and include the required
license texts and attributions with that artifact. Record the review against the
release commit and regenerate it when dependencies or features change. Do not
label this document a completed third-party notice bundle.

## FFmpeg and codecs

FFmpeg/FFprobe are separate executables, installed locally or through the user's
consented WinGet operation. This repository does not contain their binaries or
link FFmpeg libraries into the Rust executable. The current workflow requires an
FFmpeg build with libx264, delogo, PNG and, for MP4 conversion, AAC encoding.

FFmpeg's license depends on its build configuration; enabling GPL components such
as libx264 changes the applicable FFmpeg build license. Keep the installed build's
license/build information and source provenance available. If future releases
bundle FFmpeg, review that exact build, its component licenses, source provision
and codec/patent requirements before redistribution. The application's Apache
license does not grant rights over those third-party components or all codecs.
See [FFmpeg's official legal guidance](https://ffmpeg.org/legal.html) and
[component license documentation](https://ffmpeg.org/doxygen/trunk/md_LICENSE.html).

## Project artwork and imported code

The application icon is AI-generated; its generation prompt and provenance are
recorded in [assets/README.md](assets/README.md). No third-party source artwork was
supplied. Review branding and provenance before promoting it as a public identity;
generation does not establish trademark clearance or exclusive copyright.

The original Python project remains outside this Git repository. If any of its
code or other external material is imported, verify ownership and retain required
notices. Add a NOTICE file when applicable attribution requirements call for one;
do not invent upstream attribution or treat a project NOTICE as a substitute for
third-party license texts.
