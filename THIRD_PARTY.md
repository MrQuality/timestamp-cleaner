# Third-party licensing and distribution

Apache-2.0 applies to original Timestamp Cleaner material. It does not replace the
licenses of Rust crates, fonts, native libraries, codecs or external tools.

## Rust desktop dependencies

Cargo.toml and Cargo.lock record the direct and resolved dependencies. The desktop
uses eframe/egui, including its default fonts, rfd and platform libraries, plus the
image and media-support crates. A vulnerability audit is **not** a license audit.

The [generated inventory](docs/dependency-licenses.csv) covers 376 external packages
resolved by `cargo metadata --locked --all-features`, including packages for targets
and optional dependencies that may not be part of a particular binary. Every row
records the declared license, version, source/repository, lockfile fingerprint and
packaged notice-file hashes. No absolute cache paths are published.

`deny.toml` defines the accepted code/font license declarations and rejects unknown
registry/Git sources. Cargo-deny 0.20.2 passes `--locked --all-features check licenses
sources` without warnings for the current graph. SPDX declaration checks do not
inspect every embedded source/font notice and do not establish legal clearance.

Forty-eight packages have no conventional license/notice file in the downloaded
crate. Upstream licensing documents have been preserved for 46 of them, fetched
from the exact commits recorded in the registry packages' `.cargo_vcs_info.json`.
The inventory records these separately as `upstream-documents-found` and verifies
their hashes and commit provenance. `dispatch` 0.2.0 and `hexf-parse` 0.2.1 remain
`upstream-review-needed`: no standalone license document was found at those commits.
Neither appears in the current all-feature Windows x64 or Linux x64 dependency tree.
Review them before enabling features/targets that use them or distributing a
broader dependency/source bundle.

egui/font terms are also preserved in [third-party/](third-party/README.md).
Hack's notice includes Bitstream Vera terms beyond the aggregate SPDX declaration;
AccessKit's upstream Chromium attribution is preserved too. The objc2 upstream
documents raise Apple SDK-derived licensing questions: Apple targets are outside
the current Windows/Linux support scope and need separate review before support.
An upstream licensing document may link to terms rather than contain a full
redistribution notice. Presence of a document is not clearance for that artifact.
The inventory is **not a completed distribution notice bundle**. Before distributing
binaries, enumerate the exact locked graph for each release
target and feature set, inspect the included license and NOTICE files (including
font licenses), resolve ambiguous/missing declarations, and include the required
license texts and attributions with that artifact. Record the review against the
release commit and regenerate it when dependencies or features change. Do not
label this document a completed third-party notice bundle.

Regenerate and check the inventory with Python 3.10+ and Cargo:

```sh
python scripts/dependency_licenses.py
python scripts/dependency_licenses.py --check
cargo install cargo-deny --version 0.20.2 --locked
cargo deny --locked --all-features check licenses sources
```

Regeneration fetches missing registry crates through Cargo. Review changed declarations,
sources and notices before accepting inventory changes. A stale inventory fails `--check`;
adding a license allowance requires review rather than automatically copying new declarations.

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
