"""Sonix SN32F240B In-System Programming (ISP) flashing engine."""

from collections.abc import Callable
import os
import time
from typing import Any

from .constants import (
    KEYBOARD_VID,
    KEYBOARD_PID,
    BOOTLOADER_VID,
    BOOTLOADER_PID,
    BOOT_TRIGGER_CMD,
    ISP_PAGE_SIZE,
    FLASH_LIMIT,
    ISP_OP_HANDSHAKE,
    ISP_OP_READ_PAGE,
    ISP_OP_WRITE_PAGE,
    ISP_OP_ERASE,
    ISP_OP_REBOOT,
)
from .device import RK855Device

ProgressCallback = Callable[[int, str], None]
LogCallback = Callable[[str, str], None]
CancelCheck = Callable[[], bool]


def load_firmware(file_path: str) -> bytes:
    """Loads a firmware binary or Intel HEX file into a 64KB padded byte stream."""
    if not os.path.isfile(file_path):
        raise FileNotFoundError(f"Firmware file not found: {file_path}")

    ext = os.path.splitext(file_path)[1].lower()
    if ext == ".hex":
        try:
            from intelhex import IntelHex

            ih = IntelHex(file_path)
            data = bytes(ih.tobinarray(start=0, size=FLASH_LIMIT))
            return data
        except ImportError:
            raise RuntimeError("intelhex library is required to load .hex files. Install with: pip install intelhex")
    else:
        with open(file_path, "rb") as f:
            data = f.read()
        if len(data) > FLASH_LIMIT:
            raise ValueError(f"Firmware image exceeds 64 KB MCU limit ({len(data)} bytes).")
        return data.ljust(FLASH_LIMIT, b"\xFF")


def send_isp_packet(device: RK855Device, opcode: int, addr: int = 0, data: bytes = b"") -> None:
    """Constructs and sends a 65-byte Sonix ISP command packet."""
    addr_bytes = [
        addr & 0xFF,
        (addr >> 8) & 0xFF,
        (addr >> 16) & 0xFF,
        (addr >> 24) & 0xFF,
    ]
    pkt = [opcode, 0xAA, 0x55] + addr_bytes + list(data)
    device.send_feature(pkt, delay=0.002)


def get_isp_response(device: RK855Device, length: int = 65) -> bytes:
    """Reads back an ISP response report."""
    return device.get_feature(0x00, length)


def trigger_bootloader_mode(log_cb: LogCallback | None = None) -> bool:
    """Sends the unlock frame to switch from normal keyboard mode to ISP bootloader."""
    def log(lvl: str, msg: str):
        if log_cb:
            log_cb(lvl, msg)

    log("info", f"Connecting to keyboard (VID: 0x{KEYBOARD_VID:04X}, PID: 0x{KEYBOARD_PID:04X})...")
    dev = RK855Device(KEYBOARD_VID, KEYBOARD_PID)
    try:
        dev.open()
    except Exception as e:
        log("error", f"Could not open keyboard in normal mode: {e}")
        return False

    try:
        log("info", "Sending ISP bootloader trigger command...")
        dev.send_feature(BOOT_TRIGGER_CMD[1:], delay=0.05)
        log("success", "Trigger packet delivered. Keyboard will reboot into bootloader.")
        return True
    except Exception as e:
        log("error", f"Failed to send bootloader trigger: {e}")
        return False
    finally:
        dev.close()


def reboot_to_normal_mode(log_cb: LogCallback | None = None) -> bool:
    """Reboots the MCU from ISP bootloader back into user firmware."""
    def log(lvl: str, msg: str):
        if log_cb:
            log_cb(lvl, msg)

    log("info", "Connecting to Sonix ISP bootloader (0x0C45:0x82AA)...")
    dev = RK855Device(BOOTLOADER_VID, BOOTLOADER_PID)
    try:
        dev.open()
        log("info", "Sending reset/reboot opcode 0x29...")
        send_isp_packet(dev, ISP_OP_REBOOT)
        time.sleep(0.1)
        log("success", "Device reboot command sent.")
        return True
    except Exception as e:
        log("error", f"Could not reboot device: {e}")
        return False
    finally:
        dev.close()


def flash_firmware_bytes(
    fw_bytes: bytes,
    progress_cb: ProgressCallback | None = None,
    log_cb: LogCallback | None = None,
    cancel_check: CancelCheck | None = None,
    verify: bool = True,
) -> bool:
    """Flashes firmware bytes to the microcontroller via Sonix ISP protocol."""
    def log(lvl: str, msg: str):
        if log_cb:
            log_cb(lvl, msg)

    def progress(pct: int, status: str):
        if progress_cb:
            progress_cb(pct, status)

    if len(fw_bytes) != FLASH_LIMIT:
        fw_bytes = fw_bytes.ljust(FLASH_LIMIT, b"\xFF")

    # Connect to bootloader
    dev = RK855Device(BOOTLOADER_VID, BOOTLOADER_PID)
    try:
        dev.open()
    except Exception as e:
        log("error", f"Cannot open Sonix ISP Bootloader: {e}")
        log("warning", "Ensure keyboard is in bootloader mode (VID: 0x0C45, PID: 0x82AA).")
        return False

    try:
        # Handshake
        log("info", "Sending ISP Handshake (Opcode 0x01)...")
        send_isp_packet(dev, ISP_OP_HANDSHAKE)
        resp = get_isp_response(dev, 9)
        if len(resp) >= 4 and resp[1] == 0x01 and resp[2] == 0xAA and resp[3] == 0x55:
            log("success", f"ISP Handshake acknowledged: {resp[:8].hex()}")
        else:
            log("error", f"Unexpected handshake response: {resp.hex()}")
            return False

        # Erase
        log("info", "Erasing user application flash memory (Opcode 0x08)...")
        progress(5, "Erasing flash...")
        send_isp_packet(dev, ISP_OP_ERASE)
        time.sleep(0.6)
        erase_resp = get_isp_response(dev, 9)
        log("success", f"Flash erase complete: {erase_resp[:4].hex()}")

        total_pages = len(fw_bytes) // ISP_PAGE_SIZE
        pages_to_write = []
        for p in range(total_pages):
            addr = p * ISP_PAGE_SIZE
            chunk = fw_bytes[addr : addr + ISP_PAGE_SIZE]
            if not (all(b == 0xFF for b in chunk) and addr >= 0x8000):
                pages_to_write.append((p, addr, chunk))

        log("info", f"Writing {len(pages_to_write)} active pages ({len(pages_to_write) * ISP_PAGE_SIZE // 1024} KB)...")
        written = 0

        # Flash Write Loop
        for p, addr, chunk in pages_to_write:
            if cancel_check and cancel_check():
                log("warning", "Flashing cancelled by user!")
                return False

            send_isp_packet(dev, ISP_OP_WRITE_PAGE, addr=addr, data=chunk)
            written += 1

            pct = int(10 + (written / len(pages_to_write)) * 60)
            if written % 16 == 0 or written == len(pages_to_write):
                progress(pct, f"Writing flash: {addr // 1024} KB / {len(fw_bytes) // 1024} KB")

        log("success", f"Successfully programmed {written} pages.")

        # Verification Loop
        if verify:
            log("info", "Verifying flash contents against original firmware...")
            verified = 0
            for p, addr, chunk in pages_to_write:
                if cancel_check and cancel_check():
                    log("warning", "Verification cancelled by user!")
                    return False

                send_isp_packet(dev, ISP_OP_READ_PAGE, addr=addr)
                read_back = get_isp_response(dev, 65)[8 : 8 + ISP_PAGE_SIZE]
                if read_back != chunk:
                    log("error", f"Verification mismatch at offset 0x{addr:04X}!")
                    return False

                verified += 1
                pct = int(70 + (verified / len(pages_to_write)) * 25)
                if verified % 16 == 0 or verified == len(pages_to_write):
                    progress(pct, f"Verifying: {addr // 1024} KB / {len(fw_bytes) // 1024} KB")

            log("success", "Flash verification passed 100%.")

        progress(98, "Rebooting device...")
        log("info", "Rebooting MCU into user application firmware...")
        send_isp_packet(dev, ISP_OP_REBOOT)
        time.sleep(0.1)

        progress(100, "Done!")
        log("success", "Firmware update finished successfully.")
        return True

    except Exception as e:
        log("error", f"Flashing encountered error: {e}")
        return False
    finally:
        dev.close()
