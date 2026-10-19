# Royal Kludge RK68 Linux Driver & Configurator

A lightweight, native, and complete open-source Linux rewrite of the official Windows software for the **Royal Kludge RK68** (Sonix controller, hardware ID `0c45:8018`).

Reverse-engineered from the original vendor installer (`GamingForceZ-RK68-Setup-1.0.1.1.exe`) to provide seamless backlight, color, per-key RGB, and key remapping support on Linux.

---

## Features

- **No Root Required**: Includes udev rules granting unprivileged access to the keyboard's raw HID interface.
- **Single-Color Blue LED Backlight**: Tailored for the monochrome Ice Blue LED variant of the RK68 / RK855. Supports effects (Wave, Static, Breathing, Ripple, Reactive, Off), animation speed (1–5), and LED brightness levels (0–5).
- **Key Remapping**: Remap any of the 68 physical keys (e.g. swap CapsLock and Control, map Fn keys, multimedia keys).
- **Profile Management**: Save and switch named profiles stored as human-readable JSON files in `~/.config/rk68/`.
- **Dual Interface**:
  - **`rk68` / `rk68-cli`**: Fast, scriptable command-line interface.
  - **`rk68-gui`**: Modern PyQt6 graphical interface with a live interactive 68-key visual layout and authentic blue LED illumination preview.
- **Interactive Key Switch Tester**: Dedicated 68-key switch actuation tester with live pressed/tested state rendering, progress stats, scancode inspector, keystroke history, and audio feedback.

---

## Hardware Compatibility

| Parameter | Value |
| :--- | :--- |
| **Keyboard Model** | Royal Kludge RK68 / RK855 / RKW855 (68 keys) |
| **Microcontroller** | Sonix USB MCU |
| **USB Vendor ID (VID)** | `0x0C45` |
| **USB Product ID (PID)** | `0x8018` |
| **Connection Mode** | **Wired USB-C connection** (wireless/Bluetooth does not expose HID feature reports) |

---

## Installation on Linux

### Quick Install (Automated)

Clone this repository and run the setup script:

```bash
chmod +x install.sh
./install.sh
```

### Manual Installation

1. **Install udev rule** to allow non-root access:
   ```bash
   sudo cp udev/99-rk68.rules /etc/udev/rules.d/99-rk68.rules
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```
   *(Unplug and replug the keyboard after applying the rule).*

2. **Install Python package**:
   ```bash
   pip install .
   # For the GUI application:
   pip install ".[gui]"
   ```

---

## CLI Usage

### Check Device Status
```bash
rk68 info
```

### Backlight & Blue LED Controls
```bash
# Set Wave effect with custom animation speed and brightness (0-5)
rk68 light --mode wave --speed 4 --brightness 5

# Set Solid constant Blue illumination
rk68 light --mode static --brightness 4

# Set Breathing pulse
rk68 light --mode breathing --speed 2 --brightness 3

# Turn backlight off
rk68 light --mode off
```

### Key Remapping
```bash
# Remap physical CapsLock key to Left Control
rk68 remap capslock control --save

# Remap key by scancode
rk68 remap esc 0x29
```

### Profile Management
```bash
# List saved profiles
rk68 profile list

# Save current settings to a profile
rk68 profile save gaming

# Load and apply a profile
rk68 profile load gaming
```

### Key Switch Tester
```bash
# Launch interactive Key Switch Matrix Tester directly
rk68 test

# Test in mock mode without physical keyboard
rk68 test --mock
```

### Factory Reset
```bash
rk68 reset
```

---

## GUI Usage

Launch the visual application:
```bash
rk68-gui
# Or launch directly into the key tester:
rk68-gui --tester
```
The GUI provides three dedicated workspaces:
1. **💡 Backlight Lighting**:
   - **Interactive Visual Preview**: Authentic ice blue LED keyboard canvas reacting dynamically to brightness level.
   - **Lighting Sliders**: Adjust animation speed (1–5) and blue LED brightness (0–5).
   - **Monochrome Hardware Status**: Confirms fixed Single-Color Ice Blue LED hardware mode.
   - **One-Click Apply**: Write lighting parameters directly to the keyboard's onboard controller.
2. **🔄 Key Remapping**:
   - **Visual Key Selection**: Click any keycap on the interactive keyboard canvas to select it.
   - **Searchable Button Picker**: Choose from all alphanumeric keys, F-keys (F1–F12), navigation, volume, media keys, and system functions via dropdown or visual button palette dialog (`Browse Buttons...`).
   - **Active Remaps Summary**: High-visibility overview of all custom key remappings in the active profile.
   - **One-Click Apply & Restore**: Write custom scancodes to onboard memory or restore factory defaults.
3. **⌨️ Key Switch Tester**:
   - **Full 68-key actuation matrix**: Keys turn vibrant cyan while pressed and persist in emerald green once actuated.
   - **Live Keystroke Inspector**: Displays key name, USB scancode, event state, and hardware scancode.
   - **Keystroke Stream History**: Shows a rolling log of recent keystrokes.
   - **Progress Counter**: Tracks tested key percentage (e.g. `45 / 68 Keys Tested (66%)`).
   - **Audio Click Feedback**: Optional audio chirp on switch actuation.
   - **Reset Test Button**: Clears test data to test again.

---

## Technical Protocol Details

The driver interfaces with the Sonix MCU by issuing 65-byte USB HID Feature Reports via `hidapi`:
- **Report ID**: `0x00` (byte 0)
- **Report Payload**: 64 bytes (bytes 1–64)
- **Lighting Command Opcode**: `0x04`
- **Payload Layout**:
  - `Byte 1`: Opcode (`0x04`)
  - `Byte 2`: Animation Mode (`0x00`–`0x08`)
  - `Byte 3`: Speed (`1`–`5`)
  - `Byte 4`: Brightness (`0`–`5`)
  - `Byte 5`: Direction (`0` or `1`)
  - `Bytes 6..8`: RGB (`R`, `G`, `B`)
  - `Bytes 9..64`: Zero padding
