//! Sonix SN32F240B In-System Programming (ISP) flasher and firmware verification library.
//!
//! Provides:
//! - Firmware parsing (`.hex` and `.bin`) and pre-flight safety analysis (`image`).
//! - Low-level wire protocol framing and opcodes for Sonix Cortex-M0 ISP (`protocol`).
//! - Hardware-agnostic mock bootloader transport for unit tests (`mock`).
//! - Guarded high-level client for flashing, reading, erasing, and verifying flash (`client`).

pub mod client;
pub mod error;
pub mod image;
pub mod mock;
pub mod protocol;

pub use client::{FlashOptions, FlashSummary, IspClient};
pub use error::IspError;
pub use image::{
    analyze_firmware, compute_sha256, load_firmware, parse_intel_hex, validate_firmware_image,
    FirmwareAnalysis, FLASH_SIZE, KNOWN_FACTORY_BIN_SHA256, KNOWN_FACTORY_HEX_SHA256,
    VALIDITY_MARKER_ADDR, VALIDITY_MARKER_VALUE,
};
pub use mock::MockIspTransport;
pub use protocol::{
    BootloaderInfo, BOOTLOADER_PID, BOOTLOADER_VID, CHIP_ID_SN32F240B, CHIP_ID_SN32F248B,
    PAGE_COUNT, PAGE_SIZE, REPORT_ID,
};
