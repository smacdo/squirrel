from pygametools.lib.build import BuildMode

import unittest


class TestBuildMode(unittest.TestCase):
    def test_from_str(self):
        self.assertEqual(BuildMode.from_str("dev"), BuildMode.Dev)
        self.assertEqual(BuildMode.from_str("Dev"), BuildMode.Dev)
        self.assertEqual(BuildMode.from_str("release"), BuildMode.Release)
        self.assertEqual(BuildMode.from_str("Release"), BuildMode.Release)

    def test_from_str_exception_if_not_known(self):
        self.assertRaises(Exception, lambda: BuildMode.from_str("nope"))


if __name__ == "__main__":
    unittest.main()
