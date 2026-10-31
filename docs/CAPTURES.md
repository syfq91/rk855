# USB Traffic Capture Guide & Log

This guide details how to capture USB HID packets from the official Royal Kludge configurator software and firmware flasher to provide empirical golden vectors for the Rust implementation.

---

## 1. Setup Instructions

### On Windows (Recommended)
1. Install **Wireshark** with the **USBPcap** component enabled:
   ```powershell
   winget install WiresharkFoundation.Wireshark
   ```
   *(Ensure the USBPcap checkbox is ticked during installation).*
2. Open **Wireshark** as Administrator.
3. In the interface list, select the **USBPcap** interface corresponding to the USB Root Hub where your RK855 is connected (e.g., `USBPcap1` or `USBPcap2`).
4. Set the capture filter:
   ```
   usb.device_address == <device_id>
   ```
   *(Or filter post-capture in the display filter: `usb.bInterfaceClass == 0x03` or `usb.idVendor == 0x0c45 || usb.idVendor == 0x05ac`).*

### On Linux
1. Load the `usbmon` kernel module:
   ```bash
   sudo modprobe usbmon
   ```
2. Locate the USB bus of the keyboard (`lsusb`):
   ```bash
   lsusb | grep -E "0c45|05ac"
   ```
3. Capture traffic on that bus using Wireshark or `tcpdump`:
   ```bash
   sudo tcpdump -i usbmon1 -w captures/lighting_breath.pcap
   ```

---

## 2. Capture Scenarios Matrix

Save all raw capture files in the `captures/` folder following the naming convention below:

| ID | Scenario | File Name | Purpose |
|:---|:---|:---|:---|
| **CAP-01** | Breathing Mode | `captures/light_breathing.pcapng` | Verify `04 18` -> `04 11` -> `04 02` -> `04 F0` sequence and speed/brightness ranges. |
| **CAP-02** | Static Red | `captures/light_static_red.pcapng` | Verify RGB byte order (`R`, `G`, `B`) and ColorType parameter. |
| **CAP-03** | Speed & Direction | `captures/light_speed_dir.pcapng` | Verify direction byte (`0x00` vs `0x01`) and speed clamping. |
| **CAP-04** | Backlight Off | `captures/light_off.pcapng` | Verify mode 19 (`0x13`). |
| **CAP-05** | Per-Key Custom RGB | `captures/custom_rgb_single.pcapng` | Illuminate a single key (e.g., ESC in pure red) to determine matrix buffer offset. |
| **CAP-06** | Remap Single Key | `captures/remap_caps_to_esc.pcapng` | Verify 256-byte table encoding and check if `04 F0` is omitted. |
| **CAP-07** | Macro Recording | `captures/macro_simple.pcapng` | Record a simple key press + delay to determine byte layout of `04 15` blocks. |
| **CAP-08** | Flasher Handshake | `captures/flasher_info.pcapng` | Capture device rebooting into bootloader (`0C45:82AA`) and opcode `0x2A` handshake. |

---

## 3. Decoding Captures

Use the automated Python decoder to convert raw PCAP/PCAPNG packets into human-readable annotated text:

```bash
# Decode an individual file:
python tools/pcap_decode.py captures/light_breathing.pcapng

# Or decode all captures into captures/decoded/:
python tools/pcap_decode.py captures/
```

The output will be placed in `captures/decoded/<filename>.txt`.

---

## 4. Verified Captures Log & Technical Analysis

*(Traces provided from official RK Windows software reverse engineering session for [rnayabed/rangoli#13](https://github.com/rnayabed/rangoli/issues/13))*

| Capture File | SHA-256 | Source OS / Device | Findings & Protocol Verification |
|:---|:---|:---|:---|
| `captures/test_1.pcap` | `98fb297e...` | Windows / `0C45:8018` | **Explode Mode & Brightness:** Captures transition from Static to Explode Mode (`0x10`). Verifies parameter byte: brightness set to 15 (`0x0F`), then changed to 7 (`0x07`). Verifies 22-packet transaction sequence. |
| `captures/test_2.pcap` | `8ef80112...` | Windows / `0C45:8018` | **Glittering & Ripples Modes:** Captures transition to Glittering (`0x06`) and Ripples (`0x0F`). Confirms active mode block offset (Block 18) and key matrix buffers. |
| `captures/test_3.pcap` | `43ddc3e1...` | Windows / `0C45:8018` | **Software Exit Baseline:** Confirms no dangling transactions on driver teardown. |
| `captures/test_4.pcap` | `c0d83e21...` | Windows / `0C45:8018` | **Software Launch & Exit Baseline:** Confirms clean initialization sequence. |
| `captures/test_5.pcap` | `e2a4073b...` | Windows / `0C45:8018` | **Software Idle Baseline:** Verifies periodic `04 F5` read-back polling loop at ~8 Hz (every 125ms). |
| `captures/test_6.pcap` | `fa125197...` | Windows / `0C45:8018` | **Driver Re-check Baseline:** Confirms zero rogue background traffic when software is closed. |

### Empirical Transaction Structure (22 Packets)

Every configuration commit in the vendor software follows this exact atomic 22-packet transaction:
1. `04 18 [0x00 * 62]`: Begin Lighting Transaction (ACKed via `GET_REPORT` with `04 18 00 01`)
2. `04 13 00 00 00 00 00 00 12 [0x00 * 55]`: Header declaring 18 data blocks (`0x12`)
3. **Blocks 1–5 (Packets 3–7):** Preset lighting parameter tables (`01 ff e1...`, `05 ff e1...`, `09 ff e1...`, `0d ff e1...`, `11 ff e1...`), each terminated with magic `AA 55`
4. **Blocks 6–8 (Packets 8–10):** 192 bytes of zero padding (`00 00...`)
5. **Blocks 9–17 (Packets 11–19):** 576 bytes key RGB buffer (`80 00 00 00...`, where `0x80` denotes default/unassigned key status)
6. **Block 18 (Packet 20):** Active mode descriptor:
   - Byte 0: Active Light Mode ID (`0x10` Explode, `0x06` Glittering, `0x0F` Ripples)
   - Byte 1–2: Flags (`0xFF`, `0xE1`)
   - Byte 8: Color type (`0x01` custom RGB)
   - Byte 9: Brightness (`0x0F` = 15, `0x07` = 7)
   - Byte 10: Speed (`0x0A` = 10)
   - Byte 14–15: Magic trailer `AA 55`
7. `04 02 [0x00 * 62]`: Commit to Flash (ACKed with destination sector `0x85D8` / `0x85C8`)
8. `04 F0 [0x00 * 62]`: PWM Reload (triggers immediate visual update on CT16B1 timer PWM)

### Empirical Read-back & Status Polling

Between user operations, the vendor driver runs a continuous read-back loop:
- `SET_REPORT: 04 F5 01 00...` $\rightarrow$ `GET_REPORT: 00 00 01 00...` (matrix rows 0–1)
- `SET_REPORT: 04 F5 00 00...` $\rightarrow$ `GET_REPORT: 20 00 21 00...` (matrix rows 2–3)
- `SET_REPORT: 04 F5 00 00...` $\rightarrow$ `GET_REPORT: 40 00 41 00...` (matrix rows 4–5)
- `SET_REPORT: 04 F5 00 00...` $\rightarrow$ `GET_REPORT: 60 00 61 00...` (matrix rows 6–7)
