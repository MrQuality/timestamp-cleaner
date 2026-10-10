# Public repository configuration

Configured and read back through the GitHub API on 2026-10-10 for
[MrQuality/timestamp-cleaner](https://github.com/MrQuality/timestamp-cleaner).
Settings can drift; this document records the applied policy, not automatic enforcement
of every future change to GitHub configuration.

## Collaboration

The repository is public, the default branch is `main`, and the license is Apache-2.0.
Issues, Discussions and Projects are enabled. The wiki is disabled so documentation
remains versioned and reviewed in this repository. Description and topics identify the
Rust desktop application, personal/family-video purpose and local processing model.

Bug and feature templates, a pull-request template and issue-routing links are tracked.
Support, contributor, conduct and governance documents explain participation and privacy.
Labels include `good first issue`, `help wanted`, `documentation`, `accessibility`,
`testing`, `performance`, `dependencies` and Windows/Linux platform labels. A label does
not mean a task has been validated as suitable for a new contributor.

Merge commits are enabled; squash and rebase merges are disabled to preserve the small
commits and contextual message bodies requested by the maintainer. Auto-merge and branch
updates are enabled, and merged topic branches are deleted automatically. There are no
downloadable releases or published installers yet.

## Main branch protection

Protection applies to administrators as well as contributors:

- Changes require a pull request with **zero required approving reviews**. This allows
  the sole maintainer to merge their own PR after checks pass; independent review is welcome.
- Review conversations must be resolved, and the branch must be up to date with `main`.
- Force pushes and branch deletion are blocked.
- Seven checks are required and bound to the GitHub Actions application (`15368`):
  `checks (ubuntu-latest)`, `checks (windows-latest)`, `minimum-rust (ubuntu-latest)`,
  `minimum-rust (windows-latest)`, `dependency-audit`, `license-and-source-policy`,
  and `secret-scan`.

When renaming workflow jobs, update the required check names as part of the same planned
change; stale names can block merges. Do not bypass protection to land routine work.
The initial source upload established `main` before protection could be applied.

## Security and automation

Private vulnerability reporting, Dependabot alerts/security updates, secret scanning and
secret-scanning push protection are enabled. Weekly Cargo and GitHub Actions update
configuration is tracked in `.github/dependabot.yml`. The dependency graph is available
for this public repository. These controls complement the project's full-history Gitleaks
and dependency audit jobs; they do not establish that all code is vulnerability-free.

Actions defaults grant read access and cannot approve pull requests. Referenced actions
must be pinned to full commit SHAs. Workflows have read-only contents permission, bounded
job timeouts and no persisted checkout credentials. No repository secrets were added.
Push runs target `main`; pull requests and manual dispatch retain the full suite.
Superseded runs for the same workflow/ref are cancelled to avoid redundant builds.

Read back the settings using an authenticated maintainer account:

```sh
gh api repos/MrQuality/timestamp-cleaner/branches/main/protection
gh api repos/MrQuality/timestamp-cleaner/private-vulnerability-reporting
gh api repos/MrQuality/timestamp-cleaner/actions/permissions
gh api repos/MrQuality/timestamp-cleaner/actions/permissions/workflow
gh api repos/MrQuality/timestamp-cleaner
gh run list --repo MrQuality/timestamp-cleaner --workflow ci.yml
```

Hosted CI evidence is recorded in [publication-audit.md](publication-audit.md). Binary
distribution and manual GUI/installation gates remain in [the readiness checklist](open-source-readiness.md).
