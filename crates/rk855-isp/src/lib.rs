//! Sonix SN32F240B ISP bootloader client and firmware image verification.
//!
//! Provides offline firmware analysis (`.bin` and `.hex` validation) to verify
//! vector tables, memory spans, checksums, and ROM descriptors without touching hardware.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Hex parsing error: {0}")]
    Hex(String),

    #[error("Invalid firmware image: {0}")]
    InvalidImage(String),
}

/// Analysis report of an analyzed firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareAnalysis {
    pub file_size: usize,
    pub sha256_hex: String,
    pub initial_sp: u32,
    pub reset_vector: u32,
    pub has_rom_marker: bool,
    pub rom_marker_value: u32,
    pub contains_apple_vid_pid: bool,
    pub contains_isp_trigger_magic: bool,
    pub populated_bytes: usize,
}

/// Analyzes a raw flat binary firmware file (`.bin`).
pub fn analyze_firmware_bin(path: &Path) -> Result<FirmwareAnalysis, IspError> {
    let data = fs::read(path)?;
    if data.len() < 16 {
        return Err(IspError::InvalidImage(
            "Image too small (< 16 bytes)".into(),
        ));
    }

    let initial_sp = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let reset_vector = u32::from_le_bytes(data[4..8].try_into().unwrap());

    // Check SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let sha256_hex = format!("{:x}", hasher.finalize());

    // Check validity marker at 0xFFFC
    let mut has_rom_marker = false;
    let mut rom_marker_value = 0;
    if data.len() >= 0x10000 {
        rom_marker_value = u32::from_le_bytes(data[0xFFFC..0x10000].try_into().unwrap());
        has_rom_marker = rom_marker_value == 0xAAAA5555;
    }

    // Check Apple VID:PID 05AC:024F (little endian: ac 05 4f 02)
    let apple_needle = [0xAC, 0x05, 0x4F, 0x02];
    let contains_apple_vid_pid = data.windows(4).any(|w| w == apple_needle);

    // Check ISP bootloader software trigger magic
    let contains_isp_trigger_magic = data
        .windows(rk855_proto::BOOTLOADER_TRIGGER_MAGIC.len())
        .any(|w| w == rk855_proto::BOOTLOADER_TRIGGER_MAGIC);

    let populated_bytes = data.iter().filter(|&&b| b != 0xFF).count();

    Ok(FirmwareAnalysis {
        file_size: data.len(),
        sha256_hex,
        initial_sp,
        reset_vector,
        has_rom_marker,
        rom_marker_value,
        contains_apple_vid_pid,
        contains_isp_trigger_magic,
        populated_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_firmware_bin() {
        let fw_path = Path::new("../../firmware/SN32F240B_32.bin");
        if fw_path.exists() {
            let analysis = analyze_firmware_bin(fw_path).expect("Failed to analyze firmware");
            assert_eq!(analysis.file_size, 65536);
            assert_eq!(analysis.initial_sp, 0x200015B0);
            assert_eq!(analysis.reset_vector, 0x000066D1);
            assert!(analysis.has_rom_marker);
            assert_eq!(analysis.rom_marker_value, 0xAAAA5555);
            assert!(analysis.contains_apple_vid_pid);
            assert!(analysis.contains_isp_trigger_magic);
        }
    }
}
