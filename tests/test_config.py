"""Unit tests for configuration and profile management."""

import tempfile
import unittest
from pathlib import Path
from rk68.config import ConfigManager, KeyboardProfile


class TestRK68Config(unittest.TestCase):

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.cfg = ConfigManager(config_dir=Path(self.temp_dir.name))

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_default_profile(self):
        profile = self.cfg.load_profile("default")
        self.assertEqual(profile.name, "default")
        self.assertEqual(profile.lighting_mode, "wave")
        self.assertEqual(profile.brightness, 4)

    def test_save_and_load_profile(self):
        profile = KeyboardProfile(
            name="gaming_fps",
            lighting_mode="static",
            speed=5,
            brightness=5,
            color="#00FF88",
            key_remaps={"capslock": "control"},
            custom_rgb={"w": "#FF0000", "a": "#FF0000"},
        )
        self.cfg.save_profile(profile)

        loaded = self.cfg.load_profile("gaming_fps")
        self.assertEqual(loaded.name, "gaming_fps")
        self.assertEqual(loaded.lighting_mode, "static")
        self.assertEqual(loaded.speed, 5)
        self.assertEqual(loaded.color, "#00FF88")
        self.assertEqual(loaded.get_color_rgb(), (0, 255, 136))
        self.assertEqual(loaded.key_remaps.get("capslock"), "control")
        self.assertEqual(loaded.custom_rgb.get("w"), "#FF0000")

    def test_list_and_delete_profile(self):
        self.cfg.save_profile(KeyboardProfile(name="profile_a"))
        self.cfg.save_profile(KeyboardProfile(name="profile_b"))

        profiles = self.cfg.list_profiles()
        self.assertIn("profile_a", profiles)
        self.assertIn("profile_b", profiles)

        deleted = self.cfg.delete_profile("profile_a")
        self.assertTrue(deleted)
        self.assertNotIn("profile_a", self.cfg.list_profiles())

        # Cannot delete default
        self.assertFalse(self.cfg.delete_profile("default"))


if __name__ == "__main__":
    unittest.main()
