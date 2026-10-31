use rk855_proto::enums::LightMode;
use rk855_proto::keys::{PhysicalKey, PHYSICAL_KEYS};
use std::path::PathBuf;

/// Top-level navigation tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    DeviceStatus,
    Lighting,
    KeyboardLayout,
    Profiles,
    Flasher,
}

/// Keyboard layout sub-mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    PerKeyRgb,
    Remap,
}

/// Hardware backend mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    Hardware,
    Mock,
}

/// Firmware image summary from pre-flight checks.
#[derive(Debug, Clone)]
pub struct ImageSummary {
    pub path: PathBuf,
    pub size_bytes: usize,
    pub sha256: String,
    pub initial_sp: u32,
    pub reset_vector: u32,
    pub has_validity_marker: bool,
    pub has_apple_descriptor: bool,
    pub is_valid: bool,
    pub error_msg: Option<String>,
}

/// Primary application state.
#[allow(dead_code)]
pub struct AppState {
    pub active_tab: ActiveTab,
    pub backend_mode: BackendMode,
    pub device_connected: bool,
    pub connected_device_name: String,
    pub status_message: String,
    pub status_is_error: bool,

    // Lighting state
    pub light_mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub color_type: u8,
    pub direction: u8,
    pub custom_color: [u8; 3],
    pub light_persist: bool,

    // Keyboard Layout state
    pub layout_mode: LayoutMode,
    pub selected_key_index: Option<usize>,
    pub key_rgb_colors: [[u8; 3]; 68],
    pub key_remap_targets: [Option<u8>; 68],
    pub active_palette_color: [u8; 3],

    // Profiles state
    pub active_profile: u8,
    pub profile_path_input: String,

    // Flasher state
    pub flasher_advanced_unlocked: bool,
    pub flasher_warning_acknowledged: bool,
    pub flasher_image_path: String,
    pub flasher_image_summary: Option<ImageSummary>,
    pub flasher_dry_run: bool,
    pub flasher_confirm_input: String,
    pub flasher_is_running: bool,
    pub flasher_progress: f32,
    pub flasher_logs: Vec<String>,
    pub bootloader_info: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let key_rgb_colors = [[0, 150, 255]; 68];
        let key_remap_targets = [None; 68];

        Self {
            active_tab: ActiveTab::DeviceStatus,
            backend_mode: BackendMode::Hardware,
            device_connected: false,
            connected_device_name: "Scanning for device...".to_string(),
            status_message: "Ready".to_string(),
            status_is_error: false,

            light_mode: 0, // Static solid
            brightness: 3,
            speed: 2,
            color_type: 0, // Fixed
            direction: 0,  // LTR
            custom_color: [0, 200, 255],
            light_persist: false,

            layout_mode: LayoutMode::PerKeyRgb,
            selected_key_index: None,
            key_rgb_colors,
            key_remap_targets,
            active_palette_color: [255, 0, 100],

            active_profile: 1,
            profile_path_input: "profile_backup.toml".to_string(),

            flasher_advanced_unlocked: false,
            flasher_warning_acknowledged: false,
            flasher_image_path: "firmware/SN32F240B_32.hex".to_string(),
            flasher_image_summary: None,
            flasher_dry_run: true,
            flasher_confirm_input: String::new(),
            flasher_is_running: false,
            flasher_progress: 0.0,
            flasher_logs: vec!["Flasher initialized. Flashing requires Advanced mode.".to_string()],
            bootloader_info: None,
        }
    }
}

impl AppState {
    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = msg.into();
        self.status_is_error = is_error;
    }

    pub fn selected_key(&self) -> Option<&'static PhysicalKey> {
        self.selected_key_index
            .and_then(|idx| PHYSICAL_KEYS.get(idx))
    }

    pub fn light_mode_name(&self) -> &'static str {
        match LightMode::from_u8(self.light_mode) {
            Some(mode) => mode.name(),
            None => "Unknown Mode",
        }
    }
}
