//! Error types for ISP bootloader communication and firmware validation.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HEX parsing error: {0}")]
    Hex(String),

    #[error("Invalid firmware image: {0}")]
    InvalidImage(String),

    #[error("Preflight validation failed: {0}")]
    PreflightFailed(String),

    #[error("HID transport error: {0}")]
    Hid(#[from] rk855_hid::HidTransportError),

    #[error("Protocol error: {0}")]
    Proto(#[from] rk855_proto::ProtoError),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Timed out waiting for bootloader re-enumeration ({0:?})")]
    BootloaderTimeout(std::time::Duration),

    #[error("ISP handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("Flash erase failed: {0}")]
    EraseFailed(String),

    #[error("Writing page {page} at address 0x{address:04X} failed: {reason}")]
    WriteFailed {
        page: usize,
        address: u32,
        reason: String,
    },

    #[error(
        "Verification mismatch at address 0x{address:04X}! Expected: {expected_hex}, Actual: {actual_hex}"
    )]
    VerifyMismatch {
        address: u32,
        expected_hex: String,
        actual_hex: String,
    },

    #[error("Code security active: flash readout blocked by microcontroller fuse")]
    CodeSecurityBlocked,

    #[error("Flash operation cancelled by user")]
    UserAborted,

    #[error("Unexpected response from bootloader: {0}")]
    UnexpectedResponse(String),
}
