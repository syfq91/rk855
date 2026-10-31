# Protocol Status & Claims Verification Ledger

**Last Updated:** Phase 1 Verification Baseline  
**Target:** Royal Kludge RK855 (RK68) / SONiX SN32F240B Microcontroller

Per the **Ground Rules** of the Rust Rewrite Plan:
- `unverified`: Claim made in earlier documentation or notes without verified hardware or binary evidence.
- `verified-by-disassembly`: Claim verified via static PE analysis (`DeviceDriver.exe`, `RK855_firmware_flasher.exe`), Capstone disassembly of firmware (`SN32F240B_32.bin` / `.hex`), or official Sonix MCU datasheets.
- `verified-by-capture`: Claim verified by packet-for-packet comparison with USB Wireshark/USBPcap traffic traces.
- `verified-on-device`: Claim actively verified against physical keyboard hardware response.
- `blocker`: Critical discrepancy preventing safe implementation until tested.

---

## 1. Resolution of the 10 Plan Contradictions

| # | Topic | Original Conflict | Evidence & Resolution | Status |
|---|---|---|---|:---:|
| **1** | Vector Table | README: SP `0x20002000`, Reset `0x109`.<br>Analysis doc: SP `0x200015B0`, Reset `0x66D1`.<br>Flasher doc: SP `0x20000728`, Reset `0xD9`. | Initial 16 bytes of `SN32F240B_32.bin` at offset `0x00000000` are: `b0 15 00 20 d1 66 00 00`. Initial SP is `0x200015B0` (top of SRAM minus 2,640-byte reserved heap/stack buffers). Reset vector is `0x000066D1` (Thumb entry `0x66D0` jumping to C runtime `0xC0`). Resource 4000 in flasher PE (`0x20000728`, `0x000000D9`) is a generic template RAM agent. `0x20002000` was SRAM size conflated with `CurVersion=0x0109`. | `verified-by-disassembly` |
| **2** | Image Extent | README: 32 KB active code, config at `0xF800`.<br>Analysis doc: code beyond `0x8000`, 57,051 bytes used. | Total addressable span in Intel HEX is `0x00000000`–`0x0000FFFF` (64 KB). Populated non-0xFF bytes: exactly 56,088 in binary and 59,292 in HEX spanning `0x00000000`–`0x0000E797` and validity marker `0xAAAA5555` at `0xFFFC`. Configurator data resides at `0x9000`–`0x97FF`. | `verified-by-disassembly` |
| **3** | USB Identity | Firmware descriptor shows Apple `05AC:024F`. Configurator and flasher docs use `0C45:8018`. | Target firmware compiled ROM descriptor at `0x6ABC` hardcodes Apple `05AC:024F` (`bcdDevice=0x0100`). Vendor Windows filter profile in `device.xml` in `DeviceDriver.exe` filters on `0C45:8018`. Bootloader is `0C45:82AA`. Both candidate IDs are supported by the Rust tool. | `verified-by-disassembly` |
| **4** | Vendor Channel | Firmware descriptor shows vendor channel as Report ID 5 on Interface 1 (3-byte input). Configurator doc says Report ID 0. | Interface 0 declares a 64-byte Feature Report with Report ID `0x00`. On the USB wire (Windows & Linux hidraw), the packet buffer is 65 bytes (`wire[0] = 0x00`, `wire[1..64] = payload[0..63]`). Report ID 5 on Interface 1 is an unconfigured 3-byte Vendor Input Report. | `verified-by-disassembly` |
| **5** | SPI vs Matrix Pins | SSP0 listed on P1.0/P1.1, but P1.0–P1.4 were claimed as matrix rows. | GPIO mapping confirmed: P0.8–P0.15 are matrix row outputs; P3.0–P3.11 are matrix column inputs (8x16 matrix). P1.0 (SCK) and P1.1 (MOSI) are dedicated SSP0 SPI pins. CT16B1 (`0x40002000`) timer drives 18 PWM phases. | `verified-by-disassembly` |
| **6** | ISP Opcodes | Opcode table (`01`, `02`, `03`, `06`, `07`, `08`, `29`) vs Cortex-M0 commands (`20`–`2A`). | Vendor flasher `RK855_firmware_flasher.exe` embeds dual command dispatchers. SN8F 8-bit legacy uses `0x01`–`0x09`. SN32F Cortex-M0 uses `0x20`–`0x2A`. Legacy table erroneously treated `0x08` (Set Option) as erase. Erase is opcode `0x22`. | `verified-by-disassembly` |
| **7** | Flash Page Size | "64 bytes / 256 bytes depending on sector". | SN32F240B flash hardware specification (Datasheet Section 15 & FMC peripheral at `0x40062000`) has a fixed page size of **64 bytes** (`0x40`). Erase sector size is 4,096 bytes (4 KB). | `verified-by-disassembly` |
| **8** | Bootloader PIDs | `0x82AA` only vs `0x7000`–`0x7800` or `0x82AA`. | ROM bootloader PID is strictly `0C45:82AA`. `SpecialPID` keys in `UISettings_248B.ini` (`8016`, `024F`, `8008`) represent runtime application PIDs monitored for automatic ISP mode transitions. | `verified-by-disassembly` |
| **9** | Recovery Claim | "Unbrickable ROM bootloader" vs warning of grounding boot pin. | Bootloader resides in Mask ROM at `0x1FFF0300`. It cannot be overwritten. However, if corrupt flash prevents runtime software jump (`AA 42 89 5A...`), pulling pin `P2.2` (Pin 3 of LQFP64) LOW during power-on/reset forces hardware entry into Mask ROM bootloader. | `verified-by-disassembly` |
| **10** | Variant Safety | "100% safe for single-color and RGB" claimed from code inspection. | Disassembly of `main` at `0x0644` inspects flash configuration word at `0x800C`. If `0x800C == 0xAAAA`, branches to monochrome driver at `0xE402`; otherwise branches to RGB driver at `0xA4C0`. Both code paths are present in the unified binary. Hardware testing remains recommended. | `verified-by-disassembly` |

---

## 2. Configurator Protocol Claims Ledger

| Claim ID | Feature / Command | Wire / Payload Format | Status | Technical Details |
|---|---|---|:---:|---|
| **CFG-01** | Begin Lighting Config | `04 18 [0x00 * 62]` | `verified-on-device` | Captured in `test_1.pcap` at packet [0081] and verified on physical hardware. Initiates lighting transaction buffer in RAM. |
| **CFG-02** | Set Lighting Parameters | `04 11 <mode> <type> <R> <G> <B> <bright> <speed> <dir> [0x00 * 54]` | `verified-on-device` | Modes: 0..19 (`0x00`–`0x13`). Brightness: 0..4 (mapped to 0..15). Speed: 0..4 (mapped to 1..10, `0x0A` is driver maximum/default). Direction: 0 (LTR) or 1 (RTL). Color type: 0 (fixed), 1 (RGB), 7 (spectrum), 8 (rainbow). Verified on physical hardware in direct sequence mode. |
| **CFG-03** | Commit Configuration | `04 02 [0x00 * 62]` | `verified-on-device` | Captured in `test_1.pcap` (packets [0083], [0085], device ACKs with flash target `0x85D8`/`0x85C8`). Verified on physical hardware in `--persist` mode. Omitted in `--live` mode to prevent flash memory wear. |
| **CFG-04** | PWM Reload | `04 F0 [0x00 * 62]` | `verified-on-device` | Captured in `test_1.pcap` packet [0084] (10.9ms after commit). Verified on physical hardware: immediately triggers timer reload to display lighting configuration without reboot. |
| **CFG-05** | Profile Switching | `04 F1` (Profile 1), `04 F2` (Profile 2), `04 F3` (Profile 3) | `verified-by-disassembly` | Remaps active RAM pointer to Flash profile offsets `0x9000`, `0x90C0`, and `0x9340`. |
| **CFG-06** | Custom Per-Key RGB Header | `04 13 00 00 00 00 00 00 12 [0x00 * 55]` | `verified-on-device` | Captured in `test_1.pcap` packet [0082] and `test_2.pcap`. Payload byte 8 contains block count `0x12` (18 blocks). Precedes the 18 data blocks in 22-packet atomic transaction. Verified on physical hardware. |
| **CFG-07** | Custom Per-Key RGB Data | 18 blocks of 64 bytes (`0x480` = 1,152 bytes) | `verified-on-device` | Captured in `test_1.pcap` and `test_2.pcap`. Blocks 1–5 hold mode presets, Blocks 6–8 zero padding, Blocks 9–17 key records (`0x80` status), Block 18 active mode descriptor. Verified on physical hardware. |
| **CFG-08** | Begin Key Remapping | `04 20 00 00 00 00 00 00 04 [0x00 * 55]` | `verified-by-disassembly` | Payload byte 8 contains block count `0x04` (4 blocks = 256 bytes follow). |
| **CFG-09** | Key Remapping Table Data | 4 blocks of 64 bytes (256 bytes total) | `verified-by-disassembly` | 128 tuples: `[PhysicalMatrixIndex, TargetKeyCode]`. Standard categories (100–125) are mapped in hardware. Categories 126–129 (Launch App, Open URL, String Macro) require host software. |
| **CFG-10** | Begin Macro Configuration | `04 19 [0x00 * 62]` | `verified-by-disassembly` | Staged in flash user data block. |
| **CFG-11** | Macro Data Stream | `04 15 <seq> <len> [data...]` | `unverified` | Requires USB packet captures of user macro creation in vendor software to confirm timing / modifier byte layout. |
| **CFG-12** | ISP Bootloader Jump Trigger | `AA 42 89 5A FF 71 62 CC [0x00 * 56]` | `verified-by-disassembly` | Matched at ROM offset `0x1790` and `0x014C`. Disassembly validates branch to Mask ROM vector at `0x1FFF0300`. |
| **CFG-13** | Matrix / Status Read-Back | `04 F5 01` (start) then `04 F5 00` x 3 | `verified-by-capture` | Captured in `test_1.pcap` and `test_2.pcap`. Host queries status/matrix in 4 chunks of 32 bytes via `GET_REPORT` on Interface 0. |

---

## 3. Firmware Flasher & ISP Protocol Claims Ledger

| Claim ID | Feature / Command | Format / Values | Status | Technical Details |
|---|---|---|:---:|---|
| **ISP-01** | Bootloader Interface | VID `0x0C45`, PID `0x82AA` | `verified-by-disassembly` | Enumerates as USB HID device upon entering bootloader mode. |
| **ISP-02** | Get Bootloader Info | Opcode `0x2A` (Cortex-M0) | `verified-by-disassembly` | Returns chip ID (`0x240B` / `0x248B`) and Mask ROM bootloader version. |
| **ISP-03** | Erase Flash | Opcode `0x22` with erase address range | `verified-by-disassembly` | Sector erase (4 KB granularity). Legacy claim that opcode `0x08` erases is FALSE (`0x08` sets code options). |
| **ISP-04** | Write Flash Page | Opcode `0x24` followed by 64 bytes data | `verified-by-disassembly` | Writes exactly one 64-byte page to flash memory. |
| **ISP-05** | Read Flash Page | Opcode `0x25` with page offset | `verified-by-disassembly` | Reads back 64 bytes for verify pass. CodeSecurity bit in Option Word must permit readout. |
| **ISP-06** | Reset / Jump to Application | Opcode `0x27` | `verified-by-disassembly` | Disables USB, remaps vector table back to `0x00000000`, and triggers NVIC system reset. |
| **ISP-07** | Hardware Recovery Pin | Pull Pin `P2.2` (Pin 3 of LQFP64) LOW at boot | `verified-by-disassembly` | Datasheet Section 1.4 & 15.4.2 confirm P2.2 is the hardware boot pin (refuting previous speculation of P0.4). |
| **ISP-08** | Flash Validity Marker | `0xAAAA5555` at `0x0000FFFC` | `verified-by-disassembly` | Checked by Mask ROM on reset to confirm valid application image before branching. |

---

## 4. Open Blockers & Verification Plan

1. **CFG-11 (Macro Stream Layout):** Needs packet capture of vendor software recording a macro to verify byte-level delay/key formatting.
2. **Phase 3 Verified on Hardware:** Both atomic 22-packet transaction and direct sequence modes, along with `--live` (RAM update without flash wear) and `--persist` (flash commit `04 02`), have been verified against connected RK855 hardware (`0C45:8018`).
3. **Single-Color Variant Safety:** Retained as `unverified` pending physical testing on a monochrome LED board variant. Unified firmware binary contains code paths for both, but single-color users should exercise caution.
