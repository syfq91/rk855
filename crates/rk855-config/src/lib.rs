//! High-level configuration management for RK855 (lighting, remaps, macros, profiles).

use rk855_hid::{HidTransport, HidTransportError};
use rk855_proto::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("HID transport error: {0}")]
    Transport(#[from] HidTransportError),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Profile representation serializable to TOML
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub name: String,
    pub version: u32,
    pub lighting: Option<LightingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightingConfig {
    pub mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub color_rgb: [u8; 3],
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

    /// Sets lighting mode and commits
    pub fn apply_lighting(
        &mut self,
        params: &LightParams,
        persist: bool,
    ) -> Result<(), ConfigError> {
        // 1. Begin light config
        self.transport.set_feature(&build_begin_light())?;
        // 2. Set light parameters
        self.transport.set_feature(&build_set_light(params))?;
        if persist {
            // 3. Commit to flash
            self.transport.set_feature(&build_commit())?;
        }
        // 4. Reload PWM
        self.transport.set_feature(&build_pwm_reload())?;
        Ok(())
    }
}
