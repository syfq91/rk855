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

    # Check for --mock and --tester flags
    args = argv or sys.argv
    is_mock = "--mock" in args
    start_tab = "tester" if ("--tester" in args or "--test" in args) else "config"
    device = MockRK68Device() if is_mock else RK68Device()

    window = RK68MainWindow(device=device, start_tab=start_tab)
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
