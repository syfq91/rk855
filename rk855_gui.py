#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "pyqt6>=6.5.0",
#     "hidapi>=0.14.0",
#     "intelhex>=2.3.0",
# ]
# ///
"""
Royal Kludge RK855 (RK68) Cross-Platform GUI Configurator & Firmware Flasher.
Supports Linux, macOS, and Windows.
"""

import sys
from rk855_gui.cli import main

if __name__ == "__main__":
    sys.exit(main())
