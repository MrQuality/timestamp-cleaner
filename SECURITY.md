# Security model

## Trust boundaries

Media files are untrusted data. FFmpeg and FFprobe are trusted native programs installed by
the user. This application does not download executables, execute a command shell, or make
network requests. Tool discovery resolves absolute paths and excludes the current directory.
Explicit tool-path environment variables are trusted configuration.

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

Report issues privately to the repository owner until a dedicated reporting address is established.
