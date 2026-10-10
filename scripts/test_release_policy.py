import unittest

from release_policy import validate_ref


class ReleasePolicyTests(unittest.TestCase):
    def test_only_matching_beta_tags_qualify(self):
        validate_ref("push", "refs/tags/v0.1.0-beta.1", "0.1.0-beta.1")
        for ref, version in (
            ("refs/tags/v0.1.0-beta.2", "0.1.0-beta.1"),
            ("refs/tags/v0.1.0", "0.1.0"),
            ("refs/heads/main", "0.1.0-beta.1"),
            ("refs/tags/v0.1.0-beta.1;echo bad", "0.1.0-beta.1"),
        ):
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                validate_ref("push", ref, version)

    def test_manual_previews_are_main_only(self):
        validate_ref("workflow_dispatch", "refs/heads/main", "0.1.0")
        with self.assertRaises(ValueError):
            validate_ref("workflow_dispatch", "refs/heads/feature", "0.1.0")
        with self.assertRaises(ValueError):
            validate_ref("pull_request", "refs/heads/main", "0.1.0")


if __name__ == "__main__":
    unittest.main()
