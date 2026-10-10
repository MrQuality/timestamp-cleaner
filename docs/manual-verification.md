# Outstanding manual verification

All scenarios below are **pending**. Unit/headless tests exercise the control flow;
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
| MKV and MP4 native save dialogs | Selected extension matches output choice; existing outputs are protected; exported video and audio play | Pending |
| Keyboard, high-DPI and accessibility | Prompt/buttons remain reachable and readable; focus, dismissal and scaling work | Pending |

For each result, record date, app commit, Windows version/architecture, WinGet version,
FFmpeg/FFprobe versions, initial dependency state, steps, observed result and any
redacted error in VALIDATION.md. Record failures as failures; do not check a roadmap
gate until its required scenarios pass. The earlier user-reported successful video
and audio export remains valid evidence for that smoke test only.

Follow [manual recovery instructions](media-tools.md) when automatic setup fails.
Publishing a development source repository does not imply these release gates pass.
