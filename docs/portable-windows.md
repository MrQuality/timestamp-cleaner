# Timestamp Cleaner: Windows portable preview

This is an unsigned development build for Windows x64. It is not a supported
production release. No Rust compiler or Visual Studio installation is required
to use the application. Clean-machine runtime compatibility is still pending.

1. Compare the ZIP's SHA-256 with `SHA256SUMS.txt` from the same release/run:
   `Get-FileHash .\timestamp-cleaner-<version>-windows-x64.zip -Algorithm SHA256`.
   Checksums detect changed bytes; they do not authenticate the publisher.
2. Extract the **entire ZIP** to a folder you can write to. Do not run from inside
   the ZIP. Keep the included licenses and notices with redistributed copies.
3. Run `timestamp-cleaner.exe`. The application is currently unsigned; Windows
   may show a reputation warning. Download only from the project's official
   [repository](https://github.com/MrQuality/timestamp-cleaner). Do not disable
   antivirus or other system protections to run it.
4. Allow startup checks to finish. If FFmpeg/FFprobe are missing, choose whether
   to install them through WinGet. Declining leaves media operations disabled.
   Installation changes the user environment even though the application itself
   requires no installer. See the included `docs/media-tools.md` for recovery.
5. Open a video, select its timestamp rectangle, compare the repair preview and
   export one result as MKV or MP4. Keep your original recording.

The repair uses FFmpeg rectangle interpolation. Only the first eligible video
stream is exported. Subtitle/data streams are omitted; rotated, HDR, alpha and
non-square-pixel inputs are outside the supported input scope. MKV copies audio;
MP4 copies AAC/MP3 or converts other audio to AAC. See the project's README and
roadmap for complete restrictions.

To update, close the app and extract the new version into a separate folder.
To remove it, close the app and delete its extracted folder. Independently
installed FFmpeg/FFprobe are managed separately and are not removed with it.

`build-info.json` identifies the source commit, compiler and lockfile;
`file-manifest.json` records included file hashes. `dependency-licenses.csv` and
`licenses/` preserve notices for the Windows normal/build dependency graph,
including build tooling conservatively. Automated collection does not replace
the artifact-specific licensing review required before public distribution.

Report problems through [Issues](https://github.com/MrQuality/timestamp-cleaner/issues)
with the build commit and tool versions. Share synthetic examples, not family media.
