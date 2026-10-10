# Security model

## Trust boundaries

Media files are untrusted data. FFmpeg and FFprobe are trusted native programs installed by
the user or, with explicit consent, by Microsoft WinGet. Discovery and media processing do not
make network requests. The application never executes a command shell. Tool discovery resolves
absolute paths, excludes the current directory and validates both tools using version commands.
Bounded capability checks exercise one synthetic video frame to a null sink and verify
FFprobe's rotation schema before readiness. The self-test reads no user media and writes no file.
Explicit tool-path environment variables and the system package-manager configuration are trusted.

When dependencies are unavailable, the user can accept installation or continue with media
operations disabled. Acceptance invokes only the exact `Gyan.FFmpeg` package from the `winget`
source with user scope; no administrator installation, custom downloader, hash bypass, malware
scan bypass or unattended startup installation is requested. WinGet performs the download and
checksum validation. The prompt discloses acceptance of source and package terms. Installation
has a 20-minute timeout, bounded diagnostics and post-install executable checks. Failures never
enable media processing. Cancellation does not promise rollback of package-manager changes;
WinGet may have created files or completed installation before it is stopped.

Every subprocess uses structured arguments. Media paths are canonical local regular files;
they are never interpolated into filter graphs or shell commands. Filter parameters are validated
integers. Demuxer and protocol allowlists reject playlists and network inputs. Both stdout and
stderr are drained concurrently with bounded memory; probes and previews have 30-second timeouts.
Exports have a 24-hour limit. Each app instance runs one job at a time, with capped encoding
threads and bounded progress delivery.

Source files are opened as inputs only. Exports use an exclusively created temporary file in
the destination directory, validate the result, flush it, and publish it without replacing an
existing destination. Cancellation and failures remove the temporary file. Abrupt process or
machine termination can leave a hidden `.timestamp-cleaner-*.partial` file; never treat it as
a completed export.

## Limits

Protocol allowlisting is not an OS sandbox. A vulnerability in an installed native codec can
still execute with the user's privileges. Use patched, trusted FFmpeg builds and do not run
this application elevated. Native codec memory and CPU use are not restricted by OS job objects
in this milestone. A hostile local process that changes files or replaces trusted tools during
processing is outside the current threat model. Metadata is re-probed before export, but source
content is not hashed or locked against concurrent modification.

Output metadata can contain source identifying information such as creation dates and location.
Metadata stripping is planned before a public release. Application errors may contain local paths;
there is no telemetry, remote crash reporting, or persistent application log in this milestone.

## Dependency and release policy

Commit Cargo.lock. Build and test with `--locked`. CI uses immutable action revisions and
audits dependencies for known advisories. Dependency changes require review of the lockfile,
changelog and affected behavior. Do not suppress advisories without a documented, dated risk
assessment. Before a release, also review FFmpeg provenance, licensing, codec vulnerabilities,
installer signing and the manually tested Windows accessibility flow.

## Reporting vulnerabilities

Do not post exploit details, sensitive media or credentials in public issues. When this
repository is published on GitHub, maintainers must enable **Private vulnerability reporting**
before advertising that channel. Once enabled, use **Security > Advisories > Report a
vulnerability** on the repository. See [GitHub's private reporting instructions](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately).

No public repository URL or dedicated private contact has been configured yet. Establish
and verify a private reporting route before publication; this file alone does not enable it.
Include the affected commit/version, OS and tool versions, impact and synthetic reproduction
steps. Redact personal paths. There is no supported stable release or promised response SLA
yet; development fixes target the current main branch.
