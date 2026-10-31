//! Firmware image parsing, normalization, pre-flight validation, and analysis.

use crate::error::IspError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Expected total on-chip flash size for Sonix SN32F240B (64 KB).
pub const FLASH_SIZE: usize = 65536;

/// Hardware ROM flash validity signature required by Mask ROM bootloader.
pub const VALIDITY_MARKER_ADDR: usize = 0xFFFC;
pub const VALIDITY_MARKER_VALUE: u32 = 0xAAAA5555;

/// Computes lowercase hex-encoded SHA-256 digest of arbitrary bytes.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Known-good official factory firmware SHA-256 digests.
pub const KNOWN_FACTORY_BIN_SHA256: &str =
    "1e1be3a56ba5785cbd3922c9eb2ae08748796bdba6fd4e5e87df336bea311ab6";
pub const KNOWN_FACTORY_HEX_SHA256: &str =
    "ba0f38cb1276077959f8851b2bd38db524d495c980837e549f91ed8ba1d592f0";

/// Detailed pre-flight analysis report of a candidate firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareAnalysis {
    /// Total image size in bytes (normalized to 64 KB).
    pub file_size: usize,
    /// SHA-256 hex digest of the normalized flat binary image.
    pub sha256_hex: String,
    /// Initial stack pointer from vector 0 (offset 0x00).
    pub initial_sp: u32,
    /// Reset handler vector address from vector 1 (offset 0x04).
    pub reset_vector: u32,
    /// Whether the reset vector has the Cortex-M Thumb execution bit (bit 0) set.
    pub is_thumb_mode: bool,
    /// Whether the initial SP points to valid SN32F240B SRAM (`0x20000000..=0x20002000`).
    pub is_valid_sram_sp: bool,
    /// Whether the reset vector points to valid on-chip Flash (`0x00000000..0x00010000`).
    pub is_valid_flash_reset: bool,
    /// Whether the 4-byte Sonix Mask ROM validity signature at `0xFFFC` matches `0xAAAA5555`.
    pub has_rom_marker: bool,
    /// Actual value found at `0xFFFC`.
    pub rom_marker_value: u32,
    /// Contains embedded Apple USB VID:PID `05AC:024F`.
    pub contains_apple_vid_pid: bool,
    /// Contains software ISP bootloader trigger sequence `AA 42 89 5A FF 71 62 CC`.
    pub contains_isp_trigger_magic: bool,
    /// Single-color hardware flag at offset `0x800C` (0xAAAA = monochrome mode enabled).
    pub single_color_flag: u16,
    /// Count of populated (non-0xFF) bytes in flash image.
    pub populated_bytes: usize,
    /// Matches official factory firmware SHA-256 digest.
    pub is_known_factory_image: bool,
    /// Pre-flight validation error messages (empty if completely valid).
    pub errors: Vec<String>,
    /// Informational warnings (e.g. unknown custom build).
    pub warnings: Vec<String>,
}

impl FirmwareAnalysis {
    /// Returns true if all critical safety checks pass and the image is safe to flash.
    pub fn is_safe_to_flash(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Parses an Intel HEX formatted string into a 64 KB flat binary buffer.
pub fn parse_intel_hex(hex_str: &str) -> Result<Vec<u8>, IspError> {
    let mut flash = vec![0xFFu8; FLASH_SIZE];
    let reader = ihex::Reader::new(hex_str);
    let mut base_address: u32 = 0;
    let mut records_found = 0;

    for record_res in reader {
        let record = record_res.map_err(|e| IspError::Hex(format!("{e}")))?;
        records_found += 1;
        match record {
            ihex::Record::Data { offset, value } => {
                let target_addr = base_address.saturating_add(offset as u32) as usize;
                if target_addr + value.len() > FLASH_SIZE {
                    return Err(IspError::InvalidImage(format!(
                        "HEX data record at address 0x{target_addr:05X} (length {}) exceeds 64 KB flash limit",
                        value.len()
                    )));
                }
                flash[target_addr..target_addr + value.len()].copy_from_slice(&value);
            }
            ihex::Record::ExtendedLinearAddress(upper) => {
                base_address = (upper as u32) << 16;
            }
            ihex::Record::ExtendedSegmentAddress(segment) => {
                base_address = (segment as u32) << 4;
            }
            ihex::Record::EndOfFile => break,
            _ => {}
        }
    }

    if records_found == 0 {
        return Err(IspError::Hex("Intel HEX file contains no records".into()));
    }

    Ok(flash)
}

/// Loads and normalizes a firmware file (either `.hex` or `.bin`) into a 64 KB flat binary buffer.
pub fn load_firmware(path: &Path) -> Result<Vec<u8>, IspError> {
    if !path.exists() {
        return Err(IspError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Firmware file not found: {}", path.display()),
        )));
    }

    let raw_bytes = fs::read(path)?;
    if raw_bytes.is_empty() {
        return Err(IspError::InvalidImage("Firmware file is empty".into()));
    }

    // Check if Intel HEX by extension or first non-whitespace byte ':'
    let is_hex = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.eq_ignore_ascii_case("hex"))
        .unwrap_or(false)
        || raw_bytes.iter().find(|&&b| !b.is_ascii_whitespace()) == Some(&b':');

    if is_hex {
        let hex_str = String::from_utf8(raw_bytes)
            .map_err(|e| IspError::Hex(format!("Invalid UTF-8 in HEX file: {e}")))?;
        parse_intel_hex(&hex_str)
    } else {
        if raw_bytes.len() > FLASH_SIZE {
            return Err(IspError::InvalidImage(format!(
                "Binary image size ({} bytes) exceeds microcontroller 64 KB limit",
                raw_bytes.len()
            )));
        }
        let mut flash = vec![0xFFu8; FLASH_SIZE];
        flash[..raw_bytes.len()].copy_from_slice(&raw_bytes);
        Ok(flash)
    }
}

/// Validates and produces a comprehensive analysis report for a 64 KB firmware byte slice.
pub fn validate_firmware_image(data: &[u8]) -> Result<FirmwareAnalysis, IspError> {
    if data.len() < FLASH_SIZE {
        return Err(IspError::InvalidImage(format!(
            "Image length ({} bytes) is less than required 64 KB ({FLASH_SIZE} bytes)",
            data.len()
        )));
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // 1. Vector table checks
    let initial_sp = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let reset_vector = u32::from_le_bytes(data[4..8].try_into().unwrap());

    // Cortex-M0 SRAM is 8 KB at 0x20000000..=0x20002000
    let is_valid_sram_sp = (0x20000000..=0x20002000).contains(&initial_sp);
    if !is_valid_sram_sp {
        errors.push(format!(
            "Initial Stack Pointer (0x{initial_sp:08X}) is outside valid SRAM (0x20000000..0x20002000)"
        ));
    }

    // Flash reset vector must be within 0x00000000..0x00010000
    let is_valid_flash_reset = reset_vector < (FLASH_SIZE as u32);
    if !is_valid_flash_reset {
        errors.push(format!(
            "Reset Vector (0x{reset_vector:08X}) points outside on-chip flash (0x00000000..0x00010000)"
        ));
    }

    // Thumb execution bit (bit 0) MUST be 1
    let is_thumb_mode = (reset_vector & 1) == 1;
    if !is_thumb_mode {
        errors.push(format!(
            "Reset Vector (0x{reset_vector:08X}) lacks required ARM Thumb mode bit (bit 0 must be 1)"
        ));
    }

    // 2. Hardware flash validity marker at 0xFFFC
    let rom_marker_value = u32::from_le_bytes(
        data[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .try_into()
            .unwrap(),
    );
    let has_rom_marker = rom_marker_value == VALIDITY_MARKER_VALUE;
    if !has_rom_marker {
        errors.push(format!(
            "Flash Validity Signature at 0x{VALIDITY_MARKER_ADDR:04X} is invalid (found 0x{rom_marker_value:08X}, expected 0x{VALIDITY_MARKER_VALUE:08X}). MCU will refuse to boot!"
        ));
    }

    // 3. Apple VID:PID needle (05AC:024F -> little-endian AC 05 4F 02)
    let apple_needle = [0xAC, 0x05, 0x4F, 0x02];
    let contains_apple_vid_pid = data.windows(4).any(|w| w == apple_needle);
    if !contains_apple_vid_pid {
        warnings
            .push("Apple VID:PID descriptor not detected in image (may be a custom build)".into());
    }

    // 4. ISP bootloader trigger sequence
    let contains_isp_trigger_magic = data
        .windows(rk855_proto::BOOTLOADER_TRIGGER_MAGIC.len())
        .any(|w| w == rk855_proto::BOOTLOADER_TRIGGER_MAGIC);
    if !contains_isp_trigger_magic {
        warnings.push("Software ISP bootloader trigger not found in image (re-flashing over USB without P2.2 boot pin grounding may be impossible)".into());
    }

    // 5. Single-color flag at 0x800C
    let single_color_flag = u16::from_le_bytes(data[0x800C..0x800E].try_into().unwrap());

    // 6. Populated byte count
    let populated_bytes = data.iter().filter(|&&b| b != 0xFF).count();
    if populated_bytes < 10000 {
        errors.push(format!(
            "Image contains only {populated_bytes} non-0xFF bytes (suspiciously small firmware)"
        ));
    }

    // 7. SHA-256 digest
    let mut hasher = Sha256::new();
    hasher.update(data);
    let sha256_hex = format!("{:x}", hasher.finalize());
    let is_known_factory_image = sha256_hex == KNOWN_FACTORY_BIN_SHA256;

    if !is_known_factory_image {
        warnings.push(format!(
            "Image SHA-256 ({sha256_hex}) does not match official factory firmware"
        ));
    }

    Ok(FirmwareAnalysis {
        file_size: data.len(),
        sha256_hex,
        initial_sp,
        reset_vector,
        is_thumb_mode,
        is_valid_sram_sp,
        is_valid_flash_reset,
        has_rom_marker,
        rom_marker_value,
        contains_apple_vid_pid,
        contains_isp_trigger_magic,
        single_color_flag,
        populated_bytes,
        is_known_factory_image,
        errors,
        warnings,
    })
}

/// Analyzes an arbitrary firmware file on disk (supports `.hex` and `.bin`).
pub fn analyze_firmware(path: &Path) -> Result<FirmwareAnalysis, IspError> {
    let data = load_firmware(path)?;
    validate_firmware_image(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_official_hex_and_bin_match() {
        let hex_path = Path::new("../../firmware/SN32F240B_32.hex");
        let bin_path = Path::new("../../firmware/SN32F240B_32.bin");

        if !hex_path.exists() || !bin_path.exists() {
            return;
        }

        let hex_data = load_firmware(hex_path).expect("Failed to load HEX firmware");
        let bin_data = load_firmware(bin_path).expect("Failed to load BIN firmware");

        assert_eq!(hex_data.len(), FLASH_SIZE);
        assert_eq!(bin_data.len(), FLASH_SIZE);

        // Byte-for-byte exact equality between HEX and BIN!
        assert_eq!(
            hex_data, bin_data,
            "Loaded Intel HEX does not match flat binary byte-for-byte"
        );

        let analysis = validate_firmware_image(&hex_data).expect("Validation failed");
        assert!(analysis.is_safe_to_flash(), "Errors: {:?}", analysis.errors);
        assert_eq!(analysis.sha256_hex, KNOWN_FACTORY_BIN_SHA256);
        assert_eq!(analysis.initial_sp, 0x200015B0);
        assert_eq!(analysis.reset_vector, 0x000066D1);
        assert!(analysis.is_thumb_mode);
        assert!(analysis.has_rom_marker);
        assert_eq!(analysis.rom_marker_value, VALIDITY_MARKER_VALUE);
        assert!(analysis.contains_apple_vid_pid);
        assert!(analysis.contains_isp_trigger_magic);
        assert_eq!(analysis.single_color_flag, 0xAAAA);
        assert_eq!(analysis.populated_bytes, 56088);
        assert!(analysis.is_known_factory_image);
    }

    #[test]
    fn test_rejects_corrupted_marker() {
        let mut corrupt = vec![0xFFu8; FLASH_SIZE];
        // Set valid SP and reset vector
        corrupt[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        corrupt[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        // Leave marker as 0xFFFFFFFF
        let analysis = validate_firmware_image(&corrupt).expect("Validation should run");
        assert!(!analysis.is_safe_to_flash());
        assert!(analysis
            .errors
            .iter()
            .any(|e| e.contains("Flash Validity Signature")));
    }

    #[test]
    fn test_rejects_arm_mode_even_reset_vector() {
        let mut corrupt = vec![0xFFu8; FLASH_SIZE];
        corrupt[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        // Even reset vector (Thumb bit = 0)
        corrupt[4..8].copy_from_slice(&0x00001000u32.to_le_bytes());
        corrupt[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        let analysis = validate_firmware_image(&corrupt).expect("Validation should run");
        assert!(!analysis.is_safe_to_flash());
        assert!(analysis
            .errors
            .iter()
            .any(|e| e.contains("ARM Thumb mode bit")));
    }

    #[test]
    fn test_rejects_invalid_sp() {
        let mut corrupt = vec![0xFFu8; FLASH_SIZE];
        // Invalid SP pointing outside SRAM (e.g. 0x00000000)
        corrupt[0..4].copy_from_slice(&0x00000000u32.to_le_bytes());
        corrupt[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        corrupt[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        let analysis = validate_firmware_image(&corrupt).expect("Validation should run");
        assert!(!analysis.is_safe_to_flash());
        assert!(analysis.errors.iter().any(|e| e.contains("SRAM")));
    }
}
