"""Release selection must never advertise draft or foreign downloads."""

import copy
from pathlib import Path
import tempfile
import unittest

from build_site import REPO, build, release_values


def release(tag, published, draft=False):
    names = [f"timestamp-cleaner-{tag[1:]}-windows-x64.zip", "SHA256SUMS.txt"]
    return dict(tag_name=tag, published_at=published, draft=draft, prerelease="-" in tag,
                assets=[dict(name=name, browser_download_url=f"{REPO}/releases/download/{tag}/{name}")
                        for name in names])


class SiteTests(unittest.TestCase):
    def setUp(self):
        self.beta = release("v0.1.0-beta.1", "2026-10-10T20:00:00Z")

    def test_latest_publication_includes_beta_but_excludes_drafts(self):
        stable = release("v0.0.9", "2026-10-01T20:00:00Z")
        draft = release("v0.2.0", "2026-10-11T20:00:00Z", draft=True)
        self.assertEqual(release_values([draft, stable, self.beta])["VERSION"], "0.1.0-beta.1")

    def test_missing_checksum_falls_back_to_complete_release(self):
        incomplete = release("v0.2.0", "2026-10-11T20:00:00Z")
        incomplete["assets"].pop()
        self.assertEqual(release_values([incomplete, self.beta])["VERSION"], "0.1.0-beta.1")

    def test_foreign_asset_and_invalid_tag_cannot_be_rendered(self):
        foreign = copy.deepcopy(self.beta)
        foreign["assets"][0]["browser_download_url"] = "https://example.com/app.zip"
        with self.assertRaises(ValueError):
            release_values([foreign])
        invalid = copy.deepcopy(self.beta)
        invalid["tag_name"] = 'v0.1.0"><script>'
        with self.assertRaises(ValueError):
            release_values([invalid])

    def test_empty_and_draft_only_fail_closed(self):
        for releases in ([], [release("v0.2.0", None, draft=True)]):
            with self.subTest(releases=releases), self.assertRaises(ValueError):
                release_values(releases)

    def test_setup_link_is_optional_and_validated_in_same_release(self):
        values = release_values([self.beta])
        self.assertEqual(values["INSTALLER_URL"], values["RELEASE_URL"])
        name = "timestamp-cleaner-0.1.0-beta.1-windows-x64-setup.exe"
        url = f"{REPO}/releases/download/v0.1.0-beta.1/{name}"
        self.beta["assets"].append(dict(name=name, browser_download_url=url))
        self.assertEqual(release_values([self.beta])["INSTALLER_URL"], url)
        self.beta["assets"][-1]["browser_download_url"] = "https://example.com/setup.exe"
        with self.assertRaisesRegex(ValueError, "Installer URL"):
            release_values([self.beta])

    def test_build_has_resolved_downloads_and_only_expected_files(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            build([self.beta], output)
            page = (output / "index.html").read_text(encoding="utf-8")
            self.assertNotIn("{{", page)
            self.assertIn(release_values([self.beta])["DOWNLOAD_URL"], page)
            self.assertEqual({p.name for p in output.iterdir()},
                             {"index.html", "style.css", "icon.png", "media", ".nojekyll"})
            self.assertEqual({p.name for p in (output / "media").iterdir()},
                             {"app-demo.mp4", "app-demo-poster.jpg"})


if __name__ == "__main__":
    unittest.main()
