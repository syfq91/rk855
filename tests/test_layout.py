"""Unit tests for the 68-key matrix layout."""

import unittest
from rk68.layout import KEY_DEFINITIONS, find_key, KeyDefinition


class TestRK68Layout(unittest.TestCase):

    def test_key_count(self):
        # Must match the exact 68-key layout from KeyboardLayout.xml
        self.assertEqual(len(KEY_DEFINITIONS), 68)

    def test_key_bounds(self):
        for k in KEY_DEFINITIONS:
            self.assertGreater(k.rect_width, 0)
            self.assertGreater(k.rect_height, 0)
            self.assertGreaterEqual(k.rect_left, 0)
            self.assertGreaterEqual(k.rect_top, 0)
            self.assertLessEqual(k.x_percent + k.width_percent, 1.05)
            self.assertLessEqual(k.y_percent + k.height_percent, 1.05)

    def test_find_key_by_name(self):
        esc = find_key("esc")
        self.assertIsNotNone(esc)
        self.assertEqual(esc.code, 0x29)

        space = find_key("space")
        self.assertIsNotNone(space)
        self.assertEqual(space.code, 0x2C)

        fn = find_key("fn")
        self.assertIsNotNone(fn)
        self.assertEqual(fn.code, 0xAF)

    def test_find_key_by_desc(self):
        caps = find_key("Caps")
        self.assertIsNotNone(caps)
        self.assertEqual(caps.code, 0x39)

    def test_find_key_by_hex_code(self):
        key = find_key("0x14")
        self.assertIsNotNone(key)
        self.assertEqual(key.name.lower(), "q")

    def test_find_key_nonexistent(self):
        self.assertIsNone(find_key("nonexistent_key_12345"))

    def test_remap_targets_available(self):
        from rk68.layout import REMAP_TARGETS
        self.assertGreater(len(REMAP_TARGETS), 50)

        # F-keys
        f5 = find_key("f5")
        self.assertIsNotNone(f5)
        self.assertEqual(f5.code, 0x3E)

        # Media keys
        vol_up = find_key("volume_up")
        self.assertIsNotNone(vol_up)
        self.assertEqual(vol_up.code, 0xE9)

        mute = find_key("mute")
        self.assertIsNotNone(mute)
        self.assertEqual(mute.code, 0xE2)


if __name__ == "__main__":
    unittest.main()
