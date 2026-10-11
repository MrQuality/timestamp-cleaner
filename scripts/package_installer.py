"""Build an unsigned NSIS installer from the verified portable ZIP payload."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tempfile
import zipfile

from package_windows import ROOT, command, source_document, validate_executable


def checked_payload(archive):
    match = re.fullmatch(r"timestamp-cleaner-(\d+\.\d+\.\d+(?:-beta\.\d+)?)-windows-x64", archive.stem)
    if not match:
        raise ValueError("Unexpected portable archive name")
    version = match[1]
    payload = {}
    seen = set()
    reserved = {"CON", "PRN", "AUX", "NUL"} | {f"{prefix}{i}" for prefix in ("COM", "LPT") for i in range(1, 10)}
    with zipfile.ZipFile(archive) as bundle:
        if len(bundle.infolist()) > 10000 or sum(i.file_size for i in bundle.infolist()) > 100_000_000:
            raise ValueError("Portable payload exceeds packaging limits")
        for entry in bundle.infolist():
            parts = entry.filename.split("/")
            if len(parts) < 2 or parts[0] != archive.stem:
                raise ValueError("Unexpected archive root")
            name = "/".join(parts[1:])
            if (not re.fullmatch(r"[A-Za-z0-9._ /-]+", name)
                    or any(not p or p in (".", "..") or p.endswith((".", " "))
                           or p.split(".")[0].upper() in reserved
                           for p in parts)
                    or name.casefold() in seen):
                raise ValueError("Unsafe or duplicate Windows payload path")
            seen.add(name.casefold())
            payload[name] = bundle.read(entry)
    manifest = json.loads(payload["file-manifest.json"])
    if set(manifest) != set(payload) - {"file-manifest.json"}:
        raise ValueError("Portable manifest does not cover exactly the payload")
    if any(hashlib.sha256(payload[name]).hexdigest() != digest for name, digest in manifest.items()):
        raise ValueError("Portable manifest digest mismatch")
    info = json.loads(payload["build-info.json"])
    if info["version"] != version or info["target"] != "x86_64-pc-windows-msvc":
        raise ValueError("Portable build metadata mismatch")
    for required in ("LICENSE", "THIRD_PARTY.md", "START-HERE.md", "timestamp-cleaner.exe"):
        if required not in payload:
            raise ValueError(f"Missing required payload: {required}")
    validate_executable(payload["timestamp-cleaner.exe"])
    return version, payload


def write_file_lists(payload, stage):
    """Generate exact file operations, with no recursive deletion or globbing."""
    install, uninstall, directories = [], [], set()
    for name, content in sorted(payload.items()):
        relative = PurePosixPath(name)
        destination = stage / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)
        parent = str(relative.parent).replace("/", "\\")
        suffix = "" if parent == "." else "\\" + parent
        windows_name = name.replace("/", "\\")
        install += [f'SetOutPath "$INSTDIR{suffix}"', f'File "${{PAYLOAD}}\\{windows_name}"',
                    'IfErrors 0 +3', 'SetErrorLevel 2', 'Abort "Could not install application files; remove the partial version folder before retrying."']
        uninstall.append(f'Delete "$INSTDIR\\{windows_name}"')
        directories.update(str(p).replace("/", "\\") for p in relative.parents if str(p) != ".")
    remove = [f'RMDir "$INSTDIR\\{p}"' for p in sorted(directories, key=lambda p: (-p.count("\\"), p))]
    for filename, lines in (("install.nsh", install), ("uninstall.nsh", uninstall), ("directories.nsh", remove)):
        (stage.parent / filename).write_text("\n".join(lines) + "\n", encoding="utf-8")


def build(archive, compiler, output):
    version, payload = checked_payload(archive)
    archive_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
    checksum = output / "SHA256SUMS.txt"
    expected = f"{archive_hash}  {archive.name}"
    if expected not in checksum.read_text(encoding="utf-8").splitlines():
        raise ValueError("Portable ZIP does not match the release checksum")
    installer = output / f"timestamp-cleaner-{version}-windows-x64-setup.exe"
    if installer.exists():
        raise FileExistsError("Installer already exists; never replace a release asset")
    compiler = compiler.resolve()
    compiler_version = subprocess.check_output([str(compiler), "/VERSION"], text=True).strip()
    if compiler_version != "v3.12":
        raise ValueError("NSIS 3.12 is required")
    if any(name.casefold().startswith("installer/") for name in payload):
        raise ValueError("Portable ZIP uses reserved installer namespace")
    payload["installer/NSIS-COPYING.txt"] = (compiler.parent / "COPYING").read_bytes()
    installer_commit = command("git", "rev-parse", "HEAD")
    payload["installer/SETUP.md"] = source_document("docs/windows-installer.md", installer_commit)
    payload["installer/build-info.json"] = (json.dumps({"nsis": compiler_version,
        "portable_zip": archive.name, "portable_sha256": archive_hash, "installer_commit": installer_commit,
        "nsis_copying_sha256": hashlib.sha256(payload["installer/NSIS-COPYING.txt"]).hexdigest(),
        "signed": False}, indent=2) + "\n").encode()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="timestamp-cleaner-nsis-") as directory:
        stage = Path(directory) / "payload"
        write_file_lists(payload, stage)
        compiled = stage.parent / installer.name
        pe_version = version.split("-")[0] + "." + (version.split("-beta.")[1] if "-beta." in version else "0")
        defines = dict(VERSION=version, PE_VERSION=pe_version,
                       OUTPUT=str(compiled), PAYLOAD=str(stage),
                       ICON=str(ROOT / "assets/icon.ico"),
                       INSTALL_FILES=str(stage.parent / "install.nsh"),
                       UNINSTALL_FILES=str(stage.parent / "uninstall.nsh"),
                       REMOVE_DIRS=str(stage.parent / "directories.nsh"))
        subprocess.run([str(compiler), "/NOCONFIG", "/WX", *[f"/D{k}={v}" for k, v in defines.items()],
                        str(ROOT / "installer/windows.nsi")], check=True)
        with installer.open("xb") as stream:
            stream.write(compiled.read_bytes())
    with checksum.open("a", encoding="utf-8", newline="\n") as stream:
        stream.write(f"{hashlib.sha256(installer.read_bytes()).hexdigest()}  {installer.name}\n")
    return installer


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    print(build(args.archive, args.compiler, args.output))
