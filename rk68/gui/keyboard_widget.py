"""Interactive 68-key visual keyboard canvas widget."""

from typing import Dict, Optional, Tuple
from ..constants import LAYOUT_CANVAS_WIDTH, LAYOUT_CANVAS_HEIGHT
from ..layout import KEY_DEFINITIONS, KeyDefinition

try:
    from PyQt6.QtCore import QPointF, QRectF, Qt, pyqtSignal
    from PyQt6.QtGui import QBrush, QColor, QFont, QPainter, QPen
    from PyQt6.QtWidgets import QWidget
    PYQT_AVAILABLE = True
except ImportError:
    PYQT_AVAILABLE = False
    QWidget = object
    pyqtSignal = lambda *args: None


class KeyboardCanvas(QWidget):
    """Visual rendered 68-key keyboard matrix matching official layout coordinates."""

    key_selected = pyqtSignal(object) if PYQT_AVAILABLE else None

    def __init__(self, parent=None):
        if not PYQT_AVAILABLE:
            super().__init__()
            return

        super().__init__(parent)
        self.setMinimumSize(800, 274)
        self.selected_key: Optional[KeyDefinition] = None
        self.key_colors: Dict[int, Tuple[int, int, int]] = {}
        self.default_key_color = QColor(45, 45, 50)
        self.border_color = QColor(70, 70, 80)
        self.selected_border_color = QColor(0, 180, 255)
        self.text_color = QColor(220, 220, 225)
        self.setMouseTracking(True)
        self.hovered_key: Optional[KeyDefinition] = None

    def set_key_color(self, key_code: int, r: int, g: int, b: int):
        self.key_colors[key_code] = (r, g, b)
        if PYQT_AVAILABLE:
            self.update()

    def set_all_colors(self, r: int, g: int, b: int):
        for k in KEY_DEFINITIONS:
            self.key_colors[k.code] = (r, g, b)
        if PYQT_AVAILABLE:
            self.update()

    def paintEvent(self, event):
        if not PYQT_AVAILABLE:
            return

        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        # Scale factor relative to native 800x274 canvas
        sx = self.width() / LAYOUT_CANVAS_WIDTH
        sy = self.height() / LAYOUT_CANVAS_HEIGHT

        # Draw dark keyboard backplate
        backplate_rect = QRectF(0, 0, self.width(), self.height())
        painter.fillRect(backplate_rect, QColor(25, 25, 30))

        font = QFont("Segoe UI", int(9 * min(sx, sy)), QFont.Weight.Bold)
        painter.setFont(font)

        for k in KEY_DEFINITIONS:
            kx = k.rect_left * sx
            ky = k.rect_top * sy
            kw = k.rect_width * sx
            kh = k.rect_height * sy
            key_rect = QRectF(kx, ky, kw, kh)

            # Key fill color
            if k.code in self.key_colors:
                r, g, b = self.key_colors[k.code]
                fill_color = QColor(r, g, b)
            elif k == self.hovered_key:
                fill_color = QColor(60, 60, 70)
            else:
                fill_color = self.default_key_color

            painter.setBrush(QBrush(fill_color))

            # Border
            if k == self.selected_key:
                painter.setPen(QPen(self.selected_border_color, 2.5))
            else:
                painter.setPen(QPen(self.border_color, 1.2))

            painter.drawRoundedRect(key_rect, 5.0, 5.0)

            # Key label
            # If the fill color is bright, use dark text
            luminance = fill_color.red() * 0.299 + fill_color.green() * 0.587 + fill_color.blue() * 0.114
            label_color = QColor(20, 20, 20) if luminance > 140 else self.text_color
            painter.setPen(QPen(label_color))
            painter.drawText(key_rect, Qt.AlignmentFlag.AlignCenter, k.desc)

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
                self.selected_key = k
                self.update()
                if self.key_selected:
                    self.key_selected.emit(k)
                break

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
