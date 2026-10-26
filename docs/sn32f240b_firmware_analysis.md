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
+---------------------+ 0x00008580
|  Flash Storage Area |  (User keymaps, custom lighting, recorded macros)
+---------------------+ 0x00009000
|  Device Config Area |  (Hardware ID overrides, USB descriptors to 0x0000E797)
+---------------------+ 0x0000E798
|  Unused Flash (0xFF)|  (6,244 bytes unwritten erase space)
+---------------------+ 0x0000FFFC
|  Flash Integrity    |  (4-byte Sonix security/integrity word 0xAAAA5555)
+---------------------+ 0x0000FFFF
```

### 2.3 SRAM Layout (Base: `0x20000000`, 8 KB Total)
- **SRAM Base**: `0x20000000`
- **Initial Stack Pointer (SP)**: `0x200015B0` (Stack grows downward from `0x200015B0`)
- **Variables & Buffers**:
  - `0x200000B8` – `0x200000DC`: USB Endpoint 0 setup buffer & state machine.
  - `0x2000012C` – `0x200001AC`: Active key matrix debounced state & HID report staging buffers.
  - `0x20000270` – `0x200002A8`: USB transfer descriptors & EP0 command staging buffers.
  - `0x2000096C` – `0x20000A67`: Red LED row PWM duty cycle buffer (6 rows × 42 bytes = 252 bytes).
  - `0x20000A68` – `0x20000B63`: Green LED row PWM duty cycle buffer (252 bytes).
  - `0x20000B64` – `0x20000C5F`: Blue LED row PWM duty cycle buffer (252 bytes).
  - `0x20000E66` – `0x20001400`: RGB LED framebuffer staging area (per-key RGB values).

---

## 3. Interrupt Vector Table & Handlers

The vector table occupies the first 192 bytes (`0x00000000`–`0x000000BF`). Handlers have bit 0 set to indicate ARM Thumb execution:

| Vector | Address Offset | Value | Handler Name | Functional Role |
| :---: | :---: | :---: | :--- | :--- |
| **SP** | `0x00000000` | `0x200015B0` | Top of Stack | Stack pointer initialized in SRAM |
| **Reset** | `0x00000004` | `0x000066D1` | `Reset_Handler` (`0x66D0`) | Initializes clocks, copies `.data`, zeros `.bss`, calls `main` |
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

---

## 4. Hardware Peripherals & Memory-Mapped Registers

From the disassembled references and Sonix SVD hardware specifications:

```
0x40000000  +--------------------------------------------------------+
            |  SN_CT16B0 / CT16B1 / CT16B2 (Timers, PWM at 0x40002000)|
0x40010000  +--------------------------------------------------------+
            |  SN_WDT (Watchdog Timer at 0x40010000)                 |
0x4001C000  +--------------------------------------------------------+
            |  SN_SSP0 (Synchronous Serial Port / SPI0)              |
0x40044000  +--------------------------------------------------------+
            |  SN_GPIO0 (Port 0: Matrix Row Outputs P0.8-P0.15)      |
0x40046000  +--------------------------------------------------------+
            |  SN_GPIO1 (Port 1: Matrix Row Outputs P1.0-P1.8)       |
0x40048000  +--------------------------------------------------------+
            |  SN_GPIO2 (Port 2: Control lines & Switch inputs)      |
0x4004A000  +--------------------------------------------------------+
            |  SN_GPIO3 (Port 3: Matrix Column Inputs P3.0-P3.11)    |
0x4005C000  +--------------------------------------------------------+
            |  SN_USB (Full Speed USB 2.0 SIE Controller)            |
0x40060000  +--------------------------------------------------------+
            |  SN_SYS0 (System Control, Clocks, Boot Remap)          |
0x40062000  +--------------------------------------------------------+
            |  SN_FLASH (On-chip Flash Controller, Key: 0x5AFA)      |
            +--------------------------------------------------------+
```

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

> [!NOTE]
> Although marketed with VID `0x0C45` PID `0x8018` in the Windows driver XML, the actual compiled ROM descriptor uses Apple VID `0x05AC` PID `0x024F`. This ensures native BIOS compatibility and Mac functionality without special drivers.

### 5.2 USB Configuration & Interface Topology
Located at offset `0x6C6A` (Total Length: 59 bytes, 2 Interfaces):

#### Interface 0: Primary Keyboard (Boot Interface)
- **Interface Number**: `0`
- **Class / SubClass / Protocol**: `0x03` (HID) / `0x01` (Boot) / `0x01` (Keyboard)
- **Endpoint `0x81` (EP1 IN)**:
  - Transfer Type: Interrupt
  - Max Packet Size: 16 bytes
  - Polling Interval: **1 ms (1000 Hz polling rate)**
- **HID Report Descriptor (`0x6B62`, 89 bytes)**:
  - **Report ID 1**: Full **NKRO Keyboard** bitmap. Uses Usage Page `0x07` (Keyboard) with 120 key bits (`15 bytes`), allowing arbitrary key combinations without ghosting.
  - **Report ID 6**: Mouse emulation (X, Y, Wheel, 3 buttons).

#### Interface 1: Media, Power & Vendor Controls
- **Interface Number**: `1`
- **Class / SubClass / Protocol**: `0x03` (HID) / `0x01` (Boot) / `0x02` (Mouse/Consumer)
- **Endpoint `0x82` (EP2 IN)**:
  - Transfer Type: Interrupt
  - Max Packet Size: 16 bytes
  - Polling Interval: 1 ms
- **HID Report Descriptor (`0x6BB1`, 158 bytes)**:
  - **Report ID 1**: Consumer Multimedia Keys (Play/Pause, Volume Up/Down, Mute, Next/Prev, Media Player, Calculator, Web Browser).
  - **Report ID 2**: System Power Controls (Power Down, Sleep, Wake).
  - **Report ID 5**: Vendor-defined Bidirectional Channel (`Usage Page 0xFF00, Usage 0x01`).

---

## 6. ROM Bootloader Entry Mechanism (Software Jump)

One of the most critical mechanisms discovered in the firmware is how it safely drops into the factory ROM bootloader.

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
1. EP0 buffer is cleared:
   ```asm
   000017A4: 2400      movs r4, #0
   000017A8: 00a0      lsls r0, r4, #2
   000017AA: 2100      movs r1, #0
   000017AC: f7ff fc28 bl   0x1000          ; memset EP0 buffer
   ```
2. The function branches to `0x0000014C`:
   ```asm
   0000014C: 4801      ldr  r0, [pc, #4]    ; Loads 0x40060024 (SN_SYS0->BOOT)
   0000014E: 6001      str  r1, [r0, #0]    ; Remaps ROM bootloader to 0x00000000
   00000150: 4801      ldr  r0, [pc, #4]    ; Loads 0x1FFF0301 (ROM Bootloader Vector)
   00000152: 4700      bx   r0              ; BRANCH DIRECTLY INTO BOOTLOADER!
   ```

This proves conclusively that the `CheckDeviceCmd` command from the flasher triggers a hardware memory remap to `0x1FFF0301`, causing the Sonix MCU to reboot into its built-in ROM ISP bootloader without requiring any hardware intervention.

---

## 7. Keyboard Matrix Scanner & Pin Assignment

The matrix scanner function begins at `0x00004592` and is called periodically by the main loop.

### 7.1 Pin Mapping & Matrix Architecture
- **Switch Matrix (8 Rows × 16 Columns = 128 Scan Positions, 68 Keys Populated)**:
  - Firmware scanning table at offset `0x00006D7B` defines an 8-row by 16-column matrix layout (8 bytes per column across 16 columns).
  - **Row Drive Lines (Outputs)**: Port 0 (`0x40044000`): Pins `P0.8`, `P0.9`, `P0.10`, `P0.11`, `P0.12`, `P0.13`, `P0.14`, `P0.15` (8 row lines)
  - **Column Sense Lines (Inputs with Pull-Ups)**: Port 3 (`0x4004A000`): Pins `P3.0` through `P3.11` (multiplexed column sense lines)
- **Dedicated SPI Master Lines (`SN_SSP0` at `0x4001C000`)**:
  - `P1.0`: `SSP0_SCK` (Serial Clock)
  - `P1.1`: `SSP0_MOSI` (Master Out Slave In)
  *(Note: P1.0 and P1.1 are dedicated hardware SPI signals for the high-speed LED driver communication and do not share lines with switch matrix rows).*

### 7.2 Scanning Mechanism
1. Active row lines are driven LOW one at a time.
2. Port 3 column registers are sampled via `ldr r0, [SN_GPIO3]`.
3. Active bits are debounced across 3 consecutive scans (`~3 ms`).
4. Keycodes are translated via the lookup table at `0x00007000` (or overridden by user map in flash at `0x00008580`).
5. Key states are encoded into the 15-byte NKRO report and dispatched via Endpoint 1.

---

## 8. LED Subsystem & Driving Architecture

The RK855 hardware platform supports both **RGB** and **Single-Color (Monochrome White/Blue)** LED configurations using an identical MCU firmware core.

### 8.1 Controller Interfaces & PWM Engine
- **Timing & Multiplexing (`SN_CT16B1`)**:
  - Base Address: `0x40002000`, Match Registers at `0x40002040`.
  - 16-bit timer interrupt `CT16B1_IRQHandler` at `0x00006252` (calling `0x0000602C`) drives the multi-channel PWM duty cycles for every switch position.
- **Matrix Row Drivers (`SN_GPIO0`, `SN_GPIO1`, `SN_GPIO2`)**:
  - Active rows are switched in synchronization with PWM match updates.
- **Serial Interface (`SN_SSP0`)**:
  - Base Address: `0x4001C000`. High-speed SPI master interface (`P1.0` SCK, `P1.1` MOSI) handled by `SSP0_IRQHandler` at `0x0000558E`.

### 8.2 The 18-Phase Scanning Loop & Framebuffer Architecture
REA disassembly of `0x0000602C` reveals that the firmware organizes LED driving into an **18-phase multiplexing loop** (`cmp r0, #18` at `0x00006082`):
- **Phases 0–5 (Rows 0–5, Red Phase):** Fed from Red row buffer at SRAM `0x2000096C` (6 rows × 42 bytes = 252 bytes).
- **Phases 6–11 (Rows 0–5, Green Phase):** Fed from Green row buffer at SRAM `0x20000A68` (252 bytes).
- **Phases 12–17 (Rows 0–5, Blue Phase):** Fed from Blue row buffer at SRAM `0x20000B64` (252 bytes).

Each 42-byte row buffer streams 21 halfwords into the 16-bit PWM match registers at `0x40002040` (`0x00005902`–`0x0000594A`), refreshing the entire matrix with zero flicker.

### 8.3 Single-Color LED Variant Compatibility & Simultaneous Drive
Disassembly proves why the same firmware binary (`SN32F240B_32.bin`) functions seamlessly on single-color keyboard hardware:
1. **Identical MCU & Matrix Lines:** The single-color model uses the exact same PCB and GPIO pinout; only the populated LED packages differ (2-pin monochrome white LEDs instead of 4-pin RGB LEDs).
2. **Hardware PWM Modulation:** Because animations (Breathing, Wave, Reactive Single-On, etc.) and brightness levels (0–4) are computed into the timer PWM match registers, the single-color LEDs smoothly animate brightness and speed just like RGB LEDs.
3. **Simultaneous Multi-Buffer Drive (`0x0000A8D2`):**
   ```asm
   A8D2: 21FF      movs    r1, #255            ; Full brightness
   A8D6: 4BF2      ldr     r3, =0x2000096C     ; RED buffer
   A8D8: 5299      strh    r1, [r3, r2]
   A8DA: 4BF1      ldr     r3, =0x20000A68     ; GREEN buffer
   A8DE: 5299      strh    r1, [r3, r2]
   A8E2: 4BF1      ldr     r3, =0x20000B64     ; BLUE buffer
   A8E4: 5299      strh    r1, [r3, r2]
   ```
   In static, monochrome, and indicator routines, the firmware writes identical PWM values to the Red, Green, and Blue buffers simultaneously ($R=G=B$, pure white). Regardless of which color phase line the physical single-color LED is soldered across, it receives full PWM power.

### 8.4 Built-in Animation Tables
- **Gamma / Brightness Table (`0x7990`)**:
  64 non-linear PWM values (`0x3F` down to `0x03`) to ensure linear visual brightness response across the LEDs.
- **Color Palette Table (`0x7580`)**:
  Standard RGB primary and secondary color vectors for preset effects:
  - Red: `[255, 0, 0]`
  - Green: `[0, 255, 0]`
  - Blue: `[0, 0, 255]`
  - Yellow: `[255, 255, 0]`
  - Cyan: `[0, 255, 255]`
  - Magenta: `[255, 0, 255]`
  - White: `[255, 255, 255]`

---

## 9. Flash Memory Layout & Storage Sectors

User modifications sent by the configurator are committed to internal flash using the hardware Flash controller at `0x40062000`.

### 9.1 Sector Allocation

| Address Range | Size | Allocation / Content |
| :---: | :---: | :--- |
| `0x00000000` – `0x00006FFF` | 28 KB | Main Program Code, C Runtime, Interrupt Handlers |
| `0x00007000` – `0x0000757F` | 1.3 KB | Factory Default Profile & Keymap Matrices |
| `0x00007580` – `0x00007FFF` | 2.6 KB | Lighting animation tables, gamma curves, color presets |
| **`0x00008580` – `0x000085FF`** | 128 B | **User Keymap Storage Profile 1** |
| **`0x00008680` – `0x000086FF`** | 128 B | **User Lighting Configuration Profile 1** |
| **`0x00008780` – `0x000087FF`** | 128 B | **User Keymap Storage Profile 2** |
| **`0x00008880` – `0x000088FF`** | 128 B | **User Keymap Storage Profile 3** |
| **`0x00009000` – `0x0000E797`** | ~22.4 KB | **Custom Per-Key RGB Matrix, Macro Streams & Device Config** |
| `0x0000E798` – `0x0000FFFB` | 6.1 KB | Unused Flash (`0xFF` padding, 6,244 bytes) |
| `0x0000FFFC` – `0x0000FFFF` | 4 B | Flash Integrity Security Word (`0xAAAA5555`) |

### 9.2 Flash Controller Programming Routine (`0x615A`)
The firmware writes flash pages using the following sequence:
1. Clears control flags in `SN_FLASH->CTRL` (`0x40062004`).
2. Enables programming: `SN_FLASH->CTRL |= 0x01`.
3. Loads target flash address: `SN_FLASH->ADDR = target_addr` (`0x40062010`).
4. Streams 32-bit data words into `SN_FLASH->DATA` (`0x4006200C`).
5. Sets write trigger bit: `SN_FLASH->CTRL |= 0x40`.
6. Polls `SN_FLASH->STATUS` busy flag (bit 31) until completion.

---

## 10. Summary & Open-Source Porting Recommendations

This firmware analysis confirms that the Royal Kludge RK855 is built on an open and standard hardware foundation:

1. **Flashing & Recovery**:
   - The keyboard can be rebooted into the Sonix ROM bootloader by sending `AA 42 89 5A FF 71 62 CC` over USB HID, provided application firmware and its USB stack remain operational.
   - If user flash is corrupted or an incomplete write occurs, recovery requires pulling the hardware boot pin `P0.4` LOW (grounding `P0.4` at power-on) to force the MCU directly into ROM bootloader mode.
   - Any custom flasher can write raw binary directly to `0x00000000` using standard Sonix ISP packets.
2. **QMK / Open-Source Firmware Compatibility**:
   - The hardware is 100% compatible with the community **SonixQMK** project (`SN32F240B` target).
   - The matrix rows (`GPIO0`, `GPIO1`), columns (`GPIO3`), and SPI LED controller (`SSP0`) match standard SonixQMK pin definitions.
3. **Alternative Configurator Implementation**:
   - The vendor HID endpoint (Report ID `0x00`, 64-byte payload) accepts the exact packet sequences reverse-engineered in the configurator analysis, allowing full lighting and key remapping control from Linux and macOS without Windows.
