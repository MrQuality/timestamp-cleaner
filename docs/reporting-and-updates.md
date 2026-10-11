# Bug reporting and application updates

Status: accepted feature requirements; not implemented in beta.2. The owner selected
reviewed GitHub reports and automatic update checks with approval before installation.
Current builds still use manual reporting and side-by-side updates.

## Reviewed bug reports

Add **Help > Report a problem** and a **Report this problem** action for failed
operations. Cancellation is not an error. Reporting must work when media tools are
missing and when the application is offline.

Build a versioned, bounded report from an explicit allowlist: application version,
build/source identifier when available, OS/architecture, operation identifier, error
category and detected tool versions. Never collect videos, previews, filenames,
paths, command lines, media metadata, environment variables, account identifiers
or credentials. Arbitrary subprocess stderr is not safe diagnostic data: categorise
it locally rather than automatically including its text. Do not claim regex-based
redaction can make arbitrary logs safe.

Show the exact report in an editable preview. The user adds reproduction steps,
expected/observed behavior and optional synthetic examples. Provide Copy and Save
for offline use; warn that GitHub issues are public. Opening GitHub is a separate
explicit action, and the user submits there using their own account. Do not ship a
GitHub token, create issues silently, or upload attachments automatically.

Use the fixed repository's existing bug-report template. Keep browser URLs bounded;
large reports use Copy/Save plus an ordinary template link. Anything placed in a URL
can enter browser history, so the displayed preview must explain that handoff. Keep
a separate link to private security reporting for suspected vulnerabilities.

Start with recoverable operation/worker failures. An in-process dialog cannot handle
every crash: crash-on-next-start reporting requires a separate design for a minimal
local marker, consent, retention and startup cleanup. Memory dumps and raw panic
payloads are excluded from this first feature.

Acceptance tests must cover absent tools, cancellation, worker failure, offline
Copy/Save, bounded reports, correct URL encoding and oversized-report fallback.
Seed errors with paths, names, location metadata and tokens and assert they never
enter the generated report. Manually verify that opening the browser does not submit
an issue and that the preview exactly matches the handoff.

## Automatic checks; approved installation

After startup, check for updates on a separate bounded worker without delaying media
dependency detection or blocking the UI. Explain the GitHub network request and offer
a persistent opt-out, **Check for updates**, and a dismissed-version preference.
Automatic checks run at most once per day across launches; manual checks can retry.
Persist settings atomically in user configuration, outside versioned install folders.
No background service or scheduled task is needed.

Query published releases from the fixed official repository. Compare parsed semantic
versions, not strings or publication dates. Stable installations accept stable releases;
beta installations accept newer betas and stable releases. Offer explicit beta-channel
selection and never downgrade automatically. Drafts, deleted/incomplete packages,
unsupported architectures and unknown version schemes are ineligible. GitHub's
`/releases/latest` endpoint alone cannot discover this project's beta releases.

Show installed/available versions, release notes, package type and download size.
The owner approved automatic checking only: obtain approval before downloading or
installing. Allow **Later**, skip this version, and cancellation. Never launch an
installer during preview/export/tool installation or discard unsaved work. Offline,
TLS, timeout, malformed-data and API-rate-limit failures leave video work usable.

Installed Windows builds should offer the setup EXE; portable/source builds should
offer the ZIP/release page and never silently switch installation mode. Define a
reliable install-mode marker before enabling installer launch; do not infer it solely
from the executable's location. Preserve the existing per-user, side-by-side setup
and manual removal of older versions. Installation completion is not proven merely
by launching setup, and installer failure must leave the current version usable.

Before execution, stream downloads into an exclusively created private temporary
location with size/time limits, verify the expected version/architecture and bytes,
and reject mismatches. Redirects require explicit HTTPS host policy for GitHub asset
delivery; never trust arbitrary metadata URLs or execute through a command shell.
Require a signed update manifest binding version, channel, asset, size and SHA-256,
verified against a public key embedded in the app. HTTPS and a checksum downloaded
beside the EXE alone do not provide independent update authenticity. Manifest signing
can use free tooling and is separate from paid Windows Authenticode certificates.

Release CI must produce and verify the signed manifest for both ZIP and EXE before
publication. Define private-key storage, protected signing permissions, rotation and
recovery; commit only public verification keys. No direct installer execution is
enabled until this trust path and its negative tests pass. Until then an update
notification may open the official release page for manual download.

Acceptance tests must cover beta.9 versus beta.10, stable promotion, channel changes,
draft exclusion, same/older versions, missing assets, malformed metadata, rate limits,
timeouts, opt-out persistence, concurrent checks and export-in-progress deferral.
Download tests require invalid signatures, wrong keys, tampered/truncated/oversized
files, hostile redirects, cancellation and interrupted-install recovery. Disposable
Windows tests must exercise installed and portable modes and cross-version handoff;
interactive and clean-host checks remain separate manual gates.

## Delivery order

1. Add reviewed bug reports with allowlisted diagnostics and tests.
2. Add persistent preferences, release selection and non-blocking update notifications.
3. Add signed release manifests and verification to CI and the client.
4. Add consent-gated installer handoff after download/security tests pass.
5. Record exact-package desktop, upgrade and clean-host results before claiming support.

Use small contextual commits and protected PRs for each increment. Update security,
FAQ, release and privacy documentation alongside behavior changes. These requirements
do not change beta.2 binaries or establish successful runtime verification.

References: [GitHub issue URL parameters](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue),
[GitHub release API](https://docs.github.com/en/rest/releases/releases),
[current security model](../SECURITY.md), [release process](releases.md).
