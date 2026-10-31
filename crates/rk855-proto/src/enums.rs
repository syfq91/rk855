//! Enumerations for lighting modes, colors, directions, profiles, and remap categories.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub const ALL: [LightMode; 20] = [
        Self::Static,
        Self::SingleOn,
        Self::SingleOff,
        Self::Glittering,
        Self::Falling,
        Self::ColorCycling,
        Self::Breathing,
        Self::Spectrum,
        Self::Outward,
        Self::Scrolling,
        Self::Wave,
        Self::Accumulate,
        Self::Windmill,
        Self::ColorfulWaterfall,
        Self::BlossomingRipples,
        Self::Flowing,
        Self::ConstantRainbow,
        Self::SineWave,
        Self::Shadow,
        Self::BacklightOff,
    ];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ColorType {
    FixedSingleColor = 0x00,
    RgbCustom = 0x01,
    SpectrumCycle = 0x07,
    DynamicRainbow = 0x08,
}

impl ColorType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::FixedSingleColor),
            0x01 => Some(Self::RgbCustom),
            0x07 => Some(Self::SpectrumCycle),
            0x08 => Some(Self::DynamicRainbow),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Direction {
    LeftToRight = 0x00,
    RightToLeft = 0x01,
}

impl Direction {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x00 => Some(Self::LeftToRight),
            0x01 => Some(Self::RightToLeft),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ProfileSlot {
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
}

impl ProfileSlot {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Slot1),
            2 => Some(Self::Slot2),
            3 => Some(Self::Slot3),
            _ => None,
        }
    }
}

/// Remap categories defined in the vendor firmware and software architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum RemapCategory {
    StandardKey = 120,
    Macro = 121,
    Media = 122,
    Mouse = 123,
    Shortcut = 124,
    ProfileSwitch = 125,
    LaunchApp = 126,
    OpenUrl = 127,
    StringText = 128,
    Disabled = 129,
    Special = 130,
}

impl RemapCategory {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            120 => Some(Self::StandardKey),
            121 => Some(Self::Macro),
            122 => Some(Self::Media),
            123 => Some(Self::Mouse),
            124 => Some(Self::Shortcut),
            125 => Some(Self::ProfileSwitch),
            126 => Some(Self::LaunchApp),
            127 => Some(Self::OpenUrl),
            128 => Some(Self::StringText),
            129 => Some(Self::Disabled),
            130 => Some(Self::Special),
            _ => None,
        }
    }

    /// Returns true if this action is executed entirely within the keyboard MCU firmware.
    pub fn is_hardware_handled(self) -> bool {
        matches!(
            self,
            Self::StandardKey
                | Self::Macro
                | Self::Media
                | Self::Mouse
                | Self::Shortcut
                | Self::ProfileSwitch
                | Self::Disabled
        )
    }

    /// Returns true if this action requires host-side software daemon handling.
    pub fn is_host_handled(self) -> bool {
        matches!(self, Self::LaunchApp | Self::OpenUrl | Self::StringText)
    }
}
