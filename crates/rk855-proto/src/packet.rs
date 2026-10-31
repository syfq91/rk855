//! Packet primitives, builders, wire conversion, and parsing logic.

use crate::constants::*;
use crate::enums::{ColorType, Direction, LightMode, ProfileSlot};
use crate::lighting::LightParams;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtoError {
    #[error("Packet length too short: expected at least {expected}, got {actual}")]
    TooShort { expected: usize, actual: usize },

    #[error("Invalid command header opcode: expected 0x04, got 0x{0:02X}")]
    InvalidHeader(u8),

    #[error("Invalid light mode id: 0x{0:02X}")]
    InvalidLightMode(u8),

    #[error("Invalid color type id: 0x{0:02X}")]
    InvalidColorType(u8),

    #[error("Invalid direction id: 0x{0:02X}")]
    InvalidDirection(u8),

    #[error("Invalid profile slot id: 0x{0:02X}")]
    InvalidProfileSlot(u8),
}

/// Fixed 64-byte payload transmitted over the USB HID Feature Report channel.
pub type Packet = [u8; PAYLOAD_SIZE];

/// Fixed 65-byte wire buffer including the 1-byte Report ID prefix (0x00).
pub type WireReport = [u8; WIRE_REPORT_SIZE];

/// Converts a 64-byte payload into a 65-byte wire report with Report ID 0x00.
pub fn to_wire_report(payload: &Packet) -> WireReport {
    let mut wire = [0u8; WIRE_REPORT_SIZE];
    wire[0] = FEATURE_REPORT_ID;
    wire[1..].copy_from_slice(payload);
    wire
}

/// Extracts a 64-byte payload from a 65-byte wire report buffer.
pub fn from_wire_report(wire: &[u8]) -> Result<Packet, ProtoError> {
    if wire.len() < WIRE_REPORT_SIZE {
        return Err(ProtoError::TooShort {
            expected: WIRE_REPORT_SIZE,
            actual: wire.len(),
        });
    }
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt.copy_from_slice(&wire[1..1 + PAYLOAD_SIZE]);
    Ok(pkt)
}

/// Strongly-typed representation of parsed protocol packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedPacket {
    BeginLight,
    SetLight(LightParams),
    CustomRgbHeader { total_blocks: u8 },
    BeginRemap { block_count: u8 },
    BeginMacro,
    Commit,
    CommitAck { status: u8, flash_address: u16 },
    PwmReload,
    ProfileSwitch(ProfileSlot),
    ReadBackRequest { block_index: u8 },
    IspTrigger,
    Raw(Packet),
}

/// Builds packet to commit staged configuration to flash (`04 02`).
pub fn build_commit() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_COMMIT;
    pkt
}

/// Builds packet to reload PWM controller immediately (`04 F0`).
pub fn build_pwm_reload() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_PWM_RELOAD;
    pkt
}

/// Builds packet to switch onboard profile slot (`04 F1`, `04 F2`, `04 F3`).
pub fn build_switch_profile(slot: ProfileSlot) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_PROFILE_BASE + (slot as u8);
    pkt
}

/// Builds packet to initiate software ISP bootloader jump.
pub fn build_isp_trigger() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[..8].copy_from_slice(&BOOTLOADER_TRIGGER_MAGIC);
    pkt
}

/// Builds packet to request matrix or status read-back (`04 F5 <block>`).
pub fn build_read_back_request(block_index: u8) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_READ_BACK;
    pkt[2] = block_index;
    pkt
}

/// Builds packet to begin macro definition (`04 19`).
pub fn build_begin_macro() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_BEGIN_MACRO;
    pkt
}

/// Builds macro data chunk packet (`04 15 <seq> <data>`).
pub fn build_macro_data(seq: u8, data: &[u8]) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_MACRO_DATA;
    pkt[2] = seq;
    let copy_len = data.len().min(PAYLOAD_SIZE - 3);
    pkt[3..3 + copy_len].copy_from_slice(&data[..copy_len]);
    pkt
}

/// Parses a 64-byte payload into a strongly-typed [`ParsedPacket`].
pub fn parse_packet(data: &[u8]) -> Result<ParsedPacket, ProtoError> {
    if data.len() < 2 {
        return Err(ProtoError::TooShort {
            expected: 2,
            actual: data.len(),
        });
    }

    if data.starts_with(&BOOTLOADER_TRIGGER_MAGIC) {
        return Ok(ParsedPacket::IspTrigger);
    }

    if data[0] != CMD_OPCODE_HEADER {
        let mut raw = [0u8; PAYLOAD_SIZE];
        let copy_len = data.len().min(PAYLOAD_SIZE);
        raw[..copy_len].copy_from_slice(&data[..copy_len]);
        return Ok(ParsedPacket::Raw(raw));
    }

    let subcmd = data[1];
    match subcmd {
        SUBCMD_BEGIN_LIGHT => Ok(ParsedPacket::BeginLight),
        SUBCMD_SET_LIGHT => {
            if data.len() < 10 {
                return Err(ProtoError::TooShort {
                    expected: 10,
                    actual: data.len(),
                });
            }
            let mode = LightMode::from_u8(data[2]).ok_or(ProtoError::InvalidLightMode(data[2]))?;
            let color_type =
                ColorType::from_u8(data[3]).ok_or(ProtoError::InvalidColorType(data[3]))?;
            let r = data[4];
            let g = data[5];
            let b = data[6];
            let brightness = data[7];
            let speed = data[8];
            let direction =
                Direction::from_u8(data[9]).ok_or(ProtoError::InvalidDirection(data[9]))?;

            Ok(ParsedPacket::SetLight(LightParams {
                mode,
                color_type,
                r,
                g,
                b,
                brightness,
                speed,
                direction,
            }))
        }
        SUBCMD_CUSTOM_RGB_HEADER => {
            let total_blocks = if data.len() > 8 { data[8] } else { 0 };
            Ok(ParsedPacket::CustomRgbHeader { total_blocks })
        }
        SUBCMD_BEGIN_REMAP => {
            let block_count = if data.len() > 8 { data[8] } else { 0 };
            Ok(ParsedPacket::BeginRemap { block_count })
        }
        SUBCMD_BEGIN_MACRO => Ok(ParsedPacket::BeginMacro),
        SUBCMD_COMMIT => {
            if data.len() >= 6 && (data[2] != 0 || data[3] != 0) {
                let status = data[3];
                let flash_address = (data[4] as u16) | ((data[5] as u16) << 8);
                Ok(ParsedPacket::CommitAck {
                    status,
                    flash_address,
                })
            } else {
                Ok(ParsedPacket::Commit)
            }
        }
        SUBCMD_PWM_RELOAD => Ok(ParsedPacket::PwmReload),
        SUBCMD_READ_BACK => {
            let block_index = if data.len() > 2 { data[2] } else { 0 };
            Ok(ParsedPacket::ReadBackRequest { block_index })
        }
        0xF1..=0xF3 => {
            let slot_val = subcmd - SUBCMD_PROFILE_BASE;
            let slot =
                ProfileSlot::from_u8(slot_val).ok_or(ProtoError::InvalidProfileSlot(slot_val))?;
            Ok(ParsedPacket::ProfileSwitch(slot))
        }
        _ => {
            let mut raw = [0u8; PAYLOAD_SIZE];
            let copy_len = data.len().min(PAYLOAD_SIZE);
            raw[..copy_len].copy_from_slice(&data[..copy_len]);
            Ok(ParsedPacket::Raw(raw))
        }
    }
}
