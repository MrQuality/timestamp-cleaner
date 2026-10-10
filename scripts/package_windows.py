"""Assemble a Windows x64 ZIP with verified dependency notices; never publish it."""

import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import posixpath
import re
import struct
import subprocess
import zipfile

from dependency_licenses import ROOT, inventory

TARGET = "x86_64-pc-windows-msvc"


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True, encoding="utf-8").strip()


def source_document(relative, commit):
    """Keep links in copied source-policy documents valid outside a Git checkout."""
    def link(match):
        target = match[2]
        if target.startswith(("https://", "http://", "#")):
            return match[0]
        target = posixpath.normpath(posixpath.join(posixpath.dirname(relative), target))
        return f"[{match[1]}](https://github.com/MrQuality/timestamp-cleaner/blob/{commit}/{target})"

    text = (ROOT / relative).read_text(encoding="utf-8")
    return re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", link, text).encode("utf-8")


def verified_notice(directory, declaration):
    relative, expected = declaration.rsplit("@sha256:", 1)
    path = (directory / relative).resolve()
    if not path.is_relative_to(directory.resolve()) or not path.is_file():
        raise ValueError(f"Invalid notice path: {relative}")
    content = path.read_bytes()
    if hashlib.sha256(content).hexdigest() != expected:
        raise ValueError(f"Notice digest mismatch: {relative}")
    return relative, content


def license_payload(rows, packages, selected):
    """Include normal/build dependency notices, including proc-macro/build tooling."""
    files = {}
    included = []
    by_key = {(row["crate"], row["version"]): row for row in rows}
    for key in sorted(selected):
        row = by_key[key]
        declarations = row["packaged_notice_files"] or row["upstream_notice_files"]
        if not declarations:
            raise ValueError(f"No notices available for {key[0]} {key[1]}")
        directory = Path(packages[key]["manifest_path"]).parent
        for field, base in (("packaged_notice_files", directory), ("upstream_notice_files", ROOT)):
            for declaration in filter(None, row[field].split(";")):
                relative, content = verified_notice(base, declaration)
                files[f"licenses/{key[0]}-{key[1]}/{relative}"] = content
        included.append(row)
    output = io.StringIO(newline="")
    writer = csv.DictWriter(output, fieldnames=list(included[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(included)
    files["dependency-licenses.csv"] = output.getvalue().encode("utf-8")
    return files


def validate_executable(content):
    if len(content) < 64 or content[:2] != b"MZ":
        raise ValueError("Expected a Windows PE executable")
    offset = struct.unpack_from("<I", content, 60)[0]
    if content[offset:offset + 6] != b"PE\0\0\x64\x86":
        raise ValueError("Expected an AMD64 Windows executable")


def write_archive(output, stem, payload):
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f"{stem}.zip"
    checksum = output / "SHA256SUMS.txt"
    if archive.exists() or checksum.exists():
        raise FileExistsError("Package/checksum already exists; choose an empty output directory")
    manifest = {name: hashlib.sha256(content).hexdigest() for name, content in sorted(payload.items())}
    payload = {**payload, "file-manifest.json": (json.dumps(manifest, indent=2) + "\n").encode()}
    # Fixed entry timestamps/order make the container reproducible for identical input bytes.
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED) as bundle:
        for name, content in sorted(payload.items()):
            entry = zipfile.ZipInfo(f"{stem}/{name}", date_time=(1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            bundle.writestr(entry, content)
    with zipfile.ZipFile(archive) as bundle:
        if bundle.testzip() is not None:
            raise ValueError("ZIP integrity check failed")
        for name, digest in manifest.items():
            if hashlib.sha256(bundle.read(f"{stem}/{name}")).hexdigest() != digest:
                raise ValueError(f"Archived file digest mismatch: {name}")
    with checksum.open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")
    return archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target" / TARGET / "release/timestamp-cleaner.exe")
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    metadata = json.loads(command("cargo", "metadata", "--locked", "--format-version", "1"))
    package = next(p for p in metadata["packages"] if p["id"] == metadata["resolve"]["root"])
    version = package["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-beta\.\d+)?", version):
        raise ValueError("Unsupported release version")
    executable = args.binary.read_bytes()
    validate_executable(executable)
    _, rows = inventory()
    packages = {(p["name"], p["version"]): p for p in metadata["packages"]}
    tree = command("cargo", "tree", "--locked", "--target", TARGET, "--edges", "normal,build", "--prefix", "none", "--format", "{p}")
    selected = set()
    for line in tree.splitlines():
        match = re.match(r"^(\S+) v(\S+)(?: |$)", line)
        if match is None:
            raise ValueError(f"Unexpected dependency tree entry: {line}")
        key = match.groups()
        if key[0] != package["name"]:
            selected.add(key)
    payload = license_payload(rows, packages, selected)
    commit = command("git", "rev-parse", "HEAD")
    for name in ("LICENSE", "CHANGELOG.md"):
        payload[name] = (ROOT / name).read_bytes()
    for source, destination in (("THIRD_PARTY.md", "THIRD_PARTY.md"), ("docs/portable-windows.md", "START-HERE.md"), ("docs/media-tools.md", "docs/media-tools.md")):
        payload[destination] = source_document(source, commit)
    payload["timestamp-cleaner.exe"] = executable
    payload["build-info.json"] = (json.dumps({
        "version": version, "target": TARGET, "features": ["desktop"],
        "commit": commit,
        "rustc": command("rustc", "--version", "--verbose"),
        "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
        "dependency_count_including_build_tools": len(selected),
        "signed": False, "ffmpeg_bundled": False,
    }, indent=2) + "\n").encode()
    archive = write_archive(args.output, f"timestamp-cleaner-{version}-windows-x64", payload)
    print(f"Created {archive.name}; {len(selected)} dependency notice sets included.")


if __name__ == "__main__":
    main()
