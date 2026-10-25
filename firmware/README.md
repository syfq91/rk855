# Royal Kludge RK855 Firmware (`SN32F240B_32`)

This directory contains the official factory firmware for the **Royal Kludge RK855 (RK68)** mechanical keyboard, extracted directly from the vendor updater utility.

---

## 1. Firmware Files & Integrity

| File | Format | File Size | Memory Span | SHA-256 Checksum | MD5 Checksum |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`SN32F240B_32.hex`** | Intel HEX | 166,843 bytes | `0x00000000` - `0x00007FFF` | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` | `f9e74d03875b600e44573e89338838dc` |
| **`SN32F240B_32.bin`** | Raw Binary | 65,536 bytes | `0x00000000` - `0x0000FFFF` | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` | `de2e1ffdd04c61d45637ae749c2ccfac` |

---

## 2. Target Microcontroller Architecture

- **Microcontroller**: SONiX SN32F240B (or pin-compatible SN32F248B)
- **CPU Core**: ARM Cortex-M0 (ARMv6-M Thumb-2, 32-bit RISC) @ 48 MHz
- **Flash ROM**: 64 KB total (Active code occupies first 32 KB, `0x00000000`–`0x00007FFF`)
- **Page Size**: 64 bytes (ISP write unit)
- **SRAM**: 8 KB (`0x20000000` – `0x20001FFF`)
- **Initial Vector Table**:
  - `SP` (Stack Pointer): `0x20002000` (Top of 8 KB SRAM)
  - `Reset_Handler`: `0x00000109` (Thumb entry address `0x00000108`)

---

## 3. Flash Memory Map

```
+--------------------------+ 0x00000000
| Vector Table (192 bytes) |
+--------------------------+ 0x000000C0
| Firmware Code & Assets   |
| - USB Descriptors        |
| - Matrix Scan Engine     |
| - SPI RGB Lighting Engine|
+--------------------------+ 0x00007FFF (32 KB boundary)
| Unused / Padding (0xFF)  |
+--------------------------+ 0x0000F800
| Persistent Config / EEPROM|
+--------------------------+ 0x0000FFFF (64 KB Flash Limit)
```

---

## 4. Origin & Extraction Details

- **Source Archive**: Embedded `RT_RCDATA` resource ID `4011` in `vendor/binaries/RK855_firmware_flasher.exe`.
- **Archive Offset**: `0x001BC3A8` (Deflate ZIP compressed stream).
- **Original Path**: `Settings/FWFiles/SN32F240B_32.hex`.
- **Conversion to Binary**: Generated using standard `objcopy`:
  ```bash
  objcopy -I ihex -O binary SN32F240B_32.hex SN32F240B_32.bin
  ```

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

For full register-level firmware analysis, refer to [docs/sn32f240b_firmware_analysis.md](../docs/sn32f240b_firmware_analysis.md).
