"""Unit tests for the CLI commands."""

import unittest
from rk855.cli import main, parse_color


class TestRK855CLI(unittest.TestCase):

    def test_parse_color_hex(self):
        self.assertEqual(parse_color("#FF0000"), (255, 0, 0))
        self.assertEqual(parse_color("00FF00"), (0, 255, 0))
        self.assertEqual(parse_color("#123456"), (0x12, 0x34, 0x56))

    def test_parse_color_names(self):
        self.assertEqual(parse_color("red"), (255, 0, 0))
        self.assertEqual(parse_color("blue"), (0, 0, 255))
        self.assertEqual(parse_color("white"), (255, 255, 255))

    def test_parse_color_invalid(self):
        with self.assertRaises(ValueError):
            parse_color("invalid_color_XYZ")

    def test_cli_info_mock(self):
        ret = main(["--mock", "info"])
        self.assertEqual(ret, 0)

    def test_cli_light_mock(self):
        ret = main(["--mock", "light", "--mode", "wave", "--speed", "4", "--brightness", "5"])
        self.assertEqual(ret, 0)

    def test_cli_remap_mock(self):
        ret = main(["--mock", "remap", "capslock", "control"])
        self.assertEqual(ret, 0)

    def test_cli_profile_mock(self):
        ret = main(["--mock", "profile", "list"])
        self.assertEqual(ret, 0)

        ret_save = main(["--mock", "profile", "save", "test_profile"])
        self.assertEqual(ret_save, 0)

        ret_load = main(["--mock", "profile", "load", "test_profile"])
        self.assertEqual(ret_load, 0)

    def test_cli_reset_mock(self):
        ret = main(["--mock", "reset"])
        self.assertEqual(ret, 0)

    def test_aliases_and_exports(self):
        from rk855 import RK855Device, MockRK855Device, RK68Device, MockRK68Device
        self.assertIs(RK68Device, RK855Device)
        self.assertIs(MockRK68Device, MockRK855Device)

        from rk855.gui.window import RK855MainWindow, RK68MainWindow
        self.assertIs(RK68MainWindow, RK855MainWindow)

    def test_cli_parser_prog(self):
        from rk855.cli import create_parser
        parser = create_parser()
        self.assertEqual(parser.prog, "rk855")


if __name__ == "__main__":
    unittest.main()
