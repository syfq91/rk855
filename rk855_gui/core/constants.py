"""Constants and specifications for Royal Kludge RK855 (RK68)."""

from dataclasses import dataclass
from typing import NamedTuple

# USB Device Identifiers
KEYBOARD_VID = 0x0C45
KEYBOARD_PID = 0x8018
BOOTLOADER_VID = 0x0C45
BOOTLOADER_PID = 0x82AA

# ISP Bootloader Magic Command Packet
BOOT_TRIGGER_CMD = bytes([0x00, 0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC])

# ISP Architecture Limits
ISP_PAGE_SIZE = 64
FLASH_LIMIT = 0x10000  # 64 KB total flash on SONiX SN32F240B

# HID Feature Report Constants
REPORT_ID = 0x00
FEATURE_REPORT_SIZE = 65

# Configuration Subcommands
CMD_BEGIN_LIGHT = 0x18
CMD_SET_LIGHT = 0x11
CMD_CUSTOM_RGB_CHUNK = 0x16
CMD_COMMIT_FLASH = 0x02
CMD_SYNC_LIVE = 0xF0

# ISP Bootloader Opcodes
ISP_OP_HANDSHAKE = 0x01
ISP_OP_READ_PAGE = 0x06
ISP_OP_WRITE_PAGE = 0x07
ISP_OP_ERASE = 0x08
ISP_OP_REBOOT = 0x29


@dataclass(frozen=True)
class LightingMode:
    id: int
    name: str
    description: str
    has_color: bool = True
    has_speed: bool = True
    has_direction: bool = False


LIGHTING_MODES: dict[int, LightingMode] = {
    0: LightingMode(0, "Static", "Continuous solid illumination across all keys", has_speed=False),
    1: LightingMode(1, "Single On", "Key lights up on keypress then fades out"),
    2: LightingMode(2, "Single Off", "All keys stay lit; pressed key turns off temporarily"),
    3: LightingMode(3, "Glittering", "Keys twinkle with random starburst intensity"),
    4: LightingMode(4, "Falling Stars", "Beams of light descend like falling rain"),
    5: LightingMode(5, "Color Cycling", "Smooth full-spectrum rainbow color cycling", has_color=False),
    6: LightingMode(6, "Breathing", "Pulsing breath rhythm fade in and out"),
    7: LightingMode(7, "Spectrum Wave", "Smooth continuous rainbow spectrum wave", has_color=False),
    8: LightingMode(8, "Outward Ripple", "Shockwave expands outwards from center"),
    9: LightingMode(9, "Scrolling Bands", "Vertical bands scroll across keyboard", has_direction=True),
    10: LightingMode(10, "Wave / Rolling", "Horizontal wave travels across keys", has_direction=True),
    11: LightingMode(11, "Accumulate", "Falling drops fill up keyboard row by row"),
    12: LightingMode(12, "Windmill / Rotating", "Light rotates like windmill blades", has_direction=True),
    13: LightingMode(13, "Colorful Waterfall", "Cascade of vibrant colors flowing down"),
    14: LightingMode(14, "Blossoming / Ripples", "Concentric waves ripple outward from pressed key"),
    15: LightingMode(15, "Flowing Stream", "Fluid ribbons flowing across the keyboard", has_direction=True),
    16: LightingMode(16, "Constant Rainbow", "Static multi-color rainbow gradient", has_speed=False, has_color=False),
    17: LightingMode(17, "Sine Wave", "Sinusoidal luminous wave oscillations", has_direction=True),
    18: LightingMode(18, "Shadow Follow", "Trailing shadow follows behind keystrokes"),
    19: LightingMode(19, "Backlight Off", "Turns keyboard illumination completely off", has_color=False, has_speed=False),
}


class KeyDefinition(NamedTuple):
    index: int
    scancode: int
    name: str
    label: str
    left: int
    top: int
    width: int
    height: int
    row: int


# Extracted from vendor KeyboardLayout.xml (Layout855 - 68 physical keys)
PHYSICAL_KEYS: list[KeyDefinition] = [
    # Row 0 (Top / Numbers row)
    KeyDefinition(0, 0x29, "esc", "ESC", 6, 10, 48, 48, 0),
    KeyDefinition(1, 0x1E, "1", "1 !", 55, 10, 48, 48, 0),
    KeyDefinition(2, 0x1F, "2", "2 @", 104, 10, 48, 48, 0),
    KeyDefinition(3, 0x20, "3", "3 #", 153, 10, 48, 48, 0),
    KeyDefinition(4, 0x21, "4", "4 $", 203, 10, 48, 48, 0),
    KeyDefinition(5, 0x22, "5", "5 %", 252, 10, 48, 48, 0),
    KeyDefinition(6, 0x23, "6", "6 ^", 302, 10, 48, 48, 0),
    KeyDefinition(7, 0x24, "7", "7 &", 351, 10, 48, 48, 0),
    KeyDefinition(8, 0x25, "8", "8 *", 401, 10, 48, 48, 0),
    KeyDefinition(9, 0x26, "9", "9 (", 450, 10, 48, 48, 0),
    KeyDefinition(10, 0x27, "0", "0 )", 500, 10, 48, 48, 0),
    KeyDefinition(11, 0x2D, "-_", "- _", 550, 10, 48, 48, 0),
    KeyDefinition(12, 0x2E, "=+", "= +", 600, 10, 48, 48, 0),
    KeyDefinition(13, 0x2A, "backspace", "Bksp", 650, 10, 97, 48, 0),
    KeyDefinition(14, 0x35, "`~", "` ~", 748, 10, 48, 48, 0),

    # Row 1 (QWERTY row)
    KeyDefinition(15, 0x2B, "tab", "Tab", 6, 60, 72, 48, 1),
    KeyDefinition(16, 0x14, "Q", "Q", 80, 60, 48, 48, 1),
    KeyDefinition(17, 0x1A, "W", "W", 130, 61, 48, 48, 1),
    KeyDefinition(18, 0x08, "E", "E", 179, 61, 48, 48, 1),
    KeyDefinition(19, 0x15, "R", "R", 228, 61, 48, 48, 1),
    KeyDefinition(20, 0x17, "T", "T", 277, 61, 48, 48, 1),
    KeyDefinition(21, 0x1C, "Y", "Y", 326, 61, 48, 48, 1),
    KeyDefinition(22, 0x18, "U", "U", 376, 61, 48, 48, 1),
    KeyDefinition(23, 0x0C, "I", "I", 426, 61, 48, 48, 1),
    KeyDefinition(24, 0x12, "O", "O", 476, 61, 48, 48, 1),
    KeyDefinition(25, 0x13, "P", "P", 525, 61, 48, 48, 1),
    KeyDefinition(26, 0x2F, "[{", "[ {", 575, 61, 48, 48, 1),
    KeyDefinition(27, 0x30, "]}", "] }", 624, 61, 48, 48, 1),
    KeyDefinition(28, 0x31, "\\|", "\\ |", 674, 61, 72, 48, 1),
    KeyDefinition(29, 0x4C, "del", "Del", 748, 61, 48, 48, 1),

    # Row 2 (Home row)
    KeyDefinition(30, 0x39, "capsLock", "Caps", 6, 113, 85, 48, 2),
    KeyDefinition(31, 0x04, "A", "A", 93, 113, 48, 48, 2),
    KeyDefinition(32, 0x16, "S", "S", 143, 113, 48, 48, 2),
    KeyDefinition(33, 0x07, "D", "D", 192, 113, 48, 48, 2),
    KeyDefinition(34, 0x09, "F", "F", 242, 113, 48, 48, 2),
    KeyDefinition(35, 0x0A, "G", "G", 291, 113, 48, 48, 2),
    KeyDefinition(36, 0x0B, "H", "H", 341, 113, 48, 48, 2),
    KeyDefinition(37, 0x0D, "J", "J", 390, 113, 48, 48, 2),
    KeyDefinition(38, 0x0E, "K", "K", 440, 113, 48, 48, 2),
    KeyDefinition(39, 0x0F, "L", "L", 489, 113, 48, 48, 2),
    KeyDefinition(40, 0x33, ";:", "; :", 538, 113, 48, 48, 2),
    KeyDefinition(41, 0x34, "'\"", "' \"", 587, 113, 48, 48, 2),
    KeyDefinition(42, 0x28, "enter", "Enter", 637, 113, 109, 48, 2),
    KeyDefinition(43, 0x4B, "page up", "PgUp", 748, 113, 48, 48, 2),

    # Row 3 (Bottom row)
    KeyDefinition(44, 0xE1, "shift", "Shift", 6, 165, 110, 48, 3),
    KeyDefinition(45, 0x1D, "Z", "Z", 118, 165, 48, 48, 3),
    KeyDefinition(46, 0x1B, "X", "X", 167, 165, 48, 48, 3),
    KeyDefinition(47, 0x06, "C", "C", 216, 165, 48, 48, 3),
    KeyDefinition(48, 0x19, "V", "V", 266, 165, 48, 48, 3),
    KeyDefinition(49, 0x05, "B", "B", 316, 165, 48, 48, 3),
    KeyDefinition(50, 0x11, "N", "N", 365, 165, 48, 48, 3),
    KeyDefinition(51, 0x10, "M", "M", 415, 165, 48, 48, 3),
    KeyDefinition(52, 0x36, "<,", "< ,", 464, 165, 48, 48, 3),
    KeyDefinition(53, 0x37, ">.", "> .", 513, 165, 48, 48, 3),
    KeyDefinition(54, 0x38, "/?", "/ ?", 563, 165, 48, 48, 3),
    KeyDefinition(55, 0xE5, "shift", "Shift", 613, 165, 86, 48, 3),
    KeyDefinition(56, 0x52, "top", "▲", 699, 165, 48, 48, 3),
    KeyDefinition(57, 0x4E, "page down", "PgDn", 748, 165, 48, 48, 3),

    # Row 4 (Modifiers row)
    KeyDefinition(58, 0xE0, "control", "Ctrl", 6, 217, 60, 48, 4),
    KeyDefinition(59, 0xE3, "win", "Win", 68, 217, 60, 48, 4),
    KeyDefinition(60, 0xE2, "alt", "Alt", 130, 217, 60, 48, 4),
    KeyDefinition(61, 0x2C, "space", "Space", 192, 217, 306, 48, 4),
    KeyDefinition(62, 0xE6, "alt", "Alt", 500, 217, 48, 48, 4),
    KeyDefinition(63, 0xAF, "fn", "Fn", 550, 217, 48, 48, 4),
    KeyDefinition(64, 0xE4, "control", "Ctrl", 599, 217, 48, 48, 4),
    KeyDefinition(65, 0x50, "left", "◀", 649, 217, 48, 48, 4),
    KeyDefinition(66, 0x51, "bottom", "▼", 699, 217, 48, 48, 4),
    KeyDefinition(67, 0x4F, "right", "▶", 748, 217, 48, 48, 4),
]

# Quick selection presets for keyboard lighting customization
KEY_PRESETS: dict[str, list[int]] = {
    "All Keys": list(range(68)),
    "WASD": [17, 31, 32, 33],  # W, A, S, D
    "Arrow Keys": [56, 65, 66, 67],  # Up, Left, Down, Right
    "Number Row": list(range(1, 14)),  # 1 through =+
    "Modifiers": [15, 30, 44, 55, 58, 59, 60, 62, 63, 64],  # Tab, Caps, Shifts, Ctrls, Wins, Alts, Fn
    "Navigation": [14, 29, 43, 57],  # ~`, Del, PgUp, PgDn
}

# Standard Preset Color Swatches (R, G, B)
PRESET_COLORS: list[tuple[str, tuple[int, int, int]]] = [
    ("Red", (255, 30, 30)),
    ("Orange", (255, 120, 0)),
    ("Yellow", (255, 230, 0)),
    ("Green", (0, 255, 60)),
    ("Cyan", (0, 240, 255)),
    ("Blue", (30, 80, 255)),
    ("Purple", (170, 0, 255)),
    ("Pink", (255, 50, 180)),
    ("White", (255, 255, 255)),
    ("Off", (0, 0, 0)),
]
