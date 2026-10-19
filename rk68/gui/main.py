"""Entrypoint for the PyQt6 graphical interface."""

import sys
from ..device import RK68Device, MockRK68Device
from .window import RK68MainWindow, PYQT_AVAILABLE


def main(argv=None):
    if not PYQT_AVAILABLE:
        print("Error: PyQt6 is not installed.", file=sys.stderr)
        print("To install, run: pip install PyQt6", file=sys.stderr)
        return 1

    from PyQt6.QtWidgets import QApplication

    app = QApplication(argv or sys.argv)
    app.setApplicationName("RK68 Linux Configurator")

    # Check for --mock flag
    is_mock = "--mock" in (argv or sys.argv)
    device = MockRK68Device() if is_mock else RK68Device()

    window = RK68MainWindow(device=device)
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
