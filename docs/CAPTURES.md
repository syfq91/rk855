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

## 4. Verified Captures Log

*As captures are collected, document their SHA-256 hashes, timestamps, and findings in this table.*

| Capture File | SHA-256 | Source OS | Findings / Verification Notes |
|---|---|---|---|
| *(Pending user captures)* | | | |
