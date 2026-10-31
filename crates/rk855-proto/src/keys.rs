//! Physical 68-key matrix definitions and scancode lookup table for the RK855 (RK68).

/// Definition of a single physical key on the keyboard layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicalKey {
    /// Physical 0-based layout index (0..67)
    pub index: usize,
    /// Standard USB HID scan code (or internal modifier code)
    pub scancode: u8,
    /// Hardware matrix index (1..128) used in remap tables and custom RGB offsets
    pub matrix_index: u8,
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
        matrix_index: 1,
        name: "esc",
        label: "ESC",
        row: 0,
    },
    PhysicalKey {
        index: 1,
        scancode: 0x1E,
        matrix_index: 20,
        name: "1",
        label: "1 !",
        row: 0,
    },
    PhysicalKey {
        index: 2,
        scancode: 0x1F,
        matrix_index: 21,
        name: "2",
        label: "2 @",
        row: 0,
    },
    PhysicalKey {
        index: 3,
        scancode: 0x20,
        matrix_index: 22,
        name: "3",
        label: "3 #",
        row: 0,
    },
    PhysicalKey {
        index: 4,
        scancode: 0x21,
        matrix_index: 23,
        name: "4",
        label: "4 $",
        row: 0,
    },
    PhysicalKey {
        index: 5,
        scancode: 0x22,
        matrix_index: 24,
        name: "5",
        label: "5 %",
        row: 0,
    },
    PhysicalKey {
        index: 6,
        scancode: 0x23,
        matrix_index: 25,
        name: "6",
        label: "6 ^",
        row: 0,
    },
    PhysicalKey {
        index: 7,
        scancode: 0x24,
        matrix_index: 26,
        name: "7",
        label: "7 &",
        row: 0,
    },
    PhysicalKey {
        index: 8,
        scancode: 0x25,
        matrix_index: 27,
        name: "8",
        label: "8 *",
        row: 0,
    },
    PhysicalKey {
        index: 9,
        scancode: 0x26,
        matrix_index: 28,
        name: "9",
        label: "9 (",
        row: 0,
    },
    PhysicalKey {
        index: 10,
        scancode: 0x27,
        matrix_index: 29,
        name: "0",
        label: "0 )",
        row: 0,
    },
    PhysicalKey {
        index: 11,
        scancode: 0x2D,
        matrix_index: 30,
        name: "-_",
        label: "- _",
        row: 0,
    },
    PhysicalKey {
        index: 12,
        scancode: 0x2E,
        matrix_index: 31,
        name: "=+",
        label: "= +",
        row: 0,
    },
    PhysicalKey {
        index: 13,
        scancode: 0x2A,
        matrix_index: 103,
        name: "backspace",
        label: "Bksp",
        row: 0,
    },
    PhysicalKey {
        index: 14,
        scancode: 0x35,
        matrix_index: 19,
        name: "`~",
        label: "` ~",
        row: 0,
    },
    // Row 1 (QWERTY row)
    PhysicalKey {
        index: 15,
        scancode: 0x2B,
        matrix_index: 37,
        name: "tab",
        label: "Tab",
        row: 1,
    },
    PhysicalKey {
        index: 16,
        scancode: 0x14,
        matrix_index: 38,
        name: "q",
        label: "Q",
        row: 1,
    },
    PhysicalKey {
        index: 17,
        scancode: 0x1A,
        matrix_index: 39,
        name: "w",
        label: "W",
        row: 1,
    },
    PhysicalKey {
        index: 18,
        scancode: 0x08,
        matrix_index: 40,
        name: "e",
        label: "E",
        row: 1,
    },
    PhysicalKey {
        index: 19,
        scancode: 0x15,
        matrix_index: 41,
        name: "r",
        label: "R",
        row: 1,
    },
    PhysicalKey {
        index: 20,
        scancode: 0x17,
        matrix_index: 42,
        name: "t",
        label: "T",
        row: 1,
    },
    PhysicalKey {
        index: 21,
        scancode: 0x1C,
        matrix_index: 43,
        name: "y",
        label: "Y",
        row: 1,
    },
    PhysicalKey {
        index: 22,
        scancode: 0x18,
        matrix_index: 44,
        name: "u",
        label: "U",
        row: 1,
    },
    PhysicalKey {
        index: 23,
        scancode: 0x0C,
        matrix_index: 45,
        name: "i",
        label: "I",
        row: 1,
    },
    PhysicalKey {
        index: 24,
        scancode: 0x12,
        matrix_index: 46,
        name: "o",
        label: "O",
        row: 1,
    },
    PhysicalKey {
        index: 25,
        scancode: 0x13,
        matrix_index: 47,
        name: "p",
        label: "P",
        row: 1,
    },
    PhysicalKey {
        index: 26,
        scancode: 0x2F,
        matrix_index: 48,
        name: "[{",
        label: "[ {",
        row: 1,
    },
    PhysicalKey {
        index: 27,
        scancode: 0x30,
        matrix_index: 49,
        name: "]}",
        label: "] }",
        row: 1,
    },
    PhysicalKey {
        index: 28,
        scancode: 0x31,
        matrix_index: 67,
        name: "\\|",
        label: "\\ |",
        row: 1,
    },
    PhysicalKey {
        index: 29,
        scancode: 0x4C,
        matrix_index: 119,
        name: "del",
        label: "Del",
        row: 1,
    },
    // Row 2 (Home row)
    PhysicalKey {
        index: 30,
        scancode: 0x39,
        matrix_index: 55,
        name: "caps",
        label: "Caps",
        row: 2,
    },
    PhysicalKey {
        index: 31,
        scancode: 0x04,
        matrix_index: 56,
        name: "a",
        label: "A",
        row: 2,
    },
    PhysicalKey {
        index: 32,
        scancode: 0x16,
        matrix_index: 57,
        name: "s",
        label: "S",
        row: 2,
    },
    PhysicalKey {
        index: 33,
        scancode: 0x07,
        matrix_index: 58,
        name: "d",
        label: "D",
        row: 2,
    },
    PhysicalKey {
        index: 34,
        scancode: 0x09,
        matrix_index: 59,
        name: "f",
        label: "F",
        row: 2,
    },
    PhysicalKey {
        index: 35,
        scancode: 0x0A,
        matrix_index: 60,
        name: "g",
        label: "G",
        row: 2,
    },
    PhysicalKey {
        index: 36,
        scancode: 0x0B,
        matrix_index: 61,
        name: "h",
        label: "H",
        row: 2,
    },
    PhysicalKey {
        index: 37,
        scancode: 0x0D,
        matrix_index: 62,
        name: "j",
        label: "J",
        row: 2,
    },
    PhysicalKey {
        index: 38,
        scancode: 0x0E,
        matrix_index: 63,
        name: "k",
        label: "K",
        row: 2,
    },
    PhysicalKey {
        index: 39,
        scancode: 0x0F,
        matrix_index: 64,
        name: "l",
        label: "L",
        row: 2,
    },
    PhysicalKey {
        index: 40,
        scancode: 0x33,
        matrix_index: 65,
        name: ";:",
        label: "; :",
        row: 2,
    },
    PhysicalKey {
        index: 41,
        scancode: 0x34,
        matrix_index: 66,
        name: "'\"",
        label: "' \"",
        row: 2,
    },
    PhysicalKey {
        index: 42,
        scancode: 0x28,
        matrix_index: 85,
        name: "enter",
        label: "Enter",
        row: 2,
    },
    PhysicalKey {
        index: 43,
        scancode: 0x4B,
        matrix_index: 118,
        name: "pgup",
        label: "PgUp",
        row: 2,
    },
    // Row 3 (Bottom row)
    PhysicalKey {
        index: 44,
        scancode: 0xE1,
        matrix_index: 73,
        name: "lshift",
        label: "Shift",
        row: 3,
    },
    PhysicalKey {
        index: 45,
        scancode: 0x1D,
        matrix_index: 74,
        name: "z",
        label: "Z",
        row: 3,
    },
    PhysicalKey {
        index: 46,
        scancode: 0x1B,
        matrix_index: 75,
        name: "x",
        label: "X",
        row: 3,
    },
    PhysicalKey {
        index: 47,
        scancode: 0x06,
        matrix_index: 76,
        name: "c",
        label: "C",
        row: 3,
    },
    PhysicalKey {
        index: 48,
        scancode: 0x19,
        matrix_index: 77,
        name: "v",
        label: "V",
        row: 3,
    },
    PhysicalKey {
        index: 49,
        scancode: 0x05,
        matrix_index: 78,
        name: "b",
        label: "B",
        row: 3,
    },
    PhysicalKey {
        index: 50,
        scancode: 0x11,
        matrix_index: 79,
        name: "n",
        label: "N",
        row: 3,
    },
    PhysicalKey {
        index: 51,
        scancode: 0x10,
        matrix_index: 80,
        name: "m",
        label: "M",
        row: 3,
    },
    PhysicalKey {
        index: 52,
        scancode: 0x36,
        matrix_index: 81,
        name: "<,",
        label: "< ,",
        row: 3,
    },
    PhysicalKey {
        index: 53,
        scancode: 0x37,
        matrix_index: 82,
        name: ">.",
        label: "> .",
        row: 3,
    },
    PhysicalKey {
        index: 54,
        scancode: 0x38,
        matrix_index: 83,
        name: "/?",
        label: "/ ?",
        row: 3,
    },
    PhysicalKey {
        index: 55,
        scancode: 0xE5,
        matrix_index: 84,
        name: "rshift",
        label: "Shift",
        row: 3,
    },
    PhysicalKey {
        index: 56,
        scancode: 0x52,
        matrix_index: 101,
        name: "up",
        label: "▲",
        row: 3,
    },
    PhysicalKey {
        index: 57,
        scancode: 0x4E,
        matrix_index: 121,
        name: "pgdn",
        label: "PgDn",
        row: 3,
    },
    // Row 4 (Modifiers row)
    PhysicalKey {
        index: 58,
        scancode: 0xE0,
        matrix_index: 91,
        name: "lctrl",
        label: "Ctrl",
        row: 4,
    },
    PhysicalKey {
        index: 59,
        scancode: 0xE3,
        matrix_index: 92,
        name: "lwin",
        label: "Win",
        row: 4,
    },
    PhysicalKey {
        index: 60,
        scancode: 0xE2,
        matrix_index: 93,
        name: "lalt",
        label: "Alt",
        row: 4,
    },
    PhysicalKey {
        index: 61,
        scancode: 0x2C,
        matrix_index: 94,
        name: "space",
        label: "Space",
        row: 4,
    },
    PhysicalKey {
        index: 62,
        scancode: 0xE6,
        matrix_index: 95,
        name: "ralt",
        label: "Alt",
        row: 4,
    },
    PhysicalKey {
        index: 63,
        scancode: 0xAF,
        matrix_index: 96,
        name: "fn",
        label: "Fn",
        row: 4,
    },
    PhysicalKey {
        index: 64,
        scancode: 0xE4,
        matrix_index: 98,
        name: "rctrl",
        label: "Ctrl",
        row: 4,
    },
    PhysicalKey {
        index: 65,
        scancode: 0x50,
        matrix_index: 99,
        name: "left",
        label: "◀",
        row: 4,
    },
    PhysicalKey {
        index: 66,
        scancode: 0x51,
        matrix_index: 100,
        name: "down",
        label: "▼",
        row: 4,
    },
    PhysicalKey {
        index: 67,
        scancode: 0x4F,
        matrix_index: 102,
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
    let direct = PHYSICAL_KEYS
        .iter()
        .find(|k| k.name.eq_ignore_ascii_case(name) || k.label.eq_ignore_ascii_case(name));
    if direct.is_some() {
        return direct;
    }

    // Common key name aliases
    if name.eq_ignore_ascii_case("capslock") || name.eq_ignore_ascii_case("caps_lock") {
        return Some(&PHYSICAL_KEYS[30]);
    }
    if name.eq_ignore_ascii_case("escape") {
        return Some(&PHYSICAL_KEYS[0]);
    }
    if name.eq_ignore_ascii_case("lctrl")
        || name.eq_ignore_ascii_case("leftctrl")
        || name.eq_ignore_ascii_case("ctrl")
    {
        return Some(&PHYSICAL_KEYS[58]);
    }
    if name.eq_ignore_ascii_case("rctrl") || name.eq_ignore_ascii_case("rightctrl") {
        return Some(&PHYSICAL_KEYS[64]);
    }
    if name.eq_ignore_ascii_case("lshift")
        || name.eq_ignore_ascii_case("leftshift")
        || name.eq_ignore_ascii_case("shift")
    {
        return Some(&PHYSICAL_KEYS[44]);
    }
    if name.eq_ignore_ascii_case("rshift") || name.eq_ignore_ascii_case("rightshift") {
        return Some(&PHYSICAL_KEYS[55]);
    }
    if name.eq_ignore_ascii_case("lalt")
        || name.eq_ignore_ascii_case("leftalt")
        || name.eq_ignore_ascii_case("alt")
    {
        return Some(&PHYSICAL_KEYS[60]);
    }
    if name.eq_ignore_ascii_case("ralt") || name.eq_ignore_ascii_case("rightalt") {
        return Some(&PHYSICAL_KEYS[62]);
    }
    if name.eq_ignore_ascii_case("super")
        || name.eq_ignore_ascii_case("gui")
        || name.eq_ignore_ascii_case("lwin")
    {
        return Some(&PHYSICAL_KEYS[59]);
    }
    if name.eq_ignore_ascii_case("bksp") {
        return Some(&PHYSICAL_KEYS[13]);
    }
    if name.eq_ignore_ascii_case("spacebar") {
        return Some(&PHYSICAL_KEYS[61]);
    }
    if name.eq_ignore_ascii_case("delete") {
        return Some(&PHYSICAL_KEYS[29]);
    }
    if name.eq_ignore_ascii_case("pageup") {
        return Some(&PHYSICAL_KEYS[43]);
    }
    if name.eq_ignore_ascii_case("pagedown") {
        return Some(&PHYSICAL_KEYS[57]);
    }
    if name.eq_ignore_ascii_case("up") {
        return Some(&PHYSICAL_KEYS[56]);
    }
    if name.eq_ignore_ascii_case("down") {
        return Some(&PHYSICAL_KEYS[66]);
    }

    None
}

/// Finds a key by scan code.
pub fn find_key_by_scancode(scancode: u8) -> Option<&'static PhysicalKey> {
    PHYSICAL_KEYS.iter().find(|k| k.scancode == scancode)
}

impl PhysicalKey {
    /// Byte offset of this key\'s RGB quad [0x80, R, G, B] in the 1,152-byte custom RGB payload.
    /// Hardware formula: 512 + (matrix_index * 4).
    pub const fn rgb_buffer_offset(&self) -> usize {
        512 + 4 * (self.matrix_index as usize)
    }
}

/// Finds a key by hardware matrix index (1..128).
pub fn find_key_by_matrix_index(matrix_index: u8) -> Option<&'static PhysicalKey> {
    PHYSICAL_KEYS
        .iter()
        .find(|k| k.matrix_index == matrix_index)
}

/// Resolves standard scan code to physical hardware matrix index.
pub fn scancode_to_matrix_index(scancode: u8) -> Option<u8> {
    find_key_by_scancode(scancode).map(|k| k.matrix_index)
}
