# Outstanding manual verification

Scenarios below are **pending** unless partial evidence is noted. Unit/headless tests exercise the control flow;
they do not establish native interaction or successful installation on a clean host.
Use disposable Windows accounts or VMs rather than removing tools from a working
machine. Keep synthetic media for export checks.

| Scenario | Required observation | Status |
| --- | --- | --- |
| Neither tool installed; accept | App prompts, remains responsive, installs for the current user, checks both tools, then enables media operations | Pending |
| Only FFmpeg available | Missing FFprobe is identified; installing the package makes both usable | Pending |
| Only FFprobe available | Missing FFmpeg is identified; installing the package makes both usable | Pending |
| Decline, Escape or dismiss prompt | No installer starts; app stays open with media operations disabled | Pending |
| WinGet unavailable | Installation fails with actionable instructions; manual setup and Check again recover | Pending |
| Offline or blocked download | Failure is visible; app stays disabled; retry works after connectivity returns | Pending |
| Invalid explicit tool override | Error identifies the override; correcting/removing it and restarting recovers | Pending |
| Cancel or close during installation | App closes/cancels without hanging; inspect package-manager state before retrying; no rollback promise | Pending |
| Manual archive setup | Both version checks pass; explicit paths or refreshed PATH enable the app | Pending |
| MKV and MP4 native save dialogs | Selected extension matches output choice; existing outputs are protected; exported video and audio play | Partial: owner reports both MKV and MP4 work; detailed observations remain pending |
| Keyboard, high-DPI and accessibility | Prompt/buttons remain reachable and readable; focus, dismissal and scaling work | Pending |

For each result, record date, app commit, Windows version/architecture, WinGet version,
FFmpeg/FFprobe versions, initial dependency state, steps, observed result and any
redacted error in VALIDATION.md. Record failures as failures; do not check a roadmap
gate until its required scenarios pass. The earlier user-reported successful video
and audio export remains valid evidence for that smoke test only.

Follow [manual recovery instructions](media-tools.md) when automatic setup fails.
Publishing a development source repository does not imply these release gates pass.

## Native-dialog upgrade: PR #2

The rfd 0.17.2 update changes Linux portal internals and the Windows native bindings.
The application still uses the same `FileDialog` open/save methods. Automated layout
and media tests do not operate the native picker. The owner reported successful MKV
and MP4 manual smoke tests after the PR repair; see [the scoped evidence](../VALIDATION.md#user-reported-mkv-and-mp4-smoke-tests).
The detailed checks below remain **pending** on
both Windows and a Linux desktop with its portal backend (and separately with zenity
fallback if that configuration is supported):

1. Open a synthetic video whose path contains spaces and non-ASCII characters;
   confirm the selected file loads and previews correctly.
2. Cancel the open dialog and then the save dialog; confirm the current video,
   selection and filesystem remain unchanged and no media job begins.
3. Export MKV and MP4 through their save dialogs. Confirm filename suggestions,
   filters and output extensions match the selection; verify an existing destination
   is protected and exported video/audio play correctly.

Record the tested commit, OS, desktop/portal or fallback backend, filenames used and
observed results in VALIDATION.md. This smoke test can supplement CI; it does not
replace the broader clean-installation, accessibility or performance release checks.
