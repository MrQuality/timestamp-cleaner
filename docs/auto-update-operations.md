# Update publication and key operations

Proposed operating model for [the update ADR](auto-update-architecture.md). This is
not the current release workflow. No keys, secrets, schedules, branches or environments
are created by this design. Implementation requires the feasibility gates below.

The [maintainer onboarding checklist](updater-maintainer-onboarding.md) separates
current review, later owner setup, free-service boundaries and ongoing maintenance.

## Ownership and signing

The application owns version/channel policy; TUF owns metadata verification; NSIS
owns installation; GitHub hosts bytes. No component substitutes for another's checks.
Start with four separate Ed25519 role keys and threshold 1 for each role. The single
maintainer is the trust authority; two keys stored by the same person would not create
independent human review. Threshold increases are a future governance decision.

| Role | Private key location | Initial lifetime policy | Renewal owner |
| --- | --- | --- | --- |
| Root | Offline encrypted storage, separate backup | 2 years | Maintainer; review annually and rotate before expiry |
| Targets | Offline encrypted storage, separate from CI | 180 days | Maintainer; release approval or scheduled maintenance |
| Snapshot | Offline encrypted storage | 90 days | Maintainer; each approved generation, review monthly |
| Timestamp | Restricted Actions environment secret | 7 days | Daily workflow refresh against the approved snapshot |

These lifetimes are proposed operational choices, not TUF defaults. Offline here means
unavailable to GitHub Actions and ordinary build processes; signing occurs in a
deliberate local signing session with verified tools. Keep encrypted backups and test
recovery before enabling client execution. Encrypting a key does not help if its
decryption password is stored beside it in CI. Never print secret keys in workflow logs.

Only timestamp renewal is unattended. A timestamp-key compromise cannot authorize
new package hashes; it can still disrupt freshness and availability. Root compromise
and targets-signing compromise are serious publisher incidents. An offline signer
must inspect release identity and hashes: approving malicious build output remains
possible even with authentic metadata. Code review and protected CI remain necessary.

## Release promotion

1. Existing tag CI builds ZIP/setup, verifies hashes and runs installer tests. It
   creates the ordinary draft. It has no offline signing keys or update publication
   authority. Preserve current immutability and required checks.
2. Maintainer downloads the exact candidate, checks provenance, package manifest,
   sizes, notices and recorded beta gates, then publishes the authorized release.
   Publication alone does not enroll it in automatic updates.
3. A reproducible publisher command constructs catalog v1 from those verified bytes,
   not from arbitrary assets selected by filename alone. Match tag/version/source,
   target, shared checksums and installed application hash. Carry both stable and
   beta offers as appropriate; the client still selects by semantic version.
4. In the offline signing session, verify current trusted repository state, sign new
   targets and snapshot metadata and verify the complete staged generation with the
   standard TUF tooling. Increasing metadata versions must not decrease the highest
   offered version in either channel. Emergency withdrawal can remove an offer, but
   cannot silently turn an older release into a downgrade candidate.
5. Commit only public metadata and the tiny catalog into `updates/`, with a protected
   PR and contextual commit. Keep all versioned roots and content-addressed catalogs;
   never overwrite immutable names. No binary packages or private review files enter
   this tree. New root generations follow TUF's old/new-threshold signing procedure.
6. After merge, the unified Pages publisher verifies the approved generation, produces
   a fresh timestamp with the online key and deploys the website plus update metadata
   together. Only fully published releases with exact matching assets are eligible.
7. A fresh client and a previously initialized client verify the live catalog. Verify
   both package downloads and record version/hash/expiry evidence. A partial deployment
   must fail verification; it must never cause installation from mixed generations.

This retains one deliberate release approval and adds one offline metadata signing
session. No paid signing service is necessary. Generate keys with the maintainer in
control during implementation; do not fabricate production keys in a development PR.
The first updater-capable release embeds the reviewed public root and installation
marker. Beta.2 users must install it manually; existing published binaries cannot
acquire an updater through a server-side change.

## Daily refresh and one deployment owner

Extend the existing Pages workflow rather than introducing a competing Pages deployer.
Main-source changes, published release events, manual dispatch and a daily schedule
all build the same combined website/update tree. Always deploy reviewed main content;
PR previews never receive the timestamp secret and never deploy. A dedicated
main-only publishing environment scopes that secret; configure this explicitly rather
than reusing the existing environment's broader tag permissions without review.

Serialize deployments without canceling a signing/publish operation. Timestamp
generation uses the latest approved snapshot unchanged; it cannot renew expired
snapshot/targets metadata or introduce new hashes. Allocate a strictly increasing
timestamp version under this serialized publisher (UTC seconds checked against the
currently served version is a candidate). Reject clock regression, reuse or a lower
snapshot version than the live verified generation. This version allocator and
cross-run ordering require an integration test before selecting the final mechanism.

Build from the latest main generation and check it again before deploying; if superseded,
retry with the newer generation rather than publish stale metadata. Pages deployments
may be atomically switched, but CDN caches are not an atomic consistency guarantee.
Use consistent snapshots and preserve immutable references; client hash/version checks
must handle temporarily stale timestamp responses safely. Never regenerate previously
signed JSON with different byte serialization during website build.

GitHub can delay or disable scheduled workflows. Alert the maintainer through failed
workflow status and documented expiry checks; a scheduled job is not a freshness SLA.
Public-repository schedules can be disabled after 60 days without repository activity;
document manual re-enablement and check renewal health during the monthly review.
Warn at 30 days remaining on offline metadata, fail publication at expiry, and perform
the monthly signing review even when no app release is planned. If maintenance stops,
automatic updates stop; the video application continues to work. Do not extend expiry
in the client or sign new offline metadata using the online timestamp key.

## Recovery

| Event | Required action |
| --- | --- |
| Missing asset, bad catalog or failed setup | Withdraw/reissue through a newer approved catalog if necessary; retain current app and existing versions. No destructive automatic cleanup. |
| Expired metadata or failed renewal | Show updates unavailable; renew with the authorized roles. Do not execute a cached candidate or disable verification. |
| Incorrect local clock | Explain the clock issue and offer manual retry; never adopt an unauthenticated server time as authority. |
| Timestamp key compromised | Offline root rotates that role key; publish the sequential root chain and fresh metadata. Investigate replay/availability impact. |
| Targets/snapshot key compromised | Treat as an incident; offline root replaces affected keys and issues new metadata. Targets compromise can authorize code, so do not promise automatic remediation of affected hosts. |
| Root threshold compromised or trusted root lost | Out-of-band recovery through a newly trusted manual release; do not promise authenticated in-band recovery. |
| Local trust store damaged | Preserve evidence, disable executable update handoff and require explicit manual recovery; resetting preferences does not reset trust. |

Ordinary key rotation preserves the complete root chain. Reinstalling an application
or clearing its data can reset local rollback history; document that boundary. A
previously installed app version remains a manual fallback, not permission for the
updater to downgrade or to migrate future project files backwards.

## Implementation slices and release gates

The owner accepted the direction on 2026-10-12. The next deliverable is a disposable
TUF proof of concept and a dated go/no-go report, before application integration.
Use generated test-only keys and a local test repository; no production secrets or
real installer execution are needed. The report must record the pinned dependency
versions/lock, Windows and Rust 1.92 results, dependency audit/license findings,
binary-size/runtime-memory measurements and any unresolved integration limits.
Demonstrate successful catalog verification across process restarts and root rotation,
then rejection of expired metadata, rollback, tampered catalog/package bytes and
interrupted transfers. Explicitly test metadata renewal for the same application
version: a release remaining current for a year must not require an app rebuild.
Expired update metadata must disable update handoff without disabling video work.
If the spike passes, proceed to the policy/notification slice; if it fails, record
the evidence and revisit the architecture rather than weaken verification.

1. Compatibility spike: pin candidate `tough`/`tuftool`; validate Rust 1.92, platform
   support, licenses/audit, transport injection, cancellation, persistent root handling
   and binary/runtime cost. Stop for design review if this fails; do not quietly
   substitute a custom verifier. Include real signed fixtures and failure cases.
2. Pure policy and notification UI: typed versions/channel/state, per-user preferences,
   bounded independent worker and official-release-page fallback. No installer launch.
3. Test repository and publisher: public test keys only, expiry/rotation drills,
   consistent snapshots, unified Pages tree and publication ordering tests. Provision
   production keys/environment separately under maintainer control.
4. Authenticated client catalog and streamed download: real verifier, persisted state,
   clock/rollback tests, host policy, exact-size/hash checks and cancellation cleanup.
5. NSIS marker and installed-mode detection: two-version integration fixture, current
   app preserved, successful/failed/canceled setup and exact post-install identity.
6. Consent/idle handoff and manual package verification: installer cannot start during
   media work; interactive upgrade, portable fallback, accessibility and clean-host
   behavior must be documented for the exact release.

Each slice gets a small contextual commit/PR, appropriate tests and updated evidence.
No automatic merge is part of this design. Documentation review is not proof that
the dependency spike, key recovery, signing pipeline or Windows upgrade works.

References: [TUF specification](https://theupdateframework.github.io/specification/latest/),
[tough tooling](https://github.com/awslabs/tough),
[GitHub scheduled events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows),
[current release workflow](../.github/workflows/release.yml),
[current Pages workflow](../.github/workflows/pages.yml).
