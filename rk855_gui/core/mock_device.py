"""Hardware simulator for Royal Kludge RK855 (for demo / offline testing)."""

import time
from typing import Any
from .constants import (
    KEYBOARD_VID,
    KEYBOARD_PID,
    BOOTLOADER_VID,
    BOOTLOADER_PID,
    ISP_PAGE_SIZE,
    FLASH_LIMIT,
)
from .device import DeviceMode
from .flasher import ProgressCallback, LogCallback, CancelCheck


class MockRK855Device:
    """Simulates an RK855 keyboard and its Sonix ISP bootloader."""

    def __init__(self):
        self.mode = DeviceMode.NORMAL
        self.is_open = False
        self.active_lighting_mode = 0
        self.brightness = 3
        self.speed = 2
        self.color = (255, 0, 255)
        self.direction = 0
        self.per_key_colors = [(255, 0, 255)] * 68
        self.remap_table: dict[int, int] = {}
        self.flash_memory = bytearray([0xFF] * FLASH_LIMIT)
        self.sent_packets: list[bytes] = []
        self._receiving_remap = False
        self._remap_buffer = bytearray()

    def open(self) -> None:
        self.is_open = True

    def close(self) -> None:
        self.is_open = False

    def send_feature(self, payload: bytes | list[int], delay: float = 0.001) -> None:
        data = bytes(payload)
        self.sent_packets.append(data)

        # Parse lighting and remap commands
        if len(data) >= 2 and data[0] == 0x04:
            subcmd = data[1]
            if subcmd == 0x11 and len(data) >= 10:
                self.active_lighting_mode = data[2]
                self.color = (data[4], data[5], data[6])
                self.brightness = data[7]
                self.speed = data[8]
                self.direction = data[9]
            elif subcmd == 0x20:
                self._receiving_remap = True
                self._remap_buffer = bytearray()
            elif subcmd == 0x02:
                if self._receiving_remap and self._remap_buffer:
                    # Parse remap entries
                    for i in range(0, min(len(self._remap_buffer), 256), 2):
                        p = self._remap_buffer[i]
                        t = self._remap_buffer[i + 1]
                        if p != 0:
                            self.remap_table[p] = t
                    self._receiving_remap = False
        elif self._receiving_remap:
            self._remap_buffer.extend(data[:64])

    def get_feature(self, report_id: int = 0x00, length: int = 65) -> bytes:
        # Default mock response with ISP ACK header
        resp = bytearray([0x00, 0x01, 0xAA, 0x55])
        resp.extend([0x00] * (length - len(resp)))
        return bytes(resp)


def simulate_flashing(
    fw_bytes: bytes,
    progress_cb: ProgressCallback | None = None,
    log_cb: LogCallback | None = None,
    cancel_check: CancelCheck | None = None,
    verify: bool = True,
    step_delay: float = 0.015,
) -> bool:
    """Simulates the entire ISP flashing and verification lifecycle."""
    def log(lvl: str, msg: str):
        if log_cb:
            log_cb(lvl, msg)

    def progress(pct: int, status: str):
        if progress_cb:
            progress_cb(pct, status)

    log("info", "[Simulator] Connecting to simulated Sonix ISP bootloader (0x0C45:0x82AA)...")
    time.sleep(0.1)

    log("info", "[Simulator] Sending ISP Handshake (0x01)...")
    time.sleep(0.05)
    log("success", "[Simulator] ISP Handshake acknowledged: 0001aa55")

    log("info", "[Simulator] Erasing microcontroller flash memory (Opcode 0x08)...")
    progress(5, "Erasing flash...")
    time.sleep(0.3)
    log("success", "[Simulator] Flash erase completed successfully.")

    total_pages = len(fw_bytes) // ISP_PAGE_SIZE
    pages_to_write = []
    for p in range(total_pages):
        addr = p * ISP_PAGE_SIZE
        chunk = fw_bytes[addr : addr + ISP_PAGE_SIZE]
        if not (all(b == 0xFF for b in chunk) and addr >= 0x8000):
            pages_to_write.append((p, addr, chunk))

    log("info", f"[Simulator] Writing {len(pages_to_write)} active pages ({len(pages_to_write) * ISP_PAGE_SIZE // 1024} KB)...")
    written = 0

    for p, addr, chunk in pages_to_write:
        if cancel_check and cancel_check():
            log("warning", "[Simulator] Flashing cancelled by user!")
            return False

        time.sleep(step_delay)
        written += 1
        pct = int(10 + (written / len(pages_to_write)) * 60)
        if written % 8 == 0 or written == len(pages_to_write):
            progress(pct, f"Writing flash: {addr // 1024} KB / {len(fw_bytes) // 1024} KB")

    log("success", f"[Simulator] Successfully written {written} pages.")

    if verify:
        log("info", "[Simulator] Verifying flash contents against original firmware...")
        verified = 0
        for p, addr, chunk in pages_to_write:
            if cancel_check and cancel_check():
                log("warning", "[Simulator] Verification cancelled by user!")
                return False

            time.sleep(step_delay * 0.6)
            verified += 1
            pct = int(70 + (verified / len(pages_to_write)) * 25)
            if verified % 8 == 0 or verified == len(pages_to_write):
                progress(pct, f"Verifying: {addr // 1024} KB / {len(fw_bytes) // 1024} KB")

        log("success", "[Simulator] Flash verification passed 100%.")

    progress(98, "Rebooting device...")
    log("info", "[Simulator] Rebooting MCU into user application firmware...")
    time.sleep(0.1)

    progress(100, "Done!")
    log("success", "[Simulator] Firmware update finished successfully.")
    return True
