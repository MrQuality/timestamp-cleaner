# Windows setup EXE

The free NSIS build produces `timestamp-cleaner-<version>-windows-x64-setup.exe`.
It is a conventional Windows setup EXE, not an MSI. The portable ZIP remains available.
Download official builds from [Releases](https://github.com/MrQuality/timestamp-cleaner/releases).
Beta.1 contains only a ZIP; installer availability starts with the next release.

## Install and launch

1. Compare `Get-FileHash .\timestamp-cleaner-<version>-windows-x64-setup.exe -Algorithm SHA256`
   with `SHA256SUMS.txt` from the same release. A checksum detects changed bytes;
   it does not authenticate the publisher. Both application and installer are unsigned.
2. Close any setup already running for this version and run the downloaded EXE.
   Read the license and complete setup. No administrator privileges are requested.
   Do not disable Windows security protections if a download is blocked.
3. Find **Timestamp Cleaner <version>** in Start. Setup does not launch the app automatically.
4. Install Microsoft's [Visual C++ v14 Redistributable (x64)](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist/)
   if the app reports a missing or incompatible runtime. It remains external.
5. Let the app check FFmpeg and FFprobe. It offers WinGet installation with your consent;
   [manual setup and recovery](https://github.com/MrQuality/timestamp-cleaner/blob/main/docs/media-tools.md)
   is available if that fails. Setup itself downloads or installs no prerequisites.

Setup uses `%LOCALAPPDATA%\Programs\TimestampCleaner\<version>` and the current user's
Start menu and Apps settings. The destination is fixed; `/D` is intentionally ignored.
Application files, notices and build metadata match the verified ZIP. Additional
`installer/` files document setup, NSIS redistribution terms and the source ZIP hash.
The installed `file-manifest.json` covers the portable payload, not these extra files
or the generated uninstaller. No personal videos are bundled.

## Update, reinstall and remove

Install a new version alongside the old one. Each has its own shortcut and uninstall entry.
There is no automatic updater or automatic removal of older versions. After checking
the new version, close the old app and remove it through **Settings > Apps** or its
`uninstall.exe`. Uninstalling an older version does not remove a newer version's entry.

Reinstalling the same version into an existing folder is refused. Uninstall it first;
move any remaining personal files out of that version's folder before reinstalling.
Keep your own videos and exports outside the installation directory.

Uninstall deletes only known packaged files, its own shortcut, registry entry and
uninstaller. It removes empty directories without recursive deletion. Files you added
are preserved. Independently installed FFmpeg/FFprobe and the Microsoft runtime are
left installed. Do not move the installation folder or uninstaller; location checks
intentionally reject relocated uninstallers.

Close the application before uninstalling. If files are locked, close the programs
using them and retry. A failed installation can leave a partial version folder. Use
its uninstaller if present; otherwise remove only that version's folder after checking
it contains no personal files. Setup is not transactional and does not promise rollback.

## Build and verification

Install [NSIS 3.12](https://nsis.sourceforge.io/Download), build/package the ZIP as
described in [releases.md](releases.md), then run:

```powershell
python scripts/package_installer.py dist/timestamp-cleaner-<version>-windows-x64.zip --compiler 'C:\Program Files (x86)\NSIS\makensis.exe'
```

The builder validates ZIP membership, Windows paths, hashes, version and architecture,
and adds the setup EXE hash to the existing `SHA256SUMS.txt`. It refuses an existing EXE.
NSIS compilation uses `/NOCONFIG` and `/WX`; build configuration cannot silently inject
local compiler configuration. CI pins the NSIS package and records its version.

On a disposable Windows runner, `scripts/test_installer.ps1` verifies silent installation,
payload hashes, registry/shortcut creation, same-version collision protection,
uninstallation and preservation of added files. `/S` is case sensitive; the uninstaller
is copied to a temporary location by NSIS, so the test waits for its completion evidence.
This does not establish interactive setup, clean-host runtime, high-DPI, accessibility,
or security reputation behavior. Those require the [manual checklist](manual-verification.md).
