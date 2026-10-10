"""Packaging regressions: licensing integrity, architecture and archive contents."""

import hashlib
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

from package_windows import license_payload, source_document, validate_executable, verified_notice, write_archive


class PackagingTests(unittest.TestCase):
    def test_copied_recovery_document_links_to_exact_source_revision(self):
        document = source_document("docs/media-tools.md", "abc123").decode()
        self.assertIn("/blob/abc123/docs/manual-verification.md)", document)
        self.assertIn("/blob/abc123/VALIDATION.md)", document)
        self.assertIn("(https://ffmpeg.org/download.html)", document)

    def test_notice_tampering_and_directory_escape_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            notice = root / "LICENSE"
            notice.write_bytes(b"original terms")
            declaration = "LICENSE@sha256:" + hashlib.sha256(notice.read_bytes()).hexdigest()
            self.assertEqual(verified_notice(root, declaration)[1], b"original terms")
            notice.write_bytes(b"changed terms")
            with self.assertRaisesRegex(ValueError, "digest mismatch"):
                verified_notice(root, declaration)
            with self.assertRaisesRegex(ValueError, "Invalid notice path"):
                verified_notice(root, "../LICENSE@sha256:unused")

    def test_selected_dependency_without_notices_blocks_packaging(self):
        row = {"crate": "missing", "version": "1.0.0", "packaged_notice_files": "", "upstream_notice_files": ""}
        with self.assertRaisesRegex(ValueError, "No notices available"):
            license_payload([row], {}, {("missing", "1.0.0")})
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "LICENSE").write_bytes(b"terms")
            selected = {"crate": "used", "version": "1.0.0", "packaged_notice_files": "LICENSE@sha256:" + hashlib.sha256(b"terms").hexdigest(), "upstream_notice_files": ""}
            files = license_payload([row, selected], {("used", "1.0.0"): {"manifest_path": str(root / "Cargo.toml")}}, {("used", "1.0.0")})
            self.assertEqual(files["licenses/used-1.0.0/LICENSE"], b"terms")
            self.assertNotIn(b"missing", files["dependency-licenses.csv"])

    def test_architecture_validation_rejects_non_pe_and_x86(self):
        content = bytearray(128)
        content[:2] = b"MZ"
        struct.pack_into("<I", content, 60, 64)
        content[64:70] = b"PE\0\0\x64\x86"
        validate_executable(content)
        content[68:70] = b"\x4c\x01"
        with self.assertRaisesRegex(ValueError, "AMD64"):
            validate_executable(content)
        with self.assertRaisesRegex(ValueError, "Windows PE"):
            validate_executable(b"not an executable")

    def test_archive_manifest_checksums_reproducibility_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            payload = {"timestamp-cleaner.exe": b"binary", "LICENSE": b"terms"}
            archive = write_archive(root / "one", "package", payload)
            with zipfile.ZipFile(archive) as bundle:
                manifest = json.loads(bundle.read("package/file-manifest.json"))
                self.assertEqual(set(manifest), set(payload))
                for name, digest in manifest.items():
                    self.assertEqual(hashlib.sha256(bundle.read("package/" + name)).hexdigest(), digest)
            checksum = (archive.parent / "SHA256SUMS.txt").read_text()
            self.assertEqual(checksum, hashlib.sha256(archive.read_bytes()).hexdigest() + "  package.zip\n")
            second = write_archive(root / "two", "package", payload)
            self.assertEqual(archive.read_bytes(), second.read_bytes())
            with self.assertRaises(FileExistsError):
                write_archive(root / "one", "package", {"different": b"content"})


if __name__ == "__main__":
    unittest.main()
