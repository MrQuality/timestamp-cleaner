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

Configure the actual FFmpeg/FFprobe binaries, not executable launchers that spawn another
process. In particular, Chocolatey's shim executables can leave a descendant holding pipe
handles after the launcher is terminated, blocking cancellation/timeout cleanup. Hosted CI
uses the actual package binaries through explicit tool-path overrides. Process-tree supervision
is a remaining production gate; the current supervisor terminates its immediate child only.

Output metadata can contain source identifying information such as creation dates and location.
Metadata stripping remains production-readiness work; current betas can retain this information. Application errors may contain local paths;
there is no telemetry, remote crash reporting, or persistent application log in this milestone.

## Dependency and release policy

Commit Cargo.lock. Build and test with `--locked`. CI uses immutable action revisions and
audits dependencies for known advisories. Dependency changes require review of the lockfile,
changelog and affected behavior. Do not suppress advisories without a documented, dated risk
assessment. Before a release, also review FFmpeg provenance, licensing, codec vulnerabilities,
installer signing and the manually tested Windows accessibility flow.

## Windows setup boundary

The NSIS setup installs for the current user into a fixed versioned LocalAppData folder.
It requests no elevation, changes no PATH values and downloads no prerequisites.
Start shortcuts and uninstall entries are version-specific. Uninstall uses an exact
packaged-file list and empty-directory removal; it does not recursively delete user files.
Known packaged filenames are application-owned and will be removed even if edited.
Same-version reinstall is refused, and setup/uninstall serialize through a versioned mutex.
Installed files are not an OS sandbox; hostile same-user filesystem/process changes
remain outside this threat model. Setup is not transactional and a failure may leave
partial files. See [setup recovery](docs/windows-installer.md).

CI verifies the ZIP's hashes before generating setup, compiles with pinned NSIS and no
local compiler configuration, and tests installation on a disposable runner. Application
and installer remain unsigned; checksums do not authenticate the publisher. Clean-host
runtime compatibility, interactive setup and accessibility still require manual evidence.

## Reporting vulnerabilities

Do not post exploit details, sensitive media or credentials in public issues. Private
vulnerability reporting is enabled for this repository. Use
[Report a vulnerability](https://github.com/MrQuality/timestamp-cleaner/security/advisories/new)
or **Security > Advisories > Report a vulnerability**. See
[GitHub's private reporting instructions](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately).

Include the affected commit/version, OS and tool versions, impact and synthetic reproduction
steps. Redact personal paths. There is no supported stable release or promised response SLA
yet; development fixes target the current main branch.
