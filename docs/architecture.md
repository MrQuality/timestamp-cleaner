# Architecture

## Processing

The UI owns presentation state and a single job handle. Workers receive immutable request
snapshots and a shared cancellation token. They return typed completion messages; they do not
mutate UI state. Progress uses a bounded channel and never blocks the encoder pipe reader.
Changing source coordinates discards the repaired preview.

The core has no GUI dependency when built with `--no-default-features`. Its public entry points
can be exercised by tests and future command-line clients without launching a window.

FFmpeg owns demuxing, decoding, filtering, encoding and muxing in one streaming process.
This intentionally avoids a raw-frame pipe, which would discard original per-frame timestamps.
The video filter receives source-pixel coordinates before preview scaling. MKV copies all audio;
MP4 copies AAC/MP3 and transcodes other audio to AAC, preserving track count. Output validation
checks codecs against that typed format policy. `fps_mode=passthrough` and the demuxer time base avoid a synthetic
fixed-FPS clock. Matroska quantizes timestamps to its output time base; tests allow 2 ms of
timestamp rounding, not wholesale timing changes.

The subprocess supervisor drains both pipes concurrently, kills and waits for the child on
cancel/timeout, and retains only bounded diagnostics. FFmpeg emits progress into that supervisor.
Ordinary output capture fails on overflow; progress output is intentionally a rolling tail.

Publication occurs only after successful exit and output probing. Output dimensions, audio
codec list and duration (250 ms tolerance for muxer/codec padding) are checked. Runtime validation
does not fully decode the exported file or compare every audio packet; integration tests perform
stronger frame-timestamp and audio-payload checks on controlled fixtures.

Startup checks run on the same background-job boundary as media processing. Dependency UI state
is explicit: pending, checking, ready, unavailable or installing. Only ready exposes usable tools.
An unavailable state opens a consent prompt; declining dismisses it without starting an installer.
Accepted installation runs the exact user-scope WinGet package and re-checks both tools. Failed
installation offers recovery without automatically retrying or enabling the application. Known
WinGet link directories are searched directly so the current process does not depend on refreshed PATH.

## Performance

One selected configuration produces one streaming export. There are no six full-video passes,
unbounded frame queues, or whole-video buffers. Preview output is capped at 1280×720; memory use
does not grow with video duration. UI textures are uploaded only when a preview completes.
The encoder uses four threads, filters two, and preview encoding two. These are conservative
initial limits; benchmark representative 1080p and 4K footage before changing them.

Current before/after previews seek and decode the selected frame twice. A future frame cache can
remove that duplication after profiling; source and processing revisions must key any such cache.

## OpenCV integration decision

The original six presets are still required for algorithm parity. This host lacks a native
OpenCV/Clang build toolchain. Do not relabel FFmpeg interpolation as Telea or Navier–Stokes,
and do not add a Python subprocess as an architectural shortcut.

Before adding OpenCV, establish a reproducible Windows build and choose a timestamp-aware frame
integration boundary. A naive OpenCV VideoCapture/VideoWriter replacement would regress audio,
variable timing, rotation and color handling. Candidate: a native FFmpeg binding that carries
frame PTS/time bases into a synchronous OpenCV filter and timestamp-aware encoder/muxer. The
binding, native library versions and runtime DLL distribution need an isolated prototype and
tests before the default backend changes. Avoid maintaining two independent muxing implementations.
