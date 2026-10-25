"""Color picker widget with palette swatches, RGB sliders, and hex input."""

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtGui import QColor
from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QLabel,
    QSlider,
    QSpinBox,
    QLineEdit,
    QPushButton,
    QColorDialog,
    QGridLayout,
    QFrame,
)
from ..core.constants import PRESET_COLORS


class ColorPickerWidget(QWidget):
    """Provides palette presets, RGB sliders, hex text box, and color dialog."""

    color_changed = pyqtSignal(tuple)  # (r, g, b)

    def __init__(self, parent: QWidget | None = None, initial_color: tuple[int, int, int] = (0, 200, 255)):
        super().__init__(parent)
        self._r, self._g, self._b = initial_color
        self._updating = False

        self._setup_ui()
        self._update_all_controls()

    def _setup_ui(self) -> None:
        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 8, 8, 8)
        layout.setSpacing(10)

        # 1. Preset Swatches
        swatch_group = QWidget()
        swatch_layout = QGridLayout(swatch_group)
        swatch_layout.setContentsMargins(0, 0, 0, 0)
        swatch_layout.setSpacing(6)

        for i, (name, (r, g, b)) in enumerate(PRESET_COLORS):
            btn = QPushButton()
            btn.setToolTip(f"{name} (#{r:02X}{g:02X}{b:02X})")
            btn.setFixedSize(32, 28)
            btn.setStyleSheet(f"""
                QPushButton {{
                    background-color: rgb({r}, {g}, {b});
                    border: 1px solid #444;
                    border-radius: 4px;
                }}
                QPushButton:hover {{
                    border: 2px solid #fff;
                }}
            """)
            btn.clicked.connect(lambda _, col=(r, g, b): self.set_color(col))
            row = i // 5
            col = i % 5
            swatch_layout.addWidget(btn, row, col)

        layout.addWidget(swatch_group)

        # 2. Sliders Section (R, G, B)
        sliders_box = QWidget()
        sliders_layout = QVBoxLayout(sliders_box)
        sliders_layout.setContentsMargins(0, 0, 0, 0)
        sliders_layout.setSpacing(4)

        self.sliders = {}
        self.spinboxes = {}

        for ch, name, color_code in [("r", "Red", "#ff4444"), ("g", "Green", "#44dd44"), ("b", "Blue", "#4488ff")]:
            row = QHBoxLayout()
            lbl = QLabel(name[:1])
            lbl.setFixedWidth(16)
            lbl.setStyleSheet(f"font-weight: bold; color: {color_code};")

            slider = QSlider(Qt.Orientation.Horizontal)
            slider.setRange(0, 255)
            slider.valueChanged.connect(self._on_slider_changed)

            spin = QSpinBox()
            spin.setRange(0, 255)
            spin.setFixedWidth(54)
            spin.valueChanged.connect(self._on_spin_changed)

            row.addWidget(lbl)
            row.addWidget(slider)
            row.addWidget(spin)
            sliders_layout.addLayout(row)

            self.sliders[ch] = slider
            self.spinboxes[ch] = spin

        layout.addWidget(sliders_box)

        # 3. Preview Swatch + Hex Input + Custom Button
        bottom_row = QHBoxLayout()
        bottom_row.setSpacing(8)

        self.preview_box = QFrame()
        self.preview_box.setFixedSize(36, 28)
        self.preview_box.setFrameShape(QFrame.Shape.Box)
        self.preview_box.setStyleSheet("border-radius: 4px; border: 1px solid #666;")

        self.hex_input = QLineEdit()
        self.hex_input.setPlaceholderText("#RRGGBB")
        self.hex_input.setMaxLength(7)
        self.hex_input.setFixedWidth(80)
        self.hex_input.setStyleSheet("font-family: monospace; font-size: 11px;")
        self.hex_input.textEdited.connect(self._on_hex_edited)

        self.btn_dialog = QPushButton("More...")
        self.btn_dialog.setStyleSheet("padding: 4px 8px;")
        self.btn_dialog.clicked.connect(self._open_color_dialog)

        bottom_row.addWidget(self.preview_box)
        bottom_row.addWidget(self.hex_input)
        bottom_row.addWidget(self.btn_dialog)
        bottom_row.addStretch()

        layout.addLayout(bottom_row)

    def get_color(self) -> tuple[int, int, int]:
        return (self._r, self._g, self._b)

    def set_color(self, color: tuple[int, int, int], emit: bool = True) -> None:
        self._r = max(0, min(255, color[0]))
        self._g = max(0, min(255, color[1]))
        self._b = max(0, min(255, color[2]))
        self._update_all_controls()
        if emit:
            self.color_changed.emit((self._r, self._g, self._b))

    def _update_all_controls(self) -> None:
        self._updating = True
        try:
            self.sliders["r"].setValue(self._r)
            self.sliders["g"].setValue(self._g)
            self.sliders["b"].setValue(self._b)

            self.spinboxes["r"].setValue(self._r)
            self.spinboxes["g"].setValue(self._g)
            self.spinboxes["b"].setValue(self._b)

            hex_str = f"#{self._r:02X}{self._g:02X}{self._b:02X}"
            self.hex_input.setText(hex_str)
            self.preview_box.setStyleSheet(f"background-color: {hex_str}; border-radius: 4px; border: 1px solid #777;")
        finally:
            self._updating = False

    def _on_slider_changed(self) -> None:
        if self._updating:
            return
        self._r = self.sliders["r"].value()
        self._g = self.sliders["g"].value()
        self._b = self.sliders["b"].value()
        self._update_all_controls()
        self.color_changed.emit((self._r, self._g, self._b))

    def _on_spin_changed(self) -> None:
        if self._updating:
            return
        self._r = self.spinboxes["r"].value()
        self._g = self.spinboxes["g"].value()
        self._b = self.spinboxes["b"].value()
        self._update_all_controls()
        self.color_changed.emit((self._r, self._g, self._b))

    def _on_hex_edited(self, text: str) -> None:
        clean = text.strip()
        if clean.startswith("#"):
            clean = clean[1:]
        if len(clean) == 6:
            try:
                r = int(clean[0:2], 16)
                g = int(clean[2:4], 16)
                b = int(clean[4:6], 16)
                self.set_color((r, g, b))
            except ValueError:
                pass

    def _open_color_dialog(self) -> None:
        cur = QColor(self._r, self._g, self._b)
        chosen = QColorDialog.getColor(cur, self, "Select Color")
        if chosen.isValid():
            self.set_color((chosen.red(), chosen.green(), chosen.blue()))
