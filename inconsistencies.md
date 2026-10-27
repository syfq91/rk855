# Firmware and Protocol Inconsistency Resolutions

This document records the verified technical resolutions for all discrepancies identified during cross-document, binary, and firmware analysis of the RK855 codebase using **REA (Reverse Engineer Anything)**, Capstone disassembly, and static PE analysis.

---

### 1. Vector Table (Items A1, X3)
- **Resolution:** The true application firmware (`SN32F240B_32.bin` and `SN32F240B_32.hex`) vector table is:
  - **Initial Stack Pointer (SP):** `0x200015B0` (located at flash offset `0x00000000`)
  - **Reset Handler:** `0x000066D1` (Thumb entry address `0x000066D0` at flash offset `0x00000004`)
  - **Execution Path:** The instruction at `0x000066D0` (`ldr r0, [pc, #0x18]; bx r0`) loads literal `0x000000C1` and branches to `0x000000C0` (`__main` C runtime startup), which initializes data/BSS and enters `main` at `0x00000620`.
  - First 16 bytes: `b0 15 00 20 d1 66 00 00 d1 66 00 00 d1 66 00 00`
- **Origin of other values:**
  - `0x20000728` / `0x000000D9`: Vector table of the flasher executable's embedded generic template/RAM agent payload (Resource ID `4000` at file offset `0x001AC378` in `RK855_firmware_flasher.exe`, SHA-256 `8324f1bc...`). This is NOT the keyboard firmware. The actual target keyboard firmware is bundled inside Resource `4011` (ZIP archive containing `SN32F240B_32.hex`).
  - `0x20002000` / `0x0109`: An earlier documentation note conflating top of 8 KB SRAM (`0x20000000` + 8 KB = `0x20002000`) and the version constant `CurVersion=0x0109` in `UISettings.ini`.

---

### 2. Image Size, Flash Extent, & Integrity Marker (Items A2, N1)
- **Resolution:** The firmware memory span covers the full 64 KB on-chip flash: `0x00000000` to `0x0000FFFF`.
  - **Populated Data in HEX:** Exactly **59,292 bytes** spanning `0x00000000`–`0x0000E797` (59,288 B) plus `0x0000FFFC`–`0x0000FFFF` (4 B hardware validity marker `0xAAAA5555`).
  - **Unpopulated Padding:** 6,244 bytes from `0x0000E798` to `0x0000FFFB` (unwritten flash erased as `0xFF`).
  - **Flat Binary:** 65,536 bytes (64 KB) with 56,088 non-`0xFF` bytes.
  - Converting the HEX with `objcopy -I ihex -O binary --gap-fill 0xff SN32F240B_32.hex SN32F240B_32.bin` reproduces `SN32F240B_32.bin` bit-for-bit.
  - **Hardware Flash Validity Marker (`0xAAAA5555` at `0xFFFC`)**: This alternating bit pattern (`1010101010101010 0101010101010101`) is a constant hardware signature checked by the Sonix Mask ROM bootloader prior to booting user code, NOT a calculated arithmetic checksum. Flasher tools must write this word to `0xFFFC`.

---

### 3. USB Identifiers & Channel Architecture (Items B1, B2, X4)
- **Resolution:**
  - **Factory ROM Default Descriptor:** `05AC:024F` (`bcdDevice: 0x0100`). There is only one device descriptor compiled into ROM (at `0x00006ABC`). It mimics an Apple Aluminum Keyboard for native macOS and BIOS interoperability.
  - **Windows Vendor Tool Matching:** `0C45:8018` is defined in `device.xml` (`<device name="RK KeyBoard 68K" type="1" vid="0c45" pid="8018"...>`) and `UISettings_248B.ini`.
  - **Bootloader Mode:** `0C45:82AA` (Sonix ISP Bootloader Device).
  - **Channel & Report Lengths:** The configuration channel is defined in Interface 0 (`&MI_00`) as a **64-byte Feature Report with Report ID `0x00`**. On the wire via Windows `HidD_SetFeature` or `hidapi`, buffer length is 65 bytes (`wire[0] = 0x00`, `wire[1..64] = payload[0..63]`).
  - **Report ID 5:** Declared in Interface 1 (Mouse interface) as a 3-byte Vendor-defined **INPUT report** (`Usage Page 0xFFFF, Usage 0x01`), completely independent of the configuration Feature Report channel.

---

### 4. Switch Matrix & Pin Map (Items E1, E2, X5)
- **Resolution:** The switch scanning matrix is **8 Rows × 16 Columns** (128 scan positions, 68 keys populated):
  - Confirmed from the firmware scan table at offset `0x00006D7B` (16 column entries of 8 bytes each).
  - **Row Drive Lines:** Port 0 (`P0.8`–`P0.15`, 8 outputs, configured at `0x000061CC`).
  - **Column Sense Lines:** Port 3 (`P3.0`–`P3.11`, multiplexed inputs with pull-ups).
  - `P1.0` and `P1.1` are dedicated hardware SPI lines (`SN_SSP0_SCK`, `SN_SSP0_MOSI`) and do not share lines with switch rows.

---

### 5. Lighting Mode Names & Parameter Ranges (Items C3, C6)
- **Resolution:** Mode names are standardized to the official vendor strings in `vendor/configurator/app/language/1033.lan`:
  - `0`: Static, `1`: SingleOn, `2`: SingleOff, `3`: Glittering, `4`: Falling, `5`: Colourful, `6`: Breath, `7`: Spectrum, `8`: Outward, `9`: Scrolling, `10`: Rolling, `11`: Rotating, `12`: Explode, `13`: Launch, `14`: Ripples, `15`: Flowing, `16`: Pulsating, `17`: Tilt, `18`: Shuttle, `19`: Close Backlight (Off). Total: 20 modes (18 dynamic animation effects + Static + Off).
  - Speed: Initialized to `0x0A` in `DeviceDriver.exe` at `0x40AF0F` before UI slider adjustment; UI slider range is `0`–`4`. Firmware clamps out-of-range values to maximum speed.

---

### 6. Per-Key RGB & Remap Header Count Rules (Items C1, C2, X1, X2)
- **Resolution:**
  - **16-Byte Record Layout:** The 1,216-byte custom RGB buffer comprises **76 records of 16 bytes each**:
    - Byte 0: Status flag (`0x80` active)
    - Bytes 1–3: Red, Green, Blue
    - Bytes 4–13: Key attributes & parameters (`0x1001`, `0x0C`, etc.)
    - Bytes 14–15: Magic delimiter bytes `0xAA, 0x55`
  - **Header Count Rule (Item X1):** In `DeviceDriver.exe`, the header count byte at `payload[8]` (`wire[9]`) is command-specific:
    - Custom RGB (`04 13`): `payload[8] = 0x12` (Block count minus one: $19 - 1 = 18$).
    - Key Remap (`04 20`): `payload[8] = 0x04` (Exact block count: 4 blocks = 256 B).
  - **Remap Encoding (Item C2):** 256 bytes contain 128 two-byte tuples `[PhysicalMatrixIndex, TargetCode]`. Target code is a 1-byte value (standard USB HID keycode or internal action code `0x73`–`0x7C`). Categories 126–129 (Launch App, Open URL, Text strings) are handled host-side in `DeviceDriver.exe`.

---

### 7. Profile Memory Layout & Flash Storage (Item C5)
- **Resolution:** The firmware EP0 handler (`0x00004BEA`–`0x00004C98`) maps user profiles to dedicated on-chip flash sectors:
  - **Profile 1:** Keymap at `0x00009000` (128 B), Lighting at `0x00009080` (64 B). Selected via `0x04 0xF1`.
  - **Profile 2:** Keymap at `0x000090C0` (128 B), Lighting at `0x00009140` (64 B). Selected via `0x04 0xF2`.
  - **Profile 3:** Keymap at `0x00009340` (128 B), Lighting at `0x000093C0` (64 B). Selected via `0x04 0xF3`.
  - **Custom RGB Framebuffer:** `0x000097C0`–`0x00009C7F` (1,216 B).

---

### 8. Official Sonix ISP Opcode Architecture (Item D1, D5)
- **Resolution:** Disassembly of `RK855_firmware_flasher.exe` (`0x54F340`–`0x54F519`) reveals dual command sets:
  - **SN8F 8-Bit MCUs:** `0x01`–`0x09` (`0x01` Get Version, `0x02` Password, `0x03` Encryption, `0x04` Erase, `0x05` Program, `0x06` Checksum, `0x07` Return User Mode, `0x08` Set Code Option, `0x09` Get Code Option).
  - **SN32F 32-Bit Cortex-M0 MCUs:** `0x20`–`0x2A` (`0x20` Enter ISP, `0x21` Get Version, `0x22` Password, `0x23` Encryption, `0x24` Enable Erase, `0x25` Enable Program, `0x26` Get Checksum, `0x27` Return User Mode, `0x28` Set Code Option, `0x29` Get Code Option, `0x2A` Check Data).
  - **Safety Hazard in Legacy Scripts:** Shipped Python tools implemented unverified opcodes where `0x08` was treated as erase (in reality `Set Code Option`), `0x07` as write page (in reality `Return User Mode`), and `0x29` as reset (in reality `Get Code Option`). Safety warnings added.

---

### 9. Single-Color Variant Hardware Marker (Item E3)
- **Resolution:** In `main` at `0x00000644`–`0x00000678`, the firmware loads halfword `0x0000800C` and compares against `0xAAAA`:
  - If `0x800C == 0xAAAA`: Sets monochrome mode flag and branches to dedicated monochrome LED driver `0xE402`.
  - Otherwise branches to RGB driver `0xA4C0`.
  - In `SN32F240B_32.bin`, `0x800C` contains `0xAAAA`.
  - Both hardware models share unified firmware, with variant execution controlled by flash address `0x800C`. Believed compatible; hardware testing recommended.

---

### 10. Bootloader Jump & Peripheral Addresses (Items E4, F1)
- **Resolution:**
  - Bootloader Jump: Confirmed Capstone disassembly at `0x00001790`: verifies 8-byte key `AA 42 89 5A FF 71 62 CC`, writes to `SYS0->REMAP` (`0x40060024`), and branches to Mask ROM entry `0x1FFF0301`.
  - Flash Programming (`0x0000615A`): Writes directly to Flash Memory Controller `0x40062000` without requiring any `0x5AFA` key.
  - CT16B1 PWM timer base: `0x40002000` (Match registers at `0x40002040`).
