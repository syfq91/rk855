"""Unit tests for RK855 Key Switch Tester and key mapping resolution."""

import unittest
from rk855.gui.key_tester import KeyMapper, KeyTesterCanvas, KeyTesterWidget, PYQT_AVAILABLE
from rk855.layout import KEY_DEFINITIONS, CODE_TO_KEY
from rk855.cli import create_parser

if PYQT_AVAILABLE:
    from PyQt6.QtCore import Qt
    from PyQt6.QtWidgets import QApplication

    # Ensure a single headless QApplication instance exists for widget tests
    app = QApplication.instance() or QApplication([])


class TestKeyMapper(unittest.TestCase):

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for Qt key tests")
    def test_alphanumeric_mapping(self):
        # Letters
        k_a = KeyMapper.resolve_key(Qt.Key.Key_A)
        self.assertIsNotNone(k_a)
        self.assertEqual(k_a.code, 0x04)
        self.assertEqual(k_a.name.upper(), "A")

        k_z = KeyMapper.resolve_key(Qt.Key.Key_Z)
        self.assertIsNotNone(k_z)
        self.assertEqual(k_z.code, 0x1D)

        # Numbers
        k_1 = KeyMapper.resolve_key(Qt.Key.Key_1)
        self.assertIsNotNone(k_1)
        self.assertEqual(k_1.code, 0x1E)

        k_0 = KeyMapper.resolve_key(Qt.Key.Key_0)
        self.assertIsNotNone(k_0)
        self.assertEqual(k_0.code, 0x27)

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for Qt key tests")
    def test_navigation_and_control_mapping(self):
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Escape).code, 0x29)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Tab).code, 0x2B)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Backspace).code, 0x2A)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Return).code, 0x28)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Enter).code, 0x28)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Space).code, 0x2C)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Delete).code, 0x4C)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_PageUp).code, 0x4B)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_PageDown).code, 0x4E)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Up).code, 0x52)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Down).code, 0x51)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Left).code, 0x50)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Right).code, 0x4F)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_CapsLock).code, 0x39)

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for Qt key tests")
    def test_symbols_mapping(self):
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Minus).code, 0x2D)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Equal).code, 0x2E)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_BracketLeft).code, 0x2F)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_BracketRight).code, 0x30)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Backslash).code, 0x31)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Semicolon).code, 0x33)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Apostrophe).code, 0x34)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Comma).code, 0x36)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Period).code, 0x37)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_Slash).code, 0x38)
        self.assertEqual(KeyMapper.resolve_key(Qt.Key.Key_QuoteLeft).code, 0x35)

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for Qt key tests")
    def test_modifier_distinction(self):
        # Shift
        lshift = KeyMapper.resolve_key(Qt.Key.Key_Shift, native_scancode=50)
        self.assertEqual(lshift.code, 0xE1)
        rshift = KeyMapper.resolve_key(Qt.Key.Key_Shift, native_scancode=62)
        self.assertEqual(rshift.code, 0xE5)

        # Control
        lctrl = KeyMapper.resolve_key(Qt.Key.Key_Control, native_scancode=37)
        self.assertEqual(lctrl.code, 0xE0)
        rctrl = KeyMapper.resolve_key(Qt.Key.Key_Control, native_scancode=105)
        self.assertEqual(rctrl.code, 0xE4)

        # Alt
        lalt = KeyMapper.resolve_key(Qt.Key.Key_Alt, native_scancode=64)
        self.assertEqual(lalt.code, 0xE2)
        ralt = KeyMapper.resolve_key(Qt.Key.Key_Alt, native_scancode=108)
        self.assertEqual(ralt.code, 0xE6)

        # Super / Win
        win = KeyMapper.resolve_key(Qt.Key.Key_Meta)
        self.assertEqual(win.code, 0xE3)

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for Qt key tests")
    def test_text_fallback(self):
        k = KeyMapper.resolve_key(0, native_scancode=0, text="w")
        self.assertIsNotNone(k)
        self.assertEqual(k.code, 0x1A)


class TestKeyTesterCanvas(unittest.TestCase):

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for widget tests")
    def test_tester_canvas_states(self):
        canvas = KeyTesterCanvas()
        self.assertEqual(canvas.get_tested_count(), 0)
        self.assertEqual(len(canvas.pressed_keys), 0)

        # Press 'A' (0x04)
        canvas.set_key_pressed(0x04, True)
        self.assertIn(0x04, canvas.pressed_keys)
        self.assertIn(0x04, canvas.tested_keys)
        self.assertEqual(canvas.get_tested_count(), 1)

        # Release 'A'
        canvas.set_key_pressed(0x04, False)
        self.assertNotIn(0x04, canvas.pressed_keys)
        self.assertIn(0x04, canvas.tested_keys)
        self.assertEqual(canvas.get_tested_count(), 1)

        # Press 'B' (0x05)
        canvas.set_key_pressed(0x05, True)
        self.assertEqual(canvas.get_tested_count(), 2)

        # Reset
        canvas.reset_tester()
        self.assertEqual(canvas.get_tested_count(), 0)
        self.assertEqual(len(canvas.pressed_keys), 0)
        self.assertEqual(len(canvas.tested_keys), 0)


class TestKeyTesterWidget(unittest.TestCase):

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for widget tests")
    def test_tester_widget_keystroke_flow(self):
        widget = KeyTesterWidget()
        self.assertEqual(widget.canvas.get_tested_count(), 0)
        self.assertEqual(widget.progress_bar.value(), 0)

        k_space = CODE_TO_KEY[0x2C]
        widget._record_keystroke(k_space, "TEST", 65)

        self.assertIn("Space", widget.history)
        self.assertIn("Space", widget.inspector_key_label.text())
        self.assertIn("0x2C", widget.inspector_code_label.text())

        # Test rolling history limit (12 items)
        for i in range(20):
            widget._record_keystroke(k_space, "TEST", 65)
        self.assertLessEqual(len(widget.history), 12)

        # Reset
        widget.reset_test()
        self.assertEqual(len(widget.history), 0)
        self.assertEqual(widget.progress_bar.value(), 0)
        self.assertIn("Waiting", widget.history_label.text())


class TestCLIParserKeyTester(unittest.TestCase):

    def test_cli_has_test_command(self):
        parser = create_parser()
        args = parser.parse_args(["test", "--mock"])
        self.assertEqual(args.command, "test")
        self.assertTrue(args.mock)


class TestKeyPaletteDialog(unittest.TestCase):

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for dialog tests")
    def test_palette_dialog_creation(self):
        from rk855.gui.remap_dialog import KeyPaletteDialog
        dlg = KeyPaletteDialog()
        self.assertIsNotNone(dlg)


class TestMainWindowTabs(unittest.TestCase):

    @unittest.skipUnless(PYQT_AVAILABLE, "PyQt6 is required for window tests")
    def test_window_tabs_split(self):
        from rk855.gui.window import RK855MainWindow
        from rk855.device import MockRK855Device
        from rk855.config import ConfigManager
        import tempfile
        import os

        with tempfile.TemporaryDirectory() as tmpdir:
            cfg = ConfigManager(config_dir=tmpdir)
            dev = MockRK855Device()

            win = RK855MainWindow(device=dev, config_manager=cfg, start_tab="lighting")
            self.assertEqual(win.tabs.count(), 3)
            self.assertIn("Lighting", win.tabs.tabText(0))
            self.assertIn("Remapping", win.tabs.tabText(1))
            self.assertIn("Tester", win.tabs.tabText(2))
            self.assertEqual(win.tabs.currentIndex(), 0)

            # Test start_tab="remap"
            win_remap = RK855MainWindow(device=dev, config_manager=cfg, start_tab="remap")
            self.assertEqual(win_remap.tabs.currentIndex(), 1)

            # Test start_tab="tester"
            win_tester = RK855MainWindow(device=dev, config_manager=cfg, start_tab="tester")
            self.assertEqual(win_tester.tabs.currentIndex(), 2)

            # Test key selection updates label and remap canvas
            k_space = CODE_TO_KEY[0x2C]
            win.remap_canvas.key_selected.emit(k_space)
            self.assertIn("Space", win.key_info_label.text())
            self.assertEqual(win.remap_canvas.selected_key.code, 0x2C)
            self.assertEqual(win.keyboard_canvas.selected_key.code, 0x2C)


if __name__ == "__main__":
    unittest.main()

