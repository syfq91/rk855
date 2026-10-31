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

    pub fn slug(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::SingleOn => "single-on",
            Self::SingleOff => "single-off",
            Self::Glittering => "glittering",
            Self::Falling => "falling",
            Self::ColorCycling => "color-cycling",
            Self::Breathing => "breathing",
            Self::Spectrum => "spectrum",
            Self::Outward => "outward",
            Self::Scrolling => "scrolling",
            Self::Wave => "wave",
            Self::Accumulate => "accumulate",
            Self::Windmill => "windmill",
            Self::ColorfulWaterfall => "colorful-waterfall",
            Self::BlossomingRipples => "blossoming-ripples",
            Self::Flowing => "flowing",
            Self::ConstantRainbow => "constant-rainbow",
            Self::SineWave => "sine-wave",
            Self::Shadow => "shadow",
            Self::BacklightOff => "backlight-off",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Static => "Solid color backlight without animation",
            Self::SingleOn => "Keys light up when pressed, then turn off",
            Self::SingleOff => "Keys turn off when pressed, then turn on",
            Self::Glittering => "Glittering star effect across keys",
            Self::Falling => "Vertical falling light drops",
            Self::ColorCycling => "Cycles continuously through solid colors",
            Self::Breathing => "Smooth pulsing breath effect",
            Self::Spectrum => "Full keyboard color spectrum animation",
            Self::Outward => "Light ripples outward from center",
            Self::Scrolling => "Light waves scroll horizontally",
            Self::Wave => "Smooth continuous wave propagation",
            Self::Accumulate => "Falling blocks accumulate at bottom",
            Self::Windmill => "Rotating windmill / pinwheel light effect",
            Self::ColorfulWaterfall => "Multi-color vertical cascading waterfall",
            Self::BlossomingRipples => "Expanding ripple rings on key press",
            Self::Flowing => "Horizontal flowing rainbow stream",
            Self::ConstantRainbow => "Dynamic rotating full-spectrum rainbow",
            Self::SineWave => "Undulating sine wave light pattern",
            Self::Shadow => "Reactive fading trail behind keypresses",
            Self::BacklightOff => "Turn off all backlight LEDs",
        }
    }

    pub fn supports_color(self) -> bool {
        !matches!(
            self,
            Self::BacklightOff
                | Self::Spectrum
                | Self::ColorCycling
                | Self::ConstantRainbow
                | Self::Flowing
                | Self::ColorfulWaterfall
        )
    }

    pub fn supports_speed(self) -> bool {
        !matches!(self, Self::Static | Self::BacklightOff)
    }

    pub fn supports_direction(self) -> bool {
        matches!(
            self,
            Self::Scrolling | Self::Wave | Self::Windmill | Self::Flowing | Self::SineWave
        )
    }

    pub fn from_name_or_slug(s: &str) -> Option<Self> {
        let s = s.trim();
        if let Ok(num) = s.parse::<u8>() {
            return Self::from_u8(num);
        }
        if let Some(hex_str) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            if let Ok(num) = u8::from_str_radix(hex_str, 16) {
                return Self::from_u8(num);
            }
        }
        for mode in Self::ALL {
            if mode.slug().eq_ignore_ascii_case(s) || mode.name().eq_ignore_ascii_case(s) {
                return Some(mode);
            }
        }
        // Common abbreviations & aliases
        if s.eq_ignore_ascii_case("breath") {
            Some(Self::Breathing)
        } else if s.eq_ignore_ascii_case("off") {
            Some(Self::BacklightOff)
        } else if s.eq_ignore_ascii_case("ripple") || s.eq_ignore_ascii_case("ripples") {
            Some(Self::BlossomingRipples)
        } else if s.eq_ignore_ascii_case("rainbow") || s.eq_ignore_ascii_case("explode") {
            Some(Self::ConstantRainbow)
        } else if s.eq_ignore_ascii_case("waterfall") {
            Some(Self::ColorfulWaterfall)
        } else if s.eq_ignore_ascii_case("flow") {
            Some(Self::Flowing)
        } else if s.eq_ignore_ascii_case("scroll") {
            Some(Self::Scrolling)
        } else if s.eq_ignore_ascii_case("glitter") {
            Some(Self::Glittering)
        } else if s.eq_ignore_ascii_case("cycle") {
            Some(Self::ColorCycling)
        } else {
            None
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

    pub fn slug(self) -> &'static str {
        match self {
            Self::FixedSingleColor => "fixed",
            Self::RgbCustom => "custom",
            Self::SpectrumCycle => "spectrum",
            Self::DynamicRainbow => "rainbow",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("fixed") || s.eq_ignore_ascii_case("single") {
            Some(Self::FixedSingleColor)
        } else if s.eq_ignore_ascii_case("rgb") || s.eq_ignore_ascii_case("custom") {
            Some(Self::RgbCustom)
        } else if s.eq_ignore_ascii_case("spectrum") || s.eq_ignore_ascii_case("cycle") {
            Some(Self::SpectrumCycle)
        } else if s.eq_ignore_ascii_case("rainbow") || s.eq_ignore_ascii_case("dynamic") {
            Some(Self::DynamicRainbow)
        } else if let Ok(val) = s.parse::<u8>() {
            Self::from_u8(val)
        } else {
            None
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

    pub fn slug(self) -> &'static str {
        match self {
            Self::LeftToRight => "ltr",
            Self::RightToLeft => "rtl",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("ltr")
            || s.eq_ignore_ascii_case("left")
            || s.eq_ignore_ascii_case("left-to-right")
            || s.eq_ignore_ascii_case("clockwise")
            || s == "0"
        {
            Some(Self::LeftToRight)
        } else if s.eq_ignore_ascii_case("rtl")
            || s.eq_ignore_ascii_case("right")
            || s.eq_ignore_ascii_case("right-to-left")
            || s.eq_ignore_ascii_case("counter-clockwise")
            || s == "1"
        {
            Some(Self::RightToLeft)
        } else {
            None
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
