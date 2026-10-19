"""Royal Kludge RK68 Linux Driver & Configuration Suite."""

__version__ = "1.0.0"

from .constants import VENDOR_ID, PRODUCT_ID, LightingMode
from .device import RK68Device, MockRK68Device
from .layout import KEY_DEFINITIONS, find_key
from .protocol import (
    build_lighting_packet,
    build_key_remap_packet,
    build_custom_rgb_frame,
    build_reset_packet,
)
from .config import ConfigManager, KeyboardProfile

__all__ = [
    "VENDOR_ID",
    "PRODUCT_ID",
    "LightingMode",
    "RK68Device",
    "MockRK68Device",
    "KEY_DEFINITIONS",
    "find_key",
    "build_lighting_packet",
    "build_key_remap_packet",
    "build_custom_rgb_frame",
    "build_reset_packet",
    "ConfigManager",
    "KeyboardProfile",
]
