//! Protocol constants for Royal Kludge RK855 (SONiX SN32F240B).

/// Vendor Windows configurator profile VID (0x0C45: Sonix)
pub const VENDOR_VID: u16 = 0x0C45;
/// Vendor Windows configurator profile PID
pub const VENDOR_PID: u16 = 0x8018;

/// Factory ROM Apple profile VID (0x05AC: Apple Inc.)
pub const FACTORY_ROM_VID: u16 = 0x05AC;
/// Factory ROM Apple profile PID
pub const FACTORY_ROM_PID: u16 = 0x024F;

/// Sonix Cortex-M0 ROM Bootloader VID
pub const BOOTLOADER_VID: u16 = 0x0C45;
/// Sonix Cortex-M0 ROM Bootloader PID
pub const BOOTLOADER_PID: u16 = 0x82AA;

/// Special PID 0 listed in UISettings_248B.ini
pub const SPECIAL_PID_0: u16 = 0x8016;
/// Special PID 2 listed in UISettings_248B.ini
pub const SPECIAL_PID_2: u16 = 0x8008;

/// Candidate USB IDs for keyboard runtime interfaces
pub const CANDIDATE_KEYBOARD_IDS: &[(u16, u16)] = &[
    (VENDOR_VID, VENDOR_PID),
    (FACTORY_ROM_VID, FACTORY_ROM_PID),
    (VENDOR_VID, SPECIAL_PID_0),
    (VENDOR_VID, SPECIAL_PID_2),
];

// --- Protocol Geometry & Timing ---

/// Payload size inside Feature Reports (64 bytes)
pub const PAYLOAD_SIZE: usize = 64;

/// Wire length of HID Feature Report including the 1-byte Report ID prefix (65 bytes)
pub const WIRE_REPORT_SIZE: usize = 65;

/// Report ID for the Interface 0 Feature Report
pub const FEATURE_REPORT_ID: u8 = 0x00;

/// Default inter-packet pacing delay in milliseconds (measured from captures: ~10.3 to 10.9 ms)
pub const DEFAULT_INTER_PACKET_DELAY_MS: u64 = 10;

/// Magic sequence for software-triggered jump to Mask ROM bootloader (8 bytes)
pub const BOOTLOADER_TRIGGER_MAGIC: [u8; 8] = [0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC];

/// Trailer magic pattern for lighting and macro data blocks (2 bytes: 0xAA, 0x55)
pub const BLOCK_TRAILER_MAGIC: [u8; 2] = [0xAA, 0x55];

/// Size of the key remapping table in bytes (256 bytes)
pub const REMAP_TABLE_SIZE: usize = 256;

/// Total number of 64-byte blocks in a full key remap table transfer
pub const REMAP_BLOCK_COUNT: u8 = 4;

/// Total number of data blocks declared in Custom RGB / Lighting Header (18 blocks)
pub const LIGHTING_BLOCK_COUNT: u8 = 18;

// --- Primary Opcode Constants ---

pub const CMD_OPCODE_HEADER: u8 = 0x04;

pub const SUBCMD_BEGIN_LIGHT: u8 = 0x18;
pub const SUBCMD_SET_LIGHT: u8 = 0x11;
pub const SUBCMD_CUSTOM_RGB_HEADER: u8 = 0x13;
pub const SUBCMD_BEGIN_REMAP: u8 = 0x20;
pub const SUBCMD_BEGIN_MACRO: u8 = 0x19;
pub const SUBCMD_MACRO_DATA: u8 = 0x15;
pub const SUBCMD_COMMIT: u8 = 0x02;
pub const SUBCMD_PWM_RELOAD: u8 = 0xF0;
pub const SUBCMD_PROFILE_BASE: u8 = 0xF0; // 0xF1=Slot 1, 0xF2=Slot 2, 0xF3=Slot 3
pub const SUBCMD_READ_BACK: u8 = 0xF5;
