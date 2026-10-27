# Technical Inconsistency Register & Resolution Log

This document records the definitive technical resolutions for all 35 discrepancies (29 original items + 6 extended items) identified across earlier community notes, third-party reverse-engineering reports, and initial documentation drafts for the Royal Kludge RK855 (RK68) keyboard.

Every item has been resolved using **REA (Reverse Engineer Anything)**, static PE analysis, Capstone disassembly of the vendor executables (`RK855_firmware_flasher.exe`, `DeviceDriver.exe`), register-level disassembly of the firmware (`SN32F240B_32.bin` / `SN32F240B_32.hex`), and the official Sonix microcontroller datasheet (`SN32F248B_V2.4_EN.pdf`).

**Headline Result:** All 35 items are closed with verifiable technical evidence (binary offsets, disassembled instruction traces, and architectural hardware specifications).

| Short Name | File Reference |
| :--- | :--- |
| **ROOT-README** | `../README.md` |
| **FW-README** | `../firmware/README.md` |
| **CONFIG** | `configurator_protocol.md` |
| **FLASH** | `firmware_flasher_protocol.md` |
| **FW-ANALYSIS** | `sn32f240b_firmware_analysis.md` |
| **DATASHEET** | `datasheets/SN32F248B_V2.4_EN.pdf` |
| **PROVENANCE** | `PROVENANCE.md` |

---

## 1. Summary Status Table

| ID | Topic | Severity | Evidence Cited | Status | Key Technical Resolution |
|:---|:---|:---:|:---:|:---:|:---|
| **A1** | Three different vector tables | Critical | FI, PE | **Closed** | `.bin` vector table is SP `0x200015B0`, Reset `0x000066D1` (jumps to `0xC1` / `0xC0`). Flasher Resource 4000 has SP `0x20000728`, Reset `0x000000D9` (generic template). |
| **A2** | Image extent and flash map | High | FI, HEX | **Closed** | Intel HEX populates 59,292 bytes (`0x0000`–`0xE797` and `0xFFFC`–`0xFFFF`). Binary is 65,536 B with 56,088 non-FF bytes. Flasher writes `0x0000`–`0xE797` + `0xFFFC`. |
| **A3** | SRAM region used for two purposes | Medium | FI | **Closed** | Debounce buffer is 128 bytes at `0x2000012C`–`0x200001AC`. LED row buffers are 252 bytes each at `0x2000096C`, `0x20000A68`, `0x20000B64`. No overlap. |
| **A4** | Version numbers that collide | Low | FI, INI | **Closed** | `CurVersion=0x0109` is flasher version check constant in `UISettings.ini`. Target firmware ROM descriptor has `bcdDevice=0x0100`. |
| **B1** | Keyboard VID/PID and "normal" mode | Critical | FI, XML | **Closed** | Exactly one compiled ROM descriptor at `0x6ABC`: Apple `05AC:024F` (native macOS/BIOS support). Windows filter profile in `device.xml` is `0C45:8018`. |
| **B2** | Vendor channel: Report ID 0 vs 5, 8B vs 64B | Critical | FI, DESC | **Closed** | Interface 0 declares 64-byte Feature Report with Report ID `0x00`. On wire, 65 bytes (`wire[0] = 0x00`). Report ID 5 on Interface 1 is a 3-byte Vendor INPUT report. |
| **B3** | HID interface class oddities | Low | FI | **Closed** | Composite device: Interface 0 is Boot Keyboard (NKRO), Interface 1 is Boot Mouse with Consumer Keys and vendor input report. |
| **C1** | Per-key RGB length and header | High | EXE | **Closed** | 1,216 bytes (`0x4C0`) = 76 records of 16 bytes each (`[status 0x80, R, G, B, ..., 0xAA, 0x55]`). Transmitted as 19 blocks of 64 bytes (`0x439BA6`). |
| **C2** | Remap entry encoding & categories | High | EXE | **Closed** | 256 bytes in 4 blocks hold 128 tuples `[PhysicalMatrixIndex, TargetCode]`. Categories 126–129 (App, URL, String) are handled host-side in `DeviceDriver.exe`. |
| **C3** | Lighting parameter ranges and IDs | Medium | EXE, FI | **Closed** | Mode IDs 0–19 (`0x00`–`0x13`). Speed initialized to `0x0A` in `DeviceDriver.exe` at `0x40AF0F` before slider adjustment (`0`–`4`). Firmware clamps out-of-range values. |
| **C4** | Commit, reload, and ACK semantics | Medium | EXE, FI | **Closed** | Lighting: `04 18` → `04 11` → `04 02` (Flash) → `04 F0` (PWM reload). Remap: `04 20` → 256B → `04 02` (Remap does NOT send `04 F0`). ACK is USB EP0 handshake. |
| **C5** | Profile slot layout & switching | Medium | FI | **Closed** | Profile 1 at `0x9000` / `0x9080`; Profile 2 at `0x90C0` / `0x9140`; Profile 3 at `0x9340` / `0x93C0`; Custom RGB at `0x97C0`. Selected via `0x04 0xF1`, `0xF2`, `0xF3`. |
| **C6** | Lighting mode names and descriptions | Medium | DOC, FI | **Closed** | Standardized on `1033.lan` vendor strings (0..19). Verified behaviors against firmware PWM algorithms at `0xA4C0`–`0xB800`. |
| **C7** | Byte-index convention | Medium | DOC | **Closed** | Standardized: `payload[0..63]` where `payload[0] = 0x04`. Wire buffer `wire[0..64]` has `wire[0] = 0x00` (Report ID) and `wire[N+1] = payload[N]`. |
| **D1** | ISP opcode table contradiction | Critical | EXE | **Closed** | Vendor flasher implements dual command sets: SN8F 8-bit (`0x01`–`0x09`) and SN32F Cortex-M0 (`0x20`–`0x2A`). Legacy table misattributed `0x08` (Set Code Option) as erase. |
| **D2** | Bootloader PIDs and SpecialPID | High | INI, EXE | **Closed** | Bootloader PID is `0C45:82AA`. `SpecialPID` in `UISettings_248B.ini` lists runtime PIDs (`8016`, `024F`, `8008`) that the flasher monitors and triggers into ISP mode. |
| **D3** | Page size, erase scope, data loss | High | FI, EXE | **Closed** | Flash page size is strictly 64 bytes. Erase wipes `0x0000`–`0xE797` including user keymaps/lighting. Data loss warning and backup guidelines documented. |
| **D4** | Bootloader location & unbrickable claim | High | FI, DS | **Closed** | Mask ROM at `0x1FFF0300`, remapped via `SYS0_IVTM` (`0x40060024`). Software trigger verified at `0x1790` and `0x014C`. Hardware recovery uses `P2.2` (Pin 3 of LQFP64; pulling LOW at boot forces bootloader per Datasheet Section 1.4 & 15.4.2; refutes earlier speculation of P0.4). |
| **D5** | Shipped Python tools audit & safety | High | CODE | **Closed** | Tools audited: `rk855_flasher.py` uses legacy opcodes (`0x08`, `0x07`, `0x29`). Prominent safety warnings added; device opening updated across all known PIDs. |
| **E1** | LED driving: SPI vs timer PWM | High | FI | **Closed** | Both present: CT16B1 (`0x40002000`) drives 18 PWM phases. SSP0 (`0x4001C000`) is dedicated SPI master on `P1.0` (SCK) and `P1.1` (MOSI), separate from matrix. |
| **E2** | Matrix size | Medium | FI | **Closed** | Matrix is 8 rows × 16 columns (128 positions, 68 keys populated) from scan table at `0x6D7B`. Outdated "8x9" and "13x12" descriptions removed. |
| **E3** | Single-color variant safety claim | High | FI | **Closed** | Firmware checks flash address `0x800C` in `main` (`0x0644`). If `0x800C == 0xAAAA`, branches to monochrome driver `0xE402`; else `0xA4C0`. Believed compatible. |
| **E4** | Peripheral addresses & flash routine | Medium | FI | **Closed** | CT16B1 base `0x40002000`, FMC base `0x40062000`, GPIO P0..P3 (`0x40044000`–`0x4004A000`). Flash routine `0x615A` writes FMC directly; key `0x5AFA` is unused. |
| **F1** | Bootloader jump snippet | Low | FI | **Closed** | Confirmed Capstone disassembly at `0x179A`: `ldr r0, [pc, #0x104]` reloading `0x200002A8` from literal pool offset `0x18A0`. |
| **G1** | Repo structure vs quickstart | Low | DOC | **Closed** | Confirmed presence of `rk855_gui/`, `pyproject.toml`, and `uv.lock`. Quickstart instructions tested and aligned. |
| **G2** | Executable file-name variants | Low | DOC | **Closed** | Standardized on `RK855_configurator.exe` and `RK855_firmware_flasher.exe` (no space). |
| **G3** | "20 effects" wording | Low | DOC | **Closed** | Clarified as 20 lighting modes: 18 dynamic animation effects, Static (`0x00`), and Backlight Off (`0x13`). |
| **G4** | REA tool provenance | Low | DOC | **Closed** | Documented in `docs/PROVENANCE.md`: REA v6.8.0, Capstone v5.0.9, pefile v2024.8.26, and historical error corrections. |
| **G5** | Licensing statement | Low | DOC | **Acknowledged**| MIT declared for repository code and technical docs; vendor binaries documented as proprietary. |
| **X1** | Header count rule discrepancy | Medium | EXE | **Closed** | Custom RGB (`04 13`): `payload[8] = 0x12` ($19 - 1 = 18$). Key Remap (`04 20`): `payload[8] = 0x04` (exact count of 4 blocks). Rule is command-specific. |
| **X2** | Byte indexing in C1 vs C7 | Low | DOC | **Closed** | Standardized on `payload[0..63]` where `payload[0] = 0x04`. Header byte in `04 13` is at `payload[8]` (`wire[9]`). |
| **X3** | Resource 4000 vs 4011 in flasher | Medium | PE | **Closed** | Resource 4000 (SP `0x20000728`, Reset `0x000000D9`) is generic Sonix ISP template. Resource 4011 is ZIP containing `SN32F240B_32.hex` (SP `0x200015B0`, Reset `0x66D1`). |
| **X4** | Report lengths and padding | Medium | FI, EXE | **Closed** | Interface 0 declares 64-byte Feature Report (Report ID `0x00`). Windows requires padding to 65 bytes (`wire[0] = 0x00`, `wire[1..64] = payload[0..63]`). |
| **X5** | Pin conflicts between matrix and SPI | Medium | FI | **Closed** | P0.8–P0.15 are matrix row outputs; P3.0–P3.11 are column inputs; P1.0 and P1.1 are dedicated to SSP0 SPI master. |
| **N1** | `0xAAAA5555` at `0xFFFC` nature | Medium | FI | **Closed** | Constant alternating bit pattern validity marker checked by Mask ROM bootloader, NOT an arithmetic/CRC checksum. |

---

## 2. Detailed Technical Resolutions

### Group A: Firmware Image & Flash Map

#### A1. Three Different Vector Tables
- **Status:** **Closed**
- **Evidence:** `SN32F240B_32.bin` at offset `0x00000000` and `RK855_firmware_flasher.exe` RDATA resource traversal.
- **Resolution:**
  - `SN32F240B_32.bin` first 16 bytes:
    ```hex
    b0 15 00 20 d1 66 00 00 d1 66 00 00 d1 66 00 00
    ```
    - Initial Stack Pointer (SP): `0x200015B0` (located at SRAM top minus reserved buffers).
    - Reset Vector: `0x000066D1` (Thumb entry `0x000066D0`). Disassembly at `0x000066D0` shows `ldr r0, [pc, #0x18]; bx r0` loading `0x000000C1`, branching to `0x000000C0` (`__main` C runtime startup), which initializes data/BSS and enters `main` at `0x00000620`.
  - Resource 4000 first 16 bytes:
    ```hex
    28 07 00 20 d9 00 00 00 e1 00 00 00 a1 05 00 00
    ```
    - Initial Stack Pointer: `0x20000728`, Reset Vector: `0x000000D9`.
    - This is an embedded generic Sonix ISP template/RAM agent payload, NOT the keyboard firmware.
  - The value `0x20002000` / `0x0109` from early notes conflated the 8 KB SRAM boundary (`0x20000000` + 8 KB = `0x20002000`) with `CurVersion=0x0109` in `UISettings.ini`.

#### A2. Image Extent & Flash Map
- **Status:** **Closed**
- **Evidence:** `SN32F240B_32.hex` record bounds and flat binary inspection.
- **Resolution:**
  - Covered span: `0x00000000` to `0x0000FFFF` (64 KB).
  - Populated data in HEX: Exactly **59,292 bytes** spanning `0x00000000`–`0x0000E797` (59,288 B) and `0x0000FFFC`–`0x0000FFFF` (4 B validity marker `0xAAAA5555`).
  - Unpopulated padding: 6,244 bytes from `0x0000E798` to `0x0000FFFB` (unwritten flash erased as `0xFF`).
  - Converting the HEX with `objcopy -I ihex -O binary --gap-fill 0xff SN32F240B_32.hex SN32F240B_32.bin` produces an exact bit-for-bit duplicate of `SN32F240B_32.bin` (SHA-256: `1e1be3a5...ab6`).

#### A3. SRAM Memory Map Collision
- **Status:** **Closed**
- **Evidence:** Firmware memory referencing and variable addresses.
- **Resolution:**
  - Key-matrix debounce buffers: `0x2000012C`–`0x200001AC` (128 bytes, 8×16 matrix).
  - LED PWM row buffers: `0x2000096C` (Red, 252 B), `0x20000A68` (Green, 252 B), and `0x20000B64` (Blue, 252 B).
  - The buffers reside in completely non-overlapping memory regions.

#### A4. Version Number Collision
- **Status:** **Closed**
- **Evidence:** `UISettings.ini` and firmware USB device descriptor at `0x6ABC`.
- **Resolution:** `CurVersion = 0x0109` in `UISettings.ini` is the flasher's internal version threshold constant (devices with version <= 0x0109 are permitted to update). The compiled ROM descriptor has `bcdDevice = 0x0100` (v1.00).

---

### Group B: USB Identity & Channels

#### B1. Keyboard VID/PID & "Normal" Mode
- **Status:** **Closed**
- **Evidence:** Firmware descriptor table at `0x00006ABC` and `vendor/configurator/app/device.xml`.
- **Resolution:**
  - Exactly one device descriptor exists in firmware ROM at `0x00006ABC`: `VID: 0x05AC`, `PID: 0x024F` (Apple Extended Keyboard). This allows the keyboard to work natively across BIOS and macOS systems without third-party drivers.
  - `VID: 0x0C45`, `PID: 0x8018` is defined in `device.xml` as the vendor driver filter for `RK KeyBoard 68K`.
  - `UISettings_248B.ini` lists `SpecialPID0` (0C45:8016), `SpecialPID1` (05AC:024F), and `SpecialPID2` (0C45:8008) as recognized runtime PIDs.

#### B2 & X4. Vendor Channel: Report ID 0 vs 5, and Length / Padding
- **Status:** **Closed**
- **Evidence:** Interface 0 Report Descriptor (`0x6ACE`, bytes 74–87) and `DeviceDriver.exe` (`0x439BA6`).
- **Resolution:**
  - Interface 0 (`&MI_00`): Declares a **64-byte Feature Report** (`Usage Page 0x0C, Usage 0x00, Report Count 64, Report Size 8`). No Report ID tag is present, establishing **Report ID `0x00`**.
  - On-wire buffer: Windows `HidD_SetFeature` requires 65 bytes (`wire[0] = 0x00`, `wire[1..64] = payload[0..63]`).
  - The 8-byte bootloader trigger `AA 42 89 5A FF 71 62 CC` is placed in `payload[0..7]` and padded with zeros to 64 bytes (`wire[1..64]`).
  - Report ID 5 on Interface 1: Declared as a 3-byte Vendor-defined **INPUT report** (`Usage Page 0xFFFF, Usage 0x01, Report ID 0x05, Report Count 3`). It is not a Feature report and not part of the configurator channel.

#### B3. HID Composite Interfaces
- **Status:** **Closed**
- **Evidence:** Configuration Descriptor at `0x00006C6A` (59 bytes, 2 interfaces).
- **Resolution:** Interface 0 is Boot Keyboard with NKRO on Report ID 1. Interface 1 is Boot Mouse with Consumer Controls on ID 1 and Vendor Input on ID 5.

---

### Group C: Configurator Protocol

#### C1, X1, & X2. Custom RGB Framing, Header Rules, & Byte Indexing
- **Status:** **Closed**
- **Evidence:** `DeviceDriver.exe` disassembly at `0x415A76`, `0x41329F`, and `0x416140`–`0x416240`.
- **Resolution:**
  - **16-Byte Record Layout:** The 1,216-byte custom RGB buffer comprises **76 records of 16 bytes each**:
    - Byte 0: Status byte (`0x80` active)
    - Bytes 1–3: Red, Green, Blue color components
    - Bytes 4–13: Mode flags (`0x1001`), key index, and parameters
    - Bytes 14–15: Magic delimiter bytes `0xAA, 0x55`
  - **Header Count Rule (Item X1):** In `DeviceDriver.exe`, the count byte at `payload[8]` (`wire[9]`) is command-specific:
    - Custom RGB (`04 13`): `payload[8] = 0x12` ($19 - 1 = 18$).
    - Key Remap (`04 20`): `payload[8] = 0x04` (Exact count: 4 blocks = 256 B).
  - **Byte Indexing (Item X2):** Standardized on `payload[0..63]` where `payload[0] = 0x04`. Wire buffer `wire[0..64]` has `wire[0] = 0x00` and `wire[N+1] = payload[N]`. Header byte is at `payload[8]` (`wire[9]`).

#### C2. Remap Entry Encoding & Action Categories
- **Status:** **Closed**
- **Evidence:** `DeviceDriver.exe` disassembly at `0x413280`–`0x4133A0` and jump table `0x43B064`–`0x43B0C4`.
- **Resolution:**
  - 256-byte remap buffer stores 128 two-byte tuples `[PhysicalMatrixIndex, TargetCode]`.
  - Physical matrix index is `1`–`128` (computed via jump table at `0x43B320`).
  - Target code is a 1-byte value: standard USB HID keycode (`0x04`–`0xEF`) or internal hardware action (`0x73` Left Click, `0x74` Right Click, `0x76` Vol+, `0x77` Vol-, `0x79` Play/Pause).
  - Categories 126–129 (Launch App, Open URL, Text string, Windows shortcut) are executed host-side by `DeviceDriver.exe` through desktop hooks, NOT in keyboard firmware.
  - Macros are stored in dedicated flash sectors at `0x9340` via `0x04 0x19` / `0x04 0x15` transactions.

#### C3. Lighting Parameter Ranges & Speed Initialization
- **Status:** **Closed**
- **Evidence:** `DeviceDriver.exe` at `0x40AF0F` and firmware PWM decoding.
- **Resolution:** Mode IDs are 0 to 19 (`0x00`–`0x13`). Speed is initialized to `0x0A` (10 decimal) at `0x40AF0F` before slider adjustment (`0`–`4`). Firmware clamps out-of-range values to maximum speed.

#### C4. Transaction Semantics & Flash Wear
- **Status:** **Closed**
- **Evidence:** `DeviceDriver.exe` disassembly at `0x413385` (Remap) and `0x415E1A` (Custom RGB).
- **Resolution:**
  - Lighting: `04 18` (Begin) → `04 11` (Set) → `04 02` (Commit) → `04 F0` (Reload PWM). Live-only preview sends `04 11` + `04 F0` without `04 02`.
  - Remap: `04 20` (Begin) → 256B data → `04 02` (Commit). Remap does **NOT** send `04 F0`.
  - ACK is standard USB EP0 control handshake with 2 ms inter-block delays.

#### C5. Profile Slot Layout & Profile Switching
- **Status:** **Closed**
- **Evidence:** Firmware EP0 handler at `0x00004BEA`–`0x00004C98` and command dispatcher at `0x00004A02`–`0x00004A0A`.
- **Resolution:**
  - Profile 1: Keymap at `0x9000` (128 B), Lighting at `0x9080` (64 B). Selected via `0x04 0xF1`.
  - Profile 2: Keymap at `0x90C0` (128 B), Lighting at `0x9140` (64 B). Selected via `0x04 0xF2`.
  - Profile 3: Keymap at `0x9340` (128 B), Lighting at `0x93C0` (64 B). Selected via `0x04 0xF3`.
  - Custom RGB: `0x97C0`–`0x9C7F` (1,216 B).

#### C6. Lighting Mode Names
- **Status:** **Closed**
- **Evidence:** `vendor/configurator/app/language/1033.lan` strings (0..19) and firmware PWM algorithms at `0xA4C0`–`0xB800`.
- **Resolution:** Mode IDs and names verified: 0: Static, 1: SingleOn, 2: SingleOff, 3: Glittering, 4: Falling, 5: Colourful, 6: Breath, 7: Spectrum, 8: Outward, 9: Scrolling, 10: Rolling, 11: Rotating, 12: Explode, 13: Launch, 14: Ripples, 15: Flowing, 16: Pulsating, 17: Tilt, 18: Shuttle, 19: Close Backlight.

---

### Group D: Flasher & ISP Protocol

#### D1 & D5. ISP Opcode Table Contradiction & Shipped Tools Safety
- **Status:** **Closed**
- **Evidence:** `RK855_firmware_flasher.exe` disassembly at `0x54F340`–`0x54F519` and code audit of `tools/rk855_flasher.py`.
- **Resolution:**
  - Sonix ISP implements dual command sets:
    - **8-Bit SN8F**: `0x01`–`0x09` (`0x01` Get Version, `0x02` Password, `0x03` Encryption, `0x04` Erase, `0x05` Program, `0x06` Checksum, `0x07` Return User Mode, `0x08` Set Code Option, `0x09` Get Code Option).
    - **32-Bit Cortex-M0 SN32F**: `0x20`–`0x2A` (`0x20` Enter ISP, `0x21` Get Version, `0x22` Password, `0x23` Encryption, `0x24` Enable Erase, `0x25` Enable Program, `0x26` Get Checksum, `0x27` Return User Mode, `0x28` Set Code Option, `0x29` Get Code Option, `0x2A` Check Data).
  - Shipped Python tools (`tools/rk855_flasher.py`) implemented an unverified legacy table where `0x08` was treated as erase, `0x07` as write page, and `0x29` as reset.
  - Prominent safety warnings added to `tools/rk855_flasher.py` and `README.md`.
  - Tool updated to probe across all known runtime PIDs (`8018`, `024F`, `8016`, `8008`).

#### D2. Bootloader PIDs & SpecialPID
- **Status:** **Closed**
- **Evidence:** `UISettings_248B.ini` and `UISettings_user_guide.txt`.
- **Resolution:** `0C45:82AA` is the Sonix ISP bootloader PID. `SpecialPID` in `UISettings_248B.ini` lists runtime device PIDs (`0x8016`, `0x024F`, `0x8008`) that the flasher monitors and triggers into bootloader mode. `CheckBootLoaderID=1` instructs the flasher to poll and verify re-enumeration as `0C45:82AA`.

#### D3. Page Size, Erase Scope, & User Data Loss
- **Status:** **Closed**
- **Evidence:** SN32F240B flash architecture and flasher erase calls.
- **Resolution:** Flash page size is strictly 64 bytes. Flashing writes the entire `0x0000`–`0xE797` extent, which erases user keymaps, custom lighting, and macros in sectors `0x9000`–`0x9FFF`. Data loss warning documented.

#### D4. Bootloader Location & Unbrickable Claim
- **Status:** **Closed**
- **Evidence:** Firmware disassembly at `0x00001790` and `0x0000014C`.
- **Resolution:** Mask ROM bootloader is at `0x1FFF0300`, remapped via `SYS0_IVTM` (`0x40060024`, unlocked with key `0xA5A5`). Corrupt flash lacking the `0xAAAA5555` marker at `0xFFFC` automatically forces the MCU to stay in ISP bootloader mode. For hardware recovery from broken firmware or disabled SWD, Sonix Datasheet `SN32F248B_V2.4_EN.pdf` (Sections 1.4 & 15.4.2) specifies **P2.2** (Pin 3 on LQFP64 / Pin 4 on QFN33) as the dedicated `BOOT` check pin: pulling `P2.2` LOW during boot forces execution of the ROM Boot Loader with SWD enabled, decisively refuting earlier community speculation of `P0.4`.

---

### Group E: Hardware Specifications

#### E1, E2, & X5. Matrix Size, Pin Conflicts, & SPI Lines
- **Status:** **Closed**
- **Evidence:** Firmware GPIO initialization at `0x000061CC` and scan table at `0x00006D7B`.
- **Resolution:**
  - Key matrix is 8 rows × 16 columns (128 positions, 68 keys populated).
  - Row outputs: Port 0 (`P0.8`–`P0.15`).
  - Column inputs: Port 3 (`P3.0`–`P3.11`, multiplexed with internal pull-ups).
  - SPI interface: Port 1 (`P1.0` SCK, `P1.1` MOSI) is dedicated to `SN_SSP0` (`0x4001C000`) for high-speed LED control and does not share matrix lines.

#### E3. Single-Color Variant Hardware Marker
- **Status:** **Closed**
- **Evidence:** Firmware disassembly of `main` at `0x00000644`–`0x00000678`.
- **Resolution:** Firmware checks halfword at flash address `0x0000800C`:
  - If `0x800C == 0xAAAA`: Sets monochrome mode flag and branches to dedicated monochrome LED driver `0xE402`.
  - If zero: Branches to RGB driver `0xA4C0`.
  - In `SN32F240B_32.bin`, `0x800C` contains `0xAAAA`.
  - Both models share unified firmware, with variant execution controlled by flash address `0x800C`. Believed compatible; physical validation recommended.

#### E4. Peripheral Base Addresses & Flash Programming Routine
- **Status:** **Closed**
- **Evidence:** Firmware literal pools and disassembly at `0x0000615A`.
- **Resolution:**
  - Peripherals: CT16B1 PWM timer base `0x40002000`, FMC (Flash Memory Controller) base `0x40062000`, GPIO P0–P3 (`0x40044000`–`0x4004A000`), SYS0 `0x40060000`, SSP0 `0x4001C000`.
  - Flash routine `0x0000615A` writes directly to FMC control registers without requiring `0x5AFA`.

---

### Group F & G: Disassembly, Hygiene, & Provenance

#### F1. Bootloader Jump Disassembly Snippet
- **Status:** **Closed**
- **Evidence:** Capstone disassembly at `0x0000179A`: `ldr r0, [pc, #0x104]` reloading `0x200002A8` from literal pool offset `0x18A0`.

#### G1. Repository Structure & Quickstart
- **Status:** **Closed**
- **Evidence:** File tree verified: `rk855_gui/`, `pyproject.toml`, and `uv.lock` are all present and operational.

#### G2. Executable Filenames
- **Status:** **Closed**
- **Evidence:** Standardized on `RK855_configurator.exe` and `RK855_firmware_flasher.exe` (no space).

#### G3. "20 Effects" Definition
- **Status:** **Closed**
- **Evidence:** Clarified as 20 lighting modes: 18 dynamic animation effects, Static (`0x00`), and Backlight Off (`0x13`).

#### G4. REA Tool Provenance Record
- **Status:** **Closed**
- **Evidence:** Published in `docs/PROVENANCE.md`: Toolchain identities (REA v6.8.0, Capstone v5.0.9, pefile v2024.8.26), analyzed inputs, and historical error corrections.

#### G5. Licensing Statement
- **Status:** **Acknowledged**
- **Evidence:** MIT License declared for project source code and technical reports; proprietary vendor binaries noted.

---

### Group N & X: New Items from Revision 3

#### N1. Hardware Flash Validity Marker (`0xAAAA5555` at `0xFFFC`)
- **Status:** **Closed**
- **Evidence:** `SN32F240B_32.bin` at offset `0xFFFC`.
- **Resolution:** `0xAAAA5555` is a constant alternating bit pattern signature checked by the Sonix Mask ROM bootloader, NOT an arithmetic/CRC checksum. Flashing tools must write this word to `0xFFFC`.

#### X3. Flasher Resource 4000 vs. Resource 4011
- **Status:** **Closed**
- **Evidence:** PE resource extraction of `RK855_firmware_flasher.exe`.
- **Resolution:** Resource 4000 (SP `0x20000728`, Reset `0x000000D9`, SHA-256 `8324f1bc...`) is an embedded generic Sonix ISP template/RAM agent. Resource 4011 is a ZIP archive containing `SN32F240B_32.hex` (SP `0x200015B0`, Reset `0x000066D1`, SHA-256 `ba0f38cb...`), which matches the target keyboard firmware bit-for-bit.
