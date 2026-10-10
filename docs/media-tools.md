# FFmpeg and FFprobe setup and recovery

Timestamp Cleaner needs **both** FFmpeg and FFprobe. They normally come in the same
FFmpeg package; installing the whole package is the recommended recovery even when
only one executable is missing. The application remains open but disables media
operations until both tools pass identity and capability checks. FFmpeg runs a
one-frame synthetic H.264/delogo/timing self-test to a null sink; FFprobe must expose
the rotation side-data schema. No family media is used or output file written.

Automatic installation is implemented for Windows through WinGet. Its clean-account
installation and offline-failure scenarios have **not yet been verified end to end**.
See [the manual verification checklist](manual-verification.md) and
[local test evidence](../VALIDATION.md). These instructions are a fallback, not a
claim that an installer has been tested on every supported configuration.

## Windows: install from a terminal

Open a normal PowerShell window under the same Windows account that runs the app.
Check Microsoft WinGet first:

```powershell
winget --version
winget show --id Gyan.FFmpeg --exact --source winget
winget install --id Gyan.FFmpeg --exact --source winget --scope user
```

Review any source/package agreements in the terminal. Do not add hash-check bypass
options. If WinGet is unavailable, install or update Microsoft's **App Installer**
using the [Microsoft setup instructions](https://learn.microsoft.com/en-us/windows/package-manager/winget/).
If the package manager reports that the package is already installed, inspect its
reported error and installed location before retrying; repeatedly clicking Install
does not repair a bad PATH or an invalid explicit override.

Open a new PowerShell window after installation, then check:

```powershell
Get-Command ffmpeg, ffprobe
ffmpeg -version
ffprobe -version
```

Choose **Check again** in the app. Known WinGet link directories are searched without
restarting; other PATH changes require restarting the app so it inherits the new PATH.
Microsoft documents installation flags and diagnostic logs in the
[WinGet install reference](https://learn.microsoft.com/en-us/windows/package-manager/winget/install).

## Windows: install without WinGet

1. Start at the [FFmpeg download page](https://ffmpeg.org/download.html), which links
   Windows build providers, including [Gyan](https://www.gyan.dev/ffmpeg/builds/).
   FFmpeg itself supplies source code; these executable packages are third-party builds.
2. Download a release archive for your machine from a linked provider. Verify the
   archive against the checksum published by that provider, and review its license.
3. Extract the package to a stable folder you control, such as
   `C:\Users\YourName\Apps\ffmpeg`. Locate its `bin` folder containing **both**
   `ffmpeg.exe` and `ffprobe.exe`. Do not move only one executable out of the package.
4. Add that `bin` folder to your **user PATH** through Windows Environment Variables,
   then restart the app and terminal. Do not add the application working directory
   to PATH: Timestamp Cleaner deliberately excludes it from implicit discovery.
5. Alternatively, launch the app with explicit absolute tool paths as shown below.

Replace the example paths with actual extracted paths. These variables affect only
this PowerShell session and the application started from it:

```powershell
$env:TIMESTAMP_CLEANER_FFMPEG = 'C:\Users\YourName\Apps\ffmpeg\bin\ffmpeg.exe'
$env:TIMESTAMP_CLEANER_FFPROBE = 'C:\Users\YourName\Apps\ffmpeg\bin\ffprobe.exe'
& $env:TIMESTAMP_CLEANER_FFMPEG -version
& $env:TIMESTAMP_CLEANER_FFPROBE -version
& '.\target\release\timestamp-cleaner.exe'
```

For a downloaded application, replace the last command with its executable path.
Invalid overrides take precedence over discovery. Correct them, or remove the
session overrides with `Remove-Item Env:TIMESTAMP_CLEANER_FFMPEG` and
`Remove-Item Env:TIMESTAMP_CLEANER_FFPROBE`. Overrides saved in Windows user/system
environment settings must also be corrected there, followed by an app restart.

## Linux

Use your distribution's package manager. For Debian/Ubuntu:

```sh
sudo apt-get update
sudo apt-get install ffmpeg
ffmpeg -version
ffprobe -version
```

Other distributions use different package names and repositories; follow the
[distribution links on FFmpeg's download page](https://ffmpeg.org/download.html).
Check the installed version and capabilities: Ubuntu 22.04's FFmpeg 4.4 package
is too old for this pipeline, even though both executable names exist. Use a
distribution with a current package, or a current Linux build linked from FFmpeg's
download page. Verify the provider's checksum, extract the complete package into
a stable user folder, and launch with explicit paths if you cannot update PATH:

```sh
TIMESTAMP_CLEANER_FFMPEG=/absolute/path/to/ffmpeg/bin/ffmpeg \
TIMESTAMP_CLEANER_FFPROBE=/absolute/path/to/ffmpeg/bin/ffprobe \
./target/release/timestamp-cleaner
```

Automatic Linux installation is not implemented. Restart the app if PATH changed,
or choose **Check again** after installing into an existing PATH location.

## Both tools exist but processing still fails

If you installed through Chocolatey, use the actual binaries in its FFmpeg package
directory rather than `ChocolateyInstall/bin/ffmpeg.exe` and `ffprobe.exe` launcher
shims. Set the explicit overrides described above to the package's actual executable
paths. The current supervisor terminates its immediate child; a launcher can leave
FFmpeg running with open pipes and block cancellation or timeout cleanup. CI resolves
the package binaries directly. Process-tree supervision remains a production gate.

Startup checks establish executable identity, the basic repair/timing pipeline and
FFprobe's required side-data schema. They do not prove compatibility with every input
codec. This application requires `libx264`, `delogo`, `lavfi`, PNG encoding and the
`-fps_mode passthrough` / `-enc_time_base demux` timing options; MP4 audio conversion
also requires AAC encoding. Inspect the build:

```powershell
ffmpeg -hide_banner -encoders
ffmpeg -hide_banner -filters
```

Look for `libx264`, `png`, `aac` in the encoders and `delogo` in the filters. Use a
trusted build containing those components if any are absent. For support, report
the app revision, OS, tool versions, which component failed and the exact error.
Redact personal paths; do not upload family footage or sensitive package-manager logs.

If an older tool earlier in PATH hides a newly installed package, correct the user
PATH or use explicit paths to the current package. Discovery preserves PATH/override
precedence; accepting installation does not silently rewrite your configuration.

From a source checkout, diagnose the exact desktop readiness checks without opening
a window. Exit status is zero only when both tools are usable:

```sh
cargo run --locked --no-default-features --example check_dependencies
```
