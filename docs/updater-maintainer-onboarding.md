# Maintainer onboarding for authenticated updates

This is a staged checklist for the single maintainer. It does not enable an updater
or ask you to create keys now. The current beta cannot gain updater code through
a server change; the first updater-capable version will need manual installation.
See the [architecture](auto-update-architecture.md),
[operations](auto-update-operations.md) and
[measured feasibility gates](../spikes/tuf-updates/RESULTS.md).

## Cost boundary

The repository is public. Current updater experiments use local tools and standard
`ubuntu-latest` / `windows-latest` hosted runners. GitHub currently provides standard
runner use free for public repositories:
[GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions).
The spike uploads no artifacts and adds no caches, external service, domain, cloud
account, paid certificate or hosted runner. Compiling an AWS SDK dependency does
not provision or use an AWS account.

Keep this boundary during implementation. Paid/larger runners, paid cloud services,
signing certificates and changes to billing settings require a separate explicit
owner decision. Existing account-wide charges and coding-assistant plan usage are
outside the repository's control. Automatic PR checks now build/test packages without uploading previews. Main,
release and explicitly dispatched previews retain their documented artifact uploads. Do not treat public runner minutes as a
promise that every GitHub product or storage operation is free.

For a billing-side safeguard you can apply now:

1. Open [personal billing](https://github.com/settings/billing), then **Budgets and alerts**.
2. Create or edit an **Actions** product budget. Choose this repository's scope if
   available; an account-wide scope also affects your other repositories.
3. Set the paid budget to **$0** and enable **Stop usage when budget limit is reached**.
   Save it and inspect the included-usage alerts. An alert-only budget does not stop
   charges; do not increase a spending limit to get a workflow through.
4. Check current usage and any previously accrued storage. A new limit does not
   remove charges that have already accrued.

Follow the current [GitHub budget instructions](https://docs.github.com/en/billing/how-tos/set-up-budgets)
if the UI differs. The current CLI token lacks billing access; no broader token is
requested. These settings remain under your control and are not changed by this PR.

## Stage 1: now ? review evidence, no updater credentials

- Changes are developed in draft PR 15, stacked on design PR 13. Neither is to be
  automatically merged. Inspect tests and findings before a later merge decision.
- No production key or repository secret is needed for the experiment. Do not send
  signing keys, passwords or recovery copies through chat or an issue.
- Engineering must finish the publisher/transport/trust-store/license gates and
  report hosted results. A green experimental job is not updater release approval.
- Continue using the published beta normally. Update tests use synthetic catalog
  and package fixtures; your private family videos are unnecessary.

## Stage 2: after reviewed tools exist ? offline custody and recovery

Engineering will provide the exact version-pinned generation, signing and recovery
commands after they are implemented and tested. Do not improvise production key
commands from the disposable fixture test.

1. Choose existing encrypted storage for the root, targets and snapshot private
   keys, unavailable to CI and ordinary build jobs. Keep role keys distinct.
2. Prepare a separately stored encrypted recovery copy and keep its unlocking
   material separately. No new hardware or subscription is required by this design.
3. Run the reviewed local generation command during a deliberate signing session.
   Inspect the public root, role assignments, threshold and expiry before adoption.
4. Prove recovery using a test repository and a separate working directory. Record
   public key IDs and expiry dates, not private material, in the maintenance record.
5. Only after recovery succeeds, commit the reviewed public bootstrap root and
   sequential public rotation chain. Private keys never enter Git history.

The online timestamp key is a fourth, separate key. Losing offline keys can prevent
future renewal or require manual out-of-band recovery; encrypted backups must be
verified before enabling installer handoff.

## Stage 3: after publishing integration passes ? protected online renewal

The reviewed workflow must exist before configuring production credentials.

1. Create a publishing environment restricted to reviewed main-branch workflow
   runs. Keep the timestamp secret out of PR, tag-build and preview contexts.
2. Store only the timestamp private key in its scoped secret. Root, targets and
   snapshot keys remain offline. Final environment/secret names will be documented
   with the implementation; they are deliberately not invented here.
3. Confirm the publisher has only the permissions it needs, serializes renewals,
   preserves approved bytes and rejects expiry, version regression and stale main.
4. Run the reviewed manual dispatch and inspect verified public metadata before
   enabling the daily schedule. Test a failed refresh and its notification path.
5. Check that website publication and update metadata have one deployment owner.
   Confirm previews cannot access the secret or deploy production metadata.

The CLI's generic `update` command is not the daily renewal command: it rebuilds
offline-signed roles. A narrow command using the upstream timestamp-role signer is
required. Never provide offline keys to CI to make generic CLI update succeed.

## Stage 4: first beta with executable update handoff

1. Engineering supplies two distinct disposable setup versions and the exact test
   checklist. Confirm approval is required before download/install; declining keeps
   the current app working. Portable ZIP users retain manual download behavior.
2. Exercise media-job exclusion, interrupted download, changed package bytes,
   expired metadata, missing assets and failed setup. Confirm the previous version
   and user files remain available. Never execute an unverified cached candidate.
3. Verify fresh/previously initialized clients and persisted root rotation. Run an
   interactive Windows setup check; a clean Windows machine remains a later beta
   gate when available. Do not purchase a VM for this task.
4. Review actual candidate hashes, notices and remaining beta limitations. Perform
   the approved offline metadata signing session against the exact release assets.
5. Manually install the first updater-capable release, then check the staged offer
   and approved side-by-side installation. Publishing a release alone does not
   enroll it in automatic updates.

## Ongoing work even when no new app version is released

- Daily: the online job renews timestamp metadata against an unchanged approved
  snapshot. Check failed-run notifications; scheduling is not a freshness SLA.
- Monthly: inspect renewal health and offline metadata expiry, then sign a new
  approved generation when needed. Snapshot/targets lifetimes still apply when the
  app version remains current for a year. This uses maintainer time, not a paid service.
- Before root expiry: renew/rotate with the reviewed old/new-root ceremony and
  preserve every sequential root. Recheck the encrypted recovery copy.
- After inactivity: verify the GitHub schedule is still enabled; public-repository
  schedules can be disabled after 60 days without repository activity. See the
  [operations runbook](auto-update-operations.md) for recovery and expiry policy.

If any trust or freshness gate fails, executable updating stays unavailable while
video processing continues. Do not erase trust history, extend expiry in the client,
or disable signature verification as a recovery shortcut.
