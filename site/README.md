# Project website

GitHub Pages serves this static page at https://mrquality.github.io/timestamp-cleaner/.
The source is plain HTML/CSS, with no browser JavaScript, remote fonts or analytics.
The existing project icon is copied during the build; screenshots can be added later.

`scripts/build_site.py` selects the most recently published release containing both
the expected Windows x64 ZIP and checksum, including prereleases and excluding drafts.
It validates asset URLs against this repository and escapes template substitutions.
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
