"""Interactive Key Switch Tester for Royal Kludge RK68 / RK855."""

from typing import Dict, List, Optional, Set, Tuple
from ..constants import LAYOUT_CANVAS_WIDTH, LAYOUT_CANVAS_HEIGHT
from ..layout import KEY_DEFINITIONS, KeyDefinition, CODE_TO_KEY, NAME_TO_KEY

try:
    from PyQt6.QtCore import QPointF, QRectF, Qt, pyqtSignal
    from PyQt6.QtGui import QBrush, QColor, QFont, QPainter, QPen
    from PyQt6.QtWidgets import (
        QApplication,
        QCheckBox,
        QFrame,
        QHBoxLayout,
        QLabel,
        QProgressBar,
        QPushButton,
        QVBoxLayout,
        QWidget,
    )
    PYQT_AVAILABLE = True
except ImportError:
    PYQT_AVAILABLE = False
    QWidget = object
    pyqtSignal = lambda *args: None


class KeyMapper:
    """Resolves Qt key events and Linux evdev scancodes to RK68 KeyDefinitions."""

    # Explicit mapping from Qt.Key constants to RK68 HID usage codes
    QT_KEY_MAP: Dict[int, int] = {}

    @classmethod
    def _init_map(cls):
        if not PYQT_AVAILABLE or cls.QT_KEY_MAP:
            return

        cls.QT_KEY_MAP = {
            # Row 1
            Qt.Key.Key_Escape: 0x29,
            Qt.Key.Key_1: 0x1E,
            Qt.Key.Key_Exclam: 0x1E,
            Qt.Key.Key_2: 0x1F,
            Qt.Key.Key_At: 0x1F,
            Qt.Key.Key_3: 0x20,
            Qt.Key.Key_NumberSign: 0x20,
            Qt.Key.Key_4: 0x21,
            Qt.Key.Key_Dollar: 0x21,
            Qt.Key.Key_5: 0x22,
            Qt.Key.Key_Percent: 0x22,
            Qt.Key.Key_6: 0x23,
            Qt.Key.Key_AsciiCircum: 0x23,
            Qt.Key.Key_7: 0x24,
            Qt.Key.Key_Ampersand: 0x24,
            Qt.Key.Key_8: 0x25,
            Qt.Key.Key_Asterisk: 0x25,
            Qt.Key.Key_9: 0x26,
            Qt.Key.Key_ParenLeft: 0x26,
            Qt.Key.Key_0: 0x27,
            Qt.Key.Key_ParenRight: 0x27,
            Qt.Key.Key_Minus: 0x2D,
            Qt.Key.Key_Underscore: 0x2D,
            Qt.Key.Key_Equal: 0x2E,
            Qt.Key.Key_Plus: 0x2E,
            Qt.Key.Key_Backspace: 0x2A,
            Qt.Key.Key_QuoteLeft: 0x35,
            Qt.Key.Key_AsciiTilde: 0x35,

            # Row 2
            Qt.Key.Key_Tab: 0x2B,
            Qt.Key.Key_Q: 0x14,
            Qt.Key.Key_W: 0x1A,
            Qt.Key.Key_E: 0x08,
            Qt.Key.Key_R: 0x15,
            Qt.Key.Key_T: 0x17,
            Qt.Key.Key_Y: 0x1C,
            Qt.Key.Key_U: 0x18,
            Qt.Key.Key_I: 0x0C,
            Qt.Key.Key_O: 0x12,
            Qt.Key.Key_P: 0x13,
            Qt.Key.Key_BracketLeft: 0x2F,
            Qt.Key.Key_BraceLeft: 0x2F,
            Qt.Key.Key_BracketRight: 0x30,
            Qt.Key.Key_BraceRight: 0x30,
            Qt.Key.Key_Backslash: 0x31,
            Qt.Key.Key_Bar: 0x31,
            Qt.Key.Key_Delete: 0x4C,

            # Row 3
            Qt.Key.Key_CapsLock: 0x39,
            Qt.Key.Key_A: 0x04,
            Qt.Key.Key_S: 0x16,
            Qt.Key.Key_D: 0x07,
            Qt.Key.Key_F: 0x09,
            Qt.Key.Key_G: 0x0A,
            Qt.Key.Key_H: 0x0B,
            Qt.Key.Key_J: 0x0D,
            Qt.Key.Key_K: 0x0E,
            Qt.Key.Key_L: 0x0F,
            Qt.Key.Key_Semicolon: 0x33,
            Qt.Key.Key_Colon: 0x33,
            Qt.Key.Key_Apostrophe: 0x34,
            Qt.Key.Key_QuoteDbl: 0x34,
            Qt.Key.Key_Return: 0x28,
            Qt.Key.Key_Enter: 0x28,
            Qt.Key.Key_PageUp: 0x4B,

            # Row 4
            Qt.Key.Key_Z: 0x1D,
            Qt.Key.Key_X: 0x1B,
            Qt.Key.Key_C: 0x06,
            Qt.Key.Key_V: 0x19,
            Qt.Key.Key_B: 0x05,
            Qt.Key.Key_N: 0x11,
            Qt.Key.Key_M: 0x10,
            Qt.Key.Key_Comma: 0x36,
            Qt.Key.Key_Less: 0x36,
            Qt.Key.Key_Period: 0x37,
            Qt.Key.Key_Greater: 0x37,
            Qt.Key.Key_Slash: 0x38,
            Qt.Key.Key_Question: 0x38,
            Qt.Key.Key_Up: 0x52,
            Qt.Key.Key_PageDown: 0x4E,

            # Row 5
            Qt.Key.Key_Meta: 0xE3,
            Qt.Key.Key_Super_L: 0xE3,
            Qt.Key.Key_Super_R: 0xE3,
            Qt.Key.Key_Space: 0x2C,
            Qt.Key.Key_Left: 0x50,
            Qt.Key.Key_Down: 0x51,
            Qt.Key.Key_Right: 0x4F,
        }

    @classmethod
    def resolve_key(
        cls,
        qt_key: int,
        native_scancode: int = 0,
        text: str = ""
    ) -> Optional[KeyDefinition]:
        """Resolve a Qt key press event into an RK68 KeyDefinition."""
        cls._init_map()

        # 1. Modifiers with Left vs Right distinction via native scancode (Linux evdev / X11)
        # Shift
        if qt_key == Qt.Key.Key_Shift:
            if native_scancode in (62, 54):  # X11 / evdev RShift
                return CODE_TO_KEY.get(0xE5)
            return CODE_TO_KEY.get(0xE1)  # LShift default

        # Control
        if qt_key == Qt.Key.Key_Control:
            if native_scancode in (105, 97):  # X11 / evdev RCtrl
                return CODE_TO_KEY.get(0xE4)
            return CODE_TO_KEY.get(0xE0)  # LCtrl default

        # Alt
        if qt_key == Qt.Key.Key_Alt:
            if native_scancode in (108, 100):  # X11 / evdev RAlt
                return CODE_TO_KEY.get(0xE6)
            return CODE_TO_KEY.get(0xE2)  # LAlt default

        # 2. Direct Qt Key mapping lookup
        if qt_key in cls.QT_KEY_MAP:
            code = cls.QT_KEY_MAP[qt_key]
            if code in CODE_TO_KEY:
                return CODE_TO_KEY[code]

        # 3. Fallback: match by typed text character
        if text:
            char = text.upper()
            if char in NAME_TO_KEY:
                return NAME_TO_KEY[char]
            for k in KEY_DEFINITIONS:
                if char in k.desc:
                    return k

        return None


class KeyTesterCanvas(QWidget):
    """Interactive visual keyboard switch matrix specifically designed for key testing."""

    key_clicked = pyqtSignal(object) if PYQT_AVAILABLE else None

    def __init__(self, parent=None):
        if not PYQT_AVAILABLE:
            super().__init__()
            return

        super().__init__(parent)
        self.setMinimumSize(800, 274)

        # Tester states
        self.pressed_keys: Set[int] = set()    # Codes of keys currently physically held down
        self.tested_keys: Set[int] = set()     # Codes of keys that have actuated at least once
        self.hovered_key: Optional[KeyDefinition] = None

        # Color schemes for tester states
        self.color_untested_fill = QColor(40, 42, 48)
        self.color_untested_border = QColor(65, 68, 78)
        self.color_untested_text = QColor(190, 195, 205)

        # Pressed state: Electric Cyan glow
        self.color_pressed_fill = QColor(0, 229, 255)
        self.color_pressed_border = QColor(128, 245, 255)
        self.color_pressed_text = QColor(10, 20, 25)

        # Tested state: Emerald Green confirmation
        self.color_tested_fill = QColor(46, 125, 50)
        self.color_tested_border = QColor(129, 199, 132)
        self.color_tested_text = QColor(255, 255, 255)

        self.setMouseTracking(True)

    def set_key_pressed(self, code: int, is_pressed: bool):
        """Update active press state and record tested history."""
        if is_pressed:
            self.pressed_keys.add(code)
            self.tested_keys.add(code)
        else:
            self.pressed_keys.discard(code)
        self.update()

    def reset_tester(self):
        """Reset all tested and pressed keys back to untested state."""
        self.pressed_keys.clear()
        self.tested_keys.clear()
        self.update()

    def get_tested_count(self) -> int:
        """Return the count of uniquely tested keys."""
        return len(self.tested_keys)

    def paintEvent(self, event):
        if not PYQT_AVAILABLE:
            return

        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        # Scaling ratio matching native 800x274 layout coordinates
        sx = self.width() / LAYOUT_CANVAS_WIDTH
        sy = self.height() / LAYOUT_CANVAS_HEIGHT

        # Background bezel
        painter.fillRect(QRectF(0, 0, self.width(), self.height()), QColor(22, 23, 27))

        font = QFont("Segoe UI", int(9 * min(sx, sy)), QFont.Weight.Bold)
        painter.setFont(font)

        for k in KEY_DEFINITIONS:
            kx = k.rect_left * sx
            ky = k.rect_top * sy
            kw = k.rect_width * sx
            kh = k.rect_height * sy
            key_rect = QRectF(kx, ky, kw, kh)

            is_pressed = k.code in self.pressed_keys
            is_tested = k.code in self.tested_keys

            # Determine colors based on key actuation state
            if is_pressed:
                fill_color = self.color_pressed_fill
                border_color = self.color_pressed_border
                text_color = self.color_pressed_text
                border_width = 2.4
            elif is_tested:
                fill_color = self.color_tested_fill
                border_color = self.color_tested_border
                text_color = self.color_tested_text
                border_width = 1.6
            elif k == self.hovered_key:
                fill_color = QColor(60, 64, 75)
                border_color = QColor(90, 95, 110)
                text_color = QColor(240, 240, 245)
                border_width = 1.2
            else:
                fill_color = self.color_untested_fill
                border_color = self.color_untested_border
                text_color = self.color_untested_text
                border_width = 1.0

            # Draw keycap body
            painter.setBrush(QBrush(fill_color))
            painter.setPen(QPen(border_color, border_width))
            painter.drawRoundedRect(key_rect, 5.0, 5.0)

            # Draw key label
            painter.setPen(QPen(text_color))
            label = k.desc
            # Highlight Fn key firmware note
            if k.code == 0xAF:
                label = "Fn*"
            painter.drawText(key_rect, Qt.AlignmentFlag.AlignCenter, label)

    def mousePressEvent(self, event):
        if not PYQT_AVAILABLE or event.button() != Qt.MouseButton.LeftButton:
            return

        sx = self.width() / LAYOUT_CANVAS_WIDTH
        sy = self.height() / LAYOUT_CANVAS_HEIGHT

        for k in KEY_DEFINITIONS:
            kx = k.rect_left * sx
            ky = k.rect_top * sy
            kw = k.rect_width * sx
            kh = k.rect_height * sy
            if QRectF(kx, ky, kw, kh).contains(event.position()):
                # Allow mouse click testing as well
                self.set_key_pressed(k.code, True)
                if self.key_clicked:
                    self.key_clicked.emit(k)
                break

    def mouseReleaseEvent(self, event):
        if not PYQT_AVAILABLE or event.button() != Qt.MouseButton.LeftButton:
            return
        # Release all mouse-pressed keys
        for code in list(self.pressed_keys):
            self.set_key_pressed(code, False)

    def mouseMoveEvent(self, event):
        if not PYQT_AVAILABLE:
            return

        sx = self.width() / LAYOUT_CANVAS_WIDTH
        sy = self.height() / LAYOUT_CANVAS_HEIGHT

        hover = None
        for k in KEY_DEFINITIONS:
            kx = k.rect_left * sx
            ky = k.rect_top * sy
            kw = k.rect_width * sx
            kh = k.rect_height * sy
            if QRectF(kx, ky, kw, kh).contains(event.position()):
                hover = k
                break
        if hover != self.hovered_key:
            self.hovered_key = hover
            self.update()


class KeyTesterWidget(QWidget):
    """Full-featured keyboard actuation test tab."""

    def __init__(self, parent=None):
        if not PYQT_AVAILABLE:
            super().__init__()
            return

        super().__init__(parent)
        self.history: List[str] = []
        self.sound_enabled: bool = False
        self.total_keys = len(KEY_DEFINITIONS)

        # Allow receiving key events directly
        self.setFocusPolicy(Qt.FocusPolicy.StrongFocus)

        self._init_ui()

    def _init_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(12)
        layout.setContentsMargins(14, 14, 14, 14)

        # Top Control & Stats Bar
        stats_box = QFrame()
        stats_box.setStyleSheet("""
            QFrame {
                background-color: #252830;
                border: 1px solid #383c48;
                border-radius: 6px;
                padding: 6px 12px;
            }
        """)
        stats_layout = QHBoxLayout(stats_box)
        stats_layout.setContentsMargins(6, 6, 6, 6)

        self.stats_label = QLabel(f"Tested: 0 / {self.total_keys} Keys (0%)")
        self.stats_label.setStyleSheet("font-weight: bold; font-size: 14px; color: #61afef;")
        stats_layout.addWidget(self.stats_label)

        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, self.total_keys)
        self.progress_bar.setValue(0)
        self.progress_bar.setFixedHeight(14)
        self.progress_bar.setTextVisible(False)
        self.progress_bar.setStyleSheet("""
            QProgressBar {
                background-color: #1e2025;
                border: 1px solid #3e4451;
                border-radius: 7px;
            }
            QProgressBar::chunk {
                background-color: #98c379;
                border-radius: 6px;
            }
        """)
        stats_layout.addWidget(self.progress_bar, stretch=1)

        self.sound_checkbox = QCheckBox("Audio Feedback")
        self.sound_checkbox.setStyleSheet("color: #abb2bf; font-size: 13px;")
        self.sound_checkbox.toggled.connect(self._on_sound_toggled)
        stats_layout.addWidget(self.sound_checkbox)

        self.reset_btn = QPushButton("Reset Test")
        self.reset_btn.setStyleSheet("""
            QPushButton {
                background-color: #e06c75;
                color: #ffffff;
                border: none;
                border-radius: 4px;
                padding: 6px 14px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #be5059;
            }
        """)
        self.reset_btn.clicked.connect(self.reset_test)
        stats_layout.addWidget(self.reset_btn)

        layout.addWidget(stats_box)

        # Visual Keyboard Matrix Canvas
        self.canvas = KeyTesterCanvas(self)
        self.canvas.key_clicked.connect(self._on_key_clicked)
        layout.addWidget(self.canvas, alignment=Qt.AlignmentFlag.AlignCenter)

        # Bottom Inspector & Keystroke History
        bottom_box = QFrame()
        bottom_box.setStyleSheet("""
            QFrame {
                background-color: #252830;
                border: 1px solid #383c48;
                border-radius: 6px;
                padding: 6px 12px;
            }
        """)
        bottom_layout = QHBoxLayout(bottom_box)
        bottom_layout.setContentsMargins(8, 8, 8, 8)

        # Live Key Inspector Details
        inspector_layout = QVBoxLayout()
        self.inspector_key_label = QLabel("Last Key Pressed: None")
        self.inspector_key_label.setStyleSheet("font-weight: bold; font-size: 14px; color: #e5c07b;")
        inspector_layout.addWidget(self.inspector_key_label)

        self.inspector_code_label = QLabel("USB Scancode: -- | Event: IDLE | Native Code: --")
        self.inspector_code_label.setStyleSheet("color: #abb2bf; font-size: 12px;")
        inspector_layout.addWidget(self.inspector_code_label)
        bottom_layout.addLayout(inspector_layout, stretch=2)

        # Keystroke Stream History
        history_layout = QVBoxLayout()
        history_title = QLabel("Keystroke History (Recent 12):")
        history_title.setStyleSheet("font-size: 11px; color: #5c6370; font-weight: bold;")
        history_layout.addWidget(history_title)

        self.history_label = QLabel("Waiting for keystrokes...")
        self.history_label.setStyleSheet("color: #98c379; font-family: monospace; font-size: 13px;")
        history_layout.addWidget(self.history_label)
        bottom_layout.addLayout(history_layout, stretch=3)

        layout.addWidget(bottom_box)

        # Informational Note
        info_label = QLabel(
            "💡 Press keys on your keyboard to test switches. "
            "Keys illuminate cyan while pressed and turn green once registered. "
            "(*Note: The Fn key is processed internally by RK68 firmware for onboard shortcuts and does not transmit an OS scancode.)"
        )
        info_label.setWordWrap(True)
        info_label.setStyleSheet("color: #5c6370; font-size: 11px; font-style: italic; margin-top: 2px;")
        layout.addWidget(info_label)

    def _on_sound_toggled(self, checked: bool):
        self.sound_enabled = checked

    def _on_key_clicked(self, key_def: KeyDefinition):
        self._record_keystroke(key_def, "MOUSE_CLICK", 0)

    def reset_test(self):
        """Reset test counts, canvas colors, and history."""
        self.canvas.reset_tester()
        self.history.clear()
        self.stats_label.setText(f"Tested: 0 / {self.total_keys} Keys (0%)")
        self.progress_bar.setValue(0)
        self.inspector_key_label.setText("Last Key Pressed: None")
        self.inspector_code_label.setText("USB Scancode: -- | Event: IDLE | Native Code: --")
        self.history_label.setText("Waiting for keystrokes...")
        self.setFocus()

    def handle_key_event(self, event, is_press: bool) -> bool:
        """Process a QKeyEvent for key testing. Returns True if consumed."""
        if not PYQT_AVAILABLE:
            return False

        key_def = KeyMapper.resolve_key(
            qt_key=event.key(),
            native_scancode=event.nativeScanCode(),
            text=event.text(),
        )

        if not key_def:
            # Still update code inspector for unmapped keys
            if is_press and not event.isAutoRepeat():
                self.inspector_key_label.setText(f"Last Key Pressed: Unmapped (0x{event.key():X})")
                self.inspector_code_label.setText(
                    f"USB Scancode: -- | Event: PRESSED | Native Code: {event.nativeScanCode()}"
                )
            return False

        if is_press:
            if not event.isAutoRepeat():
                self.canvas.set_key_pressed(key_def.code, True)
                self._record_keystroke(key_def, "KEY_PRESS", event.nativeScanCode())
                if self.sound_enabled:
                    QApplication.beep()
        else:
            if not event.isAutoRepeat():
                self.canvas.set_key_pressed(key_def.code, False)
                self.inspector_code_label.setText(
                    f"USB Scancode: 0x{key_def.code:02X} | Event: RELEASED | Native Code: {event.nativeScanCode()}"
                )

        return True

    def _record_keystroke(self, key_def: KeyDefinition, event_type: str, native_code: int):
        tested_count = self.canvas.get_tested_count()
        percent = int((tested_count / self.total_keys) * 100)

        self.stats_label.setText(f"Tested: {tested_count} / {self.total_keys} Keys ({percent}%)")
        self.progress_bar.setValue(tested_count)

        self.inspector_key_label.setText(f"Last Key Pressed: {key_def.desc} ({key_def.name})")
        self.inspector_code_label.setText(
            f"USB Scancode: 0x{key_def.code:02X} | Event: PRESSED | Native Code: {native_code}"
        )

        # Update rolling history (up to 12 items)
        self.history.append(key_def.desc)
        if len(self.history) > 12:
            self.history.pop(0)
        self.history_label.setText("  ▶  ".join(self.history))
