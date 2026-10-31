# Royal Kludge RK855 Hardware Recovery & Unbricking Guide

This guide details the hardware recovery procedure for the Royal Kludge RK855 (RK68) mechanical keyboard powered by the **SONiX SN32F240B / SN32F248B** ARM Cortex-M0 microcontroller.

If a firmware flash is interrupted (e.g. power loss mid-write), corrupted, or if an experimental custom build fails to boot, this hardware procedure forces the microcontroller into the permanent, read-only factory Mask ROM bootloader.

---

## 1. Safety & Architecture Overview

* **Mask ROM Bootloader**: Located at physical memory address `0x1FFF0300`. It is burned into silicon at the factory and **cannot be overwritten, corrupted, or erased by any flash command**.
* **Automatic Recovery**: If the on-chip flash memory lacks the 4-byte validity marker (`0xAAAA5555` at `0x0000FFFC`), the Mask ROM automatically stays in USB ISP bootloader mode (`0C45:82AA`) without requiring hardware pin grounding.
* **Forced Hardware Recovery**: If flash contains a valid marker but code crashes in a HardFault loop or disables USB/SWD, pulling the dedicated hardware boot pin **`P2.2`** LOW (to `GND`) during power-on overrides flash and forces Mask ROM entry.

> [!WARNING]
> This hardware recovery procedure requires opening the keyboard casing to access the printed circuit board (PCB). Disconnect the keyboard from USB power before disassembling.

---

## 2. Hardware Pin Specifications

According to the official SONiX Datasheet (*SN32F240B / SN32F248B Series 32-bit Micro-Controller with ARM Cortex-M0*, Sections 1.4 & 15.4.2):

| Pin Name | Package Pin | Direction / Mode | Function During Power-On Reset |
| :--- | :--- | :--- | :--- |
| **`P2.2` / `BOOT`** | **Pin 3** (LQFP64) / **Pin 4** (QFN33) | Input with internal weak pull-up | **Low (0V / GND)**: Force Mask ROM Bootloader mode with SWD enabled.<br>**High (3.3V)**: Normal boot into on-chip user flash. |
| **`GND`** | Pin 16, 32, 48, 64 (or USB shield) | Power Ground | Reference 0V ground |

*(Note: Earlier community documentation erroneously guessed `P0.4`. Static disassembly and the Sonix hardware datasheet verify that `P2.2` is the true dedicated boot check pin).*

---

## 3. Step-by-Step Unbricking Procedure

### Step 1: Disassemble the Keyboard
1. Unplug the USB cable from the keyboard and computer.
2. Remove keycaps to expose the mounting plate screws.
3. Unscrew all perimeter and center screws securing the PCB/plate assembly to the bottom case.
4. Carefully lift the PCB out of the plastic casing. If your model has a battery connector (wireless/dual-mode version), disconnect the battery immediately.

### Step 2: Locate the Microcontroller & Boot Pin
1. Locate the square **SONiX SN32F240B** chip on the back or front of the PCB (LQFP64 package with 16 pins per side).
2. Find **Pin 1** (identified by the small circular indent / dot in the corner of the chip package).
3. Counting counter-clockwise along the first edge:
   - Pin 1: `P2.0`
   - Pin 2: `P2.1`
   - **Pin 3**: **`P2.2` (BOOT pin)**
4. On many RK855 PCB revisions, a small circular test pad labeled `BOOT` or `P22` is located adjacent to Pin 3.

### Step 3: Ground `P2.2` and Connect USB
1. Use conductive tweezers, a paperclip, or a short jumper wire to bridge **Pin 3 (`P2.2`)** to any Ground point (e.g. the metal casing of the USB-C connector or an exposed `GND` solder pad).
2. **While holding the short to Ground**, plug the USB-C cable into your computer.
3. Keep the pin grounded for **1 to 2 seconds** after plugging in, then release the short.

### Step 4: Verify Bootloader Detection
Run the `rk855` probe command:

```bash
rk855 probe
```

Expected output:
```text
[01] 0C45:82AA [BOOTLOADER MODE (0C45:82AA)] | Iface: 0 | UsagePage: 0x0001 | Usage: 0x0000
     Product:      SN32F240B Bootloader
     Manufacturer: SONiX Technology Co., Ltd.
```

Or query the bootloader directly:

```bash
rk855 isp info
```

Expected output:
```text
============================================================
 Sonix ISP Bootloader Information
============================================================
  Chip ID:            0x240B
  Bootloader Version: 0x0102
  Microcontroller:    SONiX SN32F240B (Cortex-M0, 64 KB Flash, 8 KB SRAM)
  Hardware Support:   VERIFIED SUPPORTED (SN32F240B / SN32F248B)
```

### Step 5: Flash Factory Firmware
Once in bootloader mode, flash the verified factory firmware image:

```bash
rk855 isp flash firmware/SN32F240B_32.hex
```

The tool will:
1. Run pre-flight vector table and marker checks.
2. Connect to the bootloader and verify chip ID (`0x240B`).
3. Erase the application flash sectors.
4. Write 64-byte pages and verify every page byte-for-byte.
5. Safely reset the microcontroller.

### Step 6: Test and Reassemble
1. Once flashing completes, the keyboard will re-enumerate as `0C45:8018` (or `05AC:024F`).
2. Verify keypresses and backlighting:
   ```bash
   rk855 light set --mode breath --color cyan
   ```
3. Disconnect USB, reattach battery (if equipped), reseat PCB into the case, tighten screws, and reinstall keycaps.
