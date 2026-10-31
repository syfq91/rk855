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
}
