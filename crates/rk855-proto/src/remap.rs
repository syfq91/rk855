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
