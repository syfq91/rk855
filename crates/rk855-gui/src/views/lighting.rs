use crate::state::{AppState, BackendMode};
use egui::Ui;
use rk855_config::ConfigManager;
use rk855_hid::{HidApiTransport, MockTransport};
use rk855_proto::enums::{ColorType, Direction, LightMode};
use rk855_proto::lighting::LightParams;

const PRESET_MODES: [(u8, &str); 20] = [
    (0, "0: Static Solid"),
    (1, "1: Single On"),
    (2, "2: Single Off"),
    (3, "3: Glittering"),
    (4, "4: Falling"),
    (5, "5: Color Cycling"),
    (6, "6: Breathing"),
    (7, "7: Spectrum"),
    (8, "8: Outward Expand"),
    (9, "9: Scrolling Wave"),
    (10, "10: Rolling Wave"),
    (11, "11: Rotating Accumulate"),
    (12, "12: Explode / Windmill"),
    (13, "13: Launch / Waterfall"),
    (14, "14: Ripples"),
    (15, "15: Flowing Stream"),
    (16, "16: Constant Rainbow"),
    (17, "17: Sine Wave Tilt"),
    (18, "18: Shuttle Shadow"),
    (19, "19: Backlight Off"),
];

const QUICK_COLORS: [(&str, [u8; 3]); 8] = [
    ("Red", [255, 0, 0]),
    ("Green", [0, 255, 0]),
    ("Blue", [0, 100, 255]),
    ("Cyan", [0, 255, 255]),
    ("Magenta", [255, 0, 255]),
    ("Yellow", [255, 255, 0]),
    ("Orange", [255, 128, 0]),
    ("White", [255, 255, 255]),
];

pub fn render(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Lighting Effects & Presets");
    ui.add_space(8.0);

    egui::Grid::new("lighting_controls_grid")
        .num_columns(2)
        .spacing([24.0, 12.0])
        .show(ui, |ui| {
            // Mode Selector
            ui.label("Effect Preset:");
            egui::ComboBox::from_id_salt("mode_combo")
                .selected_text(
                    PRESET_MODES
                        .iter()
                        .find(|(id, _)| *id == state.light_mode)
                        .map(|(_, name)| *name)
                        .unwrap_or("Custom"),
                )
                .show_ui(ui, |ui| {
                    for (id, name) in PRESET_MODES {
                        ui.selectable_value(&mut state.light_mode, id, name);
                    }
                });
            ui.end_row();

            // Color Type
            ui.label("Color Mode:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.color_type, 0, "Fixed Color");
                ui.selectable_value(&mut state.color_type, 1, "Custom RGB");
                ui.selectable_value(&mut state.color_type, 8, "Rainbow");
                ui.selectable_value(&mut state.color_type, 7, "Spectrum");
            });
            ui.end_row();

            // Brightness Slider
            ui.label("Brightness (0-4):");
            ui.add(egui::Slider::new(&mut state.brightness, 0..=4));
            ui.end_row();

            // Speed Slider
            ui.label("Speed (0-4):");
            ui.add(egui::Slider::new(&mut state.speed, 0..=4));
            ui.end_row();

            // Direction Toggle
            ui.label("Animation Direction:");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.direction, 0, "◀ Left-to-Right");
                ui.selectable_value(&mut state.direction, 1, "Right-to-Left ▶");
            });
            ui.end_row();

            // Color Picker
            ui.label("Selected Color:");
            ui.horizontal(|ui| {
                let mut color32 = egui::Color32::from_rgb(
                    state.custom_color[0],
                    state.custom_color[1],
                    state.custom_color[2],
                );
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color32,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    state.custom_color = [color32.r(), color32.g(), color32.b()];
                }

                ui.label(format!(
                    "#{:02X}{:02X}{:02X}",
                    state.custom_color[0], state.custom_color[1], state.custom_color[2]
                ));
            });
            ui.end_row();

            // Quick Colors Palette
            ui.label("Quick Palettes:");
            ui.horizontal(|ui| {
                for (name, rgb) in QUICK_COLORS {
                    let col = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                    if ui.button(egui::RichText::new(name).color(col)).clicked() {
                        state.custom_color = rgb;
                        state.color_type = 1; // RGB
                    }
                }
            });
            ui.end_row();
        });

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Apply Buttons
    ui.horizontal(|ui| {
        if ui.button("⚡ Apply (Live RAM — No Flash Wear)").clicked() {
            apply_lighting(state, false);
        }

        if ui.button("💾 Apply & Save to Flash Memory").clicked() {
            apply_lighting(state, true);
        }

        ui.add_space(16.0);

        if ui.button("🛑 Turn Off Backlight").clicked() {
            state.light_mode = 19; // Off
            apply_lighting(state, false);
        }
    });
}

fn apply_lighting(state: &mut AppState, persist: bool) {
    let mode = match LightMode::from_u8(state.light_mode) {
        Some(m) => m,
        None => {
            state.set_status("Invalid lighting mode", true);
            return;
        }
    };
    let color_type = ColorType::from_u8(state.color_type).unwrap_or(ColorType::RgbCustom);
    let direction = Direction::from_u8(state.direction).unwrap_or(Direction::LeftToRight);

    // Brightness maps 0..4 to 0..15; Speed maps 0..4 to 1..10
    let brightness_raw = state.brightness * 3 + (if state.brightness > 0 { 3 } else { 0 });
    let speed_raw = state.speed * 2 + 2;

    let params = LightParams {
        mode,
        color_type,
        r: state.custom_color[0],
        g: state.custom_color[1],
        b: state.custom_color[2],
        brightness: brightness_raw.min(15),
        speed: speed_raw.min(10),
        direction,
    };

    if state.backend_mode == BackendMode::Mock {
        let mock = MockTransport::new();
        let mut mgr = ConfigManager::new(mock);
        let res = mgr.apply_lighting_transaction(&params, persist);
        match res {
            Ok(_) => state.set_status(
                format!("Applied {} (Mock, Persist={})", mode.name(), persist),
                false,
            ),
            Err(e) => state.set_status(format!("Error: {}", e), true),
        }
    } else {
        match HidApiTransport::open_first_candidate() {
            Ok(transport) => {
                let mut mgr = ConfigManager::new(transport);
                match mgr.apply_lighting_transaction(&params, persist) {
                    Ok(_) => state.set_status(
                        format!(
                            "Applied {} successfully! (Persist={})",
                            mode.name(),
                            persist
                        ),
                        false,
                    ),
                    Err(e) => state.set_status(format!("Failed to apply lighting: {}", e), true),
                }
            }
            Err(e) => state.set_status(format!("Keyboard not connected: {}", e), true),
        }
    }
}
