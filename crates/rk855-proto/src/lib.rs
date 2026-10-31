//! Pure, no-I/O protocol primitives, constants, packet builders, and parsers
//! for the Royal Kludge RK855 (RK68) mechanical keyboard (SONiX SN32F240B).

#![no_std]

// --- Known Identifiers ---

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

/// Pair list of all known candidate USB IDs for keyboard runtime interfaces
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

/// Default inter-packet pacing delay in milliseconds (measured from captures / decompilation)
pub const DEFAULT_INTER_PACKET_DELAY_MS: u64 = 5;

/// Magic sequence for software-triggered jump to Mask ROM bootloader (8 bytes)
pub const BOOTLOADER_TRIGGER_MAGIC: [u8; 8] = [0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC];

// --- Opcode Constants ---

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

/// Fixed-size 64-byte command payload
pub type Packet = [u8; PAYLOAD_SIZE];

/// Fixed-size 65-byte wire report buffer (byte 0 is Report ID 0x00)
pub type WireReport = [u8; WIRE_REPORT_SIZE];

/// Converts a 64-byte payload into a 65-byte wire report with Report ID 0x00.
pub fn to_wire_report(payload: &Packet) -> WireReport {
    let mut wire = [0u8; WIRE_REPORT_SIZE];
    wire[0] = FEATURE_REPORT_ID;
    wire[1..].copy_from_slice(payload);
    wire
}

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LightMode {
    Static = 0x00,
    SingleOn = 0x01,
    SingleOff = 0x02,
    Glittering = 0x03,
    Falling = 0x04,
    ColorCycling = 0x05,
    Breathing = 0x06,
    Spectrum = 0x07,
    Outward = 0x08,
    Scrolling = 0x09,
    Wave = 0x0A,
    Accumulate = 0x0B,
    Windmill = 0x0C,
    ColorfulWaterfall = 0x0D,
    BlossomingRipples = 0x0E,
    Flowing = 0x0F,
    ConstantRainbow = 0x10,
    SineWave = 0x11,
    Shadow = 0x12,
    BacklightOff = 0x13,
}

impl LightMode {
    pub fn from_u8(val: u8) -> Option<Self> {
        if val <= 0x13 {
            Some(unsafe { core::mem::transmute::<u8, LightMode>(val) })
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::SingleOn => "Single On",
            Self::SingleOff => "Single Off",
            Self::Glittering => "Glittering",
            Self::Falling => "Falling",
            Self::ColorCycling => "Color Cycling",
            Self::Breathing => "Breathing",
            Self::Spectrum => "Spectrum",
            Self::Outward => "Outward",
            Self::Scrolling => "Scrolling",
            Self::Wave => "Wave",
            Self::Accumulate => "Accumulate",
            Self::Windmill => "Windmill",
            Self::ColorfulWaterfall => "Colorful Waterfall",
            Self::BlossomingRipples => "Blossoming / Ripples",
            Self::Flowing => "Flowing",
            Self::ConstantRainbow => "Constant Rainbow",
            Self::SineWave => "Sine Wave",
            Self::Shadow => "Shadow",
            Self::BacklightOff => "Backlight Off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorType {
    FixedSingleColor = 0x00,
    RgbCustom = 0x01,
    SpectrumCycle = 0x07,
    DynamicRainbow = 0x08,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    LeftToRight = 0x00,
    RightToLeft = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProfileSlot {
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// --- Packet Builders ---

/// Builds packet to begin lighting configuration (`04 18`)
pub fn build_begin_light() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_BEGIN_LIGHT;
    pkt
}

/// Builds packet to set lighting parameters (`04 11`)
pub fn build_set_light(params: &LightParams) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_SET_LIGHT;
    pkt[2] = params.mode as u8;
    pkt[3] = params.color_type as u8;
    pkt[4] = params.r;
    pkt[5] = params.g;
    pkt[6] = params.b;
    pkt[7] = params.brightness.min(4);
    pkt[8] = params.speed;
    pkt[9] = params.direction as u8;
    pkt
}

/// Builds packet to commit configuration to flash (`04 02`)
pub fn build_commit() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_COMMIT;
    pkt
}

/// Builds packet to reload PWM controller immediately (`04 F0`)
pub fn build_pwm_reload() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_PWM_RELOAD;
    pkt
}

/// Builds packet to switch onboard profile slot (`04 F1`, `04 F2`, `04 F3`)
pub fn build_switch_profile(slot: ProfileSlot) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_PROFILE_BASE + (slot as u8);
    pkt
}

/// Builds packet to initiate software ISP bootloader jump
pub fn build_isp_trigger() -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[..8].copy_from_slice(&BOOTLOADER_TRIGGER_MAGIC);
    pkt
}

/// Builds header packet for custom per-key RGB (`04 13 ... 0x12`)
pub fn build_custom_rgb_header(total_blocks: u8) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_CUSTOM_RGB_HEADER;
    pkt[8] = total_blocks;
    pkt
}

/// Builds header packet for key remapping (`04 20 ... 0x04`)
pub fn build_begin_remap(block_count: u8) -> Packet {
    let mut pkt = [0u8; PAYLOAD_SIZE];
    pkt[0] = CMD_OPCODE_HEADER;
    pkt[1] = SUBCMD_BEGIN_REMAP;
    pkt[8] = block_count;
    pkt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_light_packet() {
        let pkt = build_begin_light();
        assert_eq!(pkt[0], 0x04);
        assert_eq!(pkt[1], 0x18);
        assert_eq!(&pkt[2..], &[0u8; 62]);
    }

    #[test]
    fn test_set_light_packet() {
        let params = LightParams {
            mode: LightMode::Breathing,
            color_type: ColorType::RgbCustom,
            r: 0xFF,
            g: 0x00,
            b: 0xAA,
            brightness: 3,
            speed: 2,
            direction: Direction::LeftToRight,
        };
        let pkt = build_set_light(&params);
        assert_eq!(pkt[0], 0x04);
        assert_eq!(pkt[1], 0x11);
        assert_eq!(pkt[2], 0x06); // Breathing
        assert_eq!(pkt[3], 0x01); // Custom RGB
        assert_eq!(pkt[4], 0xFF); // R
        assert_eq!(pkt[5], 0x00); // G
        assert_eq!(pkt[6], 0xAA); // B
        assert_eq!(pkt[7], 0x03); // Brightness
        assert_eq!(pkt[8], 0x02); // Speed
        assert_eq!(pkt[9], 0x00); // Direction
    }

    #[test]
    fn test_wire_report() {
        let pkt = build_commit();
        let wire = to_wire_report(&pkt);
        assert_eq!(wire[0], 0x00); // Report ID prefix
        assert_eq!(wire[1], 0x04);
        assert_eq!(wire[2], 0x02);
        assert_eq!(wire.len(), 65);
    }

    #[test]
    fn test_isp_trigger_magic() {
        let pkt = build_isp_trigger();
        assert_eq!(&pkt[..8], &[0xAA, 0x42, 0x89, 0x5A, 0xFF, 0x71, 0x62, 0xCC]);
    }
}
