# Royal Kludge RK855 Firmware (`SN32F240B_32`)

This directory contains the official factory firmware for the **Royal Kludge RK855 (RK68)** mechanical keyboard, extracted directly from the vendor updater utility.

---

## 1. Firmware Files & Integrity

| File | Format | File Size | Memory Span | SHA-256 Checksum | MD5 Checksum |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`SN32F240B_32.hex`** | Intel HEX | 166,843 bytes | `0x00000000` - `0x0000FFFF` | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` | `f9e74d03875b600e44573e89338838dc` |
| **`SN32F240B_32.bin`** | Raw Binary | 65,536 bytes | `0x00000000` - `0x0000FFFF` | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` | `de2e1ffdd04c61d45637ae749c2ccfac` |

---

## 2. Target Microcontroller Architecture

- **Microcontroller**: SONiX SN32F240B (or pin-compatible SN32F248B)
- **CPU Core**: ARM Cortex-M0 (ARMv6-M Thumb-2, 32-bit RISC) @ 48 MHz
- **Flash ROM**: 64 KB total (59,292 bytes populated in HEX across `0x00000000`–`0x0000E797` and `0x0000FFFC`–`0x0000FFFF`; 56,088 non-0xFF bytes in flat binary)
- **Page Size**: 64 bytes (ISP write unit)
- **SRAM**: 8 KB (`0x20000000` – `0x20001FFF`)
- **Initial Vector Table**:
  - `SP` (Stack Pointer): `0x200015B0` (Initialized in SRAM)
  - `Reset_Handler`: `0x000066D1` (Thumb entry address `0x000066D0`)
  *(Note: The value `0x0109` found in `UISettings.ini` corresponds to firmware version v1.09, not the reset vector. The alternative vector pair `0x20000728` / `0x000000D9` belongs to the flasher executable's embedded resource 4000).*

---

## 3. Flash Memory Map

```
+--------------------------+ 0x00000000
| Vector Table (192 bytes) |
+--------------------------+ 0x000000C0
| Firmware Code & Assets   |
| - USB Descriptors (0x6ABC)|
| - Matrix Scan Engine     |
| - SPI Master Controller  |
+--------------------------+ 0x000071FF (29,184 B Code)
| Factory Presets & Layout | 0x00007000 - 0x0000757F
+--------------------------+ 0x00007580
| RGB Animation Tables     | (Gamma curves, palettes)
+--------------------------+ 0x000081FF
| Padding (0xFF)           |
+--------------------------+ 0x00008580
| Onboard User Profiles    | (Keymaps, custom lighting,
| & Macro Flash Storage    |  and macros to 0x0000E797)
+--------------------------+ 0x0000E798
| Unused Flash (0xFF)      | (6,244 bytes unwritten erase space)
+--------------------------+ 0x0000FFFC
| Flash Integrity Keyword  | (4 bytes: 0xAAAA5555 / 55 55 AA AA)
+--------------------------+ 0x0000FFFF (64 KB Flash Limit)
```

---

## 4. Origin & Extraction Details

- **Source Archive**: Embedded `RT_RCDATA` resource ID `4011` in `vendor/binaries/RK855_firmware_flasher.exe`.
- **Archive Offset**: `0x001BC3A8` (Deflate ZIP compressed stream).
- **Original Path**: `Settings/FWFiles/SN32F240B_32.hex`.
- **Conversion to Binary**: Generated using standard `objcopy` with gap filling:
  ```bash
  objcopy -I ihex -O binary --gap-fill 0xff SN32F240B_32.hex SN32F240B_32.bin
  ```
  *(Note: `--gap-fill 0xff` is required to ensure sparse unwritten flash areas between `0x0000E798` and `0x0000FFFB` are filled with erased flash byte `0xFF`, matching the flat binary byte-for-byte).*

---

## 5. In-System Programming (ISP) Flashing

The firmware can be flashed over USB without any hardware programmer (ST-Link / J-Link) using the built-in Sonix ISP Bootloader:

1. **Trigger Bootloader Mode**:
   Send the unlock sequence from keyboard mode (`VID: 0x0C45`, `PID: 0x8018`):
   ```
   0xAA 0x42 0x89 0x5A 0xFF 0x71 0x62 0xCC
   ```
2. **Device Re-enumerates**:
   The keyboard re-enumerates as the Sonix ISP bootloader (`VID: 0x0C45`, `PID: 0x82AA`).
3. **Write Firmware**:
   Use `tools/rk855_flasher.py` or the vendor flasher utility.

> [!WARNING]
> Flashing invalid or corrupt firmware may cause the keyboard to enter an unbootable state requiring physical bootloader pin grounding (`P0.4` / `BOOT` pin pulled LOW at power-on) to recover.

---

## 6. Hardware Variant Compatibility (RGB & Single-Color Models)

Reverse engineering analysis using **REA** confirms that **`SN32F240B_32.hex` / `SN32F240B_32.bin` is the universal factory firmware for both the RGB and Single-Color (Monochrome / White LED) hardware revisions**:

- Both models use the identical SONiX `SN32F240B` MCU, PCB matrix, and ISP bootloader entry sequence.
- The firmware executes an 18-phase multiplexing loop (`0x00006082`) driving Red (`0x2000096C`), Green (`0x20000A68`), and Blue (`0x20000B64`) row buffers into 16-bit PWM match registers at `0x40002040`.
- On single-color models, monochrome LEDs respond to all 20 animation effects, brightness levels, and speeds via hardware PWM modulation.
- In static, monochrome, and indicator routines, the firmware writes identical PWM values across Red, Green, and Blue buffers simultaneously ($R=G=B$), ensuring single-color LEDs receive PWM power regardless of which color line they bridge.
- Flashing this unified firmware is believed compatible for both single-color and RGB variants based on firmware PWM analysis (physical hardware test capture recommended before production flashing).

For full register-level firmware analysis, refer to [docs/sn32f240b_firmware_analysis.md](../docs/sn32f240b_firmware_analysis.md).
