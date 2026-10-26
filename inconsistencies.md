# Firmware and Protocol Inconsistency Resolutions

This document records the verified resolutions for the 7 key inconsistencies identified during cross-document and binary analysis of the RK855 codebase using **REA** and static reverse engineering.

---

### 1. Vector Table
- **Resolution:** The true application firmware (`SN32F240B_32.bin` and `SN32F240B_32.hex`) vector table is:
  - **Initial Stack Pointer (SP):** `0x200015B0` (located at flash offset `0x00000000`)
  - **Reset Handler:** `0x000066D1` (Thumb entry address `0x000066D0` at flash offset `0x00000004`)
- **Origin of other values:**
  - `0x20000728` / `0x000000D9`: Vector table of the flasher's embedded RAM agent payload (Resource ID `4000` at file offset `0x001AC378` in `RK855_firmware_flasher.exe`).
  - `0x20002000` / `0x0109`: An earlier documentation typo conflating the top of 8 KB SRAM (`0x20000000` + 8 KB = `0x20002000`) and the version string `CurVersion=0x0109` (v1.09) in `UISettings.ini`.

---

### 2. Image Size & Flash Extent
- **Resolution:** The firmware memory span covers the full 64 KB on-chip flash: `0x00000000` to `0x0000FFFF`.
  - **Populated Data in HEX:** Exactly **59,292 bytes** spanning `0x00000000`–`0x0000E797` (59,288 B) plus `0x0000FFFC`–`0x0000FFFF` (4 B integrity footer `0xAAAA5555`).
  - **Unpopulated Padding:** 6,244 bytes from `0x0000E798` to `0x0000FFFB` (unwritten flash erased as `0xFF`).
  - **Flat Binary:** 65,536 bytes (64 KB) with 56,088 non-`0xFF` bytes.
  - Converting the HEX with `objcopy -I ihex -O binary --gap-fill 0xff SN32F240B_32.hex SN32F240B_32.bin` yields an exact 100% bit-for-bit match to `SN32F240B_32.bin`.

---

### 3. USB Identifiers
- **Resolution:** Both identifiers are valid depending on the context:
  - **Factory ROM Default Descriptor:** `05AC:024F` (Apple Extended Keyboard profile compiled into ROM at offset `0x00006ABC` for universal BIOS and macOS compatibility).
  - **Windows Vendor Tool Matching:** `0C45:8018` (used in `device.xml` for `RK KeyBoard 68K` and Sonix driver matching).
  - **Bootloader Mode:** `0C45:82AA` (Sonix ISP Bootloader Device).
  - Open tooling must probe for both `05AC:024F` and `0C45:8018` (as well as `0C45:8016` and `0C45:8008`) by HID usage page rather than relying on a single hard-coded ID.

---

### 4. Switch Matrix & Pin Map
- **Resolution:** The switch scanning matrix is **8 Rows × 16 Columns** (128 scan positions, 68 keys populated):
  - Confirmed directly from the firmware scan table at offset `0x00006D7B` (16 column entries of 8 bytes each).
  - **Row Drive Lines:** Port 0 (`P0.8`–`P0.15`, 8 outputs).
  - **Column Sense Lines:** Port 3 (`P3.0`–`P3.11`, multiplexed inputs with pull-ups).
  - `P1.0` and `P1.1` are dedicated to the hardware SPI interface (`SN_SSP0_SCK`, `SN_SSP0_MOSI`) and do not share lines with switch rows.

---

### 5. Lighting Mode Names
- **Resolution:** Mode names are standardized to the official vendor strings in `vendor/configurator/app/language/1033.lan`:
  - `0`: Static, `1`: SingleOn, `2`: SingleOff, `3`: Glittering, `4`: Falling, `5`: Colourful, `6`: Breath, `7`: Spectrum, `8`: Outward, `9`: Scrolling, `10`: Rolling, `11`: Rotating, `12`: Explode, `13`: Launch, `14`: Ripples, `15`: Flowing, `16`: Pulsating, `17`: Tilt, `18`: Shuttle, `19`: Close Backlight (Off).
  - Total: 20 modes (18 dynamic animation effects + Static + Off).

---

### 6. Per-Key RGB Buffer Structure
- **Resolution:** The 1,216-byte (`0x4C0`) buffer sent in 19 64-byte reports consists of **76 records of 16 bytes each** (`76 * 16 = 1,216 bytes`):
  - Each record stores: Byte 0 (status flag `0x80`), Bytes 1–3 (R, G, B components), Bytes 8–11 (key parameters/indices), and Bytes 14–15 (fixed delimiter `0xAA, 0x55`).
  - In `DeviceDriver.exe` (`0x439BA6`), the buffer length `0x4C0` (19 blocks) is decremented (`dec esi`) to `18` (`0x12`), which is placed into Byte 8 of the `04 13` header packet to declare the 18 subsequent payload chunk packets following the initialization block.

---

### 7. Disassembly Snippet (Bootloader Jump)
- **Resolution:** The raw firmware instructions around `0x00001790` and `0x0000014C` in `SN32F240B_32.bin` were verified:
  ```arm
  0x178C: ldr  r1, [pc, #0x110]   ; r1 = 0x200002A8 (EP0 buffer pointer)
  0x178E: strb r0, [r1, #0x12]
  0x1790: mov  r0, r1             ; r0 = 0x200002A8
  0x1792: ldr  r0, [r0, #0x14]    ; Load payload bytes 0-3
  0x1794: ldr  r1, [pc, #0x10c]   ; Literal 0x5A8942AA (AA 42 89 5A)
  0x1796: cmp  r0, r1
  0x1798: bne  #0x17ba
  0x179A: ldr  r0, [pc, #0x104]   ; Reload r0 = 0x200002A8 from literal pool!
  0x179C: ldr  r0, [r0, #0x18]    ; Load payload bytes 4-7
  0x179E: ldr  r1, [pc, #0x108]   ; Literal 0xCC6271FF (FF 71 62 CC)
  0x17A0: cmp  r0, r1
  0x17A2: bne  #0x17ba
  ...
  0x17B6: bl   #0x14c
  ...
  0x014C: ldr  r0, [pc, #4]       ; r0 = 0x40060024 (SYS0 memory remap register)
  0x014E: str  r1, [r0]           ; Set remap to 0 (ROM bootloader at 0x0)
  0x0150: ldr  r0, [pc, #4]       ; r0 = 0x1FFF0301 (ROM bootloader entry vector)
  0x0152: bx   r0                 ; Branch into ROM bootloader
  ```
  The original transcript slipped by omitting the `ldr r0, [pc, #0x104]` reload at `0x179A`. The raw code is completely self-consistent.
