"""Build a script-free Pages site from published GitHub release metadata."""

import argparse
import html
import json
from pathlib import Path
import re
import shutil


REPO = "https://github.com/MrQuality/timestamp-cleaner"
ROOT = Path(__file__).resolve().parents[1]


def release_values(releases):
    """Choose the newest published Windows ZIP, including prereleases."""
    candidates = []
    for release in releases:
        if release["draft"] or not release["published_at"]:
            continue
        tag = release["tag_name"]
        if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[A-Za-z0-9.]+)?", tag):
            continue
        version = tag[1:]
        names = {
            "DOWNLOAD_URL": f"timestamp-cleaner-{version}-windows-x64.zip",
            "CHECKSUM_URL": "SHA256SUMS.txt",
        }
        assets = {asset["name"]: asset for asset in release["assets"]}
        if not all(name in assets for name in names.values()):
            continue
        values = {key: f"{REPO}/releases/download/{tag}/{name}" for key, name in names.items()}
        if any(assets[name]["browser_download_url"] != values[key] for key, name in names.items()):
            raise ValueError("Release asset URL does not match this repository and tag")
        values.update(VERSION=version, RELEASE_URL=f"{REPO}/releases/tag/{tag}",
                      STATUS="Early beta" if release["prerelease"] else "Release")
        candidates.append((release["published_at"], values))
    if not candidates:
        raise ValueError("No published Windows ZIP with checksum found")
    return max(candidates, key=lambda candidate: candidate[0])[1]


def build(releases, output):
    values = release_values(releases)
    page = (ROOT / "site/index.html").read_text(encoding="utf-8")
    for key, value in values.items():
        page = page.replace("{{" + key + "}}", html.escape(value, quote=True))
    if "{{" in page:
        raise ValueError("Unresolved site template token")
    output.mkdir(parents=True, exist_ok=True)
    (output / "index.html").write_text(page, encoding="utf-8")
    shutil.copyfile(ROOT / "site/style.css", output / "style.css")
    shutil.copyfile(ROOT / "assets/icon-256.png", output / "icon.png")
    (output / ".nojekyll").touch()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("releases", type=Path, help="JSON array from the GitHub releases API")
    parser.add_argument("--output", type=Path, default=ROOT / "dist/site")
    args = parser.parse_args()
    build(json.loads(args.releases.read_text(encoding="utf-8-sig")), args.output)
