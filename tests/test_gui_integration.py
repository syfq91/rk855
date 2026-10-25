"""Integration test for MainWindow and UI flows in headless mode."""

import os
import unittest
from unittest.mock import patch
from PyQt6.QtWidgets import QApplication, QMessageBox
from rk855_gui.ui.main_window import MainWindow


class TestGuiIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.app = QApplication.instance() or QApplication([])

    @patch.object(QMessageBox, "information", return_value=QMessageBox.StandardButton.Ok)
    @patch.object(QMessageBox, "warning", return_value=QMessageBox.StandardButton.Ok)
    @patch.object(QMessageBox, "critical", return_value=QMessageBox.StandardButton.Ok)
    @patch.object(QMessageBox, "question", return_value=QMessageBox.StandardButton.Yes)
    def test_main_window_tabs_and_mock_actions(self, mock_q, mock_c, mock_w, mock_i):
        window = MainWindow(mock=True)

        # 1. Test tab count & switching
        self.assertEqual(window.tabs.count(), 5)
        for i in range(5):
            window.tabs.setCurrentIndex(i)
            self.assertEqual(window.tabs.currentIndex(), i)

        # 2. Test lighting effect configuration
        window.tabs.setCurrentIndex(0)
        window.combo_modes.setCurrentIndex(3)  # Glittering
        window.slider_bright.setValue(4)
        window.slider_speed.setValue(3)
        window.light_color_picker.set_color((255, 128, 0))

        # Test apply lighting
        window._on_apply_lighting()
        self.assertEqual(window._mock_device.active_lighting_mode, 3)
        self.assertEqual(window._mock_device.color, (255, 128, 0))

        # 3. Test Per-Key Studio
        window.tabs.setCurrentIndex(1)
        window.keyboard_view.select_preset("WASD")
        window.perkey_color_picker.set_color((0, 255, 0))
        window._on_upload_matrix()
        # Verify mock device received packets
        self.assertTrue(len(window._mock_device.sent_packets) > 0)

        # 4. Test Key Remapping Tab
        window.tabs.setCurrentIndex(2)
        # Select key 30 (Caps Lock, 0x39)
        window._on_remap_key_clicked(30)
        self.assertIn("0x39", window.lbl_selected_remap_key.text())

        # Select Modifiers category and Left Ctrl (0xE0)
        cat_idx = window.combo_remap_category.findText("Modifiers")
        if cat_idx >= 0:
            window.combo_remap_category.setCurrentIndex(cat_idx)
        act_idx = window.combo_target_action.findData(0xE0)
        if act_idx >= 0:
            window.combo_target_action.setCurrentIndex(act_idx)

        # Assign action
        window._on_assign_action() if hasattr(window, "_on_assign_action") else window._on_assign_remap_action()
        self.assertEqual(window._remap_dict.get(0x39), 0xE0)
        self.assertEqual(window.table_remaps.rowCount(), 1)

        # Apply remap
        window._on_apply_remap()
        self.assertEqual(window._mock_device.remap_table.get(0x39), 0xE0)

        # Test restore key
        window._on_reset_selected_key_remap()
        self.assertNotIn(0x39, window._remap_dict)
        self.assertEqual(window.table_remaps.rowCount(), 0)

        # 5. Test Firmware Flasher preloaded file
        window.tabs.setCurrentIndex(3)
        hex_file = os.path.join(os.path.dirname(__file__), "..", "firmware", "SN32F240B_32.hex")
        if os.path.isfile(hex_file):
            window.load_firmware_file(os.path.abspath(hex_file))
            self.assertIsNotNone(window._loaded_firmware_bytes)
            self.assertEqual(len(window._loaded_firmware_bytes), 65536)

        window.close()


if __name__ == "__main__":
    unittest.main()
