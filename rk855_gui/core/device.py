"""Cross-platform USB HID interface for Royal Kludge RK855."""

from enum import Enum, auto
import os
import sys
import time
from typing import Any

try:
    import hid
except ImportError:
    hid = None

from .constants import (
    KEYBOARD_VID,
    KEYBOARD_PID,
    BOOTLOADER_VID,
    BOOTLOADER_PID,
    FEATURE_REPORT_SIZE,
)


class DeviceMode(Enum):
    DISCONNECTED = auto()
    NORMAL = auto()
    BOOTLOADER = auto()
    PERMISSION_DENIED = auto()


class RK855Device:
    """Manages the raw USB HID communication with the RK855 keyboard or bootloader."""

    def __init__(self, vid: int = KEYBOARD_VID, pid: int = KEYBOARD_PID):
        self.vid = vid
        self.pid = pid
        self._dev = None

    @classmethod
    def get_status(cls) -> tuple[DeviceMode, dict[str, Any] | None, str | None]:
        """Detects current device mode, device info dict, and any error message."""
        if hid is None:
            return DeviceMode.DISCONNECTED, None, "hidapi library not available"

        try:
            devices = hid.enumerate()
        except Exception as e:
            return DeviceMode.DISCONNECTED, None, f"HID enumeration error: {e}"

        normal_info = None
        bootloader_info = None

        for d in devices:
            v = d.get("vendor_id")
            p = d.get("product_id")
            if v == KEYBOARD_VID and p == KEYBOARD_PID:
                normal_info = d
            elif v == BOOTLOADER_VID and p == BOOTLOADER_PID:
                bootloader_info = d

        if bootloader_info is not None:
            # Test opening bootloader interface
            try:
                test_dev = hid.device()
                test_dev.open(BOOTLOADER_VID, BOOTLOADER_PID)
                test_dev.close()
                return DeviceMode.BOOTLOADER, bootloader_info, None
            except Exception as e:
                return DeviceMode.PERMISSION_DENIED, bootloader_info, str(e)

        if normal_info is not None:
            # Test opening normal interface
            try:
                test_dev = hid.device()
                test_dev.open(KEYBOARD_VID, KEYBOARD_PID)
                test_dev.close()
                return DeviceMode.NORMAL, normal_info, None
            except Exception as e:
                return DeviceMode.PERMISSION_DENIED, normal_info, str(e)

        return DeviceMode.DISCONNECTED, None, None

    def open(self) -> None:
        """Opens connection to the specified HID device."""
        if hid is None:
            raise RuntimeError("hidapi module is required. Run: pip install hidapi")
        if self._dev is not None:
            return

        dev = hid.device()
        try:
            dev.open(self.vid, self.pid)
            self._dev = dev
        except Exception as e:
            raise PermissionError(f"Unable to open USB device (VID: 0x{self.vid:04X}, PID: 0x{self.pid:04X}): {e}")

    def close(self) -> None:
        """Closes the active connection if open."""
        if self._dev is not None:
            try:
                self._dev.close()
            except Exception:
                pass
            finally:
                self._dev = None

    def __enter__(self):
        self.open()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def send_feature(self, payload: bytes | list[int], delay: float = 0.005) -> None:
        """Sends a padded 65-byte feature report (Report ID 0x00)."""
        if self._dev is None:
            raise RuntimeError("Device not opened")
        buf = bytes([0x00]) + bytes(payload).ljust(FEATURE_REPORT_SIZE - 1, b"\x00")
        self._dev.send_feature_report(buf)
        if delay > 0:
            time.sleep(delay)

    def get_feature(self, report_id: int = 0x00, length: int = FEATURE_REPORT_SIZE) -> bytes:
        """Reads back a feature report from the device."""
        if self._dev is None:
            raise RuntimeError("Device not opened")
        resp = self._dev.get_feature_report(report_id, length)
        return bytes(resp)


def get_udev_rule_text() -> str:
    """Returns the recommended udev rule content for Linux systems."""
    return (
        '# Royal Kludge RK855 (RK68) - Normal Mode and Sonix ISP Bootloader\n'
        'SUBSYSTEMS=="usb", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="8018", MODE="0666", TAG+="uaccess"\n'
        'SUBSYSTEMS=="usb", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="82aa", MODE="0666", TAG+="uaccess"\n'
        'KERNEL=="hidraw*", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="8018", MODE="0666", TAG+="uaccess"\n'
        'KERNEL=="hidraw*", ATTRS{idVendor}=="0c45", ATTRS{idProduct}=="82aa", MODE="0666", TAG+="uaccess"\n'
    )
