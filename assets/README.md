# Application icon

The mark combines a film frame, a mountain/sun landscape and a restoration sparkle,
connecting family travel recordings with cleaning an intrusive timestamp. There is
no text so the silhouette can work as a desktop icon.

- `icon.png`: original generated artwork, with transparent exterior.
- `icon-256.png`: 256-pixel runtime icon embedded in the native window.
- `icon.ico`: 256-pixel PNG-backed Windows icon for future shortcut/installer packaging.

The window uses the icon at runtime. The standalone executable's Windows PE icon
resource is **not yet embedded**; Explorer/shortcut packaging needs that separate
release-build step. Native taskbar/Explorer/high-DPI appearance remains a manual check.
Smaller-size optical tuning can follow real desktop inspection.

Generated on 2026-10-10 using the built-in OpenAI image generation tool. No reference
artwork was supplied. The original is preserved; FFmpeg performed only size/format
conversion for the runtime and ICO files. These assets are offered under the project's
Apache-2.0 license to the extent rights can be licensed. AI provenance does not establish
exclusive copyright, third-party clearance or trademark registration.

## Generation prompt

> Use case: logo-brand. Create one polished native desktop application icon for Timestamp Cleaner, a local tool that restores family travel videos by removing burned-in camera timestamps. A distinctive simple film-frame outline enclosing a clean mountain-and-sun landscape; a small luminous four-point restoration sparkle at the lower right where an intrusive timestamp has vanished. Rich midnight navy rounded-square tile, restrained luminous turquoise and warm amber accents, crisp geometric vector-like shapes with subtle dimensional polish. Large readable silhouette, generous optical padding, coherent at 32 pixels, square 1024 by 1024 composition. No text, no letters, no numerals, no clocks, no watermark, no extra mockups, no multiple variations. Original artwork; avoid resemblance to existing software brands. Transparent background outside the rounded-square tile; opaque interior.

## Recreate packaged formats

Run from the repository root with FFmpeg installed. Commands intentionally refuse
to replace existing outputs; remove or version a generated output deliberately first.

```powershell
ffmpeg -hide_banner -loglevel error -i assets/icon.png -vf scale=256:256 -frames:v 1 assets/icon-256.png
ffmpeg -hide_banner -loglevel error -i assets/icon.png -vf scale=256:256 -frames:v 1 -c:v png assets/icon.ico
```
