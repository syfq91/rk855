"""Entrypoint for the PyQt6 graphical interface."""

import sys
from ..device import RK855Device, MockRK855Device, RK68Device, MockRK68Device
from .window import RK855MainWindow, RK68MainWindow, PYQT_AVAILABLE


def main(argv=None):
    if not PYQT_AVAILABLE:
        print("Error: PyQt6 is not installed.", file=sys.stderr)
        print("To install, run: pip install PyQt6", file=sys.stderr)
        return 1

    from PyQt6.QtWidgets import QApplication

    app = QApplication(argv or sys.argv)
    app.setApplicationName("RK855 Linux Configurator")

    # Check for --mock, --tester, and --remap flags
    args = argv or sys.argv
    is_mock = "--mock" in args
    if "--tester" in args or "--test" in args:
        start_tab = "tester"
    elif "--remap" in args or "--remapping" in args:
        start_tab = "remap"
    else:
        start_tab = "lighting"
    device = MockRK855Device() if is_mock else RK855Device()

    window = RK855MainWindow(device=device, start_tab=start_tab)
    window.show()

    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
