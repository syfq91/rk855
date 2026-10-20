#!/usr/bin/env bash
set -e

echo "=== Royal Kludge RK855 Linux Driver Installer ==="

# Check root for udev installation
if [ "$EUID" -ne 0 ]; then
  echo "[-] Requesting sudo privileges to install udev rules..."
  SUDO="sudo"
else
  SUDO=""
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Install udev rule
echo "[+] Installing udev rule to /etc/udev/rules.d/99-rk855.rules..."
$SUDO cp "$SCRIPT_DIR/udev/99-rk855.rules" /etc/udev/rules.d/99-rk855.rules
$SUDO chmod 644 /etc/udev/rules.d/99-rk855.rules

echo "[+] Reloading udev rules..."
$SUDO udevadm control --reload-rules
$SUDO udevadm trigger

# 2. Install python package
echo "[+] Installing rk855 Python package and dependencies..."
if command -v uv >/dev/null 2>&1; then
  uv tool install --editable "$SCRIPT_DIR[gui]" --force
else
  python3 -m pip install --break-system-packages "$SCRIPT_DIR[gui]" 2>/dev/null || python3 -m pip install "$SCRIPT_DIR[gui]"
fi

echo ""
echo "=== Installation Complete! ==="
echo "You can now control your RK855 keyboard without root:"
echo "  rk855 info"
echo "  rk855 light --mode wave --speed 3 --brightness 4"
echo "  rk855 light --mode steady --color '#FF0000'"
echo "  rk855 remap capslock control"
echo "  rk855-gui  # (Launch Graphical Configurator)"
echo ""
echo "Note: If your keyboard was already plugged in, unplug and replug the USB cable."
