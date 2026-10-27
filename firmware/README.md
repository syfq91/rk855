# Royal Kludge RK855 Firmware (`SN32F240B_32`)

This directory contains the official factory firmware for the **Royal Kludge RK855 (RK68)** mechanical keyboard, extracted directly from the vendor updater utility.

---

## 1. Firmware Files & Integrity

| File | Format | File Size | Memory Span | SHA-256 Checksum | MD5 Checksum |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`SN32F240B_32.hex`** | Intel HEX | 166,843 bytes | `0x00000000` - `0x0000FFFF` | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` | `f9e74d03875b600e44573e89338838dc` |
| **`SN32F240B_32.bin`** | Flat Binary | 65,536 bytes | `0x00000000` - `0x0000FFFF` | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` | `de2e1ffdd04c61d45637ae749c2ccfac` |

---

## 2. Target Microcontroller Architecture

- **Microcontroller**: SONiX SN32F240B (or pin-compatible SN32F248B)
- **CPU Core**: ARM Cortex-M0 (ARMv6-M Thumb, 32-bit RISC) @ 48 MHz
- **Flash ROM**: 64 KB total (59,292 bytes populated in HEX across `0x00000000`–`0x0000E797` and `0x0000FFFC`–`0x0000FFFF`; 56,088 non-0xFF bytes in flat binary)
- **Page Size**: 64 bytes (strictly 64 bytes on SN32F240B)
- **SRAM**: 8 KB (`0x20000000` – `0x20001FFF`)
- **Initial Vector Table (Target Keyboard Firmware)**:
  - `SP` (Stack Pointer): `0x200015B0` (located at flash offset `0x00000000`)
  - `Reset_Handler`: `0x000066D1` (Thumb entry address `0x000066D0` at flash offset `0x00000004`)
  - Reset Execution: Loads literal `0x000000C1` and branches to `0x000000C0` (`__main` C runtime startup), which clears BSS, copies data, and enters `main` at `0x00000620`.
  - First 16 bytes: `b0 15 00 20 d1 66 00 00 d1 66 00 00 d1 66 00 00`
- **Flasher Embedded Template Vector Table (`RT_RCDATA` Resource 4000)**:
  - `SP`: `0x20000728`, `Reset`: `0x000000D9` (SHA-256: `8324f1bc059cae33c21066c44ac8e18faa161b6389c9987dc99cdc4cd5b889c7`)
  - First 16 bytes: `28 07 00 20 d9 00 00 00 e1 00 00 00 a1 05 00 00`
  - *(Note: Resource 4000 is an embedded generic Sonix ISP template/RAM agent in `RK855_firmware_flasher.exe`, NOT the RK855 keyboard firmware. The actual target firmware is bundled in Resource 4011).*

---

## 3. Flash Memory Map

```
+--------------------------+ 0x00000000
| Vector Table (192 bytes) |
+--------------------------+ 0x000000C0
| C Runtime Startup (__main)|
+--------------------------+ 0x0000012C
| Firmware Code & Assets   |
| - Matrix Scan Engine     |
| - USB Descriptors (0x6ABC)|
+--------------------------+ 0x00007000
| Factory Presets & Layout | 0x00007000 - 0x0000757F
+--------------------------+ 0x00007580
| RGB Animation Tables     | (Gamma curves, palettes)
+--------------------------+ 0x00008000
| Hardware Config Markers  | (0x800C: 0xAAAA single-color variant flag)
+--------------------------+ 0x00009000
| Profile 1 Flash Storage  | (Keymap: 0x9000, Lighting: 0x9080)
+--------------------------+ 0x000090C0
| Profile 2 Flash Storage  | (Keymap: 0x90C0, Lighting: 0x9140)
+--------------------------+ 0x00009340
| Profile 3 Flash Storage  | (Keymap: 0x9340, Lighting: 0x93C0)
+--------------------------+ 0x000097C0
| Custom Per-Key RGB Matrix| (1,216 bytes framebuffer & macro streams)
+--------------------------+ 0x0000E798
| Unused Flash (0xFF)      | (6,244 bytes unwritten erase space)
+--------------------------+ 0x0000FFFC
| Flash Validity Marker    | (4 bytes: 0xAAAA5555 / 55 55 AA AA)
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
   Send the unlock sequence from keyboard mode (`VID: 0x05AC`, `PID: 0x024F` or `VID: 0x0C45`, `PID: 0x8018`):
   ```
   0xAA 0x42 0x89 0x5A 0xFF 0x71 0x62 0xCC
   ```
2. **Device Re-enumerates**:
   The keyboard re-enumerates as the Sonix ISP bootloader (`VID: 0x0C45`, `PID: 0x82AA`).
3. **Write Firmware**:
   Flashing tools MUST erase and program the entire `0x00000000`–`0x0000E797` block as well as the 4-byte validity marker at `0x0000FFFC` (`0xAAAA5555`).

> [!WARNING]
> Flashing invalid or corrupt firmware may cause the keyboard to enter an unbootable state requiring physical bootloader pin grounding (`P2.2` / `BOOT` pin, Pin 3 on LQFP64 pulled LOW at power-on as documented in SN32F240B/248B Datasheet Section 1.4 & 15.4.2) to recover.

---

## 6. Hardware Variant Compatibility (RGB & Single-Color Models)

Reverse engineering analysis using **REA** confirms that **`SN32F240B_32.hex` / `SN32F240B_32.bin` is the unified factory firmware for both the RGB and Single-Color (Monochrome / White LED) hardware revisions**:

- Both models use the identical SONiX `SN32F240B` MCU, PCB matrix, and ISP bootloader entry sequence.
- At power-on (`main` at `0x00000644`), the firmware checks whether halfword `0x0000800C` equals `0xAAAA`. If match, it sets the single-color flag and branches to the dedicated monochrome driver `0xE402`. If zero, it branches to the RGB driver `0xA4C0`.
- In `SN32F240B_32.bin`, `0x800C` contains `0xAAAA`.
- On single-color models, monochrome LEDs respond to all 20 animation effects, brightness levels, and speeds via hardware PWM modulation.
- Flashing this unified firmware is believed compatible across both models; physical hardware validation on a test board is recommended.

For full register-level firmware analysis, refer to [docs/sn32f240b_firmware_analysis.md](../docs/sn32f240b_firmware_analysis.md).
