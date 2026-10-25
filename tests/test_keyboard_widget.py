"""Tests for interactive KeyboardView widget in headless Qt."""

import unittest
from PyQt6.QtWidgets import QApplication
from rk855_gui.widgets.keyboard_view import KeyboardView


class TestKeyboardView(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Create single QApplication instance for unit tests
        cls.app = QApplication.instance() or QApplication([])

    def test_default_state(self):
        view = KeyboardView()
        colors = view.get_key_colors()
        self.assertEqual(len(colors), 68)
        self.assertEqual(len(view.get_selected_indices()), 0)

    def test_preset_selection(self):
        view = KeyboardView()
        view.select_preset("WASD")
        self.assertEqual(view.get_selected_indices(), [17, 31, 32, 33])

        view.select_preset("Arrow Keys")
        self.assertEqual(view.get_selected_indices(), [56, 65, 66, 67])

        view.clear_selection()
        self.assertEqual(len(view.get_selected_indices()), 0)

    def test_color_application(self):
        view = KeyboardView()
        view.select_preset("WASD")
        view.set_selected_keys_color((255, 0, 0))

        colors = view.get_key_colors()
        self.assertEqual(colors[17], (255, 0, 0))  # W
        self.assertEqual(colors[31], (255, 0, 0))  # A
        self.assertEqual(colors[32], (255, 0, 0))  # S
        self.assertEqual(colors[33], (255, 0, 0))  # D
        # Non-selected keys remain previous color
        self.assertNotEqual(colors[0], (255, 0, 0))


if __name__ == "__main__":
    unittest.main()
