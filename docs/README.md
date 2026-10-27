# Technical Analysis & Protocol Documentation

This directory contains in-depth reverse-engineering reports detailing the hardware architecture, firmware implementation, and USB communication protocols of the **Royal Kludge RK855 (RK68)** mechanical keyboard.

---

## Documentation Index

| Document | Topic | Key Findings |
| :--- | :--- | :--- |
| **[configurator_protocol.md](configurator_protocol.md)** | Configurator Protocol | 65-byte HID Feature Report framing, 20 lighting modes, per-key RGB matrix streaming, key remapping tables, physical 68-key matrix coordinate map. |
| **[firmware_flasher_protocol.md](firmware_flasher_protocol.md)** | Flasher & Bootloader Protocol | Sonix dual-command ISP architecture (SN8F vs SN32F Cortex-M0 `0x20`–`0x2A`), USB HID unlock sequence (`AA 42 89 5A FF 71 62 CC`), 64-byte page flashing, hardware `P2.2` boot pin. |
| **[sn32f240b_firmware_analysis.md](sn32f240b_firmware_analysis.md)** | SN32F240B Firmware Analysis | ARM Cortex-M0 memory map, vector table disassembly, 8×16 switch matrix scanning GPIO pins, CT16B1 18-phase PWM timers, dedicated SPI master (`P1.0`/`P1.1`), `0x800C` variant detection. |
| **[inconsistencies.md](inconsistencies.md)** | Inconsistency Register & Log | Complete 35-item register cataloguing and definitively closing all historical discrepancies across documentation drafts with machine-level evidence. |
| **[PROVENANCE.md](PROVENANCE.md)** | Methodology & Provenance | Reverse-engineering toolchain (REA v6.8.0, Capstone v5.0.9, pefile), cryptographic digests of analyzed vendor artifacts, and historical error corrections. |
| **[rust_rewrite_plan.md](rust_rewrite_plan.md)** | Rust Implementation Plan | Architectural blueprint and phased roadmap for building a memory-safe, cross-platform Rust CLI/GUI configurator and flasher. |
| **[datasheets/](datasheets/)** | Hardware Datasheets | Official manufacturer reference manual: `SN32F248B_V2.4_EN.pdf` (Sonix SN32F240B/248B Series 32-bit Cortex-M0 MCU Datasheet v2.4). |

---

## Recommended Reading Order

1. **Start with [configurator_protocol.md](configurator_protocol.md)** to understand how the keyboard is controlled during normal operation over USB HID.
2. **Read [firmware_flasher_protocol.md](firmware_flasher_protocol.md)** to see how the keyboard transitions into the hardware bootloader and is flashed over USB ISP.
3. **Explore [sn32f240b_firmware_analysis.md](sn32f240b_firmware_analysis.md)** for a deep dive into the ARM Cortex-M0 binary, GPIO pin assignments, matrix scanning logic, and low-level peripheral registers.
4. **Consult [inconsistencies.md](inconsistencies.md)** for detailed technical resolutions to all 35 previously documented discrepancies.
5. **Review [PROVENANCE.md](PROVENANCE.md)** for toolchain verification and binary hashes.
