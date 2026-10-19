#!/usr/bin/env bash
set -e

echo "=== Royal Kludge RK68 Linux Driver Installer ==="

# Check root for udev installation
if [ "$EUID" -ne 0 ]; then
  echo "[-] Requesting sudo privileges to install udev rules..."
  SUDO="sudo"
else
  SUDO=""
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Install udev rule
echo "[+] Installing udev rule to /etc/udev/rules.d/99-rk68.rules..."
$SUDO cp "$SCRIPT_DIR/udev/99-rk68.rules" /etc/udev/rules.d/99-rk68.rules
$SUDO chmod 644 /etc/udev/rules.d/99-rk68.rules

echo "[+] Reloading udev rules..."
$SUDO udevadm control --reload-rules
$SUDO udevadm trigger

# 2. Install python package
echo "[+] Installing rk68 Python package and dependencies..."
python3 -m pip install --upgrade pip
python3 -m pip install .

echo ""
echo "=== Installation Complete! ==="
echo "You can now control your RK68 keyboard without root:"
echo "  rk68 info"
echo "  rk68 light --mode wave --speed 3 --brightness 4"
echo "  rk68 light --mode steady --color '#FF0000'"
echo "  rk68 remap capslock control"
echo "  rk68-gui  # (Launch Graphical Configurator)"
echo ""
echo "Note: If your keyboard was already plugged in, unplug and replug the USB cable."
