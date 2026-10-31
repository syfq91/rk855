//! Physical 68-key matrix definitions and scancode lookup table for the RK855 (RK68).

/// Definition of a single physical key on the keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicalKey {
    /// Physical 0-based layout index (0..67)
    pub index: usize,
    /// Standard USB HID scan code (or internal modifier code)
    pub scancode: u8,
    /// Internal identifier name (e.g., "esc", "q", "space")
    pub name: &'static str,
    /// Human-friendly display label (e.g., "ESC", "Q", "Space")
    pub label: &'static str,
    /// Visual keyboard row (0 = Number row, 4 = Spacebar row)
    pub row: u8,
}

/// The 68 physical keys populated on the Royal Kludge RK855 (RK68) layout.
pub const PHYSICAL_KEYS: [PhysicalKey; 68] = [
    // Row 0 (Top / Number row)
    PhysicalKey {
        index: 0,
        scancode: 0x29,
        name: "esc",
        label: "ESC",
        row: 0,
    },
    PhysicalKey {
        index: 1,
        scancode: 0x1E,
        name: "1",
        label: "1 !",
        row: 0,
    },
    PhysicalKey {
        index: 2,
        scancode: 0x1F,
        name: "2",
        label: "2 @",
        row: 0,
    },
    PhysicalKey {
        index: 3,
        scancode: 0x20,
        name: "3",
        label: "3 #",
        row: 0,
    },
    PhysicalKey {
        index: 4,
        scancode: 0x21,
        name: "4",
        label: "4 $",
        row: 0,
    },
    PhysicalKey {
        index: 5,
        scancode: 0x22,
        name: "5",
        label: "5 %",
        row: 0,
    },
    PhysicalKey {
        index: 6,
        scancode: 0x23,
        name: "6",
        label: "6 ^",
        row: 0,
    },
    PhysicalKey {
        index: 7,
        scancode: 0x24,
        name: "7",
        label: "7 &",
        row: 0,
    },
    PhysicalKey {
        index: 8,
        scancode: 0x25,
        name: "8",
        label: "8 *",
        row: 0,
    },
    PhysicalKey {
        index: 9,
        scancode: 0x26,
        name: "9",
        label: "9 (",
        row: 0,
    },
    PhysicalKey {
        index: 10,
        scancode: 0x27,
        name: "0",
        label: "0 )",
        row: 0,
    },
    PhysicalKey {
        index: 11,
        scancode: 0x2D,
        name: "-_",
        label: "- _",
        row: 0,
    },
    PhysicalKey {
        index: 12,
        scancode: 0x2E,
        name: "=+",
        label: "= +",
        row: 0,
    },
    PhysicalKey {
        index: 13,
        scancode: 0x2A,
        name: "backspace",
        label: "Bksp",
        row: 0,
    },
    PhysicalKey {
        index: 14,
        scancode: 0x35,
        name: "`~",
        label: "` ~",
        row: 0,
    },
    // Row 1 (QWERTY row)
    PhysicalKey {
        index: 15,
        scancode: 0x2B,
        name: "tab",
        label: "Tab",
        row: 1,
    },
    PhysicalKey {
        index: 16,
        scancode: 0x14,
        name: "q",
        label: "Q",
        row: 1,
    },
    PhysicalKey {
        index: 17,
        scancode: 0x1A,
        name: "w",
        label: "W",
        row: 1,
    },
    PhysicalKey {
        index: 18,
        scancode: 0x08,
        name: "e",
        label: "E",
        row: 1,
    },
    PhysicalKey {
        index: 19,
        scancode: 0x15,
        name: "r",
        label: "R",
        row: 1,
    },
    PhysicalKey {
        index: 20,
        scancode: 0x17,
        name: "t",
        label: "T",
        row: 1,
    },
    PhysicalKey {
        index: 21,
        scancode: 0x1C,
        name: "y",
        label: "Y",
        row: 1,
    },
    PhysicalKey {
        index: 22,
        scancode: 0x18,
        name: "u",
        label: "U",
        row: 1,
    },
    PhysicalKey {
        index: 23,
        scancode: 0x0C,
        name: "i",
        label: "I",
        row: 1,
    },
    PhysicalKey {
        index: 24,
        scancode: 0x12,
        name: "o",
        label: "O",
        row: 1,
    },
    PhysicalKey {
        index: 25,
        scancode: 0x13,
        name: "p",
        label: "P",
        row: 1,
    },
    PhysicalKey {
        index: 26,
        scancode: 0x2F,
        name: "[{",
        label: "[ {",
        row: 1,
    },
    PhysicalKey {
        index: 27,
        scancode: 0x30,
        name: "]}",
        label: "] }",
        row: 1,
    },
    PhysicalKey {
        index: 28,
        scancode: 0x31,
        name: "\\|",
        label: "\\ |",
        row: 1,
    },
    PhysicalKey {
        index: 29,
        scancode: 0x4C,
        name: "del",
        label: "Del",
        row: 1,
    },
    // Row 2 (Home row)
    PhysicalKey {
        index: 30,
        scancode: 0x39,
        name: "caps",
        label: "Caps",
        row: 2,
    },
    PhysicalKey {
        index: 31,
        scancode: 0x04,
        name: "a",
        label: "A",
        row: 2,
    },
    PhysicalKey {
        index: 32,
        scancode: 0x16,
        name: "s",
        label: "S",
        row: 2,
    },
    PhysicalKey {
        index: 33,
        scancode: 0x07,
        name: "d",
        label: "D",
        row: 2,
    },
    PhysicalKey {
        index: 34,
        scancode: 0x09,
        name: "f",
        label: "F",
        row: 2,
    },
    PhysicalKey {
        index: 35,
        scancode: 0x0A,
        name: "g",
        label: "G",
        row: 2,
    },
    PhysicalKey {
        index: 36,
        scancode: 0x0B,
        name: "h",
        label: "H",
        row: 2,
    },
    PhysicalKey {
        index: 37,
        scancode: 0x0D,
        name: "j",
        label: "J",
        row: 2,
    },
    PhysicalKey {
        index: 38,
        scancode: 0x0E,
        name: "k",
        label: "K",
        row: 2,
    },
    PhysicalKey {
        index: 39,
        scancode: 0x0F,
        name: "l",
        label: "L",
        row: 2,
    },
    PhysicalKey {
        index: 40,
        scancode: 0x33,
        name: ";:",
        label: "; :",
        row: 2,
    },
    PhysicalKey {
        index: 41,
        scancode: 0x34,
        name: "'\"",
        label: "' \"",
        row: 2,
    },
    PhysicalKey {
        index: 42,
        scancode: 0x28,
        name: "enter",
        label: "Enter",
        row: 2,
    },
    PhysicalKey {
        index: 43,
        scancode: 0x4B,
        name: "pgup",
        label: "PgUp",
        row: 2,
    },
    // Row 3 (Bottom row)
    PhysicalKey {
        index: 44,
        scancode: 0xE1,
        name: "lshift",
        label: "Shift",
        row: 3,
    },
    PhysicalKey {
        index: 45,
        scancode: 0x1D,
        name: "z",
        label: "Z",
        row: 3,
    },
    PhysicalKey {
        index: 46,
        scancode: 0x1B,
        name: "x",
        label: "X",
        row: 3,
    },
    PhysicalKey {
        index: 47,
        scancode: 0x06,
        name: "c",
        label: "C",
        row: 3,
    },
    PhysicalKey {
        index: 48,
        scancode: 0x19,
        name: "v",
        label: "V",
        row: 3,
    },
    PhysicalKey {
        index: 49,
        scancode: 0x05,
        name: "b",
        label: "B",
        row: 3,
    },
    PhysicalKey {
        index: 50,
        scancode: 0x11,
        name: "n",
        label: "N",
        row: 3,
    },
    PhysicalKey {
        index: 51,
        scancode: 0x10,
        name: "m",
        label: "M",
        row: 3,
    },
    PhysicalKey {
        index: 52,
        scancode: 0x36,
        name: "<,",
        label: "< ,",
        row: 3,
    },
    PhysicalKey {
        index: 53,
        scancode: 0x37,
        name: ">.",
        label: "> .",
        row: 3,
    },
    PhysicalKey {
        index: 54,
        scancode: 0x38,
        name: "/?",
        label: "/ ?",
        row: 3,
    },
    PhysicalKey {
        index: 55,
        scancode: 0xE5,
        name: "rshift",
        label: "Shift",
        row: 3,
    },
    PhysicalKey {
        index: 56,
        scancode: 0x52,
        name: "up",
        label: "▲",
        row: 3,
    },
    PhysicalKey {
        index: 57,
        scancode: 0x4E,
        name: "pgdn",
        label: "PgDn",
        row: 3,
    },
    // Row 4 (Modifiers row)
    PhysicalKey {
        index: 58,
        scancode: 0xE0,
        name: "lctrl",
        label: "Ctrl",
        row: 4,
    },
    PhysicalKey {
        index: 59,
        scancode: 0xE3,
        name: "lwin",
        label: "Win",
        row: 4,
    },
    PhysicalKey {
        index: 60,
        scancode: 0xE2,
        name: "lalt",
        label: "Alt",
        row: 4,
    },
    PhysicalKey {
        index: 61,
        scancode: 0x2C,
        name: "space",
        label: "Space",
        row: 4,
    },
    PhysicalKey {
        index: 62,
        scancode: 0xE6,
        name: "ralt",
        label: "Alt",
        row: 4,
    },
    PhysicalKey {
        index: 63,
        scancode: 0xAF,
        name: "fn",
        label: "Fn",
        row: 4,
    },
    PhysicalKey {
        index: 64,
        scancode: 0xE4,
        name: "rctrl",
        label: "Ctrl",
        row: 4,
    },
    PhysicalKey {
        index: 65,
        scancode: 0x50,
        name: "left",
        label: "◀",
        row: 4,
    },
    PhysicalKey {
        index: 66,
        scancode: 0x51,
        name: "down",
        label: "▼",
        row: 4,
    },
    PhysicalKey {
        index: 67,
        scancode: 0x4F,
        name: "right",
        label: "▶",
        row: 4,
    },
];

/// Finds a key by index (0..67).
pub fn find_key_by_index(index: usize) -> Option<&'static PhysicalKey> {
    PHYSICAL_KEYS.get(index)
}

/// Finds a key by name (case-insensitive, e.g. "esc", "CAPS", "Space").
pub fn find_key_by_name(name: &str) -> Option<&'static PhysicalKey> {
    PHYSICAL_KEYS
        .iter()
        .find(|k| k.name.eq_ignore_ascii_case(name) || k.label.eq_ignore_ascii_case(name))
}

/// Finds a key by scan code.
pub fn find_key_by_scancode(scancode: u8) -> Option<&'static PhysicalKey> {
    PHYSICAL_KEYS.iter().find(|k| k.scancode == scancode)
}
