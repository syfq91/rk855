//! Key remapping table encoding, decoding, and transaction packet builders.

use crate::constants::{
    CMD_OPCODE_HEADER, PAYLOAD_SIZE, REMAP_BLOCK_COUNT, REMAP_TABLE_SIZE, SUBCMD_BEGIN_REMAP,
};
use crate::packet::Packet;
use alloc::vec::Vec;

/// Represents a single key mapping: physical key scancode mapped to target action/scancode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemapEntry {
    pub physical_scancode: u8,
    pub target_code: u8,
}

impl RemapEntry {
    pub const fn new(physical_scancode: u8, target_code: u8) -> Self {
        Self {
            physical_scancode,
            target_code,
        }
    }
}

/// Encodes a list of remap entries into the 256-byte flash table format.
///
/// Each remap entry occupies 2 bytes: `[physical_scancode, target_code]`.
/// Unused slots remain zero-padded up to 256 bytes.
pub fn encode_remap_table(entries: &[RemapEntry]) -> [u8; REMAP_TABLE_SIZE] {
    let mut table = [0u8; REMAP_TABLE_SIZE];
    let mut offset = 0;

    for entry in entries {
        if offset + 2 > REMAP_TABLE_SIZE {
            break;
        }
        table[offset] = entry.physical_scancode;
        table[offset + 1] = entry.target_code;
        offset += 2;
    }

    table
}

/// Decodes a 256-byte flash table back into a list of remap entries.
pub fn decode_remap_table(table: &[u8; REMAP_TABLE_SIZE]) -> Vec<RemapEntry> {
    let mut entries = Vec::new();
    for i in (0..REMAP_TABLE_SIZE).step_by(2) {
        let phys = table[i];
        let target = table[i + 1];
        if phys != 0 {
            entries.push(RemapEntry {
                physical_scancode: phys,
                target_code: target,
            });
        }
    }
    entries
}

/// Builds the header packet to initiate a key remapping transaction (`04 20 ... 0x04`).
pub fn build_begin_remap(block_count: u8) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_BEGIN_REMAP;
    pkt[8] = block_count;
    pkt
}

/// Splits the 256-byte remap table into four 64-byte chunks ready for transmission.
pub fn split_remap_table_blocks(
    table: &[u8; REMAP_TABLE_SIZE],
) -> [Packet; REMAP_BLOCK_COUNT as usize] {
    let mut blocks = [[0u8; PAYLOAD_SIZE]; REMAP_BLOCK_COUNT as usize];
    for (i, block) in blocks.iter_mut().enumerate() {
        let start = i * PAYLOAD_SIZE;
        block.copy_from_slice(&table[start..start + PAYLOAD_SIZE]);
    }
    blocks
}

/// Builds the complete atomic key remapping transaction:
/// - 1: `04 20 ... 0x04` (Begin Remap Header declaring 4 blocks)
/// - 2-5: 4 blocks of 64 bytes (256 bytes total table)
/// - 6: `04 02` (Commit Flash)
/// - 7: `04 F0` (PWM / Matrix Reload)
pub fn build_remap_transaction(entries: &[RemapEntry]) -> Vec<Packet> {
    let mut pkts = Vec::with_capacity(7);
    let table = encode_remap_table(entries);
    pkts.push(build_begin_remap(REMAP_BLOCK_COUNT));
    let blocks = split_remap_table_blocks(&table);
    for b in blocks {
        pkts.push(b);
    }
    pkts.push(crate::packet::build_commit());
    pkts.push(crate::packet::build_pwm_reload());
    pkts
}

/// Resolves a key name, symbol, shortcut, or hex code to its standard target scancode / action code.
pub fn resolve_action_code(s: &str) -> Option<u8> {
    let s = s.trim();

    // Check numeric / hex values first
    if let Some(hex_str) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        if let Ok(num) = u8::from_str_radix(hex_str, 16) {
            return Some(num);
        }
    }
    if let Ok(num) = s.parse::<u8>() {
        return Some(num);
    }

    // Letters A..Z
    if s.len() == 1 {
        let c = s.as_bytes()[0].to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            return Some(0x04 + (c - b'A'));
        }
        if c.is_ascii_digit() && c != b'0' {
            return Some(0x1E + (c - b'1'));
        }
        if c == b'0' {
            return Some(0x27);
        }
    }

    // Standard physical key lookup
    if let Some(key) = crate::keys::find_key_by_name(s) {
        return Some(key.scancode);
    }

    // Standard names and aliases
    let s_upper = s.to_ascii_uppercase();
    match s_upper.as_str() {
        "ESC" | "ESCAPE" => Some(0x29),
        "ENTER" | "RETURN" => Some(0x28),
        "BACKSPACE" | "BKSP" => Some(0x2A),
        "TAB" => Some(0x2B),
        "SPACE" | "SPACEBAR" => Some(0x2C),
        "CAPS" | "CAPSLOCK" => Some(0x39),
        "F1" => Some(0x3A),
        "F2" => Some(0x3B),
        "F3" => Some(0x3C),
        "F4" => Some(0x3D),
        "F5" => Some(0x3E),
        "F6" => Some(0x3F),
        "F7" => Some(0x40),
        "F8" => Some(0x41),
        "F9" => Some(0x42),
        "F10" => Some(0x43),
        "F11" => Some(0x44),
        "F12" => Some(0x45),
        "PRTSCN" | "PRINTSCREEN" => Some(0x46),
        "SCROLLLOCK" => Some(0x47),
        "PAUSE" | "BREAK" => Some(0x48),
        "INS" | "INSERT" => Some(0x49),
        "HOME" => Some(0x4A),
        "PGUP" | "PAGEUP" => Some(0x4B),
        "DEL" | "DELETE" => Some(0x4C),
        "END" => Some(0x4D),
        "PGDN" | "PAGEDOWN" => Some(0x4E),
        "RIGHT" | "ARROW_RIGHT" => Some(0x4F),
        "LEFT" | "ARROW_LEFT" => Some(0x50),
        "DOWN" | "ARROW_DOWN" => Some(0x51),
        "UP" | "ARROW_UP" => Some(0x52),

        // Modifiers
        "LCTRL" | "CTRL" | "CONTROL" => Some(0xE0),
        "LSHIFT" | "SHIFT" => Some(0xE1),
        "LALT" | "ALT" => Some(0xE2),
        "LWIN" | "WIN" | "SUPER" | "GUI" => Some(0xE3),
        "RCTRL" => Some(0xE4),
        "RSHIFT" => Some(0xE5),
        "RALT" => Some(0xE6),
        "RWIN" => Some(0xE7),
        "FN" => Some(0xAF),

        // Symbols
        "-" | "-_" | "MINUS" => Some(0x2D),
        "=" | "=+" | "EQUALS" => Some(0x2E),
        "[" | "[{" | "LBRACKET" => Some(0x2F),
        "]" | "]}" | "RBRACKET" => Some(0x30),
        "\\" | "\\|" | "BACKSLASH" => Some(0x31),
        ";" | ";:" | "SEMICOLON" => Some(0x33),
        "'" | "'\"" | "QUOTE" => Some(0x34),
        "`" | "`~" | "GRAVE" | "TILDE" => Some(0x35),
        "," | "<," | "COMMA" => Some(0x36),
        "." | ">." | "DOT" | "PERIOD" => Some(0x37),
        "/" | "/?" | "SLASH" => Some(0x38),

        // Multimedia
        "PLAY" | "PLAY_PAUSE" | "PLAYPAUSE" => Some(179),
        "STOP" => Some(180),
        "PREV" | "PREVIOUS" | "PREV_TRACK" => Some(181),
        "NEXT" | "NEXT_TRACK" => Some(182),
        "VOL_UP" | "VOL+" | "VOLUME_UP" => Some(183),
        "VOL_DOWN" | "VOL-" | "VOLUME_DOWN" => Some(184),
        "MUTE" => Some(185),

        // Mouse actions
        "MOUSE_LEFT" | "MOUSE_LCLICK" | "LCLICK" => Some(170),
        "MOUSE_MIDDLE" | "MOUSE_MCLICK" | "MCLICK" => Some(172),
        "MOUSE_RIGHT" | "MOUSE_RCLICK" | "RCLICK" => Some(173),
        "MOUSE_DOUBLE_CLICK" | "DOUBLE_CLICK" => Some(174),
        "MOUSE_SCROLL_UP" | "SCROLL_UP" => Some(175),
        "MOUSE_SCROLL_DOWN" | "SCROLL_DOWN" => Some(176),
        "MOUSE_BACK" | "MOUSE_B4" => Some(177),
        "MOUSE_FORWARD" | "MOUSE_B5" => Some(178),

        // Disabled / None
        "NONE" | "DISABLED" | "DISABLE" | "NULL" => Some(0x00),

        _ => None,
    }
}

/// Converts a target action code back into a canonical readable string representation.
pub fn action_code_to_name(code: u8) -> &'static str {
    match code {
        0x00 => "Disabled",
        0x04 => "A",
        0x05 => "B",
        0x06 => "C",
        0x07 => "D",
        0x08 => "E",
        0x09 => "F",
        0x0A => "G",
        0x0B => "H",
        0x0C => "I",
        0x0D => "J",
        0x0E => "K",
        0x0F => "L",
        0x10 => "M",
        0x11 => "N",
        0x12 => "O",
        0x13 => "P",
        0x14 => "Q",
        0x15 => "R",
        0x16 => "S",
        0x17 => "T",
        0x18 => "U",
        0x19 => "V",
        0x1A => "W",
        0x1B => "X",
        0x1C => "Y",
        0x1D => "Z",
        0x1E => "1",
        0x1F => "2",
        0x20 => "3",
        0x21 => "4",
        0x22 => "5",
        0x23 => "6",
        0x24 => "7",
        0x25 => "8",
        0x26 => "9",
        0x27 => "0",
        0x28 => "Enter",
        0x29 => "Escape",
        0x2A => "Backspace",
        0x2B => "Tab",
        0x2C => "Space",
        0x2D => "-_",
        0x2E => "=+",
        0x2F => "[{",
        0x30 => "]}",
        0x31 => "\\|",
        0x33 => ";:",
        0x34 => "'\"",
        0x35 => "`~",
        0x36 => "<,",
        0x37 => ">.",
        0x38 => "/?",
        0x39 => "CapsLock",
        0x3A => "F1",
        0x3B => "F2",
        0x3C => "F3",
        0x3D => "F4",
        0x3E => "F5",
        0x3F => "F6",
        0x40 => "F7",
        0x41 => "F8",
        0x42 => "F9",
        0x43 => "F10",
        0x44 => "F11",
        0x45 => "F12",
        0x46 => "PrintScreen",
        0x47 => "ScrollLock",
        0x48 => "Pause",
        0x49 => "Insert",
        0x4A => "Home",
        0x4B => "PageUp",
        0x4C => "Delete",
        0x4D => "End",
        0x4E => "PageDown",
        0x4F => "Right",
        0x50 => "Left",
        0x51 => "Down",
        0x52 => "Up",
        0xAF => "Fn",
        0xE0 => "LeftCtrl",
        0xE1 => "LeftShift",
        0xE2 => "LeftAlt",
        0xE3 => "LeftWin",
        0xE4 => "RightCtrl",
        0xE5 => "RightShift",
        0xE6 => "RightAlt",
        0xE7 => "RightWin",
        170 => "MouseLeft",
        172 => "MouseMiddle",
        173 => "MouseRight",
        174 => "MouseDoubleClick",
        176 => "MouseScrollDown",
        177 => "MouseButton4",
        178 => "MouseButton5",
        179 => "PlayPause",
        180 => "Stop",
        181 => "PrevTrack",
        182 => "NextTrack",
        183 => "VolumeUp",
        184 => "VolumeDown",
        185 => "Mute",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_remap_encode_decode_roundtrip() {
        let original = vec![
            RemapEntry::new(0x39, 0x29), // CapsLock -> Escape
            RemapEntry::new(0xE0, 0xE2), // LCtrl -> LAlt
        ];
        let table = encode_remap_table(&original);
        assert_eq!(table[0], 0x39);
        assert_eq!(table[1], 0x29);
        assert_eq!(table[2], 0xE0);
        assert_eq!(table[3], 0xE2);
        assert_eq!(&table[4..], &[0u8; 252]);

        let decoded = decode_remap_table(&table);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_split_remap_blocks() {
        let mut table = [0u8; REMAP_TABLE_SIZE];
        table[0] = 0xAA;
        table[64] = 0xBB;
        table[128] = 0xCC;
        table[192] = 0xDD;

        let blocks = split_remap_table_blocks(&table);
        assert_eq!(blocks[0][0], 0xAA);
        assert_eq!(blocks[1][0], 0xBB);
        assert_eq!(blocks[2][0], 0xCC);
        assert_eq!(blocks[3][0], 0xDD);
    }
}
