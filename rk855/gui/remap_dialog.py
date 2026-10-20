"""Visual button palette dialog for selecting key remap actions."""

from typing import Optional
from ..layout import RemapTarget, REMAP_TARGETS

try:
    from PyQt6.QtCore import Qt, pyqtSignal
    from PyQt6.QtGui import QFont
    from PyQt6.QtWidgets import (
        QDialog,
        QGridLayout,
        QGroupBox,
        QHBoxLayout,
        QLabel,
        QPushButton,
        QScrollArea,
        QTabWidget,
        QVBoxLayout,
        QWidget,
    )
    PYQT_AVAILABLE = True
except ImportError:
    PYQT_AVAILABLE = False
    QDialog = object
    pyqtSignal = lambda *args: None


class KeyPaletteDialog(QDialog):
    """Visual dialog displaying all available buttons for 1-click remap selection."""

    target_selected = pyqtSignal(object) if PYQT_AVAILABLE else None

    def __init__(self, parent=None, current_code: Optional[int] = None):
        if not PYQT_AVAILABLE:
            super().__init__()
            return

        super().__init__(parent)
        self.selected_target: Optional[RemapTarget] = None
        self.current_code = current_code

        self.setWindowTitle("Select Target Button to Remap")
        self.resize(720, 520)
        self.setStyleSheet("""
            QDialog {
                background-color: #1e1e24;
            }
            QLabel {
                color: #e0e0e0;
                font-size: 13px;
            }
            QGroupBox {
                color: #61afef;
                font-weight: bold;
                border: 1px solid #3e4451;
                margin-top: 8px;
                padding-top: 14px;
                border-radius: 6px;
            }
            QPushButton.keyBtn {
                background-color: #2c313a;
                color: #abb2bf;
                border: 1px solid #4b5263;
                border-radius: 5px;
                font-weight: bold;
                font-size: 11px;
                min-width: 44px;
                min-height: 38px;
                padding: 4px;
            }
            QPushButton.keyBtn:hover {
                background-color: #3e4451;
                color: #61afef;
                border-color: #61afef;
            }
            QPushButton.keyBtn:pressed {
                background-color: #00A8FF;
                color: #1e1e24;
            }
            QScrollArea {
                border: none;
                background-color: transparent;
            }
        """)

        self._init_ui()

    def _init_ui(self):
        layout = QVBoxLayout(self)
        layout.setSpacing(10)
        layout.setContentsMargins(16, 16, 16, 16)

        header_label = QLabel("Click any button below to assign it as the remap target:")
        header_label.setStyleSheet("font-weight: bold; font-size: 14px; color: #61afef;")
        layout.addWidget(header_label)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        container = QWidget()
        container_layout = QVBoxLayout(container)
        container_layout.setSpacing(12)

        # Categorize targets
        categories = {}
        for t in REMAP_TARGETS:
            categories.setdefault(t.category, []).append(t)

        for cat_name, targets in categories.items():
            group = QGroupBox(cat_name)
            grid = QGridLayout(group)
            grid.setSpacing(6)
            grid.setContentsMargins(8, 12, 8, 8)

            columns = 8 if cat_name in ("Letters", "Numbers", "Function Keys") else 4
            row, col = 0, 0
            for t in targets:
                btn = QPushButton(f"{t.desc}\n(0x{t.code:02X})")
                btn.setProperty("class", "keyBtn")
                btn.setStyleSheet("""
                    QPushButton {
                        background-color: #282a32;
                        color: #dcdce1;
                        border: 1px solid #3e4451;
                        border-radius: 5px;
                        font-weight: bold;
                        font-size: 11px;
                        min-width: 58px;
                        min-height: 38px;
                        padding: 4px;
                    }
                    QPushButton:hover {
                        background-color: #383c48;
                        color: #00e5ff;
                        border: 1px solid #00A8FF;
                    }
                """)
                btn.clicked.connect(lambda checked, target=t: self._on_button_clicked(target))
                grid.addWidget(btn, row, col)
                col += 1
                if col >= columns:
                    col = 0
                    row += 1

            container_layout.addWidget(group)

        scroll.setWidget(container)
        layout.addWidget(scroll)

        # Bottom Cancel button
        bottom_layout = QHBoxLayout()
        bottom_layout.addStretch()
        cancel_btn = QPushButton("Cancel")
        cancel_btn.clicked.connect(self.reject)
        bottom_layout.addWidget(cancel_btn)
        layout.addLayout(bottom_layout)

    def _on_button_clicked(self, target: RemapTarget):
        self.selected_target = target
        if self.target_selected:
            self.target_selected.emit(target)
        self.accept()
