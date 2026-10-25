"""Interactive 68-key physical layout widget for RK855 with per-key RGB visualization."""

from PyQt6.QtCore import Qt, QRectF, QPointF, pyqtSignal
from PyQt6.QtGui import (
    QPainter,
    QColor,
    QPen,
    QBrush,
    QFont,
    QMouseEvent,
    QPaintEvent,
)
from PyQt6.QtWidgets import QWidget, QToolTip

from ..core.constants import PHYSICAL_KEYS, KeyDefinition, KEY_PRESETS


class KeyboardView(QWidget):
    """Renders the physical 68-key layout with individual RGB illumination and key selection."""

    selection_changed = pyqtSignal(list)  # list of key indices
    key_clicked = pyqtSignal(int)         # single key index

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self.setMinimumSize(780, 270)
        self.setMouseTracking(True)

        # 68 RGB color tuples (R, G, B)
        self._key_colors: list[tuple[int, int, int]] = [(0, 200, 255)] * 68
        self._selected_indices: set[int] = set()
        self._hover_index: int | None = None

        # Drag selection state
        self._is_dragging: bool = False
        self._drag_start: QPointF | None = None
        self._drag_current: QPointF | None = None

        # Precompute native layout dimensions
        # Original layout size from KeyboardLayout.xml: 800 x 274
        self.native_width = 802.0
        self.native_height = 276.0

    def get_key_colors(self) -> list[tuple[int, int, int]]:
        return list(self._key_colors)

    def set_key_color(self, index: int, color: tuple[int, int, int]) -> None:
        if 0 <= index < 68:
            self._key_colors[index] = color
            self.update()

    def set_all_colors(self, colors: list[tuple[int, int, int]]) -> None:
        self._key_colors = list(colors[:68])
        if len(self._key_colors) < 68:
            self._key_colors.extend([(0, 0, 0)] * (68 - len(self._key_colors)))
        self.update()

    def set_selected_keys_color(self, color: tuple[int, int, int]) -> None:
        """Applies color to selected keys, or all keys if nothing is selected."""
        targets = self._selected_indices if self._selected_indices else set(range(68))
        for idx in targets:
            self._key_colors[idx] = color
        self.update()

    def get_selected_indices(self) -> list[int]:
        return sorted(list(self._selected_indices))

    def select_preset(self, preset_name: str) -> None:
        if preset_name in KEY_PRESETS:
            self._selected_indices = set(KEY_PRESETS[preset_name])
            self.selection_changed.emit(self.get_selected_indices())
            self.update()

    def select_all(self) -> None:
        self._selected_indices = set(range(68))
        self.selection_changed.emit(self.get_selected_indices())
        self.update()

    def clear_selection(self) -> None:
        self._selected_indices.clear()
        self.selection_changed.emit([])
        self.update()

    def _get_transform(self) -> tuple[float, float, float]:
        """Calculates scale factor and offsets to maintain aspect ratio."""
        w = self.width()
        h = self.height()
        scale = min(w / self.native_width, h / self.native_height)
        offset_x = (w - self.native_width * scale) / 2.0
        offset_y = (h - self.native_height * scale) / 2.0
        return scale, offset_x, offset_y

    def _key_rect(self, key: KeyDefinition, scale: float, ox: float, oy: float) -> QRectF:
        return QRectF(
            ox + key.left * scale,
            oy + key.top * scale,
            key.width * scale,
            key.height * scale,
        )

    def _key_at_point(self, pos: QPointF) -> int | None:
        scale, ox, oy = self._get_transform()
        for key in PHYSICAL_KEYS:
            rect = self._key_rect(key, scale, ox, oy)
            if rect.contains(pos):
                return key.index
        return None

    def paintEvent(self, event: QPaintEvent) -> None:
        painter = QPainter(self)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)

        # Background base plate
        scale, ox, oy = self._get_transform()
        board_rect = QRectF(ox - 3, oy - 3, self.native_width * scale + 6, self.native_height * scale + 6)
        painter.setPen(QPen(QColor(50, 54, 66), 2))
        painter.setBrush(QBrush(QColor(22, 24, 29)))
        painter.drawRoundedRect(board_rect, 10, 10)

        # Inner case bevel
        case_rect = QRectF(ox, oy, self.native_width * scale, self.native_height * scale)
        painter.setPen(QPen(QColor(32, 35, 42), 1))
        painter.setBrush(QBrush(QColor(16, 17, 20)))
        painter.drawRoundedRect(case_rect, 8, 8)

        # Render each key
        for key in PHYSICAL_KEYS:
            idx = key.index
            rect = self._key_rect(key, scale, ox, oy)
            r, g, b = self._key_colors[idx]
            glow_color = QColor(r, g, b)
            is_selected = idx in self._selected_indices
            is_hover = idx == self._hover_index

            # Key backlight glow under keycap
            if r > 10 or g > 10 or b > 10:
                glow_rect = rect.adjusted(-1.5, -1.5, 1.5, 1.5)
                glow_brush = QColor(r, g, b, 45)
                painter.setPen(Qt.PenStyle.NoPen)
                painter.setBrush(QBrush(glow_brush))
                painter.drawRoundedRect(glow_rect, 6, 6)

            # Keycap body
            if is_selected:
                key_bg = QColor(42, 58, 82)
                border_pen = QPen(QColor(80, 160, 255), 2.2)
            elif is_hover:
                key_bg = QColor(48, 52, 62)
                border_pen = QPen(QColor(180, 190, 210), 1.6)
            else:
                key_bg = QColor(32, 35, 43)
                border_pen = QPen(QColor(55, 60, 72), 1.2)

            painter.setPen(border_pen)
            painter.setBrush(QBrush(key_bg))
            painter.drawRoundedRect(rect, 5, 5)

            # Accent RGB indicator dot/bar inside key
            indicator_h = 3.5 * scale
            indicator_rect = QRectF(rect.left() + 4 * scale, rect.bottom() - indicator_h - 3 * scale, rect.width() - 8 * scale, indicator_h)
            painter.setPen(Qt.PenStyle.NoPen)
            painter.setBrush(QBrush(glow_color))
            painter.drawRoundedRect(indicator_rect, 2, 2)

            # Keycap Label Text
            painter.setFont(QFont("Segoe UI", max(7, int(10 * scale)), QFont.Weight.Bold))
            if is_selected:
                painter.setPen(QColor(255, 255, 255))
            else:
                painter.setPen(QColor(210, 215, 225))

            text_rect = rect.adjusted(0, -2 * scale, 0, -4 * scale)
            painter.drawText(text_rect, Qt.AlignmentFlag.AlignCenter, key.label)

        # Drag selection rubber band
        if self._is_dragging and self._drag_start and self._drag_current:
            band = QRectF(self._drag_start, self._drag_current).normalized()
            painter.setPen(QPen(QColor(0, 180, 255, 200), 1, Qt.PenStyle.DashLine))
            painter.setBrush(QBrush(QColor(0, 140, 255, 40)))
            painter.drawRect(band)

    def mousePressEvent(self, event: QMouseEvent) -> None:
        if event.button() == Qt.MouseButton.LeftButton:
            pos = event.position()
            idx = self._key_at_point(pos)
            modifiers = event.modifiers()

            if idx is not None:
                if modifiers & (Qt.KeyboardModifier.ControlModifier | Qt.KeyboardModifier.ShiftModifier):
                    if idx in self._selected_indices:
                        self._selected_indices.remove(idx)
                    else:
                        self._selected_indices.add(idx)
                else:
                    if idx in self._selected_indices and len(self._selected_indices) == 1:
                        self._selected_indices.clear()
                    else:
                        self._selected_indices = {idx}

                self.selection_changed.emit(self.get_selected_indices())
                self.key_clicked.emit(idx)
                self.update()
            else:
                # Start box drag
                self._is_dragging = True
                self._drag_start = pos
                self._drag_current = pos
                if not (modifiers & (Qt.KeyboardModifier.ControlModifier | Qt.KeyboardModifier.ShiftModifier)):
                    self._selected_indices.clear()
                    self.selection_changed.emit([])
                    self.update()

    def mouseMoveEvent(self, event: QMouseEvent) -> None:
        pos = event.position()
        if self._is_dragging:
            self._drag_current = pos
            # Update selected keys inside drag rectangle
            scale, ox, oy = self._get_transform()
            band = QRectF(self._drag_start, self._drag_current).normalized()
            for key in PHYSICAL_KEYS:
                krect = self._key_rect(key, scale, ox, oy)
                if band.intersects(krect):
                    self._selected_indices.add(key.index)
            self.selection_changed.emit(self.get_selected_indices())
            self.update()
            return

        idx = self._key_at_point(pos)
        if idx != self._hover_index:
            self._hover_index = idx
            self.update()
            if idx is not None:
                key = PHYSICAL_KEYS[idx]
                r, g, b = self._key_colors[idx]
                hex_col = f"#{r:02X}{g:02X}{b:02X}"
                QToolTip.showText(
                    event.globalPosition().toPoint(),
                    f"<b>{key.label}</b> (Key {idx}, Code: 0x{key.scancode:02X})<br/>RGB: {hex_col}",
                    self,
                )

    def mouseReleaseEvent(self, event: QMouseEvent) -> None:
        if event.button() == Qt.MouseButton.LeftButton and self._is_dragging:
            self._is_dragging = False
            self._drag_start = None
            self._drag_current = None
            self.update()
