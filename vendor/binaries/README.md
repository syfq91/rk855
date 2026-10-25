# Vendor Executables

This directory contains the original factory binaries distributed by Royal Kludge for the RK855 (RK68) mechanical keyboard. Filenames have been normalized (replacing spaces with underscores) for cross-platform compatibility.

---

## Binary Inventory & Checksums

| File | Size | SHA-256 Checksum | MD5 Checksum | Type |
| :--- | :--- | :--- | :--- | :--- |
| **`RK855_configurator.exe`** | 9,116,150 B | `d31b8e8cda53c595809be2ea7f9c86b2bdb945e277a4e0f9f2504c53e2fbe7f0` | `0a4695f53eb1daf8a8b8bd9d41e03cde` | InstallShield 32-bit Setup Wrapper |
| **`RK855_firmware_flasher.exe`** | 1,917,952 B | `976d7c345492ea2f3e6a51d110b96570d9eddcce97f1cead45613f08a77d4796` | `c7e45a184e616c06b7ca3462eb895dcc` | PE32 GUI Executable (MFC / HFD ISP Tool) |

---

## 1. `RK855_configurator.exe`
- **Purpose**: Official Windows configuration GUI for keyboard backlighting, custom layouts, macros, and profile management.
- **Container**: InstallShield package storing `data1.cab`, `data2.cab`, and setup metadata in the overlay starting at offset `0x00133000`.
- **Extracted Contents**: The unpacked standalone application is located in `vendor/configurator/app/`.
- **Analysis Document**: [docs/configurator_protocol.md](../../docs/configurator_protocol.md)

## 2. `RK855_firmware_flasher.exe`
- **Purpose**: Official Windows firmware flashing utility for the RK855 keyboard.
- **Internal Payload**: Stores an embedded ZIP file in `RT_RCDATA` resource ID `4011` containing `SN32F240B_32.hex` and `UISettings.ini`.
- **Extracted Contents**: The flasher settings and UI bitmaps are located in `vendor/flasher/`.
- **Analysis Document**: [docs/firmware_flasher_protocol.md](../../docs/firmware_flasher_protocol.md)
