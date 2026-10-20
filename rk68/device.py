"""Hardware device communication layer for Royal Kludge RK68 via HIDAPI."""

import logging
import time
from typing import Dict, List, Optional, Tuple, Union
from .constants import (
    VENDOR_ID,
    PRODUCT_ID,
    LightingMode,
    DEFAULT_BACKLIGHT_RGB,
)
from .protocol import (
    build_lighting_packet,
    build_full_lighting_transaction,
    build_key_remap_packet,
    build_key_remap_transaction,
    build_custom_rgb_frame,
    build_reset_packet,
    build_reset_transaction,
)

logger = logging.getLogger("rk68.device")

try:
    import hid  # provided by hidapi / cython-hidapi
    HID_AVAILABLE = True
except ImportError:
    hid = None
    HID_AVAILABLE = False


class RK68Device:
    """Manages the connection and USB HID communication with the RK68 keyboard."""

    def __init__(self, vendor_id: int = VENDOR_ID, product_id: int = PRODUCT_ID):
        self.vendor_id = vendor_id
        self.product_id = product_id
        self._device: Optional[object] = None
        self._is_open = False
        self._active_remaps: Dict[int, int] = {}

    @classmethod
    def enumerate_devices(cls) -> List[Dict]:
        """List all detected RK68 keyboards connected via USB."""
        if not HID_AVAILABLE:
            return []
        devices = []
        try:
            for dev in hid.enumerate(VENDOR_ID, PRODUCT_ID):
                devices.append({
                    "path": dev.get("path"),
                    "vendor_id": dev.get("vendor_id"),
                    "product_id": dev.get("product_id"),
                    "serial_number": dev.get("serial_number"),
                    "manufacturer_string": dev.get("manufacturer_string"),
                    "product_string": dev.get("product_string"),
                    "interface_number": dev.get("interface_number"),
                })
        except Exception as e:
            logger.warning("Error enumerating HID devices: %s", e)
        return devices

    @property
    def is_connected(self) -> bool:
        """Check if the device is currently opened."""
        return self._is_open and self._device is not None

    def open(self, path: Optional[bytes] = None) -> bool:
        """Open a handle to the keyboard's HID feature report endpoint."""
        if not HID_AVAILABLE:
            raise RuntimeError(
                "hidapi library is not installed. Install with: pip install hidapi"
            )

        if self.is_connected:
            return True

        self._device = hid.device()
        try:
            if path:
                self._device.open_path(path)
            else:
                self._device.open(self.vendor_id, self.product_id)
            self._is_open = True
            logger.info("Successfully connected to RK68 (0x%04X:0x%04X)", self.vendor_id, self.product_id)
            return True
        except Exception as e:
            self._device = None
            self._is_open = False
            logger.error("Failed to open RK68 device: %s", e)
            raise ConnectionError(
                f"Could not open RK68 device (0x{self.vendor_id:04X}:0x{self.product_id:04X}): {e}\n"
                "Ensure keyboard is connected via USB cable and udev rules are installed."
            ) from e

    def close(self):
        """Close the device handle."""
        if self._device is not None:
            try:
                self._device.close()
            except Exception:
                pass
            self._device = None
        self._is_open = False

    def send_feature_report(self, report_bytes: bytes) -> int:
        """Send a single 65-byte feature report to the keyboard."""
        if not self.is_connected:
            raise ConnectionError("Device is not connected. Call open() first.")
        
        result = self._device.send_feature_report(report_bytes)
        if result < 0:
            raise IOError("Failed to send feature report to RK68 keyboard")
        return result

    def send_reports(self, reports: List[bytes], delay_ms: float = 2.0) -> int:
        """Send a sequence of feature reports (transaction) with optional inter-packet delay."""
        total = 0
        for report in reports:
            total += self.send_feature_report(report)
            if delay_ms > 0:
                time.sleep(delay_ms / 1000.0)
        return total

    def set_lighting(
        self,
        mode: Union[LightingMode, int],
        speed: int = 3,
        brightness: int = 3,
        color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
        direction: int = 0,
    ) -> bool:
        """Update keyboard backlight mode and settings using the full verified transaction.

        Transmits:
          1. Preamble (0x04 0x18)
          2. Lighting bank descriptor header (0x04 0x13)
          3. Preset lighting packet (0x04 0x11)
          4. 19 lighting bank chunk packets (1216 bytes with 0x55AA sync)
          5. Commit (0x04 0x02)
          6. Flush (0x04 0xF0)
        """
        packets = build_full_lighting_transaction(
            mode=mode,
            speed=speed,
            brightness=brightness,
            color_rgb=color_rgb,
            direction=direction,
        )
        self.send_reports(packets)
        return True

    def remap_key(self, key_code: int, target_scancode: int) -> bool:
        """Remap a physical key scancode and commit to keyboard onboard memory."""
        self._active_remaps[key_code] = target_scancode
        packets = build_key_remap_transaction(self._active_remaps)
        self.send_reports(packets)
        return True

    def remap_keys(self, remaps: Dict[int, int]) -> bool:
        """Batch remap multiple physical keys in a single atomic hardware transaction."""
        self._active_remaps.update(remaps)
        packets = build_key_remap_transaction(self._active_remaps)
        self.send_reports(packets)
        return True

    def reset_defaults(self) -> bool:
        """Reset keyboard onboard configuration to factory defaults."""
        self._active_remaps.clear()
        packets = build_reset_transaction()
        self.send_reports(packets)
        return True

    def __enter__(self):
        self.open()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()


class MockRK68Device(RK68Device):
    """Virtual mock device for testing and headless verification without physical USB hardware."""

    def __init__(self):
        super().__init__()
        self.sent_reports: List[bytes] = []
        self._mock_open = False

    @property
    def is_connected(self) -> bool:
        return self._mock_open

    def open(self, path=None) -> bool:
        self._mock_open = True
        return True

    def close(self):
        self._mock_open = False

    def send_feature_report(self, report_bytes: bytes) -> int:
        if not self._mock_open:
            raise ConnectionError("Mock device is not connected.")
        if len(report_bytes) != 65:
            raise ValueError(f"Feature report must be exactly 65 bytes, got {len(report_bytes)}")
        self.sent_reports.append(report_bytes)
        return len(report_bytes)

    def send_reports(self, reports: List[bytes], delay_ms: float = 0.0) -> int:
        total = 0
        for report in reports:
            total += self.send_feature_report(report)
        return total
