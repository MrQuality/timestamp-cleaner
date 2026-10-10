# Preserved upstream terms

`egui-0.35.0/` preserves the four font notice files shipped in the
`epaint_default_fonts` 0.35.0 registry package, plus egui's MIT/Apache texts retrieved
from the exact upstream commit in that package's `.cargo_vcs_info.json`:
`6f15dc0e16b26edce1fc2a05212eaf7e749c1d05`.

The original bytes are unchanged; `provenance.json` records SHA-256 digests. The
fonts are used through eframe/egui's enabled `default_fonts` feature; font binaries
are not copied into this directory. Preserve these terms when distributing builds
containing the fonts. Hack-Regular.txt includes both MIT and Bitstream Vera terms,
which are more detailed than the crate's aggregate SPDX expression. Do not rename
or modify the fonts without reviewing their reserved-name and modification terms.

This directory is **not a complete binary license bundle**. See
[THIRD_PARTY.md](../THIRD_PARTY.md) for the full dependency inventory and remaining
upstream notice review. The application license does not replace these terms.
