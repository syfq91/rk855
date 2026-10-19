"""Layout definitions for the 68-key RK68 / RK855 keyboard matrix."""

from dataclasses import dataclass
from typing import Dict, List, Optional
from .constants import LAYOUT_CANVAS_WIDTH, LAYOUT_CANVAS_HEIGHT


@dataclass(frozen=True)
class KeyDefinition:
    code: int
    name: str
    desc: str
    rect_left: int
    rect_top: int
    rect_width: int
    rect_height: int

    @property
    def x_percent(self) -> float:
        return self.rect_left / LAYOUT_CANVAS_WIDTH

    @property
    def y_percent(self) -> float:
        return self.rect_top / LAYOUT_CANVAS_HEIGHT

    @property
    def width_percent(self) -> float:
        return self.rect_width / LAYOUT_CANVAS_WIDTH

    @property
    def height_percent(self) -> float:
        return self.rect_height / LAYOUT_CANVAS_HEIGHT


# Complete 68-key matrix parsed from original KeyboardLayout.xml
KEY_DEFINITIONS: List[KeyDefinition] = [
    # Row 1
    KeyDefinition(0x29, "esc", "ESC", 6, 10, 48, 48),
    KeyDefinition(0x1E, "1", "1", 55, 10, 48, 48),
    KeyDefinition(0x1F, "2", "2", 104, 10, 48, 48),
    KeyDefinition(0x20, "3", "3", 153, 10, 48, 48),
    KeyDefinition(0x21, "4", "4", 203, 10, 48, 48),
    KeyDefinition(0x22, "5", "5", 252, 10, 48, 48),
    KeyDefinition(0x23, "6", "6", 302, 10, 48, 48),
    KeyDefinition(0x24, "7", "7", 351, 10, 48, 48),
    KeyDefinition(0x25, "8", "8", 401, 10, 48, 48),
    KeyDefinition(0x26, "9", "9", 450, 10, 48, 48),
    KeyDefinition(0x27, "0", "0", 500, 10, 48, 48),
    KeyDefinition(0x2D, "-_", "-_", 550, 10, 48, 48),
    KeyDefinition(0x2E, "=+", "=+", 600, 10, 48, 48),
    KeyDefinition(0x2A, "backspace", "Backspace", 650, 10, 97, 48),
    KeyDefinition(0x35, "·~", "`~", 748, 10, 48, 48),

    # Row 2
    KeyDefinition(0x2B, "tab", "Tab", 6, 60, 72, 48),
    KeyDefinition(0x14, "Q", "Q", 80, 60, 48, 48),
    KeyDefinition(0x1A, "W", "W", 130, 61, 48, 48),
    KeyDefinition(0x08, "E", "E", 179, 61, 48, 48),
    KeyDefinition(0x15, "R", "R", 228, 61, 48, 48),
    KeyDefinition(0x17, "T", "T", 277, 61, 48, 48),
    KeyDefinition(0x1C, "Y", "Y", 326, 61, 48, 48),
    KeyDefinition(0x18, "U", "U", 376, 61, 48, 48),
    KeyDefinition(0x0C, "I", "I", 426, 61, 48, 48),
    KeyDefinition(0x12, "O", "O", 476, 61, 48, 48),
    KeyDefinition(0x13, "P", "P", 525, 61, 48, 48),
    KeyDefinition(0x2F, "[", "[{", 575, 61, 48, 48),
    KeyDefinition(0x30, "]", "]}", 624, 61, 48, 48),
    KeyDefinition(0x31, "\\|", "\\|", 674, 61, 72, 48),
    KeyDefinition(0x4C, "del", "Del", 748, 61, 48, 48),

    # Row 3
    KeyDefinition(0x39, "capsLock", "Caps", 6, 113, 85, 48),
    KeyDefinition(0x04, "A", "A", 93, 113, 48, 48),
    KeyDefinition(0x16, "S", "S", 143, 113, 48, 48),
    KeyDefinition(0x07, "D", "D", 192, 113, 48, 48),
    KeyDefinition(0x09, "F", "F", 242, 113, 48, 48),
    KeyDefinition(0x0A, "G", "G", 291, 113, 48, 48),
    KeyDefinition(0x0B, "H", "H", 341, 113, 48, 48),
    KeyDefinition(0x0D, "J", "J", 390, 113, 48, 48),
    KeyDefinition(0x0E, "K", "K", 440, 113, 48, 48),
    KeyDefinition(0x0F, "L", "L", 489, 113, 48, 48),
    KeyDefinition(0x33, ":;", ":;", 538, 113, 48, 48),
    KeyDefinition(0x34, "'", "'\"", 587, 113, 48, 48),
    KeyDefinition(0x28, "enter", "Enter", 637, 113, 109, 48),
    KeyDefinition(0x4B, "page up", "PgUp", 748, 113, 48, 48),

    # Row 4
    KeyDefinition(0xE1, "shift", "LShift", 6, 165, 110, 48),
    KeyDefinition(0x1D, "Z", "Z", 118, 165, 48, 48),
    KeyDefinition(0x1B, "X", "X", 167, 165, 48, 48),
    KeyDefinition(0x06, "C", "C", 216, 165, 48, 48),
    KeyDefinition(0x19, "V", "V", 266, 165, 48, 48),
    KeyDefinition(0x05, "B", "B", 316, 165, 48, 48),
    KeyDefinition(0x11, "N", "N", 365, 165, 48, 48),
    KeyDefinition(0x10, "M", "M", 415, 165, 48, 48),
    KeyDefinition(0x36, "<,", "<,", 464, 165, 48, 48),
    KeyDefinition(0x37, ">.", ">.", 513, 165, 48, 48),
    KeyDefinition(0x38, "/?", "/?", 563, 165, 48, 48),
    KeyDefinition(0xE5, "shift_r", "RShift", 613, 165, 86, 48),
    KeyDefinition(0x52, "top", "Up", 699, 165, 48, 48),
    KeyDefinition(0x4E, "page down", "PgDn", 748, 165, 48, 48),

    # Row 5
    KeyDefinition(0xE0, "control", "LCtrl", 6, 217, 60, 48),
    KeyDefinition(0xE3, "win", "Win", 68, 217, 60, 48),
    KeyDefinition(0xE2, "alt", "LAlt", 130, 217, 60, 48),
    KeyDefinition(0x2C, "space", "Space", 192, 217, 306, 48),
    KeyDefinition(0xE6, "alt_r", "RAlt", 500, 217, 48, 48),
    KeyDefinition(0xAF, "fn", "Fn", 550, 217, 48, 48),
    KeyDefinition(0xE4, "control_r", "RCtrl", 599, 217, 48, 48),
    KeyDefinition(0x50, "left", "Left", 649, 217, 48, 48),
    KeyDefinition(0x51, "bottom", "Down", 699, 217, 48, 48),
    KeyDefinition(0x4F, "right", "Right", 748, 217, 48, 48),
]

# Quick lookups
NAME_TO_KEY: Dict[str, KeyDefinition] = {
    k.name.lower(): k for k in KEY_DEFINITIONS
}
CODE_TO_KEY: Dict[int, KeyDefinition] = {
    k.code: k for k in KEY_DEFINITIONS
}


def find_key(query: str) -> Optional[KeyDefinition]:
    """Find a key definition by name, label, or scancode hex string."""
    query = query.strip().lower()
    if query in NAME_TO_KEY:
        return NAME_TO_KEY[query]
    
    # Try by desc / label
    for k in KEY_DEFINITIONS:
        if k.desc.lower() == query:
            return k
            
    # Try by hex code
    try:
        val = int(query, 16)
        if val in CODE_TO_KEY:
            return CODE_TO_KEY[val]
    except ValueError:
        pass

    return None
