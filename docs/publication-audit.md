# Local publication audit

Recorded on 2026-10-10. This is evidence for preparing a development source
repository, not certification of a production release or legal clearance.

## History and privacy

The audited application source/history tip is `06daae7`; Gitleaks 8.30.1 scanned
all 16 reachable commits using `git --redact --log-opts="--all --full-history"`
and found **zero secret findings**. No custom allowlist or baseline suppressed
findings. Scan future commits again before pushing.
Reachable commit-message bodies were also scanned through Gitleaks stdin and
checked for local user paths, with no findings.

All 107 unique reachable historical file blobs and historical paths were reviewed for local user
paths, personal material and private keys. The only user-path pattern was the
deliberate `C:\Users\YourName` example in the manual setup guide. Historical
binary assets are the generated application PNGs/ICO; no family footage is tracked.
The original PNG includes OpenAI/C2PA provenance; its metadata was not stripped.
No local user path or email-like string was found in that embedded provenance.
Upstream copyright/contact details in preserved license documents are intentional
attribution, not the maintainer's private contact configuration.

There is one project Git author identity. **The owner must decide whether the
existing author/committer name and email are appropriate for public history.**
No history was rewritten. Choosing a different future Git identity does not remove
existing commit metadata. The owner must also confirm publication rights and any
employer/external-contribution constraints; a scanner cannot establish ownership.

Scanner verification used an isolated repository containing a synthetic GitHub
token that was then deleted. Full-history scanning still detected it. No real
credential was used, and fixtures/reports are in ignored local tooling.

## Dependencies and notices

- Inventory: 376 external locked packages; no missing declared license field.
- Policy: cargo-deny 0.20.2 passes `--locked --all-features check licenses sources`
  without warnings. Unreviewed registry/Git sources are denied.
- Refreshed cargo-audit 0.22.2 check with `--deny warnings` passes against 1296
  advisories, scanning 377 locked packages including the application.
- The inventory ties versions, declarations and notice hashes to Cargo.lock, avoids
  absolute cache paths, and is reproducible on Windows and Linux.
- All six preserved egui/font texts match their provenance hashes. Supplemental
  documents for 46 packages were retrieved from registry-recorded upstream commits;
  their bytes and commits are verified by the generator.
- `dispatch` 0.2.0 and `hexf-parse` 0.2.1 lack standalone upstream license texts at
  those commits. Neither appears in current all-feature Windows x64/Linux x64
  dependency trees. Broader target/feature/distribution review remains open.
- Hack/Bitstream and Chromium terms and objc2's Apple SDK licensing commentary are
  documented in THIRD_PARTY.md. Apple targets are outside the support scope.
- A stale inventory, reversed freshness comparison and wrong upstream digest were
  each rejected during verification; original files were restored.

This review does not inspect every line of every dependency or assemble a complete
target-specific binary notice bundle. See [THIRD_PARTY.md](../THIRD_PARTY.md).

## Clean checkout and tool compatibility

A fresh independent Git clone passed all 32 tests on Windows with Rust 1.92.0,
including nine real-media integration tests. The Windows Rust 1.92 release build
and default-toolchain strict Clippy/formatting also pass. Linux WSL Ubuntu 22.04
passed all 32 tests on Rust
1.92.0 using FFmpeg/FFprobe `n8.1.3-17-gc389840894-20261010`.
The Linux minimum-Rust release build, formatting and strict all-target/all-feature
Clippy passed too. The checked-out license inventory is identical across platforms.

The Linux archive was downloaded from the BtbN provider linked by FFmpeg, checked
against its published checksum manifest, and extracted only after path/symlink
inspection. Archive SHA-256:
`e4fba844d3451bd9b386dd007180a3db0ac343909d7796b0600eb4845b442aab`.
These development tools are ignored locally and are not included in the repository.
The source-package file list was inspected as well: no `.tools`, build targets,
personal media, environment secrets or signing bundles are included. Cargo reports
missing homepage/repository metadata, which remains pending the real GitHub URL;
no placeholder URL was invented. crates.io publication remains disabled.

The initial Linux run failed seven integration tests with Ubuntu's FFmpeg/FFprobe
4.4.2: required timing options and rotation schema were absent. That exposed a
startup false-positive, now fixed by bounded capability checks. The updated shared
readiness diagnostic rejects both old tools and accepts the modern package.
An isolated reversed schema guard fails its regression on Rust 1.92.0.

WSL verification is not native GUI/accessibility validation or proof of GitHub CI
success. Clean-account Windows installation and the manual checklist remain open.

## CI and remaining public-repository steps

Actionlint 1.7.12 passes the workflow. CI defines stable quality gates, Rust 1.92
Windows/Linux checks, dependency advisories, license/source and inventory checks,
and full-history secret scanning. Actions use immutable commits, checkout does
not persist credentials, and each job has a timeout. Workflow configuration and
local checks do not establish hosted-runner results.

Before publication, the owner still needs to confirm ownership/public identity,
choose the GitHub URL, enable and verify private reporting, configure repository
security/branch rules, and inspect actual hosted CI results. Follow
[the readiness checklist](open-source-readiness.md). Release signing, installers,
native appearance and the production roadmap remain separate gates.

## Reproduce the main audit checks

```sh
gitleaks git --redact --no-banner --log-opts="--all --full-history" .
cargo deny --locked --all-features check licenses sources
cargo audit --deny warnings
python scripts/dependency_licenses.py --check
actionlint
cargo +1.92.0 test --locked --all-features
cargo +1.92.0 build --locked --release
```

Tool versions above identify the recorded run; CI pins scanner/linter versions.
For a source build, use the native prerequisites in CI and current media tools
as documented in [media-tools.md](media-tools.md). Synthetic fixtures are used;
do not publish private logs, test recordings or signing bundles.
