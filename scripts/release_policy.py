"""Validate release entry points before running the reusable quality/build workflow."""

import json
import os
import re
import subprocess


def validate_ref(event, ref, version):
    if event == "workflow_dispatch":
        if ref != "refs/heads/main":
            raise ValueError("Manual release previews must run from main")
    elif event == "push":
        if not re.fullmatch(r"refs/tags/v\d+\.\d+\.\d+-beta\.\d+", ref):
            raise ValueError("Only explicit beta tags can prepare releases")
        if ref != f"refs/tags/v{version}":
            raise ValueError("Release tag must exactly match Cargo package version")
    else:
        raise ValueError("Unsupported release event")


def main():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"], text=True,
    ))
    package = next(p for p in metadata["packages"] if p["name"] == "timestamp-cleaner")
    validate_ref(os.environ["GITHUB_EVENT_NAME"], os.environ["GITHUB_REF"], package["version"])
    # Includes annotated tags; only commits already incorporated into main qualify.
    subprocess.run(["git", "merge-base", "--is-ancestor", "HEAD", "origin/main"], check=True)
    print("Release source and version policy passed.")


if __name__ == "__main__":
    main()
