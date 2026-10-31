//! Sonix SN32F240B ISP bootloader wire protocol definitions and packet framing.

use crate::error::IspError;

pub const BOOTLOADER_VID: u16 = 0x0C45;
pub const BOOTLOADER_PID: u16 = 0x82AA;

pub const REPORT_ID: u8 = 0x00;
pub const PAYLOAD_SIZE: usize = 64;
pub const PAGE_SIZE: usize = 64;
pub const PAGE_COUNT: usize = 1024; // 64 KB / 64 = 1024 pages

// Cortex-M0 ISP Opcodes
pub const OP_ENTER_ISP: u8 = 0x20;
pub const OP_GET_VERSION: u8 = 0x21;
pub const OP_COMPARE_PWD: u8 = 0x22;
pub const OP_SET_ENCRYPTION: u8 = 0x23;
pub const OP_ENABLE_ERASE: u8 = 0x24;
pub const OP_ENABLE_PROGRAM: u8 = 0x25;
pub const OP_GET_CHECKSUM: u8 = 0x26;
pub const OP_RETURN_USER: u8 = 0x27;
pub const OP_SET_OPTION: u8 = 0x28;
pub const OP_GET_OPTION: u8 = 0x29;
pub const OP_CHECK_DATA: u8 = 0x2A;

// Sonix Magic Signatures
pub const ISP_MAGIC_AA: u8 = 0xAA;
pub const ISP_MAGIC_55: u8 = 0x55;
pub const ISP_UNLOCK_KEY: [u8; 2] = [0xFA, 0x5A]; // 0x5AFA little-endian

// Known Chip IDs
pub const CHIP_ID_SN32F240B: u16 = 0x240B;
pub const CHIP_ID_SN32F248B: u16 = 0x248B;

/// Information retrieved from the bootloader during handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootloaderInfo {
    pub chip_id: u16,
    pub bootloader_version: u16,
    pub is_supported_chip: bool,
}

impl BootloaderInfo {
    pub fn chip_name(&self) -> &'static str {
        match self.chip_id {
            CHIP_ID_SN32F240B => "SONiX SN32F240B (Cortex-M0, 64 KB Flash, 8 KB SRAM)",
            CHIP_ID_SN32F248B => "SONiX SN32F248B (Cortex-M0, 64 KB Flash, 8 KB SRAM)",
            _ => "Unknown / Unverified Sonix MCU",
        }
    }
}

/// Helper to create a zeroed 64-byte payload buffer.
pub fn new_payload() -> [u8; PAYLOAD_SIZE] {
    [0u8; PAYLOAD_SIZE]
}

/// Builds the software trigger packet sent to running keyboard to jump to Mask ROM bootloader.
pub fn build_software_bootloader_trigger() -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[..8].copy_from_slice(&rk855_proto::BOOTLOADER_TRIGGER_MAGIC);
    pkt
}

/// Builds the Enter ISP Mode packet (Opcode 0x20).
pub fn build_enter_isp_packet() -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_ENTER_ISP;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt[3] = ISP_UNLOCK_KEY[0];
    pkt[4] = ISP_UNLOCK_KEY[1];
    pkt
}

/// Builds the Get Firmware / Bootloader Version packet (Opcode 0x21).
pub fn build_get_version_packet() -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_GET_VERSION;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt
}

/// Helper to normalize response slices (skipping report ID byte 0 if present).
fn payload_slice(resp: &[u8]) -> &[u8] {
    if !resp.is_empty() && resp[0] == REPORT_ID && resp.len() >= 65 {
        &resp[1..]
    } else {
        resp
    }
}

/// Parses the response to Get Version (Opcode 0x21).
pub fn parse_version_response(raw_resp: &[u8]) -> Result<BootloaderInfo, IspError> {
    let resp = payload_slice(raw_resp);
    if resp.len() < 7 {
        return Err(IspError::UnexpectedResponse("Response too short".into()));
    }
    // Response check: resp[0] == 0x21, resp[1] == 0xAA, resp[2] == 0x55
    if resp[0] != OP_GET_VERSION || resp[1] != ISP_MAGIC_AA || resp[2] != ISP_MAGIC_55 {
        return Err(IspError::HandshakeFailed(format!(
            "Invalid version response header: {:02X?}",
            &resp[..3.min(resp.len())]
        )));
    }

    let chip_id = u16::from_le_bytes(resp[3..5].try_into().unwrap());
    let bootloader_version = u16::from_le_bytes(resp[5..7].try_into().unwrap());
    let is_supported_chip = chip_id == CHIP_ID_SN32F240B || chip_id == CHIP_ID_SN32F248B;

    Ok(BootloaderInfo {
        chip_id,
        bootloader_version,
        is_supported_chip,
    })
}

/// Builds an Enable Erase packet (Opcode 0x24) specifying the address range to erase.
pub fn build_erase_packet(start_addr: u32, end_addr: u32) -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_ENABLE_ERASE;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt[3..7].copy_from_slice(&start_addr.to_le_bytes());
    pkt[7..11].copy_from_slice(&end_addr.to_le_bytes());
    pkt
}

/// Verifies ACK response for erase or program.
pub fn verify_ack_response(raw_resp: &[u8], expected_opcode: u8) -> Result<(), IspError> {
    let resp = payload_slice(raw_resp);
    if resp.len() < 3
        || resp[0] != expected_opcode
        || resp[1] != ISP_MAGIC_AA
        || resp[2] != ISP_MAGIC_55
    {
        return Err(IspError::UnexpectedResponse(format!(
            "Expected ACK for 0x{expected_opcode:02X}, got {:02X?}",
            &resp[..3.min(resp.len())]
        )));
    }
    Ok(())
}

/// Builds an Enable Program setup packet (Opcode 0x25) setting the target page base address.
pub fn build_program_setup_packet(page_addr: u32) -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_ENABLE_PROGRAM;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt[3..7].copy_from_slice(&page_addr.to_le_bytes());
    pkt
}

/// Builds a 64-byte page data packet.
pub fn build_page_data_packet(data: &[u8; PAGE_SIZE]) -> [u8; PAYLOAD_SIZE] {
    *data
}

/// Builds an Enable Check Data (read-back) request packet (Opcode 0x2A).
pub fn build_check_data_packet(page_addr: u32, length: u32) -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_CHECK_DATA;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt[3..7].copy_from_slice(&page_addr.to_le_bytes());
    pkt[7..11].copy_from_slice(&length.to_le_bytes());
    pkt
}

/// Extracts read-back page data from a check data response.
pub fn extract_page_data(raw_resp: &[u8]) -> Result<[u8; PAGE_SIZE], IspError> {
    let resp = payload_slice(raw_resp);
    if resp.len() < PAGE_SIZE {
        return Err(IspError::UnexpectedResponse(format!(
            "Check data response length ({}) too short for 64-byte page",
            resp.len()
        )));
    }
    let mut page = [0u8; PAGE_SIZE];
    page.copy_from_slice(&resp[..PAGE_SIZE]);
    Ok(page)
}

/// Builds a Return User Mode (reset into application) packet (Opcode 0x27).
pub fn build_return_user_packet() -> [u8; PAYLOAD_SIZE] {
    let mut pkt = new_payload();
    pkt[0] = OP_RETURN_USER;
    pkt[1] = ISP_MAGIC_AA;
    pkt[2] = ISP_MAGIC_55;
    pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_framing() {
        let trig = build_software_bootloader_trigger();
        assert_eq!(&trig[..8], &rk855_proto::BOOTLOADER_TRIGGER_MAGIC);

        let isp = build_enter_isp_packet();
        assert_eq!(isp[0], OP_ENTER_ISP);
        assert_eq!(isp[1], 0xAA);
        assert_eq!(isp[2], 0x55);
        assert_eq!(&isp[3..5], &[0xFA, 0x5A]);

        let ver = build_get_version_packet();
        assert_eq!(ver[0], OP_GET_VERSION);

        let erase = build_erase_packet(0x0000, 0xE797);
        assert_eq!(erase[0], OP_ENABLE_ERASE);
        assert_eq!(u32::from_le_bytes(erase[3..7].try_into().unwrap()), 0x0000);
        assert_eq!(u32::from_le_bytes(erase[7..11].try_into().unwrap()), 0xE797);
    }

    #[test]
    fn test_parse_version() {
        let mut resp = [0u8; 64];
        resp[0] = OP_GET_VERSION;
        resp[1] = 0xAA;
        resp[2] = 0x55;
        resp[3..5].copy_from_slice(&0x240Bu16.to_le_bytes()); // SN32F240B
        resp[5..7].copy_from_slice(&0x0102u16.to_le_bytes()); // v1.02

        let info = parse_version_response(&resp).expect("Failed to parse version");
        assert_eq!(info.chip_id, 0x240B);
        assert_eq!(info.bootloader_version, 0x0102);
        assert!(info.is_supported_chip);
        assert!(info.chip_name().contains("SN32F240B"));
    }
}
