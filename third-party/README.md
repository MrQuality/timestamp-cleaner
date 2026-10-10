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

`upstream-provenance.json` maps 48 packages without packaged notices to their
exact upstream repository commits. Documents for 46 packages are preserved under
`upstream/` or the shared egui directory. SHA-256 digests and direct source URLs
are recorded; the generator verifies their bytes and matches registry VCS commits.
Two cases have no standalone upstream license document and remain explicit gaps.
These copies include upstream licensing commentary and attribution, not just
canonical license texts. Preserve their context and review the actual release
target before assembling a distribution notice bundle.
