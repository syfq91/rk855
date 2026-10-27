# Technical Deep-Dive Report: Royal Kludge RK855 Firmware (`SN32F240B_32.hex`)

## 1. Executive Summary

This report delivers a comprehensive reverse-engineering analysis of **`SN32F240B_32.hex`** (and its binary form `SN32F240B_32.bin`), the official factory firmware for the Royal Kludge RK855 (RK68) mechanical keyboard extracted from the firmware flasher utility.

Using **REA (Reverse Engineer Anything)**, ARM Cortex-M disassemblers, and Sonix architectural SVD specifications, the firmware has been analyzed from the vector table down to register-level hardware interactions. This document covers the memory layout, hardware peripherals, USB descriptor tree, matrix scanning pins, RGB LED driving pipeline, flash storage organization, and the hidden ROM bootloader entry mechanism.

---

## 2. Firmware Specifications & Memory Architecture

### 2.1 File Characteristics
- **Source File**: `SN32F240B_32.hex` (Intel HEX format, 166,843 bytes)
- **Converted Flat Binary**: `SN32F240B_32.bin` (65,536 bytes / exactly 64 KB)
- **Address Span**: `0x00000000` to `0x0000FFFF` (covers full 64 KB on-chip flash)
- **Active Code & Data**: 59,292 bytes populated in HEX across `0x00000000`–`0x0000E797` and `0x0000FFFC`–`0x0000FFFF` (56,088 non-0xFF bytes in 64 KB flat binary; 6,244 bytes unpopulated 0xFF padding between `0x0000E798`–`0x0000FFFB`)
- **Target CPU**: **ARM Cortex-M0** (ARMv6-M Thumb instruction set, little-endian)
- **Microcontroller**: **SONiX SN32F240B** / SN32F248B

### 2.2 Memory Map
```
+---------------------+ 0x00000000
|  Interrupt Vectors  |  (192 bytes: System exceptions & 32 Peripheral IRQs)
+---------------------+ 0x000000C0
|  Keil C Startup     |  (__main, data copy, BSS clear)
+---------------------+ 0x0000012C
|  Application Code   |  (Main loop, USB stack, Matrix scanner, RGB effects)
+---------------------+ 0x00006ABC
|  USB Descriptors    |  (Device, Config, HID, Report & String descriptors)
+---------------------+ 0x00007000
|  Factory Presets    |  (Default profile, lighting parameters, layout)
+---------------------+ 0x00007580
|  RGB & PWM Tables   |  (Gamma curves, preset color palettes)
+---------------------+ 0x00008000
|  Hardware Markers   |  (0x800C: 0xAAAA variant flag)
+---------------------+ 0x00008580
|  User Keymap Sectors|  (0x9000 P1, 0x90C0 P2, 0x9340 P3)
+---------------------+ 0x000097C0
|  Custom RGB Matrix  |  (1,216 bytes per-key RGB framebuffer to 0x0000E797)
+---------------------+ 0x0000E798
|  Unused Flash (0xFF)|  (6,244 bytes unwritten erase space)
+---------------------+ 0x0000FFFC
|  Flash Validity Word|  (4-byte Sonix hardware validity marker 0xAAAA5555)
+---------------------+ 0x0000FFFF
```

### 2.3 SRAM Layout (Base: `0x20000000`, 8 KB Total)
- **SRAM Base**: `0x20000000`
- **Initial Stack Pointer (SP)**: `0x200015B0` (Stack grows downward from `0x200015B0`)
- **Variables & Buffers**:
  - `0x200000B8` – `0x200000DC`: USB Endpoint 0 setup buffer & state machine.
  - `0x2000012C` – `0x200001AC`: Active key matrix debounced state & HID report staging buffers (128 bytes, corresponding to the 8×16 matrix).
  - `0x20000270` – `0x200002A8`: USB transfer descriptors & EP0 command staging buffers (`0x200002A8` reloaded in EP0 handler).
  - `0x2000096C` – `0x20000A67`: Red LED row PWM duty cycle buffer (6 rows × 42 bytes = 252 bytes).
  - `0x20000A68` – `0x20000B63`: Green LED row PWM duty cycle buffer (252 bytes).
  - `0x20000B64` – `0x20000C5F`: Blue LED row PWM duty cycle buffer (252 bytes).
  - `0x20000E66` – `0x20001400`: RGB LED framebuffer staging area (per-key RGB values).

---

## 3. Interrupt Vector Table & Reset Execution Flow

The vector table occupies the first 192 bytes (`0x00000000`–`0x000000BF`). Handlers have bit 0 set to indicate ARM Thumb execution:

| Vector | Address Offset | Value | Handler Name | Functional Role |
| :---: | :---: | :---: | :--- | :--- |
| **SP** | `0x00000000` | `0x200015B0` | Top of Stack | Stack pointer initialized in SRAM |
| **Reset** | `0x00000004` | `0x000066D1` | `Reset_Handler` (`0x66D0`) | Initial hardware reset entry point |
| **NMI** | `0x00000008` | `0x000066D5` | `NMI_Handler` | Default loop trap |
| **HardFault**| `0x0000000C` | `0x00000745` | `HardFault_Handler` | System crash recovery loop |
| **SVCall** | `0x0000002C` | `0x000066D9` | `SVC_Handler` | Supervisor call |
| **PendSV** | `0x00000038` | `0x000066DB` | `PendSV_Handler` | Context switch interrupt |
| **SysTick**| `0x0000003C` | `0x00006341` | `SysTick_Handler` (`0x6340`) | 1 ms system tick: debounces matrix, advances RGB animations |
| **IRQ 00** | `0x00000040` | `0x000066DF` | `NDT_IRQHandler` | SYS0 Non-maskable digital test |
| **IRQ 01** | `0x00000044` | `0x00000B27` | **`USB_IRQHandler` (`0x0B26`)** | **Full-Speed USB Engine, SIE, EP0/EP1/EP2 transactions** |
| **IRQ 06** | `0x00000058` | `0x0000558F` | **`SSP0_IRQHandler` (`0x558E`)** | **SPI0 Master: High-speed RGB LED driver communication** |
| **IRQ 16** | `0x00000080` | `0x00006253` | **`CT16B1_IRQHandler` (`0x6252`)** | **Timer CT16B1: LED multiplexing and PWM timing** |
| **IRQ 25** | `0x000000A4` | `0x00006425` | `WDT_IRQHandler` (`0x6424`) | Watchdog Timer refresh |
| **IRQ 28** | `0x000000B0` | `0x00000A07` | **`P3_IRQHandler` (`0x0A06`)** | **GPIO Port 3 interrupt: Key press wake from sleep** |
| **IRQ 29** | `0x000000B4` | `0x00000A31` | **`P2_IRQHandler` (`0x0A30`)** | **GPIO Port 2 interrupt: External switch changes** |

### 3.1 Reset Handler Execution Flow
Disassembly of `Reset_Handler` (`0x000066D0`) reveals how the MCU transitions from vector reset to C application logic:
1. `0x000066D0`: `ldr r0, [pc, #0x18]` loads literal `0x000000C1`.
2. `0x000066D2`: `bx r0` branches to `0x000000C0`.
3. `0x000000C0`: The Keil C runtime startup (`__main`) clears the BSS segment (`stm r1!, {r3, r4, r5, r6}` at `0x00000110`), initializes initialized data, and branches to `main` at `0x00000620`.

---

## 4. Hardware Peripherals & Memory-Mapped Registers (Item E4 Resolution)

Disassembly and literal pool extraction verify the following peripheral base addresses:

```
0x40002000  +--------------------------------------------------------+
            |  SN_CT16B1 (16-bit PWM Timer & Match Registers)       |
0x40010000  +--------------------------------------------------------+
            |  SN_USB (Full-Speed USB 2.0 SIE Controller)            |
0x4001C000  +--------------------------------------------------------+
            |  SN_SSP0 (Synchronous Serial Port / SPI0 Controller)   |
0x40044000  +--------------------------------------------------------+
            |  SN_GPIO0 (Port 0: Switch Matrix Row Outputs P0.8-P0.15)|
0x40046000  +--------------------------------------------------------+
            |  SN_GPIO1 (Port 1: P1.0 SCK, P1.1 MOSI, P1.2-P1.7)     |
0x40048000  +--------------------------------------------------------+
            |  SN_GPIO2 (Port 2: Status lines & hardware controls)   |
0x4004A000  +--------------------------------------------------------+
            |  SN_GPIO3 (Port 3: Switch Matrix Column Inputs P3.0-11)|
0x4005E000  +--------------------------------------------------------+
            |  SN_SYS2 (System Pin Multiplexing & Special Config)    |
0x40060000  +--------------------------------------------------------+
            |  SN_SYS0 (System Clocks, Reset Control, SYS0->REMAP)   |
0x40062000  +--------------------------------------------------------+
            |  SN_FMC (Flash Memory Controller)                      |
0x40082000  +--------------------------------------------------------+
            |  SN_USB_RAM (USB Packet Memory & Buffer Descriptors)   |
            +--------------------------------------------------------+
```

### 4.1 Verified Flash Programming Routine (`0x0000615A`)
The firmware's internal flash programming routine at `0x0000615A` writes user configuration directly to the hardware Flash Memory Controller (`0x40062000`):
```asm
0x615A: push {r4, lr}
0x615C: mov  r3, r0             ; r3 = Target Flash Address
0x615E: ldr  r0, [pc, #0x68]    ; r0 = 0x40062000 (SN_FMC base)
0x6166: ldr  r4, [pc, #0x60]    ; r4 = 0x40062000
0x6168: str  r0, [r4, #4]       ; FMC->CTRL: clear error flags
0x616A: movs r0, #1
0x616C: str  r0, [r4, #8]       ; FMC->CMD = 1 (Enable Program)
0x6170: str  r3, [r0, #0x10]    ; FMC->ADDR = target_address
; Loop: writes 32-bit data words into FMC->DATA (0x4006200C)
0x6188: ldr  r0, [r2]           ; Load word from source buffer
0x618A: str  r0, [r4, #0xc]     ; Store word to FMC->DATA
0x618E: adds r2, r2, #4
0x6190: subs r1, r1, #1
0x6192: cmp  r1, #0
0x6194: bne  #0x6186
; Trigger program execution
0x6198: ldr  r0, [r0, #8]
0x619C: orrs r0, #0x40          ; Bit 6 = Start Program
0x61A0: str  r0, [r4, #8]
; Poll busy bit (bit 0 of FMC->STATUS)
0x61A8: ldr  r0, [r4, #4]
0x61AC: lsls r0, r0, #0x1f
0x61B0: cmp  r0, #0
0x61B2: bne  #0x61A6            ; Wait until busy flag clears
0x61C2: pop  {r4, pc}
```
**Disassembly finding on `0x5AFA`**: The magic key `0x5AFA` does **NOT** appear anywhere in this function or in `SN32F240B_32.bin`. On the SN32F240B, runtime flash programming writes to user flash sectors directly through the FMC control registers without requiring a secret unlock keyword.

---

## 5. USB Subsystem & Descriptors

### 5.1 Device Descriptor (`0x6ABC`)
Located at offset `0x6ABC`:
```hex
12 01 00 02 00 00 00 40 ac 05 4f 02 00 01 01 02 00 01
```
- `bLength`: `0x12` (18 bytes)
- `bDescriptorType`: `0x01` (DEVICE)
- `bcdUSB`: `0x0200` (USB 2.0 Full Speed)
- `bDeviceClass`: `0x00` (Defined at Interface level)
- `bMaxPacketSize0`: `0x40` (64 bytes EP0 buffer)
- **`idVendor`**: **`0x05AC`** (Apple Inc. profile)
- **`idProduct`**: **`0x024F`** (Apple Extended Keyboard compatibility ID)
- `bcdDevice`: `0x0100` (Firmware version 1.00)
- `iManufacturer`: `1` ("SONiX")
- `iProduct`: `2` ("Krux Neo" / RK OEM core name)
- `bNumConfigurations`: `1`

### 5.2 USB Configuration & Interface Topology
Located at offset `0x6C6A` (Total Length: 59 bytes, 2 Interfaces):

#### Interface 0: Primary Keyboard (Boot Interface)
- **Interface Number**: `0` (`&MI_00`)
- **Class / SubClass / Protocol**: `0x03` (HID) / `0x01` (Boot) / `0x01` (Keyboard)
- **Endpoint `0x81` (EP1 IN)**: Max Packet Size: 16 bytes, Interval: 1 ms (1000 Hz polling rate).
- **HID Report Descriptor (`0x6ACE`, 89 bytes)**:
  - Full NKRO Keyboard bitmap (Usage Page `0x07`).
  - **Feature Report Channel (Bytes 74–87)**: Declared with `Report Count = 64, Report Size = 8` without a Report ID tag, establishing that the vendor configuration Feature Report has **Report ID `0x00`** and is **64 payload bytes long**.

#### Interface 1: Media, Power & Vendor Controls
- **Interface Number**: `1`
- **Class / SubClass / Protocol**: `0x03` (HID) / `0x01` (Boot) / `0x02` (Mouse/Consumer)
- **Endpoint `0x82` (EP2 IN)**: Max Packet Size: 16 bytes, Interval: 1 ms.
- **HID Report Descriptor (`0x6B7B`, 158 bytes)**:
  - Report ID 1: Consumer Multimedia Keys
  - Report ID 2: System Power Controls
  - Report ID 5: 3-byte Vendor-defined **INPUT report** (`Usage Page 0xFFFF, Usage 0x01`).

---

## 6. ROM Bootloader Entry Mechanism (Software Jump)

### 6.1 Command Reception & Trap Verification
Inside the EP0 setup packet handler at address `0x00001790`:

```asm
00001790: 0846      mov r0, r1              ; r1 = 0x200002A8 (EP0 buffer base address)
00001792: 4069      ldr r0, [r0, #20]       ; Load first 4 bytes of incoming payload (+0x14)
00001794: 4349      ldr r1, [pc, #268]      ; Load 0x5A8942AA (literal at 0x18A4)
00001796: 8842      cmp r0, r1              ; Check bytes: AA 42 89 5A
00001798: 0fd1      bne.n 0x17BA            ; Fail if mismatch
0000179A: 4148      ldr r0, [pc, #260]      ; Reload buffer pointer 0x200002A8 (literal at 0x18A0)
0000179C: 8069      ldr r0, [r0, #24]       ; Load next 4 bytes of incoming payload (+0x18)
0000179E: 4249      ldr r1, [pc, #264]      ; Load 0xCC6271FF (literal at 0x18A8)
000017A0: 8842      cmp r0, r1              ; Check bytes: FF 71 62 CC
000017A2: 0ad1      bne.n 0x17BA            ; Fail if mismatch
```

### 6.2 Bootloader Execution
When the 8-byte key `AA 42 89 5A FF 71 62 CC` is verified:
1. EP0 buffer is cleared at `0x000017A4`–`0x000017AC`.
2. The function branches to `0x0000014C`:
   ```asm
   0000014C: 4801      ldr  r0, [pc, #4]    ; Loads 0x40060024 (SN_SYS0->REMAP)
   0000014E: 6001      str  r1, [r0, #0]    ; Remaps ROM bootloader to 0x00000000
   00000150: 4801      ldr  r0, [pc, #4]    ; Loads 0x1FFF0301 (ROM Bootloader Vector)
   00000152: 4700      bx   r0              ; BRANCH DIRECTLY INTO BOOTLOADER!
   ```

---

## 7. Keyboard Matrix Scanner & Pin Assignment (Items E1, E2, X5 Resolution)

The matrix scanner function begins at `0x00004592` and is called periodically by the main loop.

### 7.1 Authoritative Hardware Pin Map
Firmware initialization at `0x000061CC` and scan routines confirm the pin assignments:

| Pin Group | Microcontroller Port | Hardware Function | Pin Details |
| :--- | :--- | :--- | :--- |
| **Matrix Row Outputs** | `GPIO Port 0` (`0x40044000`) | 8 Row Drive Lines (Outputs) | `P0.8`, `P0.9`, `P0.10`, `P0.11`, `P0.12`, `P0.13`, `P0.14`, `P0.15` |
| **Matrix Column Inputs**| `GPIO Port 3` (`0x4004A000`) | 16 Column Sense Lines (Inputs) | `P3.0` through `P3.11` (multiplexed with internal pull-ups) |
| **SPI Master Interface**| `GPIO Port 1` (`0x40046000`) | Dedicated High-Speed SPI | `P1.0`: `SSP0_SCK`, `P1.1`: `SSP0_MOSI` |
| **Hardware Boot Pin** | `GPIO Port 0` (`0x40044000`) | ISP Bootloader Force Pin | `P0.4` (Internal pull-up; pull LOW at power-on for hardware ISP) |

*(Note: `P1.0` and `P1.1` are dedicated to `SN_SSP0` at `0x4001C000` and do not share lines with switch rows).*

---

## 8. LED Subsystem & Hardware Variant Branching (Item E3 Resolution)

### 8.1 Controller Interfaces & PWM Engine
- **Timing & Multiplexing (`SN_CT16B1`)**:
  - Base Address: `0x40002000`, Match Registers at `0x40002040`.
  - 16-bit timer interrupt `CT16B1_IRQHandler` at `0x00006252` drives multi-channel PWM duty cycles.
- **18-Phase Scanning Loop (`0x00006082`)**:
  - Phases 0–5 (Rows 0–5, Red Phase): Fed from Red row buffer at SRAM `0x2000096C` (252 bytes).
  - Phases 6–11 (Rows 0–5, Green Phase): Fed from Green row buffer at SRAM `0x20000A68` (252 bytes).
  - Phases 12–17 (Rows 0–5, Blue Phase): Fed from Blue row buffer at SRAM `0x20000B64` (252 bytes).

### 8.2 Single-Color Variant Hardware Marker (`0x800C`)
Disassembly of `main` at `0x00000644`–`0x00000678` reveals how the firmware detects hardware variants:
```asm
0x0644: 2001        movs r0, #1
0x0646: 03C0        lsls r0, r0, #15        ; r0 = 0x8000
0x0648: 8980        ldrh r0, [r0, #0xc]     ; Load halfword at Flash 0x0000800C
0x064A: 4947        ldr  r1, [pc, #0x11c]   ; Load 0xAAAA (literal at 0x0768)
0x064C: 4288        cmp  r0, r1             ; Check if marker == 0xAAAA
0x064E: D102        bne  #0x0656
0x0650: 2001        movs r0, #1             ; If match, set Single-Color Mode flag = 1
0x0652: 4944        ldr  r1, [pc, #0x110]
0x0654: 7008        strb r0, [r1]
...
0x0666: 2800        cmp  r0, #0
0x0668: D006        beq  #0x0678            ; If flag == 0: branch to 0xA4C0 (RGB driver)
0x066A: F00D FBCA   bl   #0xE402            ; If flag == 1: branch to 0xE402 (Monochrome driver)
```
- In `SN32F240B_32.bin`, address `0x0000800C` contains `0xAAAA`.
- This proves that single-color vs. RGB execution is governed by a flash-time marker at `0x800C`.
- **Safety Status**: Believed compatible across both models; hardware validation on a sacrificial single-color keyboard recommended.

---

## 9. Flash Storage Sectors & Validity Marker (Items C5 & N1 Resolution)

### 9.1 Sector Allocation
Analysis of the firmware EP0 handler (`0x00004BEA`–`0x00004C98`) establishes the exact flash map:

| Address Range | Size | Allocation / Content |
| :---: | :---: | :--- |
| `0x00000000` – `0x00006FFF` | 28 KB | Main Program Code, C Runtime, Interrupt Handlers |
| `0x00007000` – `0x0000757F` | 1.3 KB | Factory Default Profile & Keymap Matrices |
| `0x00007580` – `0x00007FFF` | 2.6 KB | Lighting animation tables, gamma curves, color presets |
| `0x00008000` – `0x0000800F` | 16 B | Hardware Configuration Markers (`0x800C = 0xAAAA`) |
| **`0x00009000` – `0x0000907F`** | 128 B | **User Keymap Storage Profile 1** |
| **`0x00009080` – `0x000090BF`** | 64 B | **User Lighting Configuration Profile 1** |
| **`0x000090C0` – `0x0000913F`** | 128 B | **User Keymap Storage Profile 2** |
| **`0x00009140` – `0x0000917F`** | 64 B | **User Lighting Configuration Profile 2** |
| **`0x00009340` – `0x000093BF`** | 128 B | **User Keymap Storage Profile 3** |
| **`0x000093C0` – `0x000093FF`** | 64 B | **User Lighting Configuration Profile 3** |
| **`0x000097C0` – `0x0000E797`** | ~20.4 KB | **Custom Per-Key RGB Matrix (1,216 B) & Macro Streams** |
| `0x0000E798` – `0x0000FFFB` | 6.1 KB | Unused Erased Flash (`0xFF` padding, 6,244 bytes) |
| **`0x0000FFFC` – `0x0000FFFF`** | 4 B | **Hardware Flash Validity Marker (`0xAAAA5555`)** |

### 9.2 Validity Marker (`0xAAAA5555` at `0xFFFC`)
- The 4-byte word at `0xFFFC` is constant `0xAAAA5555` (little-endian byte sequence `55 55 AA AA`).
- This constant bit-alternating pattern is verified by the Sonix Mask ROM bootloader prior to executing user flash. If this marker is missing or corrupt (e.g. blank `0xFFFFFFFF`), the MCU remains in ISP bootloader mode (`0C45:82AA`).
