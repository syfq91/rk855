# Reverse Engineering Provenance & Methodology

This document records the provenance, toolchain, inputs, and verification methodology used throughout the reverse engineering of the Royal Kludge RK855 keyboard artifacts.

---

## 1. Analysis Framework & Toolchain

The reverse engineering workflow utilized the **REA (Reverse Engineer Anything)** tool suite alongside standard static analysis engines and disassemblers:

| Component | Tool / Utility | Version / Environment | Primary Purpose |
| :--- | :--- | :--- | :--- |
| **REA CLI & Core** | `rea` (`rea-agents`) | v6.8.0 / Node.js 24 | PE header inspection, import table extraction, resource directory inventory, artifact comparison |
| **ARM Cortex-M Disassembler** | Capstone Engine | v5.0.9 (Python 3.12 bindings) | Thumb-2 disassembly of `SN32F240B_32.bin` from vector table to register interactions |
| **x86 PE Disassembler** | Capstone Engine | v5.0.9 (Python 3.12 bindings) | 32-bit x86 disassembly of `DeviceDriver.exe` and `RK855_firmware_flasher.exe` |
| **PE Parser** | `pefile` | v2024.8.26 | PE resource traversal, RDATA literal pool resolution, import thunk tracing |
| **Binary & HEX Utilities** | `objcopy`, `binutils` | GNU binutils 2.42 | Intel HEX to flat binary conversion with explicit gap filling (`--gap-fill 0xff`) |
| **Hardware Reference** | SONiX Datasheets & SVD | SONiX SN32F240B/248B Series Datasheet v2.4 (`SN32F248B_V2.4_EN.pdf`), SVD v1.2 | Verification of peripheral base addresses, FMC flash procedures, low-power keys, BOOT pin (P2.2), and CT16B1 PFPA pinout |

---

## 2. Analyzed Inputs & Artifact Identities

All findings are derived from shipped vendor binaries and firmware dumps preserved in this repository:

| Artifact | File Path | File Size | SHA-256 Digest |
| :--- | :--- | :--- | :--- |
| **Configurator Installer** | `vendor/binaries/RK855_configurator.exe` | 9,116,150 B | `d31b8e8cda53c595809be2ea7f9c86b2bdb945e277a4e0f9f2504c53e2fbe7f0` |
| **Standalone Driver** | `vendor/configurator/app/DeviceDriver.exe` | 1,620,992 B | `690e9b0311a9dd97663da0f66bd49e3f595ed693f3520189206f05eb7fbd244c` |
| **Firmware Flasher** | `vendor/binaries/RK855_firmware_flasher.exe` | 1,917,952 B | `976d7c345492ea2f3e6a51d110b96570d9eddcce97f1cead45613f08a77d4796` |
| **Factory Firmware (HEX)** | `firmware/SN32F240B_32.hex` | 166,843 B | `ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0` |
| **Factory Firmware (BIN)** | `firmware/SN32F240B_32.bin` | 65,536 B | `1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6` |
| **Embedded Flasher Resource 4000** | Extracted from `RK855_firmware_flasher.exe` | 65,536 B | `8324f1bc059cae33c21066c44ac8e18faa161b6389c9987dc99cdc4cd5b889c7` |
| **Official MCU Datasheet** | `docs/datasheets/SN32F248B_V2.4_EN.pdf` | 3,756,724 B | `fde84237c27a6248755cba5b6ca177e990a4f8a58a222256cc3aead0398f83dc` |

---

## 3. Historical Inconsistencies & Corrective Analysis

Earlier iterations of the technical documentation contained misattributions that originated from preliminary automated analysis or generic third-party reverse engineering templates. Systematic re-analysis identified and resolved the following historical errors:

1. **The Disassembly Snippet Error (Item F1)**:
   - *Previous claim*: An instruction in the bootloader jump snippet was described as omitting the pointer reload.
   - *Corrective evidence*: Disassembly of `SN32F240B_32.bin` at `0x0000179A` confirmed `ldr r0, [pc, #0x104]` correctly reloads buffer pointer `0x200002A8` from literal pool offset `0x18A0`.
2. **SRAM Buffer Overlap (Item A3)**:
   - *Previous claim*: Documentation previously placed matrix debounce buffers and RGB row PWM buffers in conflicting memory regions.
   - *Corrective evidence*: Disassembly proved that matrix debounce buffers occupy lower SRAM (`0x2000012C`–`0x200001AC`, 128 bytes for the 8×16 matrix), while the 18-phase LED PWM buffers reside at `0x2000096C`, `0x20000A68`, and `0x20000B64` (252 bytes each).
3. **The ISP Opcode Table Contradiction (Item D1)**:
   - *Previous claim*: The early flasher protocol table described `0x08` as "Erase Flash", `0x07` as "Write Page", and `0x29` as "Reset".
   - *Corrective evidence*: Static disassembly of `RK855_firmware_flasher.exe` (`0x54F340`–`0x54F519`) revealed that Sonix Cortex-M0 devices use an extended opcode set (`0x20`–`0x2A`), where `0x24` is Enable Erase, `0x25` is Enable Program, and `0x27` is Return User Mode. Opcode `0x08` corresponds to `Set Code Option` and `0x29` to `Get Code Option`.
4. **Custom RGB Buffer Framing & Header Rules (Items C1, X1)**:
   - *Previous claim*: Per-key RGB size was stated as 1,216 bytes without a field layout, and the header count rule was assumed identical across commands.
   - *Corrective evidence*: Disassembly of `DeviceDriver.exe` (`0x416140`–`0x416240`) established that the 1,216 bytes consist of 76 16-byte records terminated with magic bytes `0xAA, 0x55`. The header count rule at `payload[8]` was proven to be command-specific: Custom RGB (`04 13`) decrements block count to `0x12` ($19 - 1 = 18$), whereas Remap (`04 20`) sends the exact block count `0x04`.
5. **Hardware Boot Pin Identification (Item D4)**:
   - *Previous claim*: Community notes and earlier documentation speculated pulling `P0.4` LOW to force hardware bootloader mode.
   - *Corrective evidence*: Official Sonix Datasheet `SN32F248B_V2.4_EN.pdf` (Sections 1.4, 1.5, & 15.4.2) identifies **P2.2** (Pin 3 on LQFP64 / Pin 4 on QFN33) as the dedicated `BOOT` check pin (internal pull-up; tie LOW during boot to force Boot Loader with SWD enabled). Pin `P0.4` is documented as `SCK0 / CT16B1_PWM04A`.

---

## 4. Verification Boundaries & Open Hardware Tasks

Static firmware and binary analysis provide absolute certainty regarding what compiled machine code instructs the hardware to execute. However, certain runtime boundary behaviors require physical USB bus analyzers or hardware testing:
- **USB Capture Verification**: Validating the exact live on-wire exchange between the vendor flasher and a physical keyboard.
- **Marker Invalidation Test**: Testing whether erasing the `0xAAAA5555` marker at `0xFFFC` on a sacrificial board automatically forces Mask ROM fallback into ISP mode.
- **Monochrome Hardware Validation**: Validating the physical PWM output of single-color LED variant units against the `0x800C` hardware flag.
