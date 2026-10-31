//! Lighting parameters, packet builders, and complete 22-packet atomic transaction generators.

use crate::constants::{
    BLOCK_TRAILER_MAGIC, CMD_OPCODE_HEADER, LIGHTING_BLOCK_COUNT, PAYLOAD_SIZE, SUBCMD_BEGIN_LIGHT,
    SUBCMD_COMMIT, SUBCMD_CUSTOM_RGB_HEADER, SUBCMD_PWM_RELOAD, SUBCMD_SET_LIGHT,
};
use crate::enums::{ColorType, Direction, LightMode};
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
