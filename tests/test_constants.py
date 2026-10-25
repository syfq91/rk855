"""Tests for keyboard specifications, constants, and layout matrix."""

import unittest
from rk855_gui.core.constants import (
    PHYSICAL_KEYS,
    LIGHTING_MODES,
    KEY_PRESETS,
    FLASH_LIMIT,
    ISP_PAGE_SIZE,
)


class TestConstants(unittest.TestCase):
    def test_physical_keys_count(self):
        self.assertEqual(len(PHYSICAL_KEYS), 68)

    def test_key_indices_and_bounds(self):
        seen_indices = set()
        for key in PHYSICAL_KEYS:
            self.assertNotIn(key.index, seen_indices)
            seen_indices.add(key.index)
            self.assertGreater(key.width, 0)
            self.assertGreater(key.height, 0)
            self.assertGreaterEqual(key.left, 0)
            self.assertGreaterEqual(key.top, 0)
            self.assertGreaterEqual(key.scancode, 0)
            self.assertLessEqual(key.scancode, 0xFF)

    def test_lighting_modes_coverage(self):
        self.assertEqual(len(LIGHTING_MODES), 20)
        for i in range(20):
            self.assertIn(i, LIGHTING_MODES)
            mode = LIGHTING_MODES[i]
            self.assertEqual(mode.id, i)
            self.assertTrue(len(mode.name) > 0)

    def test_key_presets(self):
        for name, key_indices in KEY_PRESETS.items():
            self.assertTrue(len(key_indices) > 0)
            for idx in key_indices:
                self.assertTrue(0 <= idx < 68)

    def test_flash_limits(self):
        self.assertEqual(FLASH_LIMIT, 65536)
        self.assertEqual(ISP_PAGE_SIZE, 64)
        self.assertEqual(FLASH_LIMIT % ISP_PAGE_SIZE, 0)


if __name__ == "__main__":
    unittest.main()
