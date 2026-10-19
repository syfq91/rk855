"""Constants and definitions for the Royal Kludge RK68 Linux driver."""

from enum import IntEnum

# Target hardware USB identifiers (Sonix controller)
VENDOR_ID = 0x0C45
PRODUCT_ID = 0x8018

# HID Report Configuration
REPORT_ID = 0x00
REPORT_PAYLOAD_SIZE = 64
REPORT_SIZE = 65  # 1 byte report ID + 64 bytes payload

# Canvas bounds from KeyboardLayout.xml
LAYOUT_CANVAS_WIDTH = 800
LAYOUT_CANVAS_HEIGHT = 274


class LightingMode(IntEnum):
    """Backlight animation modes supported by the RK68 firmware."""
    OFF = 0
    STATIC = 1
    BREATHING = 2
    WAVE = 3
    RIPPLE = 4
    RAINBOW = 5
    REACTIVE = 6
    NEON = 7
    CUSTOM_RGB = 8

    @classmethod
    def from_string(cls, name: str) -> "LightingMode":
        cleaned = name.strip().lower().replace("-", "_")
        for mode in cls:
            if mode.name.lower() == cleaned:
                return mode
        raise ValueError(f"Unknown lighting mode: '{name}'. Available: {[m.name.lower() for m in cls]}")


# Hardware LED specifications for single-color blue backlight RK68 / RK855
DEFAULT_BACKLIGHT_COLOR = "#00A8FF"
DEFAULT_BACKLIGHT_RGB = (0, 168, 255)

# Animation modes compatible with single-color monochrome blue LEDs
SINGLE_COLOR_LIGHTING_MODES = [
    LightingMode.STATIC,
    LightingMode.BREATHING,
    LightingMode.WAVE,
    LightingMode.RIPPLE,
    LightingMode.REACTIVE,
    LightingMode.OFF,
]


class CommandOpcode(IntEnum):
    """Firmware packet command opcodes."""
    SET_LIGHT_CONFIG = 0x04
    SET_CUSTOM_RGB = 0x05
    SET_KEY_REMAP = 0x06
    RESET_DEFAULT = 0x07
    QUERY_DEVICE = 0x01
