use crate::state::{ActiveTab, AppState, BackendMode};
use crate::views;
use egui::{Color32, RichText};

pub struct Rk855App {
    state: AppState,
}

impl Rk855App {
    pub fn new(mock_mode: bool) -> Self {
        let mut state = AppState::default();
        if mock_mode {
            state.backend_mode = BackendMode::Mock;
            state.device_connected = true;
            state.connected_device_name = "RK855 Virtual Simulator (Mock)".to_string();
            state.set_status("Simulator active (--mock)", false);
        } else {
            views::status::probe_device(&mut state);
        }

        Self { state }
    }
}

impl eframe::App for Rk855App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top Navigation Bar
        egui::TopBottomPanel::top("top_nav").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⌨️ RK855 Configurator").strong());
                ui.add_space(20.0);

                let is_status = self.state.active_tab == ActiveTab::DeviceStatus;
                if ui.selectable_label(is_status, "🔌 Device Status").clicked() {
                    self.state.active_tab = ActiveTab::DeviceStatus;
                }

                let is_lighting = self.state.active_tab == ActiveTab::Lighting;
                if ui.selectable_label(is_lighting, "💡 Lighting").clicked() {
                    self.state.active_tab = ActiveTab::Lighting;
                }

                let is_layout = self.state.active_tab == ActiveTab::KeyboardLayout;
                if ui.selectable_label(is_layout, "⌨️ Layout & RGB").clicked() {
                    self.state.active_tab = ActiveTab::KeyboardLayout;
                }

                let is_profiles = self.state.active_tab == ActiveTab::Profiles;
                if ui.selectable_label(is_profiles, "📑 Profiles").clicked() {
                    self.state.active_tab = ActiveTab::Profiles;
                }

                let is_flasher = self.state.active_tab == ActiveTab::Flasher;
                let flasher_label = if self.state.flasher_advanced_unlocked {
                    "⚡ Flasher (Unlocked)"
                } else {
                    "🔒 Flasher (Advanced)"
                };
                if ui.selectable_label(is_flasher, flasher_label).clicked() {
                    self.state.active_tab = ActiveTab::Flasher;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (mode_text, mode_col) = match self.state.backend_mode {
                        BackendMode::Hardware => ("Hardware Mode", Color32::from_rgb(0, 180, 255)),
                        BackendMode::Mock => ("Simulator Mode", Color32::from_rgb(255, 165, 0)),
                    };
                    ui.colored_label(mode_col, format!("[{}]", mode_text));
                });
            });
            ui.add_space(6.0);
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status_col = if self.state.status_is_error {
                    Color32::from_rgb(255, 80, 80)
                } else {
                    Color32::from_rgb(120, 220, 120)
                };
                ui.colored_label(status_col, &self.state.status_message);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Active Profile: {}", self.state.active_profile));
                });
            });
        });

        // Main Content Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| match self.state.active_tab {
                ActiveTab::DeviceStatus => views::status::render(ui, &mut self.state),
                ActiveTab::Lighting => views::lighting::render(ui, &mut self.state),
                ActiveTab::KeyboardLayout => views::keyboard::render(ui, &mut self.state),
                ActiveTab::Profiles => views::profiles::render(ui, &mut self.state),
                ActiveTab::Flasher => views::flasher::render(ui, &mut self.state),
            });
        });
    }
}
