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


@dataclass(frozen=True)
class RemapTarget:
    """Represents an assignable target button action."""
    code: int
    name: str
    desc: str
    category: str


# Comprehensive table of all assignable target buttons
REMAP_TARGETS: List[RemapTarget] = [
    # Letters (A-Z)
    RemapTarget(0x04, "a", "A", "Letters"),
    RemapTarget(0x05, "b", "B", "Letters"),
    RemapTarget(0x06, "c", "C", "Letters"),
    RemapTarget(0x07, "d", "D", "Letters"),
    RemapTarget(0x08, "e", "E", "Letters"),
    RemapTarget(0x09, "f", "F", "Letters"),
    RemapTarget(0x0A, "g", "G", "Letters"),
    RemapTarget(0x0B, "h", "H", "Letters"),
    RemapTarget(0x0C, "i", "I", "Letters"),
    RemapTarget(0x0D, "j", "J", "Letters"),
    RemapTarget(0x0E, "k", "K", "Letters"),
    RemapTarget(0x0F, "l", "L", "Letters"),
    RemapTarget(0x10, "m", "M", "Letters"),
    RemapTarget(0x11, "n", "N", "Letters"),
    RemapTarget(0x12, "o", "O", "Letters"),
    RemapTarget(0x13, "p", "P", "Letters"),
    RemapTarget(0x14, "q", "Q", "Letters"),
    RemapTarget(0x15, "r", "R", "Letters"),
    RemapTarget(0x16, "s", "S", "Letters"),
    RemapTarget(0x17, "t", "T", "Letters"),
    RemapTarget(0x18, "u", "U", "Letters"),
    RemapTarget(0x19, "v", "V", "Letters"),
    RemapTarget(0x1A, "w", "W", "Letters"),
    RemapTarget(0x1B, "x", "X", "Letters"),
    RemapTarget(0x1C, "y", "Y", "Letters"),
    RemapTarget(0x1D, "z", "Z", "Letters"),

    # Numbers (0-9)
    RemapTarget(0x1E, "1", "1", "Numbers"),
    RemapTarget(0x1F, "2", "2", "Numbers"),
    RemapTarget(0x20, "3", "3", "Numbers"),
    RemapTarget(0x21, "4", "4", "Numbers"),
    RemapTarget(0x22, "5", "5", "Numbers"),
    RemapTarget(0x23, "6", "6", "Numbers"),
    RemapTarget(0x24, "7", "7", "Numbers"),
    RemapTarget(0x25, "8", "8", "Numbers"),
    RemapTarget(0x26, "9", "9", "Numbers"),
    RemapTarget(0x27, "0", "0", "Numbers"),

    # Function Keys (F1-F12)
    RemapTarget(0x3A, "f1", "F1", "Function Keys"),
    RemapTarget(0x3B, "f2", "F2", "Function Keys"),
    RemapTarget(0x3C, "f3", "F3", "Function Keys"),
    RemapTarget(0x3D, "f4", "F4", "Function Keys"),
    RemapTarget(0x3E, "f5", "F5", "Function Keys"),
    RemapTarget(0x3F, "f6", "F6", "Function Keys"),
    RemapTarget(0x40, "f7", "F7", "Function Keys"),
    RemapTarget(0x41, "f8", "F8", "Function Keys"),
    RemapTarget(0x42, "f9", "F9", "Function Keys"),
    RemapTarget(0x43, "f10", "F10", "Function Keys"),
    RemapTarget(0x44, "f11", "F11", "Function Keys"),
    RemapTarget(0x45, "f12", "F12", "Function Keys"),

    # Modifiers & System
    RemapTarget(0xE0, "control", "Left Control (LCtrl)", "Modifiers"),
    RemapTarget(0xE4, "control_r", "Right Control (RCtrl)", "Modifiers"),
    RemapTarget(0xE1, "shift", "Left Shift (LShift)", "Modifiers"),
    RemapTarget(0xE5, "shift_r", "Right Shift (RShift)", "Modifiers"),
    RemapTarget(0xE2, "alt", "Left Alt (LAlt)", "Modifiers"),
    RemapTarget(0xE6, "alt_r", "Right Alt (RAlt)", "Modifiers"),
    RemapTarget(0xE3, "win", "Windows / Super (Win)", "Modifiers"),
    RemapTarget(0x39, "capslock", "Caps Lock", "Modifiers"),

    # Navigation & Editing
    RemapTarget(0x29, "esc", "Escape (ESC)", "Navigation"),
    RemapTarget(0x2B, "tab", "Tab", "Navigation"),
    RemapTarget(0x28, "enter", "Enter", "Navigation"),
    RemapTarget(0x2C, "space", "Spacebar", "Navigation"),
    RemapTarget(0x2A, "backspace", "Backspace", "Navigation"),
    RemapTarget(0x4C, "del", "Delete (Del)", "Navigation"),
    RemapTarget(0x49, "insert", "Insert (Ins)", "Navigation"),
    RemapTarget(0x4A, "home", "Home", "Navigation"),
    RemapTarget(0x4D, "end", "End", "Navigation"),
    RemapTarget(0x4B, "page_up", "Page Up (PgUp)", "Navigation"),
    RemapTarget(0x4E, "page_down", "Page Down (PgDn)", "Navigation"),
    RemapTarget(0x52, "up", "Arrow Up (↑)", "Navigation"),
    RemapTarget(0x51, "down", "Arrow Down (↓)", "Navigation"),
    RemapTarget(0x50, "left", "Arrow Left (←)", "Navigation"),
    RemapTarget(0x4F, "right", "Arrow Right (→)", "Navigation"),

    # Punctuation & Symbols
    RemapTarget(0x2D, "-_", "- _ (Minus)", "Symbols"),
    RemapTarget(0x2E, "=+", "= + (Equal)", "Symbols"),
    RemapTarget(0x2F, "[", "[ { (Left Bracket)", "Symbols"),
    RemapTarget(0x30, "]", "] } (Right Bracket)", "Symbols"),
    RemapTarget(0x31, "\\|", "\\ | (Backslash)", "Symbols"),
    RemapTarget(0x33, ":;", ": ; (Semicolon)", "Symbols"),
    RemapTarget(0x34, "'", "' \" (Quote)", "Symbols"),
    RemapTarget(0x36, "<,", "< , (Comma)", "Symbols"),
    RemapTarget(0x37, ">.", "> . (Period)", "Symbols"),
    RemapTarget(0x38, "/?", "/ ? (Slash)", "Symbols"),
    RemapTarget(0x35, "·~", "` ~ (Grave / Tilde)", "Symbols"),

    # Media & Audio
    RemapTarget(0xE2, "mute", "Mute Audio", "Media"),
    RemapTarget(0xE9, "volume_up", "Volume Up", "Media"),
    RemapTarget(0xEA, "volume_down", "Volume Down", "Media"),
    RemapTarget(0xCD, "play_pause", "Play / Pause", "Media"),
    RemapTarget(0xB5, "next_track", "Next Track (⏭)", "Media"),
    RemapTarget(0xB6, "prev_track", "Previous Track (⏮)", "Media"),
    RemapTarget(0xB7, "stop", "Stop Media (⏹)", "Media"),

    # System & Control
    RemapTarget(0x46, "print_screen", "Print Screen (PrtSc)", "System"),
    RemapTarget(0x47, "scroll_lock", "Scroll Lock", "System"),
    RemapTarget(0x48, "pause", "Pause / Break", "System"),
]

TARGET_NAME_TO_TARGET: Dict[str, RemapTarget] = {
    t.name.lower(): t for t in REMAP_TARGETS
}
TARGET_CODE_TO_TARGET: Dict[int, RemapTarget] = {
    t.code: t for t in REMAP_TARGETS
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

    # Try by hex or decimal code
    try:
        val = int(query, 16) if query.startswith("0x") else int(query)
        if val in CODE_TO_KEY:
            return CODE_TO_KEY[val]
        if val in TARGET_CODE_TO_TARGET:
            t = TARGET_CODE_TO_TARGET[val]
            return KeyDefinition(t.code, t.name, t.desc, 0, 0, 0, 0)
    except ValueError:
        pass

    # Check remap targets by name or desc
    if query in TARGET_NAME_TO_TARGET:
        t = TARGET_NAME_TO_TARGET[query]
        return KeyDefinition(t.code, t.name, t.desc, 0, 0, 0, 0)

    for t in REMAP_TARGETS:
        if t.desc.lower() == query:
            return KeyDefinition(t.code, t.name, t.desc, 0, 0, 0, 0)

    return None
