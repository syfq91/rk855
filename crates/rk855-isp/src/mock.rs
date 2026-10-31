//! Mock ISP transport for testing flashing workflows and failure scenarios.

use crate::image::FLASH_SIZE;
use crate::protocol::*;
use rk855_hid::{HidTransport, HidTransportError};

#[derive(Debug, Clone)]
pub struct MockIspTransport {
    pub flash: Vec<u8>,
    pub is_bootloader: bool,
    pub chip_id: u16,
    pub bootloader_version: u16,
    pub current_prog_addr: Option<u32>,
    pub pending_response: Vec<u8>,
    pub fail_on_erase: bool,
    pub fail_on_page_write: Option<usize>,
    pub corrupt_on_verify: Option<u32>,
    pub simulate_disconnect_at_page: Option<usize>,
    pub written_pages: Vec<usize>,
    pub erased_ranges: Vec<(u32, u32)>,
    pub reset_called: bool,
}

impl MockIspTransport {
    /// Creates a mock transport in bootloader mode with flash initialized to 0xFF.
    pub fn new_bootloader() -> Self {
        Self {
            flash: vec![0xFFu8; FLASH_SIZE],
            is_bootloader: true,
            chip_id: CHIP_ID_SN32F240B,
            bootloader_version: 0x0102,
            current_prog_addr: None,
            pending_response: vec![0u8; 65],
            fail_on_erase: false,
            fail_on_page_write: None,
            corrupt_on_verify: None,
            simulate_disconnect_at_page: None,
            written_pages: Vec::new(),
            erased_ranges: Vec::new(),
            reset_called: false,
        }
    }

    /// Creates a mock transport starting in normal keyboard mode.
    pub fn new_keyboard() -> Self {
        let mut mock = Self::new_bootloader();
        mock.is_bootloader = false;
        mock
    }
}

impl HidTransport for MockIspTransport {
    fn set_feature(&mut self, payload: &[u8]) -> Result<(), HidTransportError> {
        if payload.is_empty() {
            return Err(HidTransportError::InvalidLength {
                expected: 64,
                actual: 0,
            });
        }

        // If in keyboard mode, look for bootloader trigger sequence
        if !self.is_bootloader {
            if payload.len() >= 8 && payload[..8] == rk855_proto::BOOTLOADER_TRIGGER_MAGIC {
                self.is_bootloader = true;
                return Ok(());
            }
            return Err(HidTransportError::Mock(
                "Received unexpected report in keyboard mode".into(),
            ));
        }

        // Bootloader mode handling
        // Check if this is a raw page data write packet following an OP_ENABLE_PROGRAM
        if let Some(page_addr) = self.current_prog_addr.take() {
            let page_idx = (page_addr as usize) / PAGE_SIZE;

            if let Some(disc_page) = self.simulate_disconnect_at_page {
                if page_idx == disc_page {
                    return Err(HidTransportError::Mock("Simulated disconnect".into()));
                }
            }

            if let Some(fail_page) = self.fail_on_page_write {
                if page_idx == fail_page {
                    return Err(HidTransportError::Mock(format!(
                        "Simulated failure on page {page_idx}"
                    )));
                }
            }

            let page_data = if payload.len() >= 64 {
                &payload[..64]
            } else {
                return Err(HidTransportError::InvalidLength {
                    expected: 64,
                    actual: payload.len(),
                });
            };

            let start = page_addr as usize;
            if start + PAGE_SIZE <= FLASH_SIZE {
                self.flash[start..start + PAGE_SIZE].copy_from_slice(page_data);
                self.written_pages.push(page_idx);
            }

            // Prepare ACK for page program
            let mut resp = vec![0u8; 65];
            resp[0] = REPORT_ID;
            resp[1] = OP_ENABLE_PROGRAM;
            resp[2] = ISP_MAGIC_AA;
            resp[3] = ISP_MAGIC_55;
            self.pending_response = resp;
            return Ok(());
        }

        // Standard command packets
        if payload.len() < 3 {
            return Err(HidTransportError::InvalidLength {
                expected: 3,
                actual: payload.len(),
            });
        }

        let opcode = payload[0];
        match opcode {
            OP_ENTER_ISP => {
                let mut resp = vec![0u8; 65];
                resp[0] = REPORT_ID;
                resp[1] = OP_ENTER_ISP;
                resp[2] = ISP_MAGIC_AA;
                resp[3] = ISP_MAGIC_55;
                self.pending_response = resp;
                Ok(())
            }
            OP_GET_VERSION => {
                let mut resp = vec![0u8; 65];
                resp[0] = REPORT_ID;
                resp[1] = OP_GET_VERSION;
                resp[2] = ISP_MAGIC_AA;
                resp[3] = ISP_MAGIC_55;
                resp[4..6].copy_from_slice(&self.chip_id.to_le_bytes());
                resp[6..8].copy_from_slice(&self.bootloader_version.to_le_bytes());
                self.pending_response = resp;
                Ok(())
            }
            OP_ENABLE_ERASE => {
                if self.fail_on_erase {
                    return Err(HidTransportError::Mock("Simulated erase failure".into()));
                }
                let start_addr = if payload.len() >= 7 {
                    u32::from_le_bytes(payload[3..7].try_into().unwrap())
                } else {
                    0
                };
                let end_addr = if payload.len() >= 11 {
                    u32::from_le_bytes(payload[7..11].try_into().unwrap())
                } else {
                    FLASH_SIZE as u32
                };
                let start = (start_addr as usize).min(FLASH_SIZE);
                let end = (end_addr as usize).min(FLASH_SIZE);
                if start < end {
                    self.flash[start..=end.min(FLASH_SIZE - 1)].fill(0xFF);
                    self.erased_ranges.push((start_addr, end_addr));
                }

                let mut resp = vec![0u8; 65];
                resp[0] = REPORT_ID;
                resp[1] = OP_ENABLE_ERASE;
                resp[2] = ISP_MAGIC_AA;
                resp[3] = ISP_MAGIC_55;
                self.pending_response = resp;
                Ok(())
            }
            OP_ENABLE_PROGRAM => {
                let page_addr = if payload.len() >= 7 {
                    u32::from_le_bytes(payload[3..7].try_into().unwrap())
                } else {
                    0
                };
                self.current_prog_addr = Some(page_addr);
                Ok(())
            }
            OP_CHECK_DATA => {
                let page_addr = if payload.len() >= 7 {
                    u32::from_le_bytes(payload[3..7].try_into().unwrap())
                } else {
                    0
                };
                let mut resp = vec![0u8; 65];
                resp[0] = REPORT_ID;

                let start = page_addr as usize;
                if start + PAGE_SIZE <= FLASH_SIZE {
                    resp[1..1 + PAGE_SIZE].copy_from_slice(&self.flash[start..start + PAGE_SIZE]);
                }

                // If corruption injection requested at this address
                if let Some(corrupt_addr) = self.corrupt_on_verify {
                    if page_addr == corrupt_addr {
                        resp[1] ^= 0xFF; // Invert a byte to trigger mismatch
                    }
                }

                self.pending_response = resp;
                Ok(())
            }
            OP_RETURN_USER => {
                self.reset_called = true;
                self.is_bootloader = false;
                Ok(())
            }
            _ => Err(HidTransportError::Mock(format!(
                "Unhandled mock opcode 0x{opcode:02X}"
            ))),
        }
    }

    fn get_feature(&mut self, _report_id: u8, len: usize) -> Result<Vec<u8>, HidTransportError> {
        let mut out = vec![0u8; len + 1];
        let copy_len = out.len().min(self.pending_response.len());
        out[..copy_len].copy_from_slice(&self.pending_response[..copy_len]);
        Ok(out)
    }
}
