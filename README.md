# Royal Kludge RK855 (RK68) Reverse Engineering & Open Tooling

A comprehensive reverse-engineering repository, protocol specification, and cross-platform tooling suite for the **Royal Kludge RK855 (RK68)** mechanical keyboard.

This project documents the hardware architecture, microcontroller firmware, USB HID configuration protocol, and ROM In-System Programming (ISP) bootloader, providing open-source Python tools to control lighting and flash firmware on Linux, macOS, and Windows.

---

## 1. Hardware Architecture Overview

| Component | Specification |
| :--- | :--- |
| **Keyboard Model** | Royal Kludge RK855 / RK68 (65% layout, 68 physical keys) |
| **Microcontroller (MCU)** | **SONiX SN32F240B** (pin-compatible with SN32F248B) |
| **CPU Architecture** | ARM Cortex-M0 (ARMv6-M Thumb) running up to 48 MHz |
| **Flash Memory** | 64 KB total on-chip Flash (32 KB active firmware: `0x00000000`–`0x00007FFF`) |
| **SRAM** | 8 KB (`0x20000000` – `0x20001FFF`, Stack Top: `0x20002000`) |
| **USB Operating VID / PID** | `0x0C45` : `0x8018` (Normal Keyboard Mode) |
| **USB Bootloader VID / PID**| `0x0C45` : `0x82AA` (Sonix ISP Bootloader Mode) |
| **Switch Matrix** | 8 Rows × 9 Columns multiplexed matrix (68 diode-isolated switches) |
| **RGB LED Subsystem** | Per-key addressable RGB LEDs driven via SPI controller (`P1.0` SCK, `P1.1` MOSI) |
| **Hardware Boot Pin** | `P0.4` (Internal pull-up; grounded at boot to force ISP bootloader) |

---

## 2. Repository Structure

```
rk855/
├── README.md                      # Primary project overview, hardware specs, and quickstart
├── .gitignore                     # Repository ignore rules
├── docs/                          # Detailed reverse-engineering technical reports
│   ├── README.md                  # Documentation index & reading guide
│   ├── configurator_protocol.md   # Configurator analysis: HID framing, lighting modes, key remap
│   ├── firmware_flasher_protocol.md # Flasher analysis: ISP bootloader protocol & unlock command
│   └── sn32f240b_firmware_analysis.md # MCU firmware deep-dive: Memory map, vectors, matrix scan
├── firmware/                      # Factory firmware dumps and specifications
│   ├── README.md                  # Firmware memory regions, checksums, and flashing notes
│   ├── SN32F240B_32.hex           # Intel HEX format (extracted from official updater)
│   └── SN32F240B_32.bin           # Flat 64 KB binary image
├── tools/                         # Cross-platform Python utilities
│   ├── README.md                  # Tools usage manual and instructions
│   ├── requirements.txt           # Python package requirements (hidapi, intelhex)
│   ├── rk855_ctl.py               # CLI tool for lighting modes, colors, brightness & speed
│   ├── rk855_flasher.py           # CLI tool for ISP flashing, verification, and bootloader trigger
│   └── udev/
│       └── 99-rk855.rules         # Linux udev rules for non-root USB access
└── vendor/                        # Original vendor distribution files
    ├── README.md                  # Structure and runtime requirements
    ├── binaries/                  # Original Windows executables (with SHA-256 hashes)
    │   ├── README.md
    │   ├── RK855_configurator.exe # Factory configurator installer (InstallShield)
    │   └── RK855_firmware_flasher.exe # Factory firmware flasher
    ├── configurator/              # Unpacked configurator files
    │   ├── app/                   # Reconstituted standalone app (DeviceDriver.exe, XMLs, fonts)
    │   └── installer/             # Raw InstallShield cabinet archives & support DLLs
    └── flasher/                   # Extracted flasher settings, chip profiles & UI bitmaps
```

---

## 3. Quickstart & CLI Tools

### 3.1 Installation
Clone the repository and install the Python dependencies:

```bash
git clone https://github.com/syfq91/rk855.git
cd rk855

# Optional: Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -r tools/requirements.txt
```

### 3.2 Linux USB Permissions (udev)
To allow non-root users to configure and flash the keyboard:

```bash
sudo cp tools/udev/99-rk855.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### 3.3 Graphical User Interface (`rk855_gui.py`)
Launch the cross-platform desktop configurator & firmware flasher:

```bash
# Launch GUI
python3 rk855_gui.py

# Launch in Simulator / Demo mode (offline preview)
python3 rk855_gui.py --mock
```

The GUI provides an interactive 68-key physical keyboard layout for per-key RGB coloring, full controls for all 20 built-in lighting effects, and a safe ISP firmware flasher with live progress bar and terminal logs.

### 3.4 Backlight CLI Control (`rk855_ctl.py`)
Control backlight modes, custom RGB colors, brightness, and speed from the terminal:

```bash
# Detect connected keyboard
python3 tools/rk855_ctl.py info

# List built-in hardware lighting modes
python3 tools/rk855_ctl.py list-modes

# Set solid color (hex, RGB, or named color)
python3 tools/rk855_ctl.py color cyan
python3 tools/rk855_ctl.py color "#FF007F" --brightness 4

# Set dynamic effect (e.g. Ripples mode 14, speed 2, purple)
python3 tools/rk855_ctl.py mode 14 --color purple --speed 2 --brightness 3

# Turn off backlight
python3 tools/rk855_ctl.py off
```

### 3.4 Firmware Flashing (`rk855_flasher.py`)
Flash or verify firmware over the Sonix ISP USB protocol without external hardware debuggers:

```bash
# Check device operating state (normal keyboard vs. bootloader)
python3 tools/rk855_flasher.py info

# Reboot keyboard into Sonix ISP bootloader
python3 tools/rk855_flasher.py bootloader

# Flash and verify official firmware hex
python3 tools/rk855_flasher.py flash firmware/SN32F240B_32.hex

# Reboot back to keyboard firmware
python3 tools/rk855_flasher.py reboot
```

---

## 4. Protocol Summary & Reference

### 4.1 USB HID Configuration Framing
All configuration commands are transmitted as **65-byte USB HID Feature Reports** (`Report ID 0x00`):

```
+-----------+------------+---------------+--------------------------------+
| Byte 0    | Byte 1     | Byte 2        | Bytes 3 .. 64                  |
| 0x00      | 0x04       | Subcommand    | Command Arguments & Zero Pad   |
| Report ID | Magic Op   |               |                                |
+-----------+------------+---------------+--------------------------------+
```

A standard lighting configuration sequence consists of four atomic packets:
1. **Begin Configuration**: `0x04 0x18`
2. **Apply Parameters**: `0x04 0x11 [Mode] [ColorType] [R] [G] [B] [Brightness] [Speed] [Direction]`
3. **Commit to Flash**: `0x04 0x02`
4. **Live Synchronization**: `0x04 0xF0`

### 4.2 Built-in RGB Lighting Modes
| ID | Mode Name | Description |
| :---: | :--- | :--- |
| `0` | **Static** | Continuous solid color illumination across all keys. |
| `1` | **Single On** | Reactive lighting: pressed keys illuminate and fade. |
| `2` | **Single Off** | Inverted reactive: keys turn off upon press. |
| `3` | **Glittering** | Random keys twinkle independently. |
| `4` | **Falling** | Rain-like vertical color cascade. |
| `5` | **Color Cycling** | Full keyboard smooth RGB hue transition. |
| `6` | **Breathing** | Pulsing fade in and fade out. |
| `7` | **Spectrum** | Color cycling across spectrum. |
| `8` | **Outward** | Center-out radial expansion. |
| `9` | **Scrolling** | Horizontal wave scrolling. |
| `10` | **Wave** | Continuous rainbow angular wave. |
| `11` | **Accumulate** | Keys progressively fill up the board. |
| `12` | **Windmill** | Rotating angular color wheel. |
| `13` | **Colorful Waterfall** | Vertical dual-tone waterfall. |
| `14` | **Blossoming / Ripples** | Key press generates expanding circular ripple. |
| `15` | **Flowing** | Stream-like linear color movement. |
| `16` | **Constant Rainbow** | Static multi-colored gradient matrix. |
| `17` | **Sine Wave** | Sinusoidal oscillating brightness wave. |
| `18` | **Shadow** | Trailing shadows following key releases. |
| `19` | **Backlight Off** | All LEDs disabled. |

### 4.3 Bootloader Unlock Command
To switch from user firmware into the Sonix ISP bootloader, issue this 9-byte Feature Report to `VID: 0x0C45`, `PID: 0x8018`:
```
0x00 0xAA 0x42 0x89 0x5A 0xFF 0x71 0x62 0xCC
```

---

## 5. Firmware Checksums & Verification

| Target | File | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Configurator Installer** | `vendor/binaries/RK855_configurator.exe` | 9,116,150 B | `d31b8e8cda53c595809be2ea7f9c86b2bdb945e277a4e0f9f2504c53e2fbe7f0` |
| **Firmware Flasher** | `vendor/binaries/RK855_firmware_flasher.exe` | 1,917,952 B | `976d7c345492ea2f3e6a51d110b96570d9eddcce97f1cead45613f08a77d4796` |
| **Factory Firmware (HEX)** | `firmware/SN32F240B_32.hex` | 166,843 B | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` |
| **Factory Firmware (BIN)** | `firmware/SN32F240B_32.bin` | 65,536 B | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` |

---

## 6. License & Disclaimer

- The reverse-engineering documentation, Python tooling, and protocol specifications are released under the **MIT License**.
- Vendor executables, firmware binaries, and artwork remain the intellectual property of **Royal Kludge / Shenzhen Beiying Electronic Technology Co., Ltd.** and are provided for educational, interoperability, and backup purposes only.
