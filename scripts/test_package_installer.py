"""Installer input and exact deletion-list regression checks."""

import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

from package_installer import build, checked_payload, write_file_lists
from package_windows import write_archive


class InstallerTests(unittest.TestCase):
    def package(self, root):
        binary = bytearray(128)
        binary[:2] = b"MZ"
        struct.pack_into("<I", binary, 60, 64)
        binary[64:70] = b"PE\0\0\x64\x86"
        payload = {name: b"terms" for name in ("LICENSE", "THIRD_PARTY.md", "START-HERE.md", "licenses/example/LICENSE")}
        payload.update({"timestamp-cleaner.exe": bytes(binary), "build-info.json": json.dumps({
            "version": "0.1.0-beta.2", "target": "x86_64-pc-windows-msvc"}).encode()})
        return write_archive(root, "timestamp-cleaner-0.1.0-beta.2-windows-x64", payload)

    def test_validated_zip_produces_exact_nonrecursive_operations(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = self.package(root)
            version, payload = checked_payload(archive)
            self.assertEqual(version, "0.1.0-beta.2")
            write_file_lists(payload, root / "payload")
            uninstall = (root / "uninstall.nsh").read_text()
            self.assertEqual(len(uninstall.splitlines()), len(payload))
            self.assertNotIn("/r", (root / "directories.nsh").read_text())
            self.assertNotIn("*", uninstall)
            self.assertEqual((root / "payload/timestamp-cleaner.exe").read_bytes(), payload["timestamp-cleaner.exe"])

    def test_tampered_hash_and_unlisted_file_are_rejected(self):
        for name, content in (("LICENSE", b"changed"), ("unexpected.txt", b"extra")):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                archive = self.package(Path(directory))
                # Rewrite to avoid duplicate names masking the manifest regression.
                with zipfile.ZipFile(archive) as source:
                    files = {i.filename: source.read(i) for i in source.infolist()}
                files[f"{archive.stem}/{name}"] = content
                with zipfile.ZipFile(archive, "w") as target:
                    for path, data in files.items(): target.writestr(path, data)
                with self.assertRaisesRegex(ValueError, "manifest"):
                    checked_payload(archive)

    def test_traversal_case_collision_reserved_name_and_nsis_injection_are_rejected(self):
        for name in ("../outside", "licenses/../outside", "license", "CON.txt", 'bad$INSTDIR', 'bad"name'):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                archive = self.package(Path(directory))
                with zipfile.ZipFile(archive, "a") as target:
                    target.writestr(f"{archive.stem}/{name}", b"unsafe")
                with self.assertRaisesRegex(ValueError, "Unsafe"):
                    checked_payload(archive)

    def test_wrong_build_version_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = self.package(Path(directory))
            with zipfile.ZipFile(archive) as bundle:
                files = {i.filename: bundle.read(i) for i in bundle.infolist()}
            key = f"{archive.stem}/build-info.json"
            files[key] = b'{"version":"0.1.0-beta.1","target":"x86_64-pc-windows-msvc"}'
            manifest = {path.split('/', 1)[1]: hashlib.sha256(data).hexdigest()
                        for path, data in files.items() if not path.endswith('/file-manifest.json')}
            files[f"{archive.stem}/file-manifest.json"] = json.dumps(manifest).encode()
            with zipfile.ZipFile(archive, "w") as bundle:
                for path, data in files.items(): bundle.writestr(path, data)
            with self.assertRaisesRegex(ValueError, "metadata mismatch"):
                checked_payload(archive)

    def test_existing_installer_is_never_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = self.package(root)
            installer = root / f"{archive.stem}-setup.exe"
            installer.write_bytes(b"original release")
            with self.assertRaises(FileExistsError):
                build(archive, Path("unused-compiler"), root)
            self.assertEqual(installer.read_bytes(), b"original release")


if __name__ == "__main__":
    unittest.main()
