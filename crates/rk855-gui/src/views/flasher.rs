use crate::state::{AppState, BackendMode, ImageSummary};
use egui::{Color32, RichText, Ui};
use rk855_hid::HidApiTransport;
use rk855_isp::{analyze_firmware, load_firmware, FlashOptions, IspClient, MockIspTransport};
use std::path::PathBuf;

pub fn render(ui: &mut Ui, state: &mut AppState) {
    ui.heading("In-System Programming (ISP) Firmware Flasher");
    ui.add_space(8.0);

    // Advanced Safety Toggle
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            let toggle_label = if state.flasher_advanced_unlocked {
                "🔓 Advanced Mode: UNLOCKED"
            } else {
                "🔒 Advanced Mode: LOCKED"
            };
            if ui
                .selectable_label(state.flasher_advanced_unlocked, toggle_label)
                .clicked()
            {
                state.flasher_advanced_unlocked = !state.flasher_advanced_unlocked;
            }
            if !state.flasher_advanced_unlocked {
                ui.label(
                    RichText::new("Unlock to access firmware flashing capabilities").italics(),
                );
            }
        });
    });

    if !state.flasher_advanced_unlocked {
        ui.add_space(16.0);
        ui.label("The Firmware Flasher tab is locked behind the Advanced safety toggle to prevent accidental firmware corruption.");
        ui.label("All features in the Lighting, Layout, and Profiles tabs can be used safely without unlocking this tab.");
        return;
    }

    ui.add_space(12.0);

    // Warning Banner Gate
    egui::Frame::NONE
        .fill(Color32::from_rgb(60, 20, 20))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.colored_label(Color32::from_rgb(255, 120, 120), "⚠️ CRITICAL SAFETY NOTICE:");
            ui.label("Flashing writes directly to on-chip MCU flash memory. Interrupting power or flashing corrupted code will render the keyboard unbootable.");
            ui.label("Physical recovery requires opening the keyboard enclosure and grounding boot pin P2.2 (Pin 3 of the LQFP64 MCU).");
            ui.add_space(4.0);
            ui.checkbox(
                &mut state.flasher_warning_acknowledged,
                "I understand the risks of flashing and acknowledge the recovery procedure.",
            );
        });

    ui.add_space(16.0);

    // Firmware File Selection & Pre-flight Validation
    ui.horizontal(|ui| {
        ui.label("Firmware File (.hex / .bin):");
        ui.text_edit_singleline(&mut state.flasher_image_path);
        if ui.button("Browse...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Firmware Image", &["hex", "bin"])
                .pick_file()
            {
                state.flasher_image_path = path.to_string_lossy().to_string();
                validate_image(state);
            }
        }
        if ui.button("Validate Image").clicked() {
            validate_image(state);
        }
    });

    // Image Summary Display
    if let Some(summary) = &state.flasher_image_summary {
        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                if summary.is_valid {
                    ui.colored_label(
                        Color32::from_rgb(50, 205, 50),
                        "✔ PRE-FLIGHT VALIDATION PASSED",
                    );
                } else {
                    ui.colored_label(Color32::from_rgb(255, 60, 60), "❌ VALIDATION FAILED");
                }
            });
            ui.label(format!("File: {:?}", summary.path));
            ui.label(format!("SHA-256: {}", summary.sha256));
            ui.label(format!(
                "Normalized Size: {} bytes (64 KB)",
                summary.size_bytes
            ));
            ui.label(format!(
                "Initial SP: 0x{:08X} | Reset Vector: 0x{:08X} (Thumb Mode)",
                summary.initial_sp, summary.reset_vector
            ));
            ui.label(format!(
                "Validity Marker (0xAAAA5555): {}",
                if summary.has_validity_marker {
                    "Present"
                } else {
                    "Missing"
                }
            ));
            ui.label(format!(
                "Apple ROM USB Descriptor: {}",
                if summary.has_apple_descriptor {
                    "Verified"
                } else {
                    "Not found"
                }
            ));
            if let Some(err) = &summary.error_msg {
                ui.colored_label(Color32::from_rgb(255, 80, 80), format!("Error: {}", err));
            }
        });
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Execution Controls
    ui.horizontal(|ui| {
        ui.checkbox(
            &mut state.flasher_dry_run,
            "Dry-Run Simulation (No hardware writes)",
        );
    });

    ui.add_space(8.0);

    let can_flash = state.flasher_warning_acknowledged
        && state
            .flasher_image_summary
            .as_ref()
            .map(|s| s.is_valid)
            .unwrap_or(false)
        && (state.flasher_confirm_input == "FLASH" || state.flasher_dry_run)
        && !state.flasher_is_running;

    ui.horizontal(|ui| {
        ui.label("Type 'FLASH' to confirm:");
        ui.text_edit_singleline(&mut state.flasher_confirm_input);

        let btn_text = if state.flasher_dry_run {
            "🚀 Run Dry-Run Simulation"
        } else {
            "🔥 FLASH FIRMWARE NOW"
        };

        if ui
            .add_enabled(
                can_flash,
                egui::Button::new(RichText::new(btn_text).strong()),
            )
            .clicked()
        {
            run_flasher(state);
        }
    });

    ui.add_space(12.0);

    // Progress Bar
    ui.add(egui::ProgressBar::new(state.flasher_progress).show_percentage());

    ui.add_space(8.0);

    // Terminal Logs
    ui.label("Flasher Logs:");
    egui::ScrollArea::vertical()
        .max_height(160.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::canvas(ui.style())
                .fill(Color32::from_rgb(18, 20, 24))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    for line in &state.flasher_logs {
                        ui.monospace(line);
                    }
                });
        });
}

fn validate_image(state: &mut AppState) {
    let path = PathBuf::from(&state.flasher_image_path);
    state
        .flasher_logs
        .push(format!("Validating image: {:?}", path));

    match analyze_firmware(&path) {
        Ok(analysis) => {
            let is_valid = analysis.is_safe_to_flash();
            let summary = ImageSummary {
                path,
                size_bytes: analysis.file_size,
                sha256: analysis.sha256_hex,
                initial_sp: analysis.initial_sp,
                reset_vector: analysis.reset_vector,
                has_validity_marker: analysis.has_rom_marker,
                has_apple_descriptor: analysis.contains_apple_vid_pid,
                is_valid,
                error_msg: if analysis.errors.is_empty() {
                    None
                } else {
                    Some(analysis.errors.join("; "))
                },
            };
            state.flasher_logs.push(format!(
                "Image validation: safe={}. SHA-256: {}",
                summary.is_valid, summary.sha256
            ));
            state.flasher_image_summary = Some(summary);
            state.set_status("Firmware pre-flight validation complete", false);
        }
        Err(e) => {
            state.flasher_logs.push(format!("Analysis error: {}", e));
            state.set_status(format!("Analysis error: {}", e), true);
        }
    }
}

fn run_flasher(state: &mut AppState) {
    state
        .flasher_logs
        .push("Starting flashing process...".to_string());
    state.flasher_progress = 0.05;

    let path = PathBuf::from(&state.flasher_image_path);
    let raw_bytes = match load_firmware(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            state
                .flasher_logs
                .push(format!("Failed to load firmware: {}", e));
            state.set_status("Flashing aborted", true);
            return;
        }
    };

    let options = FlashOptions {
        dry_run: state.flasher_dry_run,
        skip_erase: false,
        no_verify: false,
        force: false,
    };

    if state.flasher_dry_run || state.backend_mode == BackendMode::Mock {
        state
            .flasher_logs
            .push("[DRY RUN] Initializing mock ISP transport...".to_string());
        let mock = MockIspTransport::new_bootloader();
        let mut client = IspClient::new(mock);

        match client.flash_firmware(&raw_bytes, options, |curr, total, stage| {
            tracing::info!("{stage} ({curr}/{total})");
        }) {
            Ok(summary) => {
                state.flasher_progress = 1.0;
                state.flasher_logs.push(format!(
                    "[DRY RUN] Finished: {} pages verified, elapsed {:?}",
                    summary.pages_verified, summary.elapsed
                ));
                state.set_status("Dry-run flashing completed successfully!", false);
            }
            Err(e) => {
                state
                    .flasher_logs
                    .push(format!("[DRY RUN] Simulation failed: {}", e));
                state.set_status(format!("Dry-run error: {}", e), true);
            }
        }
    } else {
        state
            .flasher_logs
            .push("Connecting to Sonix ISP Bootloader (0C45:82AA)...".to_string());
        match HidApiTransport::open_bootloader() {
            Ok(transport) => {
                let mut client = IspClient::new(transport);
                state
                    .flasher_logs
                    .push("Handshaking with bootloader...".to_string());
                match client.flash_firmware(&raw_bytes, options, |curr, total, stage| {
                    tracing::info!("{stage} ({curr}/{total})");
                }) {
                    Ok(summary) => {
                        state.flasher_progress = 1.0;
                        state.flasher_logs.push(format!(
                            "Flash programming succeeded! Verified {} pages in {:?}",
                            summary.pages_verified, summary.elapsed
                        ));
                        state
                            .flasher_logs
                            .push("Rebooting device into user application...".to_string());
                        state.set_status("Firmware flashed successfully!", false);
                    }
                    Err(e) => {
                        state
                            .flasher_logs
                            .push(format!("Flashing aborted: {}. DO NOT POWER OFF.", e));
                        state.set_status(format!("Flashing aborted: {}", e), true);
                    }
                }
            }
            Err(e) => {
                state.flasher_logs.push(format!(
                    "Device not found in ISP mode: {}. Run 'rk855 isp enter' first.",
                    e
                ));
                state.set_status("Bootloader device not detected", true);
            }
        }
    }
}
