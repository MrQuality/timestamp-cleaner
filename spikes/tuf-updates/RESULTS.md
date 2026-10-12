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
| Windows release probe | 6,768,128 bytes, thin LTO, stripped, one codegen unit | Standalone size, not incremental application growth. Public-fixture startup/OS-memory measurements below; integrated app cost remains unmeasured. |
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
must run with `--ignored` and the pinned tool path. The dedicated hosted publisher job also passes in
[run 38216595550](https://github.com/MrQuality/timestamp-cleaner/actions/runs/38216595550),
alongside both verifier jobs. That run predates the package-binding increment.

## Package-binding increment

The probe now optionally streams a dummy package from a fifth CLI URL argument.
Only after TUF catalog verification does it validate the declared nonzero length
(maximum 256 MiB) and lowercase SHA-256. It counts and hashes bytes incrementally,
rejects excess bytes immediately and rejects truncated/changed bytes at completion.
No installer is persisted, published or executed. Two added ordinary tests exercise
valid/changed/short/long data plus signed zero/oversized/malformed bindings against
fresh verifier processes. Removing the hash comparison makes the tampering test
fail because altered bytes are accepted; the comparison is restored afterwards.
The prior dependency graph/lock is unchanged: AWS-LC was already used transitively
and by fixtures, and is now a direct runtime dependency of the isolated probe.

This is a byte-integrity experiment, not the application catalog schema, protected
package origin policy or transactional installer downloader. Those remain gates.
Two further root tests check progress after later expiry failure and reject a
missing/corrupt saved root. The current ordinary suite has twelve functional tests; the publisher ceremony is an
explicit separately run test. Hosted results above are historical evidence for
their stated commits, not confirmation of unpushed or later changes.

## Local release-probe measurements

`measure_probe.py` ran five trials per condition with the 6,768,128-byte Windows
baseline release binary (source 856936d, before package/root corrections),
Python 3.10.11, Windows build 26300, and the 26-byte test catalog.
The [raw measurement record](measurements/windows-2026-10-12.json) contains all
20 invocations and sampling counts. Each trial starts a new verifier process. Fresh-state and persisted-state trials
share a fixture but do not reuse an HTTP client. Values include process creation
and 1 ms requested sampling intervals (actual scheduling is coarser).

| Transport / state | Median elapsed | Min?max | Highest sampled OS peak working set |
| --- | --- | --- | --- |
| File / fresh | 587.217 ms | 549.867?1,769.207 ms | 6.80 MiB |
| File / persisted | 588.876 ms | 535.708?613.997 ms | 6.79 MiB |
| Loopback HTTP / fresh | 1,737.610 ms | 1,726.405?1,897.914 ms | 8.15 MiB |
| Loopback HTTP / persisted | 1,718.693 ms | 1,694.882?1,738.926 ms | 7.62 MiB |

All 20 invocations verified the same catalog successfully. File trials had only
1?3 memory samples, and HTTP trials 38?43: treat sampled peaks as **lower bounds**,
not a rigorous memory ceiling. These measurements exclude TLS, real CDN/proxy
latency and the GUI. They cannot explain the earlier slow debug cancellation or
predict a reused update worker. The baseline is a standalone probe; incremental
application binary growth and worker idle/peak memory still require integration
measurements after dependency approval.

## Architectural findings

1. **Reuse saved trust explicitly.** Correction to the initial report: tough
   0.24.0 writes root.json in the datastore, including verified rotations before a
   later role may fail. It does not automatically select that file as the next
   loader's root argument. The probe now supplies state/root.json and removes its
   redundant trusted-root.json copy. Reverting to the embedded root on every load
   breaks continuity after rotation; falling back when saved state is damaged can
   discard trust progress. Missing root in nonempty state fails closed. A fresh
   store seeds the caller-trusted bootstrap using upstream self-signature validation
   before network work, so a first-request timeout can be retried normally. This
   initial write is also non-atomic and remains an explicit production gate.
2. **Storage is a production blocker.** The upstream datastore uses direct,
   non-atomic writes; root updates remove the previous file before rewriting it. In-process protection does not establish
   a multi-process transaction. We must design locking, crash-safe publication and
   fail-closed corruption handling together; deleting state to recover would discard
   rollback protection. Test partial writes, disk-full and concurrent app instances.
   A naive staged copy committed only after full catalog success also loses verified
   role progress when a later role fails. Preserve the verifier's per-role commit
   semantics. The datastore is private; resolving this may require an upstream
   change rather than pretending an outer wrapper makes its writes atomic.
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

**Proceed with standard TUF; no-go for production integration using the stock
store in this probe.** Signature, renewal and root-continuity behavior are viable.
The next engineering slice must resolve upstream storage semantics, constrained
transport and artifact notices before selecting the production runtime dependency.
Do not turn this experiment into installer code while those defects remain.

The minimal storage work should preserve verified per-role progress, atomic durable
replacement and explicit missing/corrupt-state failure under a cross-process lock.
An upstream-ready patch or supported storage hook should be evaluated against the
same fixtures and crash fault injection; a whole-generation copy committed only
on success is not an equivalent solution. No upstream issue/PR is opened by this
experiment, and no unreviewed fork is added to the app.

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
