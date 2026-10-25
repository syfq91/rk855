"""CLI launcher and application startup logic for RK855 GUI."""

import argparse
import sys
from PyQt6.QtCore import Qt, QTimer
from PyQt6.QtGui import QIcon, QPixmap, QPainter, QColor
from PyQt6.QtWidgets import QApplication

from .ui.main_window import MainWindow


def create_app_icon() -> QIcon:
    """Generates an embedded vector-style keyboard icon."""
    pix = QPixmap(64, 64)
    pix.fill(Qt.GlobalColor.transparent)
    painter = QPainter(pix)
    painter.setRenderHint(QPainter.RenderHint.Antialiasing)

    # Keyboard body
    painter.setBrush(QColor(30, 34, 42))
    painter.setPen(QColor(97, 175, 239))
    painter.drawRoundedRect(4, 12, 56, 40, 6, 6)

    # Key rows
    painter.setBrush(QColor(97, 175, 239))
    painter.setPen(Qt.PenStyle.NoPen)
    for row in range(3):
        for col in range(5):
            painter.drawRoundedRect(9 + col * 9, 17 + row * 8, 7, 5, 1, 1)

    # Spacebar
    painter.setBrush(QColor(152, 195, 121))
    painter.drawRoundedRect(18, 41, 28, 5, 1, 1)
    painter.end()
    return QIcon(pix)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Royal Kludge RK855 (RK68) Cross-Platform GUI Configurator & Firmware Flasher",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--mock",
        "--demo",
        action="store_true",
        help="Run in simulator mode without requiring physical RK855 keyboard",
    )
    parser.add_argument(
        "--flasher",
        action="store_true",
        help="Open GUI directly to the Firmware Flasher tab",
    )
    parser.add_argument(
        "--remap",
        action="store_true",
        help="Open GUI directly to the Key Remapping tab",
    )
    parser.add_argument(
        "--tab",
        choices=["lighting", "perkey", "remap", "flasher", "info"],
        default=None,
        help="Specify which tab to open on startup",
    )
    parser.add_argument(
        "--test-exit",
        action="store_true",
        help="Initialize GUI, verify widgets, and exit immediately (for automated testing)",
    )

    args = parser.parse_args(argv if argv is not None else sys.argv[1:])

    # Configure High DPI Scaling
    if hasattr(Qt.ApplicationAttribute, "AA_EnableHighDpiScaling"):
        QApplication.setAttribute(Qt.ApplicationAttribute.AA_EnableHighDpiScaling, True)
    if hasattr(Qt.ApplicationAttribute, "AA_UseHighDpiPixmaps"):
        QApplication.setAttribute(Qt.ApplicationAttribute.AA_UseHighDpiPixmaps, True)

    app = QApplication(sys.argv)
    app.setApplicationName("RK855 Configurator & Flasher")
    app.setOrganizationName("RoyalKludgeOpenTooling")
    app.setWindowIcon(create_app_icon())

    tab_index = 0
    if args.flasher:
        tab_index = 3
    elif args.remap:
        tab_index = 2
    elif args.tab == "lighting":
        tab_index = 0
    elif args.tab == "perkey":
        tab_index = 1
    elif args.tab == "remap":
        tab_index = 2
    elif args.tab == "flasher":
        tab_index = 3
    elif args.tab == "info":
        tab_index = 4

    window = MainWindow(mock=args.mock, initial_tab=tab_index)
    window.show()

    if args.test_exit:
        QTimer.singleShot(100, app.quit)

    return app.exec()
