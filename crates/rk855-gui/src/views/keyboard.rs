use crate::state::{AppState, BackendMode, LayoutMode};
use egui::{Color32, RichText, Ui, Vec2};
use rk855_config::{parse_rgb_json, ConfigManager};
use rk855_hid::{HidApiTransport, MockTransport};
use rk855_proto::keys::{PhysicalKey, PHYSICAL_KEYS};
use rk855_proto::lighting::CustomRgbMap;
use rk855_proto::remap::RemapEntry;
use std::collections::HashMap;

/// Key width units (1.0 = standard square key, e.g. 38px)
fn key_unit_width(key: &PhysicalKey) -> f32 {
    match key.name {
        "backspace" => 2.0,
        "tab" | "\\|" => 1.5,
        "caps" => 1.75,
        "enter" => 2.25,
        "lshift" => 2.25,
        "rshift" => 1.75,
        "lctrl" | "lwin" | "lalt" => 1.25,
        "space" => 6.25,
        _ => 1.0,
    }
}

pub fn render(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Keyboard Layout (68-Key Physical Matrix)");
    ui.add_space(8.0);

    // Mode Selection Bar
    ui.horizontal(|ui| {
        ui.label("Layout Tool:");
        if ui
            .selectable_label(
                state.layout_mode == LayoutMode::PerKeyRgb,
                "🎨 Per-Key RGB Backlight",
            )
            .clicked()
        {
            state.layout_mode = LayoutMode::PerKeyRgb;
        }
        if ui
            .selectable_label(state.layout_mode == LayoutMode::Remap, "⌨️ Key Remapping")
            .clicked()
        {
            state.layout_mode = LayoutMode::Remap;
        }

        ui.add_space(20.0);

        match state.layout_mode {
            LayoutMode::PerKeyRgb => {
                ui.label("Active Paint Color:");
                let mut col = Color32::from_rgb(
                    state.active_palette_color[0],
                    state.active_palette_color[1],
                    state.active_palette_color[2],
                );
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut col,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    state.active_palette_color = [col.r(), col.g(), col.b()];
                }
            }
            LayoutMode::Remap => {
                if let Some(key) = state.selected_key() {
                    ui.label(
                        RichText::new(format!(
                            "Selected: {} (Scancode 0x{:02X}, Matrix #{})",
                            key.label, key.scancode, key.matrix_index
                        ))
                        .strong(),
                    );
                } else {
                    ui.label("Click a key to remap its action");
                }
            }
        }
    });

    ui.add_space(12.0);

    // Render 68-key visual keyboard matrix
    let base_unit = 40.0;
    let key_height = 36.0;

    egui::Frame::canvas(ui.style())
        .fill(Color32::from_rgb(25, 27, 32))
        .inner_margin(12.0)
        .show(ui, |ui| {
            for row in 0..=4 {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 3.0);
                    let row_keys: Vec<&PhysicalKey> =
                        PHYSICAL_KEYS.iter().filter(|k| k.row == row).collect();
                    for key in row_keys {
                        let width = key_unit_width(key) * base_unit;
                        let size = Vec2::new(width, key_height);

                        let is_selected = state.selected_key_index == Some(key.index);

                        let (fill_color, text_color) = match state.layout_mode {
                            LayoutMode::PerKeyRgb => {
                                let rgb = state.key_rgb_colors[key.index];
                                let bg = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                let lum = (rgb[0] as f32 * 0.299
                                    + rgb[1] as f32 * 0.587
                                    + rgb[2] as f32 * 0.114)
                                    as u8;
                                let fg = if lum > 140 {
                                    Color32::BLACK
                                } else {
                                    Color32::WHITE
                                };
                                (bg, fg)
                            }
                            LayoutMode::Remap => {
                                if is_selected {
                                    (Color32::from_rgb(0, 120, 215), Color32::WHITE)
                                } else if state.key_remap_targets[key.index].is_some() {
                                    (Color32::from_rgb(45, 100, 60), Color32::WHITE)
                                } else {
                                    (
                                        Color32::from_rgb(45, 48, 56),
                                        Color32::from_rgb(220, 220, 220),
                                    )
                                }
                            }
                        };

                        let label_text = if state.layout_mode == LayoutMode::Remap {
                            if let Some(target) = state.key_remap_targets[key.index] {
                                format!("{}\n->0x{:02X}", key.label, target)
                            } else {
                                key.label.to_string()
                            }
                        } else {
                            key.label.to_string()
                        };

                        let btn = egui::Button::new(
                            RichText::new(label_text).color(text_color).size(11.0),
                        )
                        .fill(fill_color)
                        .min_size(size);

                        if ui.add(btn).clicked() {
                            state.selected_key_index = Some(key.index);
                            if state.layout_mode == LayoutMode::PerKeyRgb {
                                state.key_rgb_colors[key.index] = state.active_palette_color;
                            }
                        }
                    }
                });
            }
        });

    ui.add_space(14.0);

    // Controls footer depending on active tool mode
    match state.layout_mode {
        LayoutMode::PerKeyRgb => {
            render_rgb_controls(ui, state);
        }
        LayoutMode::Remap => {
            render_remap_controls(ui, state);
        }
    }
}

fn render_rgb_controls(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        if ui.button("⚡ Apply Colors (Live RAM)").clicked() {
            apply_custom_rgb(state, false);
        }
        if ui.button("💾 Apply & Save to Flash").clicked() {
            apply_custom_rgb(state, true);
        }
        ui.add_space(10.0);
        if ui.button("🧹 Clear All to Black").clicked() {
            state.key_rgb_colors = [[0, 0, 0]; 68];
            state.set_status("Cleared all key colors", false);
        }
        if ui.button("🎨 Fill All with Active Color").clicked() {
            for color in state.key_rgb_colors.iter_mut() {
                *color = state.active_palette_color;
            }
            state.set_status("Filled all keys with active color", false);
        }
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("📁 Import Theme JSON").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match parse_rgb_json(&content) {
                        Ok(map) => {
                            for (idx, color_opt) in map.colors.iter().enumerate() {
                                if let Some((r, g, b)) = color_opt {
                                    state.key_rgb_colors[idx] = [*r, *g, *b];
                                }
                            }
                            state.set_status(
                                format!(
                                    "Loaded RGB theme from {:?}",
                                    path.file_name().unwrap_or_default()
                                ),
                                false,
                            );
                        }
                        Err(e) => state.set_status(format!("Failed to parse JSON: {}", e), true),
                    },
                    Err(e) => state.set_status(format!("Failed to read file: {}", e), true),
                }
            }
        }

        if ui.button("💾 Export Theme JSON").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .save_file()
            {
                let mut map = HashMap::new();
                for (idx, rgb) in state.key_rgb_colors.iter().enumerate() {
                    let key = &PHYSICAL_KEYS[idx];
                    map.insert(
                        key.name.to_string(),
                        format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
                    );
                }
                if let Ok(json_str) = serde_json::to_string_pretty(&map) {
                    let _ = std::fs::write(&path, json_str);
                    state.set_status(
                        format!("Saved theme to {:?}", path.file_name().unwrap_or_default()),
                        false,
                    );
                }
            }
        }
    });
}

fn render_remap_controls(ui: &mut Ui, state: &mut AppState) {
    if let Some(key) = state.selected_key() {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Remapping key '{}' (Scancode 0x{:02X}):",
                key.label, key.scancode
            ));
            if ui.button("Remap to Caps Lock (0x39)").clicked() {
                state.key_remap_targets[key.index] = Some(0x39);
            }
            if ui.button("Remap to L-Ctrl (0xE0)").clicked() {
                state.key_remap_targets[key.index] = Some(0xE0);
            }
            if ui.button("Reset this key").clicked() {
                state.key_remap_targets[key.index] = None;
                state.set_status(format!("Reset key '{}'", key.label), false);
            }
        });
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("💾 Commit Remap Table to Hardware").clicked() {
            apply_remap(state);
        }
        if ui.button("🔄 Factory Reset All Remaps").clicked() {
            state.key_remap_targets = [None; 68];
            reset_all_remaps(state);
        }
    });
}

fn apply_custom_rgb(state: &mut AppState, persist: bool) {
    let mut rgb_map = CustomRgbMap::new();
    for (idx, rgb) in state.key_rgb_colors.iter().enumerate() {
        rgb_map.colors[idx] = Some((rgb[0], rgb[1], rgb[2]));
    }

    if state.backend_mode == BackendMode::Mock {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let res = mgr.apply_custom_rgb(&rgb_map, persist);
        match res {
            Ok(_) => state.set_status(
                format!("Applied 68-Key RGB (Mock, Persist={})", persist),
                false,
            ),
            Err(e) => state.set_status(format!("Error: {}", e), true),
        }
    } else {
        match HidApiTransport::open_first_candidate() {
            Ok(transport) => {
                let mut mgr = ConfigManager::new(transport);
                match mgr.apply_custom_rgb(&rgb_map, persist) {
                    Ok(_) => state.set_status(
                        format!("Successfully applied Per-Key RGB! (Persist={})", persist),
                        false,
                    ),
                    Err(e) => state.set_status(format!("Failed to apply RGB: {}", e), true),
                }
            }
            Err(e) => state.set_status(format!("Keyboard not connected: {}", e), true),
        }
    }
}

fn apply_remap(state: &mut AppState) {
    let mut entries = Vec::new();
    for (idx, target_opt) in state.key_remap_targets.iter().enumerate() {
        if let Some(target_scancode) = target_opt {
            let key = &PHYSICAL_KEYS[idx];
            entries.push(RemapEntry::new(key.scancode, *target_scancode));
        }
    }

    if state.backend_mode == BackendMode::Mock {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let _ = mgr.apply_remap(&entries, true);
        state.set_status("Committed remap table (Mock)", false);
    } else {
        match HidApiTransport::open_first_candidate() {
            Ok(transport) => {
                let mut mgr = ConfigManager::new(transport);
                match mgr.apply_remap(&entries, true) {
                    Ok(_) => state.set_status("Committed key remap table to hardware!", false),
                    Err(e) => state.set_status(format!("Remap error: {}", e), true),
                }
            }
            Err(e) => state.set_status(format!("Keyboard not connected: {}", e), true),
        }
    }
}

fn reset_all_remaps(state: &mut AppState) {
    if state.backend_mode == BackendMode::Mock {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let _ = mgr.reset_remap(true);
        state.set_status("Factory reset all remaps (Mock)", false);
    } else {
        match HidApiTransport::open_first_candidate() {
            Ok(transport) => {
                let mut mgr = ConfigManager::new(transport);
                match mgr.reset_remap(true) {
                    Ok(_) => state.set_status("Factory reset all remaps on keyboard!", false),
                    Err(e) => state.set_status(format!("Reset error: {}", e), true),
                }
            }
            Err(e) => state.set_status(format!("Keyboard not connected: {}", e), true),
        }
    }
}
