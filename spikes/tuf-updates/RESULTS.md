# TUF feasibility results and integration gates

Status: experimental, incomplete; measured on 2026-10-12. This is evidence for the
[accepted design](../../docs/auto-update-architecture.md), not approval to ship an
updater. No application dependency, released executable, production signing key or
update publication job is changed by this workspace.

## Evidence

| Check | Observed result | Limit |
| --- | --- | --- |
| Windows x64, Rust 1.92.0 | Release build; eight focused integration tests pass | Standard full-suite command still fails to launch the empty binary harness with OS access denied (error 5); all eight functional integration tests run. Hosted full suite also passes; local empty-harness denial remains unexplained. |
| Ubuntu 22.04 under WSL, Rust 1.92.0 | Eight-test full suite passes | WSL is not an independent hosted Linux runner. |
| Formatting / strict Clippy | Pass locally on 1.93.0 and hosted on 1.92.0 | No application graph change. |
| Hosted Windows/Linux | [Run 38215416403](https://github.com/MrQuality/timestamp-cleaner/actions/runs/38215416403): both format, strict Clippy, full test and release build jobs pass | Eight functional tests on each runner; Windows stalled request about 5.01 seconds in this run. |
| Audit | cargo-audit 0.22.2: no warnings/advisories in the 190-package lock graph | Point-in-time result; this graph includes unused platform dependencies. |
| Source policy | Pass under the existing project deny policy | License approval remains separate. |
| License policy | Rejects aws-lc-sys 0.45.0 and subtle 2.6.1 (BSD-3-Clause), webpki-root-certs 1.0.9 (CDLA-Permissive-2.0) | Preserve policy; review exact artifact contents and notices before adding app dependencies. |
| Windows release probe | 6,768,128 bytes, thin LTO, stripped, one codegen unit | Standalone size, not incremental application growth. No memory/startup baseline measured yet. |
| Publisher CLI | tuftool 0.17.0 locked/no-default-features fails on 1.92/1.93; builds and reports its version on isolated 1.94.1 | AWS SDK dependencies still require 1.94.1. Windows CLI renewal/cross-sign ceremony passes separately; initial CLI create, production publication and deployment races are untested. |

The tests use fresh temporary four-role Ed25519 keys and upstream signing code.
The process-restart test verifies the same app version after metadata renewal;
rollback and expiry fail without exposing a catalog. Modified signatures and
modified/truncated target bytes fail. Root rotation is cross-signed by old/new
root keys; the next process succeeds after the server rotation file is removed.
A fresh client using the obsolete root fails against that same repository.

Two additional probes exercise actual local HTTP sockets. A peer sending no headers
is cancelled, and a peer closing before its declared body length fails closed.
On Windows under concurrent compilation, a stalled request returned `Elapsed`
after about 12.15 seconds despite a five-second Tokio timeout. The interrupted
response also reached that outer timeout under default retries. These tests prove
failure without catalog exposure; they do **not** prove prompt cancellation or
that truncation is diagnosed before retries. The tests record latency; a strict wall-clock bound remains an unmet integration
requirement.

The hosted [TUF workflow](https://github.com/MrQuality/timestamp-cleaner/actions/workflows/tuf-spike.yml)
runs the unmodified full-suite command on both runners; it does not suppress the
locally denied empty harness. The local denial was reproduced after integration
success. Its cause is not established, and no endpoint protection or executable
permissions were changed to bypass it.

The separate Windows publisher test passes with tuftool 0.17.0: CLI renewal keeps
the application catalog unchanged, CLI old-root cross-signing enables persisted
rotation, and a timestamp key alone cannot sign a targets/snapshot generation.
The generic `tuftool update` command always rebuilds those roles; it cannot implement
the online timestamp-only job. The test demonstrates the upstream `SignedRole`
signer can renew only timestamp metadata, while snapshot/targets bytes stay identical
and the standard verifier accepts the same catalog. Production needs a narrow
publisher command around this API, plus version allocation, authorization and
freshness checks; do not upload offline keys to work around the CLI limitation.
The explicit publisher test is excluded from the ordinary eight-test suite and
must run with `--ignored` and the pinned tool path. A dedicated hosted publisher job
is added; its results are pending for this increment.

## Architectural findings

1. **Persist trust explicitly.** In tough 0.24.0, the datastore saves timestamp,
   snapshot and targets, but excludes root. The probe persists verified root
   metadata separately. Omitting this breaks subsequent trust after rotation.
2. **Storage is a production blocker.** The upstream datastore and probe root
   persistence use direct, non-atomic writes. In-process protection does not establish
   a multi-process transaction. We must design locking, crash-safe publication and
   fail-closed corruption handling together; deleting state to recover would discard
   rollback protection. Test partial writes, disk-full and concurrent app instances.
3. **Verify complete streams.** Target bytes can arrive before final hash/length
   verification. The probe buffers only the bounded catalog and parses it after
   successful stream completion. Installer downloads must go to an unpublished
   temporary file, finish verification and only then become installable.
4. **Own transport policy.** Default HTTP behavior includes retries and client
   construction. A Tokio timeout only cancels cooperative work. A production
   transport must reuse a client, constrain HTTPS origins and every redirect, define
   proxy behavior, apply byte/request limits, and support responsive cancellation
   without blocking the GUI or shutdown. Measure initialization separately.
5. **Separate client/publisher toolchains.** The verifier works on app MSRV 1.92;
   the selected CLI publisher requires a newer pinned toolchain. Do not silently
   raise the application MSRV or give the online job offline-role keys.

The upstream implementation inspected is the exact registry source for tough
0.24.0: `src/datastore.rs`, `src/lib.rs`, `src/transport.rs` and `src/http.rs`.
Public references: [tough repository](https://github.com/awslabs/tough),
[0.24.0 loader API](https://docs.rs/tough/0.24.0/tough/struct.RepositoryLoader.html),
[TUF specification](https://theupdateframework.github.io/specification/latest/).

## Review of this increment

The five-axis review covers correctness (real signatures and negative controls),
readability (small CLI and fixture helpers), architecture (standalone workspace),
security (temporary keys, no production execution/publication), and performance
(bounded catalog, measured transport delay and standalone binary size). Workflow
permissions are read-only and action references are pinned. No code is reachable
from the application. The known storage/transport/license defects block production
integration; they are explicit subjects of this experiment, not accepted app defects.

## Decision and next implementation boundary

**Proceed with evaluation; do not integrate this probe into the app.** Standard
TUF verification and durable root rotation are viable enough to continue. Storage,
transport and license gates prevent a production dependency decision today.

Next increments, in order:

- Inspect the isolated Windows/Linux CI matrix and resolve genuine portability
  issues. Measure release delta, HTTP initialization and worker peak memory with
  repeatable fixtures and a stated baseline.
- Exercise the pinned publisher CLI with test-only keys through publish, same-version
  renewal and root rotation. Verify timestamp-only publication cannot change targets.
- Review the new license expressions and per-artifact notices; choose an explicit
  dependency/publisher version policy.
- Implement and test the transactional trust-store and constrained transport as
  isolated modules, then a typed catalog and package stream verifier. Cover altered,
  truncated and oversized installers, mixed generations, redirected origins,
  cancellation, disk-full and concurrent clients before introducing GUI update actions.
- Integrate check scheduling and consent; test media-job exclusion and side-by-side
  NSIS handoff with two distinct disposable fixture releases.

Owner involvement is not needed for these test-only increments. Production onboarding
will require an offline-key custody/recovery decision and protected GitHub timestamp
signing configuration. Interactive installer and clean-Windows checks remain manual
release gates. No production keys should be sent through chat or committed.
