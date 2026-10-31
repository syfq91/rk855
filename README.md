# Royal Kludge RK855 (RK68) Reverse Engineering & Open Tooling

A comprehensive reverse-engineering repository, protocol specification, and cross-platform tooling suite for the **Royal Kludge RK855 (RK68)** mechanical keyboard.

This project documents the hardware architecture, microcontroller firmware, USB HID configuration protocol, and ROM In-System Programming (ISP) bootloader, providing both a production-grade native **Rust CLI toolchain** (`rk855`) and reference Python/PyQt6 tools for Linux, macOS, and Windows.

---

## 1. Hardware Architecture Overview

| Component | Specification |
| :--- | :--- |
| **Keyboard Model** | Royal Kludge RK855 / RK68 (65% layout, 68 physical keys) |
| **Microcontroller (MCU)** | **SONiX SN32F240B** (pin-compatible with SN32F248B) |
| **CPU Architecture** | ARM Cortex-M0 (ARMv6-M Thumb) running up to 48 MHz |
| **Flash Memory** | 64 KB total on-chip Flash (59,292 bytes populated in HEX across `0x00000000`–`0x0000E797` and `0x0000FFFC`–`0x0000FFFF`; 56,088 non-0xFF bytes in flat binary) |
| **SRAM** | 8 KB (`0x20000000` – `0x20001FFF`, Initial Stack Pointer: `0x200015B0`) |
| **USB Operating VID / PID** | `0x05AC` : `0x024F` (Factory ROM Default Profile) / `0x0C45` : `0x8018` (Windows Vendor Utility Filter) |
| **USB Bootloader VID / PID**| `0x0C45` : `0x82AA` (Sonix ISP Bootloader Mode) |
| **Switch Matrix** | 8 Rows × 16 Columns scan matrix (128 positions, 68 keys populated with diode isolation) |
| **LED Backlight Subsystem** | Multiplexed LED matrix driven via CT16B1 16-bit PWM timers (`0x40002040`) across 18 scanning phases (6 rows × 3 color phases: R, G, B) and SSP0 SPI controller (`0x4001C000`) |
| **Hardware Variations** | **RGB Variant** (per-key RGB SMD LEDs) and **Single-Color Variant** (monochrome white or blue SMD LEDs). **Both share identical MCU, PCB architecture, and unified factory firmware.** |
| **Hardware Boot Pin** | `P2.2` (Pin 3 of LQFP64; internal pull-up; grounded at boot to force ISP bootloader as verified in Sonix SN32F240B/248B Datasheet Section 1.4 & 15.4.2) |

---

## 2. Feature Status & Safety Verification

Every protocol feature in this project is audited and tracked with evidentiary backing in [`docs/PROTOCOL_STATUS.md`](docs/PROTOCOL_STATUS.md):

### Verified On Physical Hardware (`verified-on-device`)
- **Lighting Controls:** All 20 built-in lighting effects, brightness levels (0–4), speed levels (0–4), direction, and RGB color selection via atomic 22-packet configuration transactions.
- **Flash Protection Modes:** `--live` mode (instant RAM updates without flash commit, eliminating flash memory wear) and `--persist` mode (commits parameters to MCU flash).
- **Profile Switching:** Switching between Profile 1, Profile 2, and Profile 3 (`0x9000`, `0x90C0`, `0x9340` flash offsets).
- **Key Remapping:** Full scan-code remapping table generation (4 blocks of 64 bytes = 256 bytes) and hardware factory reset (`remap reset`).
- **Matrix & Status Query:** Hardware status and active matrix read-back via HID GET_REPORT chunks.
- **Per-Key RGB Customization:** 18-block atomic configuration transactions targeting the 68 physical switch matrix positions.

### Verified by Static Disassembly & Specifications (`verified-by-disassembly`)
- **Cortex-M0 ISP Bootloader Protocol:** Full command set (`0x20` ISP Enter, `0x21` Version, `0x24` Erase, `0x25` Program, `0x27` Return User Mode, `0x2A` Data Check/Read).
- **Firmware Integrity:** 64 KB image layout, ARM Thumb reset vector, Initial Stack Pointer verification, and `0xAAAA5555` flash validity marker at `0x0000FFFC`.
- **Bootloader Mode Trigger:** Software jump report `AA 42 89 5A FF 71 62 CC` verified in ROM disassembly.
- **Hardware Recovery Pin:** Pulling pin `P2.2` LOW forces Mask ROM bootloader mode without executing flash code ([`docs/RECOVERY.md`](docs/RECOVERY.md)).

### Unverified & Known Limitations (`unverified`)
- **Custom Macro Stream Format:** Payload sequence for user-defined macro playback (`04 15`) remains unverified until raw USB captures of vendor macro creation are recorded.
- **Single-Color Hardware Flashing:** Unified firmware binary contains code paths for both monochrome and RGB hardware, but firmware flashing on single-color units has not yet been physically confirmed.

> [!CAUTION]
> **Firmware Flashing Risks**: In-System Programming writes directly to microcontroller flash. A power loss, cable disconnect, or corrupt binary can interrupt flashing. The `rk855` flasher enforces rigorous pre-flight validation, dry-run simulation, and per-page read-back verification. If flash is corrupted, physical recovery requires opening the keyboard case and grounding hardware boot pin `P2.2` (Pin 3 of the LQFP64 MCU). See [`docs/RECOVERY.md`](docs/RECOVERY.md) for step-by-step instructions.

---

## 3. Platform Setup & Driver Requirements

### 3.1 Linux Permissions (`udev`)

On Linux, non-root users require access to the kernel's `hidraw` device nodes. Because the keyboard **re-enumerates** with a different Product ID when entering ISP bootloader mode (`0C45:82AA`), the udev rules must grant access to both runtime and bootloader IDs.

1. Install the provided udev rules:
   ```bash
   sudo cp udev/99-rk855.rules /etc/udev/rules.d/
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```
2. Unplug and reconnect the keyboard.
3. The rules grant access to the active seat user via `TAG+="uaccess"` for:
   - `0C45:8018` (Standard Windows Profile)
   - `05AC:024F` (Factory ROM Apple Profile)
   - `0C45:8016` (Hardware Variant Profile 0)
   - `0C45:8008` (Hardware Variant Profile 2)
   - `0C45:82AA` (Sonix SN32F240B ROM ISP Bootloader)

### 3.2 Windows Driver Setup

- **Standard In-Box HID Driver:** Windows automatically binds its built-in USB HID class driver (`hidclass.sys` / `hidusb.sys`) to both the keyboard and the Sonix ISP bootloader (`0C45:82AA`).
- **No Zadig or WinUSB Required:** The bootloader is a native USB HID class device (`bInterfaceClass = 0x03`).
- > [!WARNING]
  > **Do NOT use Zadig** to replace the bootloader or keyboard driver with WinUSB or libusb-win32. Doing so will prevent standard Windows HID APIs (`hidapi` / `hid.dll`) from opening the device.

---

## 4. Rust Tooling (`rk855` CLI)

The project includes a modular, zero-dependency protocol library and command-line utility written in Rust.

### 4.1 Building from Source

Prerequisites: Rust stable toolchain (1.75+). On Linux, install `libudev-dev` and `pkg-config`.

```bash
# Clone the repository
git clone https://github.com/syfq91/rk855.git
cd rk855

# Build the release binary
cargo build --release --bin rk855

# The binary is located at target/release/rk855 (or target/release/rk855.exe on Windows)
```

### 4.2 CLI Usage & Examples

#### Device Detection & Hardware Probing
```bash
# Scan for connected RK855 keyboards and display interface details
rk855 probe
```

#### Backlight & LED Effects
```bash
# List all 20 built-in lighting effects
rk855 light modes

# Set static solid color in live mode (RAM update without flash wear)
rk855 light set 0 --color "#FF007F" --brightness 4 --speed 2 --live

# Set dynamic effect (Wave mode 10) and commit to flash memory
rk855 light set 10 --color rainbow --speed 3 --brightness 4 --persist

# Turn off backlight
rk855 light off
```

#### Profile Management
```bash
# Switch to Profile 1, 2, or 3
rk855 profile switch 2
```

#### Key Remapping
```bash
# Remap Caps Lock to Left Control
rk855 remap set "Caps Lock" "Left Control"

# Reset all key remaps back to factory hardware matrix
rk855 remap reset

# Inspect active remap table (or query hardware matrix live)
rk855 remap dump --query
```

#### Custom Per-Key RGB
```bash
# Set individual key color
rk855 rgb set-key "Escape" "#FF0000" --live

# Load full per-key RGB coloring from JSON file
rk855 rgb load custom_theme.json --persist

# Clear all custom per-key colors
rk855 rgb clear
```

#### Firmware Verification & ISP Flashing
```bash
# Validate firmware image headers, vectors, checksum, and 0xAAAA5555 marker
rk855 firmware check firmware/SN32F240B_32.hex

# Read bootloader version and MCU ID (when connected in bootloader mode)
rk855 isp info

# Dry-run simulate full flash process against internal mock
rk855 isp flash firmware/SN32F240B_32.hex --dry-run

# Flash official firmware (requires explicit confirmation prompt)
rk855 isp flash firmware/SN32F240B_32.hex
```

### 4.3 Native Desktop GUI (`rk855-gui`)

The workspace includes a native desktop configurator built with [egui](https://github.com/emilk/egui) that communicates exclusively through `rk855-config` and `rk855-isp`:

```bash
# Launch the desktop GUI in normal hardware mode
cargo run --release -p rk855-gui

# Launch in offline simulator / mock mode (works without hardware attached)
cargo run --release -p rk855-gui -- --mock
```

**Key GUI Features:**
- **Device Status:** Connection state, VID/PID, active profile selector, and USB interface inspection.
- **Lighting Presets:** Interactive controls for all 20 built-in lighting effects, brightness/speed sliders, direction toggle, and live color pickers.
- **Visual Keyboard Layout (68-Key Matrix):** Interactive layout of the physical 65% keyboard for per-key RGB coloring and key remapping. Supports theme JSON import/export.
- **Hardware Profiles:** Switch between Profiles 1, 2, and 3, or backup/restore profile configurations to TOML/JSON.
- **Guarded Firmware Flasher:** Behind an explicit "Advanced" safety toggle and warning confirmation gate. Displays pre-flight image validation details, supports dry-run simulation, and requires typing `FLASH` to confirm.

---

## 5. Repository Structure

```
rk855/
├── .github/workflows/             # Continuous Integration and Release automation
│   ├── ci.yml                     # Multi-platform CI (fmt, clippy, unit & mock tests)
│   └── release.yml                # Automated release packager and SHA-256 generator
├── udev/
│   └── 99-rk855.rules             # Linux udev permissions rules
├── crates/                        # Native Rust workspace
│   ├── rk855-proto/               # Pure no_std packet encoders, decoders, scan matrix
│   ├── rk855-hid/                 # Transport layer abstraction (hidapi + mock transport)
│   ├── rk855-config/              # High-level configuration transactions (light, remap, RGB)
│   ├── rk855-isp/                 # Firmware image validator, parser & Cortex-M0 flasher client
│   ├── rk855-cli/                 # Command-line application (rk855)
│   └── rk855-gui/                 # Native desktop GUI configurator & flasher (egui)
├── docs/                          # Detailed reverse-engineering technical reports
│   ├── README.md                  # Documentation index & reading guide
│   ├── PROTOCOL_STATUS.md         # Status and claims verification ledger
│   ├── RECOVERY.md                # Hardware pin P2.2 unbricking guide
│   ├── CAPTURES.md                # USB packet captures analysis & packet offsets
│   ├── configurator_protocol.md   # Configurator HID framing & transaction protocol
│   ├── firmware_flasher_protocol.md # Flasher ISP bootloader protocol & opcodes
│   ├── sn32f240b_firmware_analysis.md # MCU firmware deep-dive: Memory map & PWM timers
│   ├── inconsistencies.md         # Inconsistency register: 35 items resolved
│   └── datasheets/                # Official manufacturer reference manuals
├── firmware/                      # Factory firmware dumps (unmodified extracted images)
│   ├── SN32F240B_32.hex           # Intel HEX format (extracted from official updater)
│   └── SN32F240B_32.bin           # Flat 64 KB binary image
├── tools/                         # Legacy Python utilities & reference scripts
│   ├── rk855_ctl.py               # Python lighting CLI
│   ├── rk855_flasher.py           # Python flasher script
│   └── rk855_gui.py               # PyQt6 desktop application
└── vendor/                        # Factory executables & disassembled artifacts
    ├── binaries/                  # Original Windows executables (with SHA-256 hashes)
    ├── configurator/              # Unpacked configurator files & XML profiles
    └── flasher/                   # Extracted flasher settings & profiles
```

---

## 6. Troubleshooting Guide

### 6.1 Linux: "Permission denied" or "Device not found"
- Verify that `udev/99-rk855.rules` is copied to `/etc/udev/rules.d/99-rk855.rules`.
- Run `sudo udevadm control --reload-rules && sudo udevadm trigger`.
- Ensure your user account is in the active local desktop seat (`loginctl show-user $USER`).
- Check device presence: `ls -l /dev/hidraw*` to ensure permissions reflect `0660` with your user or group.

### 6.2 Bootloader Mode Re-enumeration Delay
- When entering bootloader mode (`rk855 isp enter` or `AA 42 89 5A...`), the keyboard disconnects from USB and re-enumerates as `0C45:82AA`.
- Operating systems typically require 500ms–2000ms to enumerate the new HID device node. If running commands immediately, wait a moment or rerun `rk855 probe`.

### 6.3 Hardware Unbricking & ROM Recovery
- If firmware flashing is interrupted by power failure or a corrupted image is flashed, the keyboard may not boot into user firmware.
- The Sonix SN32F240B microcontroller includes an immutable Mask ROM bootloader at `0x1FFF0300` that cannot be erased or corrupted.
- To force bootloader mode from hardware:
  1. Unplug the keyboard.
  2. Open the keyboard enclosure and locate the SN32F240B chip.
  3. Ground Pin 3 (`P2.2`) with a jumper wire or tweezers to GND.
  4. Plug in the USB cable while holding `P2.2` low, then release after 1 second.
  5. The device will enumerate as `0C45:82AA`, ready for recovery flashing via `rk855 isp flash`.
  6. See [`docs/RECOVERY.md`](docs/RECOVERY.md) for detailed schematics and pin diagrams.

---

## 7. Firmware Verification & Hashes

| Target | File | Size | SHA-256 Checksum |
| :--- | :--- | :--- | :--- |
| **Official Firmware (HEX)** | `firmware/SN32F240B_32.hex` | 166,843 B | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` |
| **Normalized Firmware (BIN)**| `firmware/SN32F240B_32.bin` | 65,536 B | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` |
| **Configurator Installer** | `vendor/binaries/RK855_configurator.exe` | 9,116,150 B | `d31b8e8cda53c595809be2ea7f9c86b2bdb945e277a4e0f9f2504c53e2fbe7f0` |
| **Firmware Flasher Executable** | `vendor/binaries/RK855_firmware_flasher.exe` | 1,917,952 B | `976d7c345492ea2f3e6a51d110b96570d9eddcce97f1cead45613f08a77d4796` |

---

## 8. License & Disclaimers

- The reverse-engineering documentation, Rust crates, Python tools, and protocol specifications are released under the **MIT License**.
- Vendor executables, firmware binaries, and artwork remain the intellectual property of **Royal Kludge / Shenzhen Beiying Electronic Technology Co., Ltd.** and are referenced for interoperability, research, and backup purposes only.
- Pre-compiled GitHub release assets contain only open-source code and never bundle proprietary vendor firmware.
