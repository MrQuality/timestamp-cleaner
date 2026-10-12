# Project website

GitHub Pages serves this static page at https://mrquality.github.io/timestamp-cleaner/.
The source is plain HTML/CSS, with no browser JavaScript, remote fonts or analytics.
The existing project icon and creator-supplied usage demo are copied during the build.

## Usage demo

`media/app-demo.mp4` is the complete 29.92-second screen recording supplied by the
project creator for public website use on 2026-10-12. It was converted from an 86.3 MB
GIF to a 1.35 MB H.264/yuv420p MP4 at its original 1202 x 834 resolution; the private
source GIF is not committed. Container metadata was removed. `app-demo-poster.jpg`
is the frame at five seconds. The demo includes the footage and file-dialog labels
visible in the supplied recording; it has not been represented as a privacy-redacted
or synthetic sample. Do not reuse this media as a bug-report fixture.

Native video controls provide pause, seeking and replay. There is no autoplay or loop,
and `preload="none"` avoids intentionally preloading the video before playback. The
poster still loads normally. A visible text description and direct MP4 link provide
alternatives; the recording is silent. CSP allows same-origin media only. The build
copies only the two explicitly named media assets, not arbitrary files from this folder.

To regenerate from a creator-authorized GIF (FFmpeg required):

```powershell
ffmpeg -i <source.gif> -map 0:v:0 -an -vf fps=25 -c:v libx264 -preset slow -crf 23 -pix_fmt yuv420p -movflags +faststart -map_metadata -1 site/media/app-demo.mp4
ffmpeg -ss 5 -i site/media/app-demo.mp4 -frames:v 1 -q:v 3 -map_metadata -1 site/media/app-demo-poster.jpg
```

Do not broaden Apache-2.0 claims to third-party people/footage in the recording;
the request to display the demo is not a separate license grant for underlying footage.

## Build and deploy

`scripts/build_site.py` selects the most recently published release containing both
the expected Windows x64 ZIP and checksum, including prereleases and excluding drafts.
It validates asset URLs against this repository and escapes template substitutions.
When the same release includes a setup EXE, the primary button downloads it; otherwise
it links to the release page without advertising an unavailable installer.
The page does not use GitHub's `/releases/latest` redirect, which excludes prereleases.
Release metadata is resolved at deployment, so visitors do not make API requests.

For local preview, save the releases API response as JSON, then run:

```powershell
gh api repos/MrQuality/timestamp-cleaner/releases | Out-File -Encoding utf8 releases.json
python scripts/build_site.py releases.json
python -m http.server 8080 --directory dist/site
```

Open http://localhost:8080 and remove the temporary metadata file afterwards.
Run the focused checks with `python -m unittest discover -s scripts -p test_build_site.py -v`.

The Pages workflow builds PR previews without deploying them. Reviewed changes to
the site on main, published/edited releases, and manual runs rebuild and deploy it.
Only deployment receives Pages write/OIDC permissions. A workflow dispatch from
main also refreshes download links after deleting releases or removing assets.
GitHub Pages must be configured with GitHub Actions as its build source.
