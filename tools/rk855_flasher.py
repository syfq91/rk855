#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "hidapi>=0.14.0",
#     "intelhex>=2.3.0",
# ]
# ///
"""
Royal Kludge RK855 (Sonix SN32F240B) In-System Programming (ISP) Flasher Tool
Reverse-engineered from the official Windows firmware updater utility.

Protocols:
- Trigger Bootloader: HID Feature Report to VID 0x0C45, PID 0x8018
  Magic payload: AA 42 89 5A FF 71 62 CC
- Sonix ISP Bootloader: HID Feature Reports to VID 0x0C45, PID 0x82AA
"""

import argparse
import os
import sys
import time

try:
    import hid
except ImportError:
    hid = None

KEYBOARD_VID = 0x0C45
KEYBOARD_PID = 0x8018

BOOTLOADER_VID = 0x0C45
BOOTLOADER_PID = 0x82AA

# Bootloader unlock magic frame configured in UISettings.ini
BOOT_TRIGGER_CMD = bytes([0x00, 0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC])

PAGE_SIZE = 64
FLASH_LIMIT = 0x10000  # 64 KB total flash on SN32F240B


def require_hid():
    if hid is None:
        print("Error: 'hidapi' module not found. Install it with: pip install hidapi", file=sys.stderr)
        sys.exit(1)


def trigger_bootloader() -> bool:
    """Sends the magic bootloader trigger command to the running keyboard."""
    require_hid()
    print(f"[+] Connecting to keyboard (VID: 0x{KEYBOARD_VID:04X}, PID: 0x{KEYBOARD_PID:04X})...")
    dev = hid.device()
    try:
        dev.open(KEYBOARD_VID, KEYBOARD_PID)
    except Exception as e:
        print(f"[-] Could not open keyboard in normal mode: {e}")
        return False

    try:
        print("[+] Sending ISP bootloader trigger command...")
        dev.send_feature_report(BOOT_TRIGGER_CMD)
        print("[+] Trigger packet delivered. Waiting for device reboot and re-enumeration...")
        dev.close()
        return True
    except Exception as e:
        print(f"[-] Failed to send bootloader trigger: {e}")
        dev.close()
        return False


def wait_for_bootloader(timeout_sec: float = 6.0) -> hid.device | None:
    """Waits for Sonix ISP bootloader to re-enumerate on the USB bus."""
    start_time = time.time()
    while time.time() - start_time < timeout_sec:
        dev = hid.device()
        try:
            dev.open(BOOTLOADER_VID, BOOTLOADER_PID)
            print(f"[+] Connected to Sonix ISP Bootloader (VID: 0x{BOOTLOADER_VID:04X}, PID: 0x{BOOTLOADER_PID:04X})")
            return dev
        except Exception:
            time.sleep(0.3)
    return None


def send_isp_cmd(dev: hid.device, opcode: int, addr: int = 0, data: bytes = b"") -> None:
    """Constructs and sends a 65-byte Sonix ISP command packet."""
    addr_bytes = [
        addr & 0xFF,
        (addr >> 8) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 24) & 0xFF,
    ]
    pkt = [0x00, opcode, 0xAA, 0x55] + addr_bytes + list(data)
    padded = bytes(pkt).ljust(65, b"\x00")
    dev.send_feature_report(padded)
    time.sleep(0.002)


def get_isp_response(dev: hid.device, length: int = 65) -> bytes:
    """Reads back an ISP response feature report."""
    return bytes(dev.get_feature_report(0x00, length))


def perform_handshake(dev: hid.device) -> bool:
    """Initiates the ISP communication session with opcode 0x01."""
    print("[+] Sending ISP Handshake (0x01)...")
    send_isp_cmd(dev, 0x01)
    resp = get_isp_response(dev, 9)
    if len(resp) >= 4 and resp[1] == 0x01 and resp[2] == 0xAA and resp[3] == 0x55:
        print(f"[+] Handshake acknowledged: {resp[:8].hex()}")
        return True
    print(f"[-] Unexpected handshake response: {resp.hex()}")
    return False


def erase_flash(dev: hid.device) -> None:
    """Erases the user application flash memory (opcode 0x08)."""
    print("[+] Erasing microcontroller flash memory...")
    send_isp_cmd(dev, 0x08)
    time.sleep(0.6)  # Allow hardware erase cycle to complete
    resp = get_isp_response(dev, 9)
    print(f"[+] Erase completed. Status: {resp[:4].hex()}")


def reboot_to_user_code(dev: hid.device) -> None:
    """Issues reset command (opcode 0x29) to execute firmware from flash."""
    print("[+] Rebooting microcontroller into user firmware (0x29)...")
    send_isp_cmd(dev, 0x29)
    time.sleep(0.1)


def load_firmware(file_path: str) -> bytes:
    """Loads a firmware binary or Intel HEX file into a 64KB byte stream."""
    if not os.path.isfile(file_path):
        raise FileNotFoundError(f"File not found: {file_path}")

    if file_path.lower().endswith(".hex"):
        try:
            from intelhex import IntelHex

            ih = IntelHex(file_path)
            return bytes(ih.tobinarray(start=0, size=FLASH_LIMIT))
        except ImportError:
            print("[-] intelhex module not found. Run 'pip install intelhex' or provide a .bin file.", file=sys.stderr)
            sys.exit(1)
    else:
        with open(file_path, "rb") as f:
            data = f.read()
        if len(data) > FLASH_LIMIT:
            raise ValueError(f"Firmware image exceeds 64 KB MCU limit ({len(data)} bytes).")
        return data.ljust(FLASH_LIMIT, b"\xFF")


def flash_device(firmware_path: str, verify: bool = True) -> bool:
    """Flashes firmware into the SN32F240B MCU via Sonix ISP protocol."""
    fw_bytes = load_firmware(firmware_path)
    print(f"[+] Loaded firmware image: {firmware_path} ({len(fw_bytes)} bytes)")

    # 1. Check if bootloader is already open
    dev = None
    try:
        dev = hid.device()
        dev.open(BOOTLOADER_VID, BOOTLOADER_PID)
        print("[+] Device already in Sonix ISP Bootloader mode.")
    except Exception:
        dev = None

    # 2. If not, trigger it from running keyboard mode
    if dev is None:
        if not trigger_bootloader():
            print("[-] Could not trigger bootloader mode.")
            return False
        dev = wait_for_bootloader(timeout_sec=5.0)
        if dev is None:
            print("[-] Device failed to re-enumerate in bootloader mode.")
            return False

    try:
        if not perform_handshake(dev):
            return False

        erase_flash(dev)

        # Write pages (64-byte chunks)
        # Only flash non-0xFF pages
        total_pages = len(fw_bytes) // PAGE_SIZE
        written_pages = 0
        print(f"[+] Writing firmware pages to flash...")

        for page_idx in range(total_pages):
            addr = page_idx * PAGE_SIZE
            chunk = fw_bytes[addr : addr + PAGE_SIZE]

            # Skip writing trailing empty pages
            if all(b == 0xFF for b in chunk) and addr >= 0x8000:
                continue

            # Opcode 0x07: Write Page
            send_isp_cmd(dev, 0x07, addr=addr, data=chunk)
            written_pages += 1

            if page_idx % 16 == 0:
                sys.stdout.write(f"\r[+] Flashed {addr // 1024} KB / {len(fw_bytes) // 1024} KB...")
                sys.stdout.flush()

        print(f"\r[+] Flashed {written_pages} pages successfully.                   ")

        if verify:
            print("[+] Verifying flash contents...")
            for page_idx in range(total_pages):
                addr = page_idx * PAGE_SIZE
                chunk = fw_bytes[addr : addr + PAGE_SIZE]
                if all(b == 0xFF for b in chunk) and addr >= 0x8000:
                    continue

                # Opcode 0x06: Read Page
                send_isp_cmd(dev, 0x06, addr=addr)
                read_back = get_isp_response(dev, 65)[8 : 8 + PAGE_SIZE]
                if read_back != chunk:
                    print(f"\n[-] Verification mismatch at offset 0x{addr:04X}!")
                    return False
            print("[+] Flash verification passed 100%.")

        reboot_to_user_code(dev)
        print("[+] Firmware update finished successfully.")
        return True
    finally:
        dev.close()


def main():
    parser = argparse.ArgumentParser(
        description="Royal Kludge RK855 (Sonix SN32F240B) In-System Programming (ISP) Flasher",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    subparsers.add_parser("info", help="Query connected keyboard or bootloader state")
    subparsers.add_parser("bootloader", help="Switch connected RK855 keyboard into ISP bootloader mode")
    subparsers.add_parser("reboot", help="Reboot MCU from bootloader back to keyboard user mode")

    p_flash = subparsers.add_parser("flash", help="Flash firmware hex or bin into MCU")
    p_flash.add_argument("file", help="Path to SN32F240B firmware (.hex or .bin)")
    p_flash.add_argument("--no-verify", action="store_true", help="Skip reading back flash for verification")

    args = parser.parse_args()

    if not args.command or args.command == "info":
        require_hid()
        devices = hid.enumerate()
        found_kb = any(d["vendor_id"] == KEYBOARD_VID and d["product_id"] == KEYBOARD_PID for d in devices)
        found_bl = any(d["vendor_id"] == BOOTLOADER_VID and d["product_id"] == BOOTLOADER_PID for d in devices)

        if found_kb:
            print(f"[+] Found RK855 Keyboard in Normal Mode (VID: 0x{KEYBOARD_VID:04X}, PID: 0x{KEYBOARD_PID:04X})")
        if found_bl:
            print(f"[+] Found RK855 Keyboard in ISP Bootloader Mode (VID: 0x{BOOTLOADER_VID:04X}, PID: 0x{BOOTLOADER_PID:04X})")
        if not found_kb and not found_bl:
            print("[-] No RK855 keyboard or Sonix bootloader found on USB bus.")
        return

    if args.command == "bootloader":
        if trigger_bootloader():
            print("[+] Bootloader trigger command sent successfully.")
        return

    if args.command == "reboot":
        dev = hid.device()
        try:
            dev.open(BOOTLOADER_VID, BOOTLOADER_PID)
            reboot_to_user_code(dev)
            dev.close()
        except Exception as e:
            print(f"[-] Could not connect to bootloader: {e}")
        return

    if args.command == "flash":
        flash_device(args.file, verify=not args.no_verify)
        return


if __name__ == "__main__":
    main()
