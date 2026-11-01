import pathlib
import tempfile
import unittest

from craftcheck_v2 import validate


class PolicyTests(unittest.TestCase):
    def check(self, source):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "craftbook.toml"
            path.write_text(source)
            return validate(path)

    def test_valid(self):
        self.assertEqual([], self.check('version=2\nproject="app"\nprofiles=["systems","performance"]'))

    def test_invalid_profile(self):
        self.assertTrue(self.check('version=2\nproject="app"\nprofiles=["unknown"]'))

    def test_invalid_assurance(self):
        self.assertTrue(self.check('version=2\nproject="app"\n[assurance]\ntest="yes"'))


if __name__ == "__main__":
    unittest.main()
