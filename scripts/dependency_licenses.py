"""Inventory declared licenses and packaged notices; this is not a legal clearance tool."""

import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "docs" / "dependency-licenses.csv"


def inventory():
    lock_hash = hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest()
    metadata = json.loads(
        subprocess.check_output(
            ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1"],
            cwd=ROOT,
        )
    )
    rows = []
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["id"] in metadata["workspace_members"]:
            continue
        directory = Path(package["manifest_path"]).parent
        notices = []
        for path in sorted(directory.rglob("*"), key=lambda p: p.relative_to(directory).as_posix()):
            relative = path.relative_to(directory)
            if not path.is_file():
                continue
            # Font terms use upstream names such as OFL.txt, UFL.txt and Hack-Regular.txt.
            conventional = path.name.lower().startswith(
                ("license", "licence", "copying", "notice", "copyright")
            )
            font_terms = "fonts" in relative.parts and path.suffix.lower() == ".txt"
            explicit = package.get("license_file") == relative.as_posix()
            if conventional or font_terms or explicit:
                digest = hashlib.sha256(path.read_bytes()).hexdigest()
                notices.append(f"{relative.as_posix()}@sha256:{digest}")
        rows.append(
            {
                "crate": package["name"],
                "version": package["version"],
                "declared_license": package.get("license") or "UNDECLARED",
                "source": package.get("source") or "",
                "lockfile_sha256": lock_hash,
                "repository": package.get("repository") or "",
                "packaged_notice_files": ";".join(notices),
                "notice_status": "packaged-files-found" if notices else "upstream-review-needed",
            }
        )
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue(), rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Fail if the tracked inventory is stale")
    args = parser.parse_args()
    content, rows = inventory()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != content:
            print("Dependency inventory is stale; run python scripts/dependency_licenses.py", file=sys.stderr)
            return 1
    else:
        OUTPUT.write_text(content, encoding="utf-8", newline="\n")
    gaps = sum(row["notice_status"] == "upstream-review-needed" for row in rows)
    print(f"Inventoried {len(rows)} external locked packages; {gaps} need upstream notice review.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
