# GitHub and Apache-2.0 readiness

This checklist separates publishing a development **source repository** from
shipping a supported **binary release**. The public repository is
[MrQuality/timestamp-cleaner](https://github.com/MrQuality/timestamp-cleaner).
Verified settings are recorded in [repository-settings.md](repository-settings.md).
Status is a record of evidence, not a certification.
`publish = false` in Cargo.toml intentionally prevents crates.io publication; it does
not prevent hosting the application source on GitHub.

## Prepared locally

- [x] Canonical Apache License 2.0 in LICENSE; Cargo package declares `Apache-2.0`.
- [x] README states development status, actual repair algorithm and feature limits.
- [x] Contribution/commit conventions and licensing expectations are documented.
- [x] Security model, dependency policy and enabled private reporting channel are explicit.
- [x] Cargo.lock and Windows/Linux CI quality gates are tracked; actions use immutable revisions.
- [x] Manual dependency recovery and outstanding verification scenarios are documented.
- [x] Application artwork and its provenance are tracked in the project.
- [x] Full-history secret scan and local privacy review; owner identity/rights decisions remain separate.
- [x] Reproducible 333-package license inventory, source/license policy and upstream notice research.
- [x] Clean-checkout Rust 1.92 testing on Windows/Linux, including real media; release and lint checks.
- [x] CI minimum-Rust, license/source, inventory, secret-scan and workflow-syntax gates.
- [x] Privacy-aware issue templates and basic contributor collaboration expectations.

See [the dated publication audit](publication-audit.md) for scope, versions, failures,
fixes and unresolved license/distribution limitations.

## Before making the source repository public

- [x] Owner explicitly requested public publication under their authenticated GitHub account,
  retaining the existing project license, artwork and Git history. This authorization
  is not an independent assessment of ownership, employer constraints or trademark clearance.
- [x] Review **all Git history**, not just the latest tree, for credentials, personal
  data, family media, machine paths and accidental proprietary material. Commit author
  emails are public metadata: choose deliberately. `.gitignore` does not remove history.
  Scan with a maintained secret-scanning tool and manually inspect flagged content;
  revoke exposed credentials before remediation. Gitleaks found no secrets at the audited
  tip; repeat before pushing new history. Existing author metadata was retained under
  the owner's public-publication instruction; no history was rewritten.
- [x] Perform dependency declaration/source review and preserve available upstream/font
  licensing records; follow THIRD_PARTY.md. Two unused-target/feature notice gaps and
  per-binary redistribution review remain explicit; this is not legal clearance.
- [x] Publish the existing artwork with its recorded provenance and licensing limits;
  no trademark clearance or exclusive rights to generated content are claimed.
- [x] Choose the GitHub owner/repository name; create the repo and add its real URL to
  Cargo metadata and README. Do not add fabricated repository, badge or support URLs.
- [x] Enable and verify GitHub private vulnerability reporting or supply a tested private
  contact. Enable dependency graph/Dependabot alerts and available secret-scanning controls.
- [x] Require pull requests, seven successful CI checks, up-to-date branches and resolved
  conversations on `main`. Require **zero approving reviews**, as requested for a sole
  maintainer. Block force pushes and deletion, including for admins. Actions use read-only
  defaults, cannot approve PRs and must reference actions by full commit SHA.
- [x] Execute the first GitHub Windows/Linux CI run and inspect actual logs. Linux passed;
  Windows exposed a launcher-related cancellation stall. The direct-binary CI fix and
  protected rerun are tracked in [PR #3](https://github.com/MrQuality/timestamp-cleaner/pull/3).
  All seven checks must pass before merging; local checks alone do not prove hosted success.
- [x] Confirm clean-checkout build/test instructions locally and document collaboration/support limits.
- [x] Set description/topics and publish support, conduct and governance guidance. Sensitive
  conduct reports use GitHub reporting controls; no dedicated conduct inbox is claimed.

Apache-2.0 application guidance is available from the
[Apache Software Foundation](https://www.apache.org/licenses/LICENSE-2.0.html).
A project NOTICE is not automatically necessary for every new Apache-licensed project;
preserve applicable upstream notices and assess what is actually incorporated.

## Before a public binary release

- [ ] Complete the clean-Windows installation and native interaction scenarios in
  [manual-verification.md](manual-verification.md), including manual setup and offline failure.
- [ ] Close the selected release's security, accessibility, correctness and performance
  gates in ROADMAP.md; do not advertise the current development milestone as production ready.
- [ ] Produce an exact dependency/font license inventory and required notice bundle for
  each target artifact. Review any bundled FFmpeg build separately before including it.
- [ ] Choose supported OS/architecture/tool versions, versioning and update policies.
  Test install, upgrade and uninstall, and define how externally installed tools are managed.
- [ ] Embed the tracked ICO in the Windows executable/installer resources and inspect
  Explorer, shortcut, taskbar and high-DPI appearance; only the runtime window icon is wired today.
- [ ] Build from a reviewed tag in CI; archive provenance, checksums, dependency inventory
  and test evidence. Review signing and artifact-attestation options before claiming them.
- [ ] Write release notes with known limitations and recovery instructions. Run clean-host
  smoke tests on the actual downloadable artifacts, not only `cargo run` output.

Useful primary references: [GitHub repository security setup](https://docs.github.com/en/code-security/getting-started/quickstart-for-securing-your-repository),
[private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository),
and [FFmpeg licensing](https://ffmpeg.org/legal.html).
