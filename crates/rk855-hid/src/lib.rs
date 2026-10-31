//! HID transport abstraction and implementations for the Royal Kludge RK855.
//!
//! Provides the [`HidTransport`] trait to decouple protocol logic from USB I/O,
//! along with concrete [`HidApiTransport`] for physical hardware and
//! [`MockTransport`] for unit testing without attached hardware.

use rk855_proto::{
    BOOTLOADER_PID, BOOTLOADER_VID, CANDIDATE_KEYBOARD_IDS, DEFAULT_INTER_PACKET_DELAY_MS,
    FEATURE_REPORT_ID, PAYLOAD_SIZE, WIRE_REPORT_SIZE,
};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HidTransportError {
    #[error("HID API error: {0}")]
    HidApi(#[from] hidapi::HidError),

    #[error("Device not found (VID: 0x{vid:04X}, PID: 0x{pid:04X})")]
    DeviceNotFound { vid: u16, pid: u16 },

    #[error("Permission denied opening device: {0}")]
    PermissionDenied(String),

    #[error("Invalid packet length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Mock transport error: {0}")]
    Mock(String),
}

/// Abstract HID transport interface for sending and receiving Feature Reports.
pub trait HidTransport: Send {
    /// Sends a 64-byte payload inside a Feature Report.
    fn set_feature(&mut self, payload: &[u8]) -> Result<(), HidTransportError>;

    /// Reads a Feature Report with the specified report ID and expected length.
    fn get_feature(&mut self, report_id: u8, len: usize) -> Result<Vec<u8>, HidTransportError>;
}

/// Metadata describing an enumerated USB HID device interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
    pub product_string: Option<String>,
    pub manufacturer_string: Option<String>,
    pub serial_number: Option<String>,
    pub path: String,
}

impl DeviceInfo {
    /// Returns true if this device entry matches known RK855 keyboard runtime VID/PID.
    pub fn is_candidate_rk855(&self) -> bool {
        CANDIDATE_KEYBOARD_IDS
            .iter()
            .any(|&(v, p)| v == self.vendor_id && p == self.product_id)
    }

    /// Returns true if this device entry matches the Sonix Mask ROM bootloader.
    pub fn is_bootloader(&self) -> bool {
        self.vendor_id == BOOTLOADER_VID && self.product_id == BOOTLOADER_PID
    }

    /// Returns true if this interface is candidate for the configuration channel (Interface 0, UsagePage 0x0001, Usage 0x0006).
    pub fn is_config_interface(&self) -> bool {
        self.is_candidate_rk855() && self.interface_number == 0
    }
}

/// Enumerates all currently connected USB HID devices via `hidapi`.
pub fn probe_devices() -> Result<Vec<DeviceInfo>, HidTransportError> {
    let api = hidapi::HidApi::new()?;
    let mut results = Vec::new();

    for d in api.device_list() {
        let path = d.path().to_string_lossy().to_string();
        results.push(DeviceInfo {
            vendor_id: d.vendor_id(),
            product_id: d.product_id(),
            interface_number: d.interface_number(),
            usage_page: d.usage_page(),
            usage: d.usage(),
            product_string: d.product_string().map(|s| s.to_string()),
            manufacturer_string: d.manufacturer_string().map(|s| s.to_string()),
            serial_number: d.serial_number().map(|s| s.to_string()),
            path,
        });
    }

    Ok(results)
}

/// Physical HID implementation backed by `hidapi`.
pub struct HidApiTransport {
    device: hidapi::HidDevice,
    inter_packet_delay: Duration,
}

impl HidApiTransport {
    /// Opens the specified device path.
    ///
    /// Opening by exact path is required because multiple HID interfaces (Interface 0 and 1)
    /// share the exact same VID and PID on composite keyboards.
    pub fn open_path(path: &str) -> Result<Self, HidTransportError> {
        let api = hidapi::HidApi::new()?;
        let c_path = std::ffi::CString::new(path)
            .map_err(|e| HidTransportError::PermissionDenied(e.to_string()))?;
        let device = api.open_path(&c_path)?;
        Ok(Self {
            device,
            inter_packet_delay: Duration::from_millis(DEFAULT_INTER_PACKET_DELAY_MS),
        })
    }

    /// Attempts to open the primary RK855 configuration interface (Interface 0).
    pub fn open_first_candidate() -> Result<Self, HidTransportError> {
        let devices = probe_devices()?;
        let candidate = devices
            .iter()
            .find(|d| d.is_config_interface())
            .or_else(|| devices.iter().find(|d| d.is_candidate_rk855()))
            .ok_or(HidTransportError::DeviceNotFound {
                vid: rk855_proto::VENDOR_VID,
                pid: rk855_proto::VENDOR_PID,
            })?;

        Self::open_path(&candidate.path)
    }

    /// Sets custom inter-packet pacing delay (default is 10 ms).
    pub fn set_inter_packet_delay(&mut self, delay: Duration) {
        self.inter_packet_delay = delay;
    }

    /// Gets current inter-packet pacing delay.
    pub fn inter_packet_delay(&self) -> Duration {
        self.inter_packet_delay
    }
}

impl HidTransport for HidApiTransport {
    fn set_feature(&mut self, payload: &[u8]) -> Result<(), HidTransportError> {
        // Across both Windows and Linux (hidraw), hidapi's send_feature_report expects
        // byte 0 to be the Report ID. For Interface 0 Feature Reports, this is Report ID 0x00,
        // followed by the 64-byte payload (65 bytes total on the wire).
        let mut wire = [0u8; WIRE_REPORT_SIZE];
        wire[0] = FEATURE_REPORT_ID;
        let copy_len = payload.len().min(PAYLOAD_SIZE);
        wire[1..1 + copy_len].copy_from_slice(&payload[..copy_len]);

        self.device.send_feature_report(&wire)?;

        if !self.inter_packet_delay.is_zero() {
            thread::sleep(self.inter_packet_delay);
        }

        Ok(())
    }

    fn get_feature(&mut self, report_id: u8, len: usize) -> Result<Vec<u8>, HidTransportError> {
        let mut buf = vec![0u8; len + 1];
        buf[0] = report_id;
        let read = self.device.get_feature_report(&mut buf)?;
        buf.truncate(read);
        Ok(buf)
    }
}

/// In-memory mock transport for unit testing protocol packet sequences and timing without hardware.
#[derive(Default, Debug, Clone)]
pub struct MockTransport {
    pub sent_packets: Vec<Vec<u8>>,
    pub queued_responses: Vec<Vec<u8>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a scripted response for the next `get_feature` call.
    pub fn queue_response(&mut self, resp: Vec<u8>) {
        self.queued_responses.push(resp);
    }

    /// Asserts that the total number of packets sent equals `expected`.
    pub fn assert_packet_count(&self, expected: usize) {
        assert_eq!(
            self.sent_packets.len(),
            expected,
            "Expected {} packets sent, got {}",
            expected,
            self.sent_packets.len()
        );
    }

    /// Asserts that packet at `index` has opcode `[header, subcmd]`.
    pub fn assert_opcode(&self, index: usize, header: u8, subcmd: u8) {
        assert!(
            index < self.sent_packets.len(),
            "Index {} out of bounds (sent {})",
            index,
            self.sent_packets.len()
        );
        let pkt = &self.sent_packets[index];
        assert_eq!(
            pkt[0], header,
            "Packet {} header mismatch: expected 0x{:02X}, got 0x{:02X}",
            index, header, pkt[0]
        );
        if pkt.len() > 1 {
            assert_eq!(
                pkt[1], subcmd,
                "Packet {} subcmd mismatch: expected 0x{:02X}, got 0x{:02X}",
                index, subcmd, pkt[1]
            );
        }
    }

    /// Clears all recorded sent packets and queued responses.
    pub fn clear(&mut self) {
        self.sent_packets.clear();
        self.queued_responses.clear();
    }
}

impl HidTransport for MockTransport {
    fn set_feature(&mut self, payload: &[u8]) -> Result<(), HidTransportError> {
        self.sent_packets.push(payload.to_vec());
        Ok(())
    }

    fn get_feature(&mut self, _report_id: u8, _len: usize) -> Result<Vec<u8>, HidTransportError> {
        if self.queued_responses.is_empty() {
            Ok(vec![0u8; PAYLOAD_SIZE])
        } else {
            Ok(self.queued_responses.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_transport_recording_and_assertions() {
        let mut mock = MockTransport::new();
        let pkt1 = [0x04, 0x18];
        let pkt2 = [0x04, 0x02];

        mock.set_feature(&pkt1).unwrap();
        mock.set_feature(&pkt2).unwrap();

        mock.assert_packet_count(2);
        mock.assert_opcode(0, 0x04, 0x18);
        mock.assert_opcode(1, 0x04, 0x02);
    }

    #[test]
    fn test_mock_transport_response_queue() {
        let mut mock = MockTransport::new();
        let resp = vec![0x04, 0x02, 0x00, 0x01, 0xD8, 0x85];
        mock.queue_response(resp.clone());

        let received = mock.get_feature(0x00, 64).unwrap();
        assert_eq!(received, resp);

        // Subsequent read returns default empty buffer when queue exhausted
        let empty = mock.get_feature(0x00, 64).unwrap();
        assert_eq!(empty.len(), 64);
    }
}
