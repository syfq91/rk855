# Royal Kludge RK855 (RK68) Cross-Platform Tooling Suite

This folder provides cross-platform graphical and command-line tools to control and flash the **Royal Kludge RK855 (RK68)** mechanical keyboard under Linux, macOS, and Windows.

---

## 1. Graphical User Interface (`rk855_gui.py`)

A full-featured, dark-themed desktop application built with PyQt6.

```bash
# Launch GUI
python3 rk855_gui.py

# Launch in Simulator / Demo mode (offline preview without physical keyboard)
python3 rk855_gui.py --mock

# Jump directly to Key Remapping tab
python3 rk855_gui.py --remap

# Jump directly to Firmware Flasher tab
python3 rk855_gui.py --flasher
```

### Features:
- **💡 Backlight & Effects**: Full control of all 20 built-in lighting algorithms, brightness (0–4), speed (0–4), direction, and RGB palette / hex color selection.
- **🎨 Per-Key RGB Studio**: Interactive vector 68-key keyboard view matching the physical RK855 layout. Click keys, box-select, or use quick presets (WASD, Arrow Keys, Modifiers, Numbers), assign colors, and upload the per-key matrix to onboard flash.
- **⌨️ Key Remapping**: Custom button mapping for any of the 68 physical switches. Map to standard HID keys (Letters, Numbers, Function keys, Navigation, Modifiers, Symbols), Multimedia controls (Play/Pause, Volume+, Volume-, Mute), Mouse actions (Clicks, Scroll), or Disable keys. Active remapping table with one-click restore and onboard EEPROM commit.
- **⚡ Firmware Flasher**: Select official or custom `.hex` / `.bin` firmware files, transition into the Sonix ISP bootloader, execute erase/flash/verification cycles with live progress and ISP console logging, and reboot back into normal operating mode.
- **ℹ️ Device & udev Help**: Real-time USB mode detection and non-root Linux udev rule installer.

---

## 2. Prerequisites & Installation

The tools require Python 3.9+ and `hidapi` (plus `intelhex` for parsing `.hex` firmware files).

```bash
# 1. Create and activate a virtual environment
python3 -m venv .venv
source .venv/bin/activate

# 2. Install dependencies
pip install -r tools/requirements.txt
```

### Linux Udev Rules (Non-Root Access)
On Linux, normal users need permission to open the USB HID device node. Install the provided udev rules:

```bash
sudo cp tools/udev/99-rk855.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## 2. RGB Backlight Control (`rk855_ctl.py`)

Controls the onboard lighting engine via 65-byte USB HID Feature Reports.

```bash
# Scan and identify connected RK855 keyboard
python3 tools/rk855_ctl.py info

# List all 20 built-in hardware lighting effects
python3 tools/rk855_ctl.py list-modes

# Set solid static color using hex, color name, or RGB
python3 tools/rk855_ctl.py color cyan
python3 tools/rk855_ctl.py color "#FF007F" --brightness 4
python3 tools/rk855_ctl.py color 255,128,0

# Set dynamic animation effect (e.g. Breathing mode 6, speed 3, green)
python3 tools/rk855_ctl.py mode 6 --color green --speed 3 --brightness 4

# Set Ripples effect (mode 14) with maximum brightness
python3 tools/rk855_ctl.py mode 14 --color "#00E5FF" --brightness 4 --speed 2

# Turn off backlight completely
python3 tools/rk855_ctl.py off
```

---

## 3. In-System Programming (ISP) Flasher (`rk855_flasher.py`)

Interacts with the microcontroller's ROM bootloader via the reverse-engineered Sonix ISP protocol.

```bash
# Check if keyboard is in normal operating mode or bootloader mode
python3 tools/rk855_flasher.py info

# Trigger bootloader mode from user space (keyboard resets to VID 0x0C45 / PID 0x82AA)
python3 tools/rk855_flasher.py bootloader

# Flash factory firmware (.hex or .bin) with erase, page writes, and verification
python3 tools/rk855_flasher.py flash firmware/SN32F240B_32.hex

# Reboot microcontroller back to user keyboard mode
python3 tools/rk855_flasher.py reboot
```

> [!WARNING]
> Only flash verified firmware images matching the SN32F240B / SN32F248B pinout. Flashing incompatible firmware can disable USB re-enumeration, requiring physical grounding of pin `P0.4` to recover.
