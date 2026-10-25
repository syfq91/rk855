# Technical Analysis & Protocol Documentation

This directory contains in-depth reverse-engineering reports detailing the hardware architecture, firmware implementation, and USB communication protocols of the **Royal Kludge RK855 (RK68)** mechanical keyboard.

---

## Documentation Index

| Document | Topic | Key Findings |
| :--- | :--- | :--- |
| **[configurator_protocol.md](configurator_protocol.md)** | Configurator Reverse Engineering | 65-byte HID Feature Report framing, 20 RGB lighting modes, per-key RGB matrix streaming, key remapping tables, physical 68-key matrix coordinate map. |
| **[firmware_flasher_protocol.md](firmware_flasher_protocol.md)** | Flasher & Bootloader Protocol | Sonix ISP bootloader commands, USB HID unlock sequence (`AA 42 89 5A FF 71 62 CC`), flash page erasing and programming sequence, payload extraction. |
| **[sn32f240b_firmware_analysis.md](sn32f240b_firmware_analysis.md)** | SN32F240B Microcontroller Firmware | ARM Cortex-M0 memory map, vector table disassembly, 8x9 switch matrix scanning GPIO pins, SPI RGB LED timing (`P1.0`/`P1.1`), hardware bootloader entry logic. |

---

## Recommended Reading Order

1. **Start with [configurator_protocol.md](configurator_protocol.md)** to understand how the keyboard is controlled during normal operation over USB HID.
2. **Read [firmware_flasher_protocol.md](firmware_flasher_protocol.md)** to see how the keyboard can be switched into the hardware bootloader and flashed over USB ISP without an external debugger.
3. **Explore [sn32f240b_firmware_analysis.md](sn32f240b_firmware_analysis.md)** for a deep dive into the ARM Cortex-M0 binary, GPIO pin assignments, matrix scanning logic, and low-level peripheral registers.
