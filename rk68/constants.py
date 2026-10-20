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
    """Backlight animation modes supported by the RK68 Sonix MCU firmware.
    
    Firmware mode IDs correspond directly to the OEM string table (1033.lan IDs 200..219):
      0: Static, 1: SingleOn, 2: SingleOff, 3: Glittering, 4: Falling,
      5: Colourful, 6: Breath, 7: Spectrum, 8: Outward, 9: Scrolling,
      10: Rolling, 11: Rotating, 12: Explode, 13: Launch, 14: Ripples,
      15: Flowing, 16: Pulsating, 17: Tilt, 18: Shuttle, 19: Close Backlight
    """
    STATIC = 0           # 200: Solid constant illumination
    SINGLE_ON = 1        # 201: Key lights on press then fades (SingleOn / Reactive)
    SINGLE_OFF = 2       # 202: Key turns off on press then recovers
    GLITTERING = 3       # 203: Random twinkle
    FALLING = 4          # 204: Waterfall cascade
    COLOURFUL = 5        # 205: Full RGB cycle
    BREATH = 6           # 206: Breathing pulse
    SPECTRUM = 7         # 207: Spectrum shift
    OUTWARD = 8          # 208: Ripple radiating outward
    SCROLLING = 9        # 209: Horizontal wave scroll
    ROLLING = 10         # 210: Diagonal rotating wave
    ROTATING = 11        # 211: Pinwheel rotation
    EXPLODE = 12         # 212: Explosive burst
    LAUNCH = 13          # 213: Vertical launch
    RIPPLES = 14         # 214: Concentric water ripples
    FLOWING = 15         # 215: Stream flowing across rows
    PULSATING = 16       # 216: Rapid heartbeat pulse
    TILT = 17            # 217: Angular wave
    SHUTTLE = 18         # 218: Laser beam sweep
    OFF = 19             # 219: Close backlight (LEDs off)

    # Aliases for convenience & backwards compatibility
    BREATHING = 6
    WAVE = 9
    RIPPLE = 14
    REACTIVE = 1
    RAINBOW = 5
    NEON = 7
    CUSTOM_RGB = 20

    @classmethod
    def from_string(cls, name: str) -> "LightingMode":
        cleaned = name.strip().lower().replace("-", "_")
        alias_map = {
            "wave": cls.SCROLLING,
            "scrolling": cls.SCROLLING,
            "breathing": cls.BREATH,
            "breath": cls.BREATH,
            "static": cls.STATIC,
            "solid": cls.STATIC,
            "ripple": cls.RIPPLES,
            "ripples": cls.RIPPLES,
            "reactive": cls.SINGLE_ON,
            "singleon": cls.SINGLE_ON,
            "single_on": cls.SINGLE_ON,
            "singleoff": cls.SINGLE_OFF,
            "single_off": cls.SINGLE_OFF,
            "rainbow": cls.COLOURFUL,
            "colourful": cls.COLOURFUL,
            "colorful": cls.COLOURFUL,
            "spectrum": cls.SPECTRUM,
            "neon": cls.SPECTRUM,
            "glittering": cls.GLITTERING,
            "falling": cls.FALLING,
            "outward": cls.OUTWARD,
            "rolling": cls.ROLLING,
            "rotating": cls.ROTATING,
            "explode": cls.EXPLODE,
            "launch": cls.LAUNCH,
            "flowing": cls.FLOWING,
            "pulsating": cls.PULSATING,
            "tilt": cls.TILT,
            "shuttle": cls.SHUTTLE,
            "off": cls.OFF,
            "close": cls.OFF,
            "close_backlight": cls.OFF,
            "custom": cls.CUSTOM_RGB,
            "custom_rgb": cls.CUSTOM_RGB,
        }
        if cleaned in alias_map:
            return alias_map[cleaned]
        for mode in cls:
            if mode.name.lower() == cleaned:
                return mode
        raise ValueError(
            f"Unknown lighting mode: '{name}'. Available: {[m.name.lower() for m in cls]}"
        )


# Hardware LED specifications for single-color blue backlight RK68 / RK855
DEFAULT_BACKLIGHT_COLOR = "#00A8FF"
DEFAULT_BACKLIGHT_RGB = (0, 168, 255)

# Animation modes compatible with single-color monochrome blue LEDs
SINGLE_COLOR_LIGHTING_MODES = [
    LightingMode.STATIC,
    LightingMode.BREATH,
    LightingMode.SCROLLING,
    LightingMode.RIPPLES,
    LightingMode.SINGLE_ON,
    LightingMode.OFF,
]


class CommandOpcode(IntEnum):
    """Firmware packet command opcodes and sub-opcodes.

    The Sonix MCU architecture uses 0x04 as the universal primary command opcode
    for all configuration feature reports. Operations are distinguished by the
    sub-opcode in Byte 2 of the 65-byte report (Byte 1 of payload):
      - 0x18: Preamble / Transaction Begin
      - 0x11: Preset Lighting Mode Config
      - 0x13: Lighting Bank Matrix Header (19 modes)
      - 0x15: Custom Per-Key RGB Frame
      - 0x19: Custom RGB Preamble
      - 0x20: Key Remapping Block
      - 0x02: Commit / Apply to Onboard Flash
      - 0xF0: Session Flush / End
    """
    # Primary Opcode (Byte 1 of 65-byte report)
    PRIMARY = 0x04
    SET_LIGHT_CONFIG = 0x04  # Legacy alias

    # Sub-Opcodes (Byte 2 of 65-byte report)
    SUB_QUERY = 0x01
    SUB_COMMIT = 0x02
    SUB_PRESET_LIGHT = 0x11
    SUB_LIGHT_BANK = 0x13
    SUB_CUSTOM_RGB = 0x15
    SUB_PREAMBLE = 0x18
    SUB_RGB_PRE = 0x19
    SUB_KEY_REMAP = 0x20
    SUB_FLUSH = 0xF0

    # Sync magic word used in lighting bank and custom frames: 0x55AA
    SYNC_MAGIC = 0x55AA

    # Legacy aliases
    SET_CUSTOM_RGB = 0x15
    SET_KEY_REMAP = 0x20
    RESET_DEFAULT = 0x07
    QUERY_DEVICE = 0x01
