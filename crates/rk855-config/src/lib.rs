//! High-level configuration management for RK855 (lighting, remaps, macros, profiles).

use rk855_hid::{HidTransport, HidTransportError};
use rk855_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("HID transport error: {0}")]
    Transport(#[from] HidTransportError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid profile format: {0}")]
    InvalidProfile(String),

    #[error("Key resolution error: {0}")]
    KeyResolution(String),
}

/// Profile representation serializable to human-readable TOML files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileFile {
    pub name: String,
    pub version: u32,
    pub slot: Option<u8>,
    pub lighting: Option<ProfileLighting>,
    #[serde(default)]
    pub remaps: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_rgb: Option<BTreeMap<String, String>>,
}

impl Default for ProfileFile {
    fn default() -> Self {
        Self {
            name: "Default RK855 Profile".to_string(),
            version: 1,
            slot: Some(1),
            lighting: Some(ProfileLighting::default()),
            remaps: BTreeMap::new(),
            custom_rgb: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileLighting {
    pub mode: String,
    pub color: String,
    pub brightness: u8,
    pub speed: u8,
    pub direction: String,
}

impl Default for ProfileLighting {
    fn default() -> Self {
        Self {
            mode: "breathing".to_string(),
            color: "#FF00FF".to_string(),
            brightness: 3,
            speed: 2,
            direction: "ltr".to_string(),
        }
    }
}

/// Configuration manager that operates via any `HidTransport`.
pub struct ConfigManager<T: HidTransport> {
    transport: T,
}

impl<T: HidTransport> ConfigManager<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Sets lighting mode using the verified atomic transaction sequence.
    ///
    /// If `persist` is true: sends the full 22-packet transaction including flash commit (`04 02`)
    /// and PWM reload (`04 F0`), persisting across power cycles and reconnects.
    ///
    /// If `persist` is false (`--live`): omits the `04 02` commit packet, changing
    /// keyboard lighting dynamically in RAM without flash wear.
    pub fn apply_lighting(
        &mut self,
        params: &LightParams,
        persist: bool,
    ) -> Result<(), ConfigError> {
        self.apply_lighting_transaction(params, persist)
    }

    /// Sends the 22-packet transaction (or 21 packets if `persist == false`).
    pub fn apply_lighting_transaction(
        &mut self,
        params: &LightParams,
        persist: bool,
    ) -> Result<(), ConfigError> {
        let pkts = build_lighting_transaction(params);
        for (idx, pkt) in pkts.into_iter().enumerate() {
            // Packet 21 (index 20) is 04 02 Commit Flash
            if !persist && idx == 20 && pkt[0] == CMD_OPCODE_HEADER && pkt[1] == SUBCMD_COMMIT {
                tracing::debug!("Live mode: omitting flash commit packet (04 02)");
                continue;
            }
            self.transport.set_feature(&pkt)?;
        }
        Ok(())
    }

    /// Applies lighting using the lightweight direct sequence (`04 18`, `04 11`, `04 02`?, `04 F0`).
    pub fn apply_lighting_direct(
        &mut self,
        params: &LightParams,
        persist: bool,
    ) -> Result<(), ConfigError> {
        self.transport.set_feature(&build_begin_light())?;
        self.transport.set_feature(&build_set_light(params))?;
        if persist {
            self.transport.set_feature(&build_commit())?;
        }
        self.transport.set_feature(&build_pwm_reload())?;
        Ok(())
    }

    /// Applies key remapping table transaction (7 packets: begin, 4 blocks table, commit, reload).
    ///
    /// If `persist` is false (`--live`), the `04 02` commit packet is omitted.
    pub fn apply_remap(
        &mut self,
        entries: &[RemapEntry],
        persist: bool,
    ) -> Result<(), ConfigError> {
        let pkts = build_remap_transaction(entries);
        for (idx, pkt) in pkts.into_iter().enumerate() {
            // Packet 6 (index 5) is 04 02 Commit Flash
            if !persist && idx == 5 && pkt[0] == CMD_OPCODE_HEADER && pkt[1] == SUBCMD_COMMIT {
                tracing::debug!("Live mode: omitting remap commit packet (04 02)");
                continue;
            }
            self.transport.set_feature(&pkt)?;
        }
        Ok(())
    }

    /// Resets all key remappings to factory defaults (sends empty 256-byte table).
    pub fn reset_remap(&mut self, persist: bool) -> Result<(), ConfigError> {
        self.apply_remap(&[], persist)
    }

    /// Applies custom per-key RGB lighting to hardware.
    ///
    /// If `persist` is true: sends 22 packets including `04 02` commit.
    /// If `persist` is false (`--live`): sends 21 packets omitting flash commit.
    pub fn apply_custom_rgb(
        &mut self,
        rgb_map: &CustomRgbMap,
        persist: bool,
    ) -> Result<(), ConfigError> {
        let pkts = build_custom_rgb_transaction(rgb_map, persist);
        for pkt in pkts {
            self.transport.set_feature(&pkt)?;
        }
        Ok(())
    }

    /// Switches the active onboard profile (Profile 1, 2, or 3).
    pub fn switch_profile(&mut self, slot: ProfileSlot) -> Result<(), ConfigError> {
        self.transport.set_feature(&build_switch_profile(slot))?;
        self.transport.set_feature(&build_pwm_reload())?;
        Ok(())
    }

    /// Queries the keyboard MCU status / matrix readout (`04 F5`).
    pub fn query_status_matrix(&mut self) -> Result<Vec<u8>, ConfigError> {
        let mut full_data = Vec::new();

        // Initial query: 04 F5 01
        self.transport.set_feature(&build_read_back_request(1))?;
        let chunk1 = self
            .transport
            .get_feature(FEATURE_REPORT_ID, PAYLOAD_SIZE)?;
        full_data.extend_from_slice(&chunk1);

        // Follow-up queries: 3x 04 F5 00
        for _ in 0..3 {
            self.transport.set_feature(&build_read_back_request(0))?;
            let chunk = self
                .transport
                .get_feature(FEATURE_REPORT_ID, PAYLOAD_SIZE)?;
            full_data.extend_from_slice(&chunk);
        }

        Ok(full_data)
    }

    /// Exports a ProfileFile to a TOML formatted string.
    pub fn export_profile_toml(&self, profile: &ProfileFile) -> Result<String, ConfigError> {
        toml::to_string_pretty(profile).map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Imports a ProfileFile from a TOML formatted string and optionally stages it to hardware.
    pub fn import_profile_toml(
        &mut self,
        toml_str: &str,
        apply_to_device: bool,
        persist: bool,
    ) -> Result<ProfileFile, ConfigError> {
        let profile: ProfileFile =
            toml::from_str(toml_str).map_err(|e| ConfigError::InvalidProfile(e.to_string()))?;

        if profile.version != 1 {
            return Err(ConfigError::InvalidProfile(format!(
                "Unsupported profile schema version: {}. Only version 1 is supported.",
                profile.version
            )));
        }

        if apply_to_device {
            // Apply profile slot if specified
            if let Some(slot_num) = profile.slot {
                if let Some(slot) = ProfileSlot::from_u8(slot_num) {
                    self.switch_profile(slot)?;
                }
            }

            // Apply remaps
            let mut entries = Vec::new();
            for (key_name, target_name) in &profile.remaps {
                let phys_key = find_key_by_name(key_name).ok_or_else(|| {
                    ConfigError::KeyResolution(format!("Unknown physical key: '{}'", key_name))
                })?;
                let target_code = resolve_action_code(target_name).ok_or_else(|| {
                    ConfigError::KeyResolution(format!(
                        "Unknown target action / key: '{}'",
                        target_name
                    ))
                })?;
                entries.push(RemapEntry::new(phys_key.scancode, target_code));
            }
            if !entries.is_empty() {
                self.apply_remap(&entries, persist)?;
            }

            // Apply custom RGB if specified
            if let Some(ref rgb_colors) = profile.custom_rgb {
                let mut rgb_map = CustomRgbMap::new();
                for (key_name, hex_color) in rgb_colors {
                    let (r, g, b) = parse_rgb_hex(hex_color)?;
                    rgb_map.set_key_by_name(key_name, r, g, b).map_err(|_| {
                        ConfigError::KeyResolution(format!(
                            "Unknown key name '{}' in profile custom_rgb",
                            key_name
                        ))
                    })?;
                }
                self.apply_custom_rgb(&rgb_map, persist)?;
            }
        }

        Ok(profile)
    }
}

/// Parses hex color string like "FF0000", "#FF0000", "0x00FF00" into (R, G, B) tuple.
pub fn parse_rgb_hex(color_str: &str) -> Result<(u8, u8, u8), ConfigError> {
    let clean = color_str
        .trim()
        .trim_start_matches('#')
        .trim_start_matches("0x")
        .trim_start_matches("0X");

    if clean.len() != 6 {
        return Err(ConfigError::InvalidProfile(format!(
            "Invalid hex color '{}': expected 6 hex digits (e.g. 'FF0000' or '#FF0000')",
            color_str
        )));
    }

    let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| {
        ConfigError::InvalidProfile(format!("Invalid red channel in hex color '{}'", color_str))
    })?;
    let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| {
        ConfigError::InvalidProfile(format!(
            "Invalid green channel in hex color '{}'",
            color_str
        ))
    })?;
    let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| {
        ConfigError::InvalidProfile(format!("Invalid blue channel in hex color '{}'", color_str))
    })?;

    Ok((r, g, b))
}

/// Parses a JSON map of `{"key": "color"}` into a [`CustomRgbMap`].
pub fn parse_rgb_json(json_str: &str) -> Result<CustomRgbMap, ConfigError> {
    let map: BTreeMap<String, String> = serde_json::from_str(json_str)
        .map_err(|e| ConfigError::Serialization(format!("Failed to parse RGB JSON map: {e}")))?;

    let mut rgb_map = CustomRgbMap::new();
    for (key_name, hex_color) in map {
        let (r, g, b) = parse_rgb_hex(&hex_color)?;
        rgb_map.set_key_by_name(&key_name, r, g, b).map_err(|_| {
            ConfigError::KeyResolution(format!("Unknown key name '{key_name}' in RGB map"))
        })?;
    }

    Ok(rgb_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk855_hid::MockTransport;

    #[test]
    fn test_apply_lighting_transaction_persist() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let params = LightParams::default();

        mgr.apply_lighting(&params, true)
            .expect("Failed to apply lighting");
        assert_eq!(mgr.transport.sent_packets.len(), 22);
        assert_eq!(mgr.transport.sent_packets[0][0], 0x04);
        assert_eq!(mgr.transport.sent_packets[0][1], 0x18);
        assert_eq!(mgr.transport.sent_packets[1][1], 0x13);
        assert_eq!(mgr.transport.sent_packets[20][1], 0x02); // Commit
        assert_eq!(mgr.transport.sent_packets[21][1], 0xF0); // PWM Reload
    }

    #[test]
    fn test_apply_lighting_transaction_live() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let params = LightParams::default();

        mgr.apply_lighting(&params, false)
            .expect("Failed to apply lighting");
        // Omitted commit packet -> exactly 21 packets
        assert_eq!(mgr.transport.sent_packets.len(), 21);
        assert_eq!(mgr.transport.sent_packets[0][1], 0x18);
        assert_eq!(mgr.transport.sent_packets[1][1], 0x13);
        assert_eq!(mgr.transport.sent_packets[20][1], 0xF0); // PWM Reload (no 04 02)
    }

    #[test]
    fn test_apply_lighting_direct() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let params = LightParams::default();

        mgr.apply_lighting_direct(&params, true)
            .expect("Failed direct");
        assert_eq!(mgr.transport.sent_packets.len(), 4);
        assert_eq!(mgr.transport.sent_packets[0][1], 0x18);
        assert_eq!(mgr.transport.sent_packets[1][1], 0x11);
        assert_eq!(mgr.transport.sent_packets[2][1], 0x02);
        assert_eq!(mgr.transport.sent_packets[3][1], 0xF0);

        let mut mgr_live = ConfigManager::new(MockTransport::new());
        mgr_live
            .apply_lighting_direct(&params, false)
            .expect("Failed direct live");
        assert_eq!(mgr_live.transport.sent_packets.len(), 3);
        assert_eq!(mgr_live.transport.sent_packets[2][1], 0xF0);
    }

    #[test]
    fn test_apply_remap_persist() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let remaps = [RemapEntry::new(0x39, 0x29)]; // Caps -> Esc

        mgr.apply_remap(&remaps, true).expect("Failed remap");
        assert_eq!(mgr.transport.sent_packets.len(), 7);
        assert_eq!(mgr.transport.sent_packets[0][0], 0x04);
        assert_eq!(mgr.transport.sent_packets[0][1], 0x20); // Begin remap
        assert_eq!(mgr.transport.sent_packets[0][8], 0x04); // 4 blocks
        assert_eq!(mgr.transport.sent_packets[1][0], 0x39); // Phys: Caps
        assert_eq!(mgr.transport.sent_packets[1][1], 0x29); // Target: Esc
        assert_eq!(mgr.transport.sent_packets[5][1], 0x02); // Commit
        assert_eq!(mgr.transport.sent_packets[6][1], 0xF0); // Reload
    }

    #[test]
    fn test_reset_remap() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);

        mgr.reset_remap(true).expect("Failed reset");
        assert_eq!(mgr.transport.sent_packets.len(), 7);
        assert_eq!(mgr.transport.sent_packets[1], vec![0u8; 64]);
        assert_eq!(mgr.transport.sent_packets[5][1], 0x02);
    }

    #[test]
    fn test_switch_profile() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);

        mgr.switch_profile(ProfileSlot::Slot2)
            .expect("Failed slot2");
        assert_eq!(mgr.transport.sent_packets.len(), 2);
        assert_eq!(mgr.transport.sent_packets[0][0], 0x04);
        assert_eq!(mgr.transport.sent_packets[0][1], 0xF2); // Profile 2
        assert_eq!(mgr.transport.sent_packets[1][1], 0xF0); // Reload
    }

    #[test]
    fn test_profile_toml_roundtrip() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);

        let mut remaps = BTreeMap::new();
        remaps.insert("capsLock".to_string(), "Escape".to_string());
        remaps.insert("esc".to_string(), "CapsLock".to_string());

        let profile = ProfileFile {
            name: "Test Profile".to_string(),
            version: 1,
            slot: Some(2),
            lighting: Some(ProfileLighting {
                mode: "breathing".to_string(),
                color: "#FF00FF".to_string(),
                brightness: 3,
                speed: 2,
                direction: "ltr".to_string(),
            }),
            remaps,
            custom_rgb: None,
        };

        let toml_str = mgr.export_profile_toml(&profile).expect("Failed export");
        assert!(toml_str.contains("version = 1"));
        assert!(toml_str.contains("slot = 2"));
        assert!(toml_str.contains("capsLock"));

        let imported = mgr
            .import_profile_toml(&toml_str, true, true)
            .expect("Failed import");
        assert_eq!(imported.name, "Test Profile");
        assert_eq!(imported.version, 1);
        assert_eq!(imported.slot, Some(2));
        assert_eq!(imported.remaps.len(), 2);
    }

    #[test]
    fn test_custom_rgb_application() {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);

        let mut rgb_map = CustomRgbMap::new();
        rgb_map.set_key_by_name("ESC", 255, 0, 0).expect("esc set");
        rgb_map.set_key_by_name("W", 0, 255, 0).expect("w set");

        mgr.apply_custom_rgb(&rgb_map, true)
            .expect("Failed custom rgb");
        assert_eq!(mgr.transport.sent_packets.len(), 22);
        assert_eq!(mgr.transport.sent_packets[0][0], 0x04);
        assert_eq!(mgr.transport.sent_packets[0][1], 0x18);
        assert_eq!(mgr.transport.sent_packets[1][1], 0x13);
        assert_eq!(mgr.transport.sent_packets[20][1], 0x02); // Commit
        assert_eq!(mgr.transport.sent_packets[21][1], 0xF0); // Reload
    }

    #[test]
    fn test_parse_rgb_json() {
        let json_data = r##"{"esc": "#FF0000", "space": "0000FF", "caps": "0x00FF00"}"##;
        let map = parse_rgb_json(json_data).expect("Failed json parse");
        let esc_key = find_key_by_name("esc").unwrap();
        let space_key = find_key_by_name("space").unwrap();
        let caps_key = find_key_by_name("caps").unwrap();

        assert_eq!(map.colors[esc_key.index], Some((255, 0, 0)));
        assert_eq!(map.colors[space_key.index], Some((0, 0, 255)));
        assert_eq!(map.colors[caps_key.index], Some((0, 255, 0)));
    }
}
