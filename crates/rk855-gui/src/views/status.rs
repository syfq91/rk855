use crate::state::{AppState, BackendMode};
use egui::Ui;
use rk855_config::ConfigManager;
use rk855_hid::{probe_devices, HidApiTransport, MockTransport};
use rk855_proto::constants::{BOOTLOADER_PID, BOOTLOADER_VID, VENDOR_PID, VENDOR_VID};
use rk855_proto::enums::ProfileSlot;

pub fn render(ui: &mut Ui, state: &mut AppState) {
    ui.heading("Device Status & Connection");
    ui.add_space(8.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Operating Mode:");
            let is_mock = state.backend_mode == BackendMode::Mock;
            if ui
                .selectable_label(!is_mock, "🔌 Hardware Device")
                .clicked()
            {
                state.backend_mode = BackendMode::Hardware;
                probe_device(state);
            }
            if ui
                .selectable_label(is_mock, "🧪 Simulator / Mock Mode")
                .clicked()
            {
                state.backend_mode = BackendMode::Mock;
                state.device_connected = true;
                state.connected_device_name = "RK855 Virtual Simulator (Mock)".to_string();
                state.set_status("Connected to Virtual Simulator", false);
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let status_color = if state.device_connected {
                egui::Color32::from_rgb(50, 205, 50)
            } else {
                egui::Color32::from_rgb(220, 50, 50)
            };
            ui.colored_label(
                status_color,
                if state.device_connected {
                    "● CONNECTED"
                } else {
                    "○ DISCONNECTED"
                },
            );
            ui.label(format!("Device: {}", state.connected_device_name));
        });

        ui.add_space(6.0);

        if ui.button("🔄 Rescan / Probe USB Devices").clicked() {
            probe_device(state);
        }
    });

    ui.add_space(16.0);

    ui.heading("Hardware Architecture & Profiles");
    ui.add_space(8.0);

    egui::Grid::new("device_info_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Target MCU:");
            ui.label("SONiX SN32F240B (ARM Cortex-M0 @ 48 MHz)");
            ui.end_row();

            ui.label("Flash Memory:");
            ui.label("64 KB (Populated: 56,088 non-0xFF bytes)");
            ui.end_row();

            ui.label("SRAM:");
            ui.label("8 KB (Initial SP: 0x200015B0)");
            ui.end_row();

            ui.label("Standard USB IDs:");
            ui.label(format!(
                "VID: 0x{:04X}, PID: 0x{:04X} (Normal) / VID: 0x{:04X}, PID: 0x{:04X} (Bootloader)",
                VENDOR_VID, VENDOR_PID, BOOTLOADER_VID, BOOTLOADER_PID
            ));
            ui.end_row();

            ui.label("Switch Matrix:");
            ui.label("8 Rows × 16 Columns (68 physical keys populated)");
            ui.end_row();

            ui.label("Active Hardware Profile:");
            ui.horizontal(|ui| {
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
            ui.end_row();
        });
}

pub fn probe_device(state: &mut AppState) {
    if state.backend_mode == BackendMode::Mock {
        state.device_connected = true;
        state.connected_device_name = "RK855 Virtual Simulator (Mock)".to_string();
        state.set_status("Virtual Simulator Active", false);
        return;
    }

    match probe_devices() {
        Ok(devices) => {
            if let Some(dev) = devices.iter().find(|d| d.is_candidate_rk855()) {
                state.device_connected = true;
                state.connected_device_name = format!(
                    "Royal Kludge RK855 (VID: 0x{:04X}, PID: 0x{:04X}, IF: {})",
                    dev.vendor_id, dev.product_id, dev.interface_number
                );
                state.set_status("Connected to physical keyboard", false);
            } else if let Some(dev) = devices.iter().find(|d| d.is_bootloader()) {
                state.device_connected = true;
                state.connected_device_name = format!(
                    "Sonix ISP Bootloader (VID: 0x{:04X}, PID: 0x{:04X})",
                    dev.vendor_id, dev.product_id
                );
                state.set_status("Connected in ISP Bootloader mode", false);
            } else {
                state.device_connected = false;
                state.connected_device_name = "None detected".to_string();
                state.set_status("No keyboard found. Check USB cable or try Mock mode.", true);
            }
        }
        Err(e) => {
            state.device_connected = false;
            state.connected_device_name = "Probe error".to_string();
            state.set_status(format!("Error scanning USB devices: {}", e), true);
        }
    }
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
