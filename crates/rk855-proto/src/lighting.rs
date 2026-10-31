//! Lighting parameters, packet builders, and complete 22-packet atomic transaction generators.

use crate::constants::{
    BLOCK_TRAILER_MAGIC, CMD_OPCODE_HEADER, LIGHTING_BLOCK_COUNT, PAYLOAD_SIZE, SUBCMD_BEGIN_LIGHT,
    SUBCMD_COMMIT, SUBCMD_CUSTOM_RGB_HEADER, SUBCMD_PWM_RELOAD, SUBCMD_SET_LIGHT,
};
use crate::enums::{ColorType, Direction, LightMode};
use crate::keys::{find_key_by_name, PHYSICAL_KEYS};
use crate::packet::Packet;
use alloc::vec::Vec;

/// High-level lighting configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightParams {
    pub mode: LightMode,
    pub color_type: ColorType,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: Direction,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            mode: LightMode::Static,
            color_type: ColorType::RgbCustom,
            r: 255,
            g: 0,
            b: 255,
            brightness: 3,
            speed: 10, // 0x0A default from vendor driver
            direction: Direction::LeftToRight,
        }
    }
}

/// Builds packet to initiate lighting configuration (`04 18`).
pub fn build_begin_light() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_BEGIN_LIGHT;
    pkt
}

/// Builds standalone packet to set lighting parameters (`04 11`).
pub fn build_set_light(params: &LightParams) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_SET_LIGHT;
    pkt[2] = params.mode as u8;
    pkt[3] = params.color_type as u8;
    pkt[4] = params.r;
    pkt[5] = params.g;
    pkt[6] = params.b;
    pkt[7] = params.brightness.min(15);
    pkt[8] = params.speed;
    pkt[9] = params.direction as u8;
    pkt
}

/// Builds header packet declaring custom lighting data blocks (`04 13 ... total_blocks`).
pub fn build_custom_rgb_header(total_blocks: u8) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_CUSTOM_RGB_HEADER;
    pkt[8] = total_blocks;
    pkt
}

/// Constructs the active mode parameter block (Block 18 in the 22-packet transaction).
pub fn build_active_mode_block(params: &LightParams) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = params.mode as u8;
    pkt[1] = 0xFF;
    pkt[2] = 0xE1;
    pkt[8] = params.color_type as u8;
    pkt[9] = params.brightness;
    pkt[10] = params.speed;
    pkt[14..16].copy_from_slice(&BLOCK_TRAILER_MAGIC);
    pkt
}

/// Builds the full atomic 22-packet lighting transaction as verified from USB captures.
///
/// Transaction structure:
/// - 1: `04 18` (Begin Lighting Configuration)
/// - 2: `04 13` (Declare 18 blocks)
/// - 3-7: Preset slots 1-5 (`01 ff e1...`, etc.)
/// - 8-10: Zero-padding blocks
/// - 11-19: Key records buffer (default 0x80 status)
/// - 20: Active mode block (Block 18)
/// - 21: `04 02` (Commit to flash)
/// - 22: `04 F0` (PWM reload)
pub fn build_lighting_transaction(params: &LightParams) -> Vec<Packet> {
    let mut pkts = Vec::with_capacity(22);

    // 1. Begin Lighting Configuration
    pkts.push(build_begin_light());

    // 2. Custom RGB Header (18 blocks follow)
    pkts.push(build_custom_rgb_header(LIGHTING_BLOCK_COUNT));

    // 3–7. Preset mode blocks 1..5
    let preset_ids = [0x01u8, 0x05, 0x09, 0x0D, 0x11];
    for &id in &preset_ids {
        let mut blk = [0u8; PAYLOAD_SIZE];
        blk[0] = id;
        blk[1] = 0xFF;
        blk[2] = 0xE1;
        blk[8] = 0x01;
        blk[9] = params.brightness;
        blk[10] = 0x0A;
        blk[14..16].copy_from_slice(&BLOCK_TRAILER_MAGIC);
        blk[62..64].copy_from_slice(&BLOCK_TRAILER_MAGIC);
        pkts.push(blk);
    }

    // 8–10. Padding blocks (3 blocks of zeros)
    for _ in 0..3 {
        pkts.push([0u8; PAYLOAD_SIZE]);
    }

    // 11–19. Key records blocks (9 blocks with default status 0x80)
    for _ in 0..9 {
        let mut blk = [0u8; PAYLOAD_SIZE];
        for i in (0..PAYLOAD_SIZE).step_by(4) {
            blk[i] = 0x80;
        }
        pkts.push(blk);
    }

    // 20. Block 18: Active mode block
    pkts.push(build_active_mode_block(params));

    // 21. Commit to Flash
    let mut commit = [0u8; PAYLOAD_SIZE];
    commit[0] = CMD_OPCODE_HEADER;
    commit[1] = SUBCMD_COMMIT;
    pkts.push(commit);

    // 22. PWM Reload
    let mut reload = [0u8; PAYLOAD_SIZE];
    reload[0] = CMD_OPCODE_HEADER;
    reload[1] = SUBCMD_PWM_RELOAD;
    pkts.push(reload);

    pkts
}

/// Representation of custom per-key RGB colors for all 68 keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomRgbMap {
    /// 68 keys corresponding to PHYSICAL_KEYS order
    pub colors: [Option<(u8, u8, u8)>; 68],
}

impl Default for CustomRgbMap {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomRgbMap {
    pub const fn new() -> Self {
        Self { colors: [None; 68] }
    }

    pub fn set_key_color(&mut self, key_index: usize, r: u8, g: u8, b: u8) {
        if key_index < 68 {
            self.colors[key_index] = Some((r, g, b));
        }
    }

    pub fn set_key_by_name(&mut self, name: &str, r: u8, g: u8, b: u8) -> Result<(), &'static str> {
        if let Some(key) = find_key_by_name(name) {
            self.colors[key.index] = Some((r, g, b));
            Ok(())
        } else {
            Err("Unknown key name")
        }
    }

    pub fn set_all(&mut self, r: u8, g: u8, b: u8) {
        self.colors.fill(Some((r, g, b)));
    }

    pub fn clear(&mut self) {
        self.colors.fill(None);
    }
}

/// Builds the 18 data blocks (1,152 bytes) for custom per-key RGB illumination.
pub fn build_custom_rgb_blocks(rgb_map: &CustomRgbMap) -> Vec<Packet> {
    let mut blocks = Vec::with_capacity(18);

    // 1–5. Preset mode blocks (20 mode records of 16 bytes each)
    let mut preset_data = [0u8; 5 * PAYLOAD_SIZE];
    for i in 0..19 {
        let offset = i * 16;
        preset_data[offset] = (i + 1) as u8;
        preset_data[offset + 1] = 0xFF;
        preset_data[offset + 2] = 0xE1;
        preset_data[offset + 8] = 0x01;
        preset_data[offset + 9] = 0x0F; // Brightness 15
        preset_data[offset + 10] = 0x0A; // Speed 10
        preset_data[offset + 14..offset + 16].copy_from_slice(&BLOCK_TRAILER_MAGIC);
    }
    // Preset 20 (Custom RGB preset slot in Block 5)
    let p20_offset = 19 * 16;
    preset_data[p20_offset] = 0x80;
    preset_data[p20_offset + 9] = 0x0F;
    preset_data[p20_offset + 14..p20_offset + 16].copy_from_slice(&BLOCK_TRAILER_MAGIC);

    blocks.extend_from_slice(preset_data.as_chunks::<PAYLOAD_SIZE>().0);

    // 6–8. Zero-padding blocks (3 blocks)
    for _ in 0..3 {
        blocks.push([0u8; PAYLOAD_SIZE]);
    }

    // 9–17. Key records blocks (9 blocks = 576 bytes = 144 slots of 4 bytes)
    let mut key_area = [0u8; 9 * PAYLOAD_SIZE];
    // Initialize all 144 slots with 0x80 status
    for i in (0..key_area.len()).step_by(4) {
        key_area[i] = 0x80;
    }

    // Fill configured key colors using reversed hardware matrix mapping
    for (i, key) in PHYSICAL_KEYS.iter().enumerate() {
        if let Some((r, g, b)) = rgb_map.colors[i] {
            let offset = 4 * (key.matrix_index as usize);
            if offset + 3 < key_area.len() {
                key_area[offset] = 0x80;
                key_area[offset + 1] = r;
                key_area[offset + 2] = g;
                key_area[offset + 3] = b;
            }
        }
    }

    blocks.extend_from_slice(key_area.as_chunks::<PAYLOAD_SIZE>().0);

    // 18. Active mode block: Custom RGB active indicator
    let mut active_blk = [0u8; PAYLOAD_SIZE];
    active_blk[0] = 0x80; // Active custom RGB mode
    active_blk[1] = 0xFF;
    active_blk[2] = 0xE1;
    active_blk[8] = 0x01; // Color type
    active_blk[9] = 0x0F; // Brightness 15
    active_blk[10] = 0x0A; // Speed 10
    active_blk[14..16].copy_from_slice(&BLOCK_TRAILER_MAGIC);
    active_blk[62..64].copy_from_slice(&BLOCK_TRAILER_MAGIC);
    blocks.push(active_blk);

    blocks
}

/// Builds the complete custom RGB transaction.
/// - If `persist` is true: 22 packets (Begin + Header + 18 blocks + Commit + Reload)
/// - If `persist` is false (live preview): 21 packets (Begin + Header + 18 blocks + Reload)
pub fn build_custom_rgb_transaction(rgb_map: &CustomRgbMap, persist: bool) -> Vec<Packet> {
    let mut pkts = Vec::with_capacity(22);
    pkts.push(build_begin_light());
    pkts.push(build_custom_rgb_header(LIGHTING_BLOCK_COUNT));
    pkts.extend(build_custom_rgb_blocks(rgb_map));

    if persist {
        let mut commit = [0u8; PAYLOAD_SIZE];
        commit[0] = CMD_OPCODE_HEADER;
        commit[1] = SUBCMD_COMMIT;
        pkts.push(commit);
    }

    let mut reload = [0u8; PAYLOAD_SIZE];
    reload[0] = CMD_OPCODE_HEADER;
    reload[1] = SUBCMD_PWM_RELOAD;
    pkts.push(reload);

    pkts
}
