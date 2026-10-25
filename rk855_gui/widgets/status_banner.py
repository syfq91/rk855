"""Header status banner widget for device connection state and simulator toggle."""

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QWidget,
    QHBoxLayout,
    QLabel,
    QPushButton,
    QCheckBox,
    QFrame,
)
from ..core.device import DeviceMode


class StatusBanner(QFrame):
    """Shows keyboard connection badge, simulator toggle, and refresh trigger."""

    refresh_requested = pyqtSignal()
    simulator_toggled = pyqtSignal(bool)

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        self.setFrameShape(QFrame.Shape.StyledPanel)
        self.setStyleSheet("""
            StatusBanner {
                background-color: #1a1c23;
                border-bottom: 1px solid #2d3139;
                padding: 6px 12px;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 6, 12, 6)
        layout.setSpacing(12)

        # Title / Model
        self.lbl_title = QLabel("RK855 Configurator & Flasher")
        self.lbl_title.setStyleSheet("font-size: 14px; font-weight: bold; color: #61afef;")
        layout.addWidget(self.lbl_title)

        # Status badge
        self.badge = QLabel()
        self.badge.setStyleSheet("""
            background-color: #282c34;
            color: #abb2bf;
            border-radius: 12px;
            padding: 3px 10px;
            font-size: 11px;
            font-weight: bold;
        """)
        layout.addWidget(self.badge)

        layout.addStretch()

        # Simulator checkbox
        self.chk_mock = QCheckBox("Simulator / Demo Mode")
        self.chk_mock.setToolTip("Emulate keyboard and bootloader without physical hardware")
        self.chk_mock.toggled.connect(self.simulator_toggled.emit)
        layout.addWidget(self.chk_mock)

        # Refresh button
        self.btn_refresh = QPushButton("Refresh Device")
        self.btn_refresh.setStyleSheet("padding: 3px 10px; font-size: 11px;")
        self.btn_refresh.clicked.connect(self.refresh_requested.emit)
        layout.addWidget(self.btn_refresh)

        self.update_status(DeviceMode.DISCONNECTED, is_simulator=False)

    def update_status(self, mode: DeviceMode, is_simulator: bool = False, details: str | None = None) -> None:
        if is_simulator:
            self.badge.setText("🔵 SIMULATOR MODE ACTIVE")
            self.badge.setStyleSheet("""
                background-color: #1c3d5a;
                color: #61afef;
                border: 1px solid #2b6cb0;
                border-radius: 12px;
                padding: 3px 10px;
                font-size: 11px;
                font-weight: bold;
            """)
            return

        if mode == DeviceMode.NORMAL:
            self.badge.setText("🟢 CONNECTED (Normal Mode: 0x0C45:0x8018)")
            self.badge.setStyleSheet("""
                background-color: #1e3a2f;
                color: #98c379;
                border: 1px solid #3e6848;
                border-radius: 12px;
                padding: 3px 10px;
                font-size: 11px;
                font-weight: bold;
            """)
        elif mode == DeviceMode.BOOTLOADER:
            self.badge.setText("🟠 BOOTLOADER (Sonix ISP: 0x0C45:0x82AA)")
            self.badge.setStyleSheet("""
                background-color: #3d2f1e;
                color: #e5c07b;
                border: 1px solid #7a5e3e;
                border-radius: 12px;
                padding: 3px 10px;
                font-size: 11px;
                font-weight: bold;
            """)
        elif mode == DeviceMode.PERMISSION_DENIED:
            self.badge.setText("🔴 PERMISSION DENIED (Check udev rules)")
            self.badge.setStyleSheet("""
                background-color: #3d1e22;
                color: #e06c75;
                border: 1px solid #7a3e44;
                border-radius: 12px;
                padding: 3px 10px;
                font-size: 11px;
                font-weight: bold;
            """)
        else:
            self.badge.setText("⚪ NO KEYBOARD DETECTED")
            self.badge.setStyleSheet("""
                background-color: #282c34;
                color: #abb2bf;
                border: 1px solid #3e4451;
                border-radius: 12px;
                padding: 3px 10px;
                font-size: 11px;
                font-weight: bold;
            """)
