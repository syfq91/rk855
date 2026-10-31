use crate::state::{AppState, BackendMode};
use egui::Ui;
use rk855_config::{ConfigManager, ProfileFile, ProfileLighting};
use rk855_hid::{HidApiTransport, MockTransport};
use rk855_proto::enums::ProfileSlot;

pub fn render(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Hardware Profiles Management");
    ui.add_space(8.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label("The SONiX SN32F240B flash stores 3 independent hardware profile slots:");
        ui.label("• Profile 1: Flash offset 0x9000");
        ui.label("• Profile 2: Flash offset 0x90C0");
        ui.label("• Profile 3: Flash offset 0x9340");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Active Profile:");
            for p in 1..=3 {
                let is_active = state.active_profile == p;
                if ui
                    .selectable_label(is_active, format!("Profile {}", p))
                    .clicked()
                {
                    switch_profile(state, p);
                }
            }
        });
    });

    ui.add_space(16.0);
    ui.heading("Profile Backup & Restore (TOML / JSON)");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("📁 Load Profile from Disk").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Configuration", &["toml", "json"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let parsed: Result<ProfileFile, _> =
                            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                                serde_json::from_str(&content).map_err(|e| e.to_string())
                            } else {
                                toml::from_str(&content).map_err(|e| e.to_string())
                            };

                        match parsed {
                            Ok(profile) => {
                                if let Some(lighting) = profile.lighting {
                                    state.brightness = lighting.brightness;
                                    state.speed = lighting.speed;
                                }
                                state.set_status(
                                    format!(
                                        "Loaded profile '{}' from {:?}",
                                        profile.name,
                                        path.file_name().unwrap_or_default()
                                    ),
                                    false,
                                );
                            }
                            Err(e) => {
                                state.set_status(format!("Failed to parse profile: {}", e), true)
                            }
                        }
                    }
                    Err(e) => state.set_status(format!("Failed to read file: {}", e), true),
                }
            }
        }

        if ui.button("💾 Export Current Profile").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("TOML", &["toml"])
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let current_profile = ProfileFile {
                    name: format!("Profile {}", state.active_profile),
                    version: 1,
                    slot: Some(state.active_profile),
                    lighting: Some(ProfileLighting {
                        mode: state.light_mode_name().to_string(),
                        color: format!(
                            "#{:02X}{:02X}{:02X}",
                            state.custom_color[0], state.custom_color[1], state.custom_color[2]
                        ),
                        brightness: state.brightness,
                        speed: state.speed,
                        direction: if state.direction == 0 {
                            "ltr".to_string()
                        } else {
                            "rtl".to_string()
                        },
                    }),
                    remaps: std::collections::BTreeMap::new(),
                    custom_rgb: None,
                };

                let output_res = if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    serde_json::to_string_pretty(&current_profile).map_err(|e| e.to_string())
                } else {
                    toml::to_string_pretty(&current_profile).map_err(|e| e.to_string())
                };

                match output_res {
                    Ok(data) => {
                        let _ = std::fs::write(&path, data);
                        state.set_status(
                            format!(
                                "Exported profile to {:?}",
                                path.file_name().unwrap_or_default()
                            ),
                            false,
                        );
                    }
                    Err(e) => state.set_status(format!("Serialization error: {}", e), true),
                }
            }
        }
    });
}

fn switch_profile(state: &mut AppState, profile: u8) {
    state.active_profile = profile;
    let slot = match profile {
        1 => ProfileSlot::Slot1,
        2 => ProfileSlot::Slot2,
        3 => ProfileSlot::Slot3,
        _ => ProfileSlot::Slot1,
    };

    if state.backend_mode == BackendMode::Mock {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let _ = mgr.switch_profile(slot);
        state.set_status(format!("Switched to Profile {} (Mock)", profile), false);
    } else {
        match HidApiTransport::open_first_candidate() {
            Ok(transport) => {
                let mut mgr = ConfigManager::new(transport);
                match mgr.switch_profile(slot) {
                    Ok(_) => state.set_status(format!("Switched to Profile {}", profile), false),
                    Err(e) => state.set_status(format!("Failed to switch profile: {}", e), true),
                }
            }
            Err(e) => state.set_status(format!("Device not accessible: {}", e), true),
        }
    }
}
