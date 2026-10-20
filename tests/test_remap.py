"""Unit tests for the safe Linux systemd-hwdb key remapping module."""

import unittest
from rk855.remap import (
    resolve_scancode,
    resolve_linux_keycode,
    generate_hwdb_content,
    set_key_remap,
    set_key_remaps,
    clear_all_remaps,
    load_user_remaps,
    save_user_remaps,
)


class TestRK855Remap(unittest.TestCase):

    def test_resolve_scancode(self):
        # Known keys by name
        self.assertEqual(resolve_scancode("capslock"), 0x39)
        self.assertEqual(resolve_scancode("esc"), 0x29)
        self.assertEqual(resolve_scancode("tab"), 0x2B)
        self.assertEqual(resolve_scancode("a"), 0x04)
        # By alias
        self.assertEqual(resolve_scancode("ctrl"), 0xE0)
        self.assertEqual(resolve_scancode("lctrl"), 0xE0)
        self.assertEqual(resolve_scancode("win"), 0xE3)
        # By int
        self.assertEqual(resolve_scancode(0x39), 0x39)

    def test_resolve_linux_keycode(self):
        self.assertEqual(resolve_linux_keycode(0x39), "capslock")
        self.assertEqual(resolve_linux_keycode(0xE0), "leftctrl")
        self.assertEqual(resolve_linux_keycode("control"), "leftctrl")
        self.assertEqual(resolve_linux_keycode("ctrl"), "leftctrl")
        self.assertEqual(resolve_linux_keycode("capslock"), "capslock")
        self.assertEqual(resolve_linux_keycode("esc"), "esc")
        self.assertEqual(resolve_linux_keycode("f1"), "f1")

    def test_generate_hwdb_content(self):
        remaps = {"capslock": "control", "esc": "grave"}
        content = generate_hwdb_content(remaps)
        self.assertIn("evdev:input:b0003v0C45p8018*", content)
        # 0x39 -> 70039=leftctrl
        self.assertIn(" KEYBOARD_KEY_70039=leftctrl", content)
        # 0x29 -> 70029=grave
        self.assertIn(" KEYBOARD_KEY_70029=grave", content)

    def test_set_and_clear_remaps_mock(self):
        success, msg = set_key_remap("capslock", "control", is_mock=True)
        self.assertTrue(success)
        self.assertEqual(load_user_remaps().get("capsLock"), "leftctrl")

        success, msg = clear_all_remaps(is_mock=True)
        self.assertTrue(success)
        self.assertEqual(load_user_remaps(), {})


if __name__ == "__main__":
    unittest.main()
