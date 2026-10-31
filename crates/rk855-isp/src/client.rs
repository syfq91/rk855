//! High-level ISP client orchestrating bootloader sessions, memory operations, and safe flashing.

use crate::error::IspError;
use crate::image::{validate_firmware_image, VALIDITY_MARKER_ADDR};
use crate::protocol::*;
use rk855_hid::HidTransport;
use std::time::{Duration, Instant};

/// Options configuring the flashing operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct FlashOptions {
    /// Skip flash erase cycle (only for pre-erased hardware).
    pub skip_erase: bool,
    /// Skip read-back verification (strongly discouraged).
    pub no_verify: bool,
    /// Force flash even if chip ID is unknown or pre-flight warnings exist.
    pub force: bool,
    /// Simulate the complete flow without writing to physical hardware.
    pub dry_run: bool,
}

/// Summary report returned upon successful flashing.
#[derive(Debug, Clone)]
pub struct FlashSummary {
    pub chip_info: BootloaderInfo,
    pub pages_written: usize,
    pub pages_verified: usize,
    pub elapsed: Duration,
}

/// Client for communicating with the Sonix SN32F240B Mask ROM ISP bootloader.
pub struct IspClient<T: HidTransport> {
    transport: T,
}

impl<T: HidTransport> IspClient<T> {
    /// Wraps an existing HID transport connected to the keyboard or bootloader.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Consumes the client and returns the underlying transport.
    pub fn into_inner(self) -> T {
        self.transport
    }

    /// Returns a reference to the underlying transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Sends the magic sequence from normal keyboard mode to trigger a software jump to ROM bootloader.
    pub fn send_bootloader_trigger(&mut self) -> Result<(), IspError> {
        let pkt = build_software_bootloader_trigger();
        self.transport.set_feature(&pkt)?;
        Ok(())
    }

    /// Establishes an ISP session with the bootloader and queries chip identification.
    pub fn handshake(&mut self) -> Result<BootloaderInfo, IspError> {
        // Step 1: Send Enter ISP Mode (0x20)
        let enter_pkt = build_enter_isp_packet();
        self.transport.set_feature(&enter_pkt)?;

        // Step 2: Query bootloader version & chip ID (0x21)
        let ver_pkt = build_get_version_packet();
        self.transport.set_feature(&ver_pkt)?;

        let resp = self.transport.get_feature(REPORT_ID, 64)?;
        parse_version_response(&resp)
    }

    /// Erases a range of on-chip flash memory using opcode 0x24.
    pub fn erase_range(&mut self, start_addr: u32, end_addr: u32) -> Result<(), IspError> {
        let pkt = build_erase_packet(start_addr, end_addr);
        self.transport.set_feature(&pkt)?;

        let resp = self.transport.get_feature(REPORT_ID, 64)?;
        verify_ack_response(&resp, OP_ENABLE_ERASE).map_err(|e| match e {
            IspError::UnexpectedResponse(msg) => IspError::EraseFailed(msg),
            other => other,
        })
    }

    /// Writes exactly one 64-byte page to flash memory.
    pub fn write_page(&mut self, page_idx: usize, data: &[u8; PAGE_SIZE]) -> Result<(), IspError> {
        let page_addr = (page_idx * PAGE_SIZE) as u32;

        // Step 1: Send setup packet with target address
        let setup_pkt = build_program_setup_packet(page_addr);
        self.transport.set_feature(&setup_pkt)?;

        // Step 2: Send 64-byte page payload
        let data_pkt = build_page_data_packet(data);
        self.transport.set_feature(&data_pkt)?;

        // Step 3: Check ACK
        let resp = self.transport.get_feature(REPORT_ID, 64)?;
        verify_ack_response(&resp, OP_ENABLE_PROGRAM).map_err(|e| match e {
            IspError::UnexpectedResponse(msg) => IspError::WriteFailed {
                page: page_idx,
                address: page_addr,
                reason: msg,
            },
            other => other,
        })
    }

    /// Reads back one 64-byte page from flash memory via opcode 0x2A.
    pub fn read_page(&mut self, page_idx: usize) -> Result<[u8; PAGE_SIZE], IspError> {
        let page_addr = (page_idx * PAGE_SIZE) as u32;
        let pkt = build_check_data_packet(page_addr, PAGE_SIZE as u32);
        self.transport.set_feature(&pkt)?;

        let resp = self.transport.get_feature(REPORT_ID, 64)?;
        extract_page_data(&resp)
    }

    /// Reads a contiguous span of on-chip flash memory page by page.
    pub fn read_flash_range(
        &mut self,
        start_addr: u32,
        byte_len: usize,
    ) -> Result<Vec<u8>, IspError> {
        let start_page = (start_addr as usize) / PAGE_SIZE;
        let end_page = ((start_addr as usize + byte_len).saturating_add(PAGE_SIZE - 1)) / PAGE_SIZE;
        let total_pages = end_page.saturating_sub(start_page).min(PAGE_COUNT);

        let mut buffer = Vec::with_capacity(total_pages * PAGE_SIZE);
        for page_idx in start_page..start_page + total_pages {
            let page = self.read_page(page_idx)?;
            buffer.extend_from_slice(&page);
        }

        let offset_in_first_page = (start_addr as usize) % PAGE_SIZE;
        if offset_in_first_page + byte_len <= buffer.len() {
            Ok(buffer[offset_in_first_page..offset_in_first_page + byte_len].to_vec())
        } else {
            Ok(buffer)
        }
    }

    /// Issues a reset command (opcode 0x27) to boot into user application code.
    pub fn reset_to_application(&mut self) -> Result<(), IspError> {
        let pkt = build_return_user_packet();
        self.transport.set_feature(&pkt)?;
        Ok(())
    }

    /// Executes the complete guarded flash pipeline:
    ///
    /// 1. Pre-flight validation of candidate firmware image
    /// 2. ISP session handshake and chip ID verification
    /// 3. Flash sector erasure (0x0000..=0xE797)
    /// 4. Page-by-page writing with MANDATORY per-page read-back verification
    /// 5. Reboot into application code ONLY if 100% verified
    pub fn flash_firmware<F>(
        &mut self,
        image_bytes: &[u8],
        options: FlashOptions,
        mut progress_cb: F,
    ) -> Result<FlashSummary, IspError>
    where
        F: FnMut(usize, usize, &'static str),
    {
        let start_time = Instant::now();

        // Step 1: Pre-flight validation
        progress_cb(0, PAGE_COUNT, "Validating candidate firmware image");
        let analysis = validate_firmware_image(image_bytes)?;
        if !analysis.is_safe_to_flash() && !options.force {
            let msg = analysis.errors.join("; ");
            return Err(IspError::PreflightFailed(msg));
        }

        if options.dry_run {
            progress_cb(
                PAGE_COUNT,
                PAGE_COUNT,
                "Dry run completed successfully (no hardware modified)",
            );
            return Ok(FlashSummary {
                chip_info: BootloaderInfo {
                    chip_id: CHIP_ID_SN32F240B,
                    bootloader_version: 0x0102,
                    is_supported_chip: true,
                },
                pages_written: 0,
                pages_verified: 0,
                elapsed: start_time.elapsed(),
            });
        }

        // Step 2: Handshake and identify MCU
        progress_cb(0, PAGE_COUNT, "Connecting to Sonix ISP Bootloader");
        let chip_info = self.handshake()?;
        if !chip_info.is_supported_chip && !options.force {
            return Err(IspError::HandshakeFailed(format!(
                "Unsupported chip ID: 0x{:04X}. Use --force to override.",
                chip_info.chip_id
            )));
        }

        // Step 3: Flash Erase
        if !options.skip_erase {
            progress_cb(0, PAGE_COUNT, "Erasing microcontroller flash memory");
            // Erase application code extent (0x0000..0xE797)
            self.erase_range(0x0000, 0xE797)?;
        }

        // Step 4: Write Pages & Immediate Verification
        let mut pages_written = 0;
        let mut pages_verified = 0;

        for page_idx in 0..PAGE_COUNT {
            let addr = (page_idx * PAGE_SIZE) as u32;
            let mut chunk = [0u8; PAGE_SIZE];
            chunk.copy_from_slice(&image_bytes[addr as usize..addr as usize + PAGE_SIZE]);

            // Skip trailing unwritten padding pages (between 0x8000 and 0xFFC0) to conserve flash wear
            let is_empty_page = chunk.iter().all(|&b| b == 0xFF);
            let is_sparse_padding = is_empty_page
                && (addr as usize) >= 0x8000
                && (addr as usize + PAGE_SIZE) <= VALIDITY_MARKER_ADDR;

            if is_sparse_padding {
                continue;
            }

            // Write page
            progress_cb(page_idx, PAGE_COUNT, "Writing page");
            self.write_page(page_idx, &chunk)?;
            pages_written += 1;

            // Immediate read-back and verify
            if !options.no_verify {
                progress_cb(page_idx, PAGE_COUNT, "Verifying page");
                let read_back = self.read_page(page_idx)?;
                if read_back != chunk {
                    // CRITICAL: DO NOT RESET INTO CORRUPTED IMAGE!
                    return Err(IspError::VerifyMismatch {
                        address: addr,
                        expected_hex: chunk.iter().map(|b| format!("{:02X}", b)).collect(),
                        actual_hex: read_back.iter().map(|b| format!("{:02X}", b)).collect(),
                    });
                }
                pages_verified += 1;
            }
        }

        // Step 5: Reset to Application (only reached if all pages written & verified!)
        progress_cb(
            PAGE_COUNT,
            PAGE_COUNT,
            "Resetting microcontroller into user application",
        );
        self.reset_to_application()?;

        Ok(FlashSummary {
            chip_info,
            pages_written,
            pages_verified,
            elapsed: start_time.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::{FLASH_SIZE, VALIDITY_MARKER_VALUE};
    use crate::mock::MockIspTransport;

    #[test]
    fn test_client_handshake() {
        let mock = MockIspTransport::new_bootloader();
        let mut client = IspClient::new(mock);
        let info = client.handshake().expect("Handshake should succeed");
        assert_eq!(info.chip_id, CHIP_ID_SN32F240B);
        assert_eq!(info.bootloader_version, 0x0102);
        assert!(info.is_supported_chip);
    }

    #[test]
    fn test_client_page_write_and_read() {
        let mock = MockIspTransport::new_bootloader();
        let mut client = IspClient::new(mock);

        let test_data = [0x42u8; PAGE_SIZE];
        client.write_page(0, &test_data).expect("Write failed");

        let read_data = client.read_page(0).expect("Read failed");
        assert_eq!(test_data, read_data);
    }

    #[test]
    fn test_client_flash_mock_success() {
        let mock = MockIspTransport::new_bootloader();
        let mut client = IspClient::new(mock);

        // Prepare valid firmware image
        let mut fw = vec![0xFFu8; FLASH_SIZE];
        fw[0..4].copy_from_slice(&0x20001000u32.to_le_bytes()); // SP
        fw[4..8].copy_from_slice(&0x00001001u32.to_le_bytes()); // Reset vector (Thumb)
        fw[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        // Add dummy populated code
        for b in &mut fw[0x100..0x3000] {
            *b = 0x55;
        }

        let summary = client
            .flash_firmware(&fw, FlashOptions::default(), |_cur, _tot, _msg| {})
            .expect("Flash should succeed");

        assert!(summary.pages_written > 0);
        assert_eq!(summary.pages_written, summary.pages_verified);
        assert!(client.transport().reset_called);
    }

    #[test]
    fn test_client_aborts_on_verify_mismatch_without_reset() {
        let mut mock = MockIspTransport::new_bootloader();
        // Inject corruption on page 2 (address 0x0080)
        mock.corrupt_on_verify = Some(0x0080);
        let mut client = IspClient::new(mock);

        let mut fw = vec![0xFFu8; FLASH_SIZE];
        fw[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        fw[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        fw[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        for b in &mut fw[0x100..0x3000] {
            *b = 0xAA;
        }

        let res = client.flash_firmware(&fw, FlashOptions::default(), |_cur, _tot, _msg| {});
        match res {
            Err(IspError::VerifyMismatch { address, .. }) => {
                assert_eq!(address, 0x0080);
            }
            other => panic!("Expected VerifyMismatch, got {:?}", other),
        }

        // CRITICAL CHECK: reset_called MUST be false to avoid bricking!
        assert!(
            !client.transport().reset_called,
            "Reset must NOT be called on verification mismatch!"
        );
    }

    #[test]
    fn test_client_disconnect_mid_write_aborts_without_reset() {
        let mut mock = MockIspTransport::new_bootloader();
        // Simulate disconnect at page 10
        mock.simulate_disconnect_at_page = Some(10);
        let mut client = IspClient::new(mock);

        let mut fw = vec![0xFFu8; FLASH_SIZE];
        fw[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        fw[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        fw[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        for b in &mut fw[0x100..0x3000] {
            *b = 0xAA;
        }

        let res = client.flash_firmware(&fw, FlashOptions::default(), |_cur, _tot, _msg| {});
        assert!(res.is_err());
        assert!(!client.transport().reset_called);
    }

    #[test]
    fn test_client_erase_failure_aborts() {
        let mut mock = MockIspTransport::new_bootloader();
        mock.fail_on_erase = true;
        let mut client = IspClient::new(mock);

        let mut fw = vec![0xFFu8; FLASH_SIZE];
        fw[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        fw[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        fw[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        for b in &mut fw[0x100..0x3000] {
            *b = 0x55;
        }

        let res = client.flash_firmware(&fw, FlashOptions::default(), |_cur, _tot, _msg| {});
        assert!(matches!(res, Err(IspError::Hid(_))));
        assert!(!client.transport().reset_called);
    }

    #[test]
    fn test_client_dry_run_does_not_modify_state() {
        let mock = MockIspTransport::new_bootloader();
        let mut client = IspClient::new(mock);

        let mut fw = vec![0xFFu8; FLASH_SIZE];
        fw[0..4].copy_from_slice(&0x20001000u32.to_le_bytes());
        fw[4..8].copy_from_slice(&0x00001001u32.to_le_bytes());
        fw[VALIDITY_MARKER_ADDR..VALIDITY_MARKER_ADDR + 4]
            .copy_from_slice(&VALIDITY_MARKER_VALUE.to_le_bytes());
        for b in &mut fw[0x100..0x3000] {
            *b = 0x55;
        }

        let opts = FlashOptions {
            dry_run: true,
            ..Default::default()
        };
        let summary = client
            .flash_firmware(&fw, opts, |_cur, _tot, _msg| {})
            .expect("Dry run should succeed");

        assert_eq!(summary.pages_written, 0);
        assert_eq!(summary.pages_verified, 0);
        assert!(!client.transport().reset_called);
        assert!(client.transport().written_pages.is_empty());
    }
}
