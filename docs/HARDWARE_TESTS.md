# Hardware Verification Checklist & Test Log

This document serves as the operational checklist for physical hardware testing on Royal Kludge RK855 (RK68) keyboards across the implementation phases.

---

## 1. Test Equipment & Environment

- **Host Operating System:** Windows 11 / Linux (Kernel 5.15+)
- **Keyboard Model:** Royal Kludge RK855 (RK68)
- **Hardware Variant:** [ ] RGB (Multi-color) / [ ] Single-color (Monochrome)
- **Reported VID:PID:** [ ] `05AC:024F` (Factory ROM) / [ ] `0C45:8018` (Vendor filter)
- **Firmware Version (bcdDevice):** `0x0100`

---

## 2. Phase-by-Phase Checklist

### Phase 1: Device Enumeration & Discovery
- [x] Run `rk855 probe` on connected keyboard.
- [x] Verify that all HID interfaces (Interface 0 and Interface 1) are enumerated.
- [x] Confirm interface 0 matches the expected Feature Report configuration channel.
- [x] Record whether device reports Apple VID:PID `05AC:024F` or Sonix `0C45:8018`.
- [x] Verify offline image analysis with `rk855 firmware check firmware/SN32F240B_32.bin`.

### Phase 2: Protocol & Transport Verification (Mocked)
- [ ] Run pure protocol tests in `rk855-proto` (`cargo test -p rk855-proto`).
- [ ] Run transport tests in `rk855-hid` (`cargo test -p rk855-hid`).
- [ ] Verify golden capture vectors against mock transport.

### Phase 3: Built-in Lighting (First Hardware Writes)
- [ ] Test static lighting: `rk855 light set --mode static --color ff0000`.
- [ ] Test dynamic breathing effect: `rk855 light set --mode breath --speed 2 --brightness 3`.
- [ ] Test backlight toggle: `rk855 light set --mode off`.
- [ ] Replug keyboard: verify whether changes persist across power-cycle.
- [ ] If testing single-color variant: verify hardware responds safely without LED driver fault.

### Phase 4: Key Remapping & Profiles
- [ ] Backup existing configuration: `rk855 profile export --out backup.toml`.
- [ ] Remap CapsLock to Escape: `rk855 remap set --key CAPS --to ESC`.
- [ ] Test physical key press: confirm key code Escape is emitted by operating system.
- [ ] Test `remap reset`: verify keyboard returns to factory keymap.
- [ ] Test profile switching: switch to Profile 2 and Profile 3.

### Phase 5: Per-Key RGB & Macros
- [ ] Test individual key illumination (e.g., ESC in red, WASD in blue).
- [ ] Validate full 68-key matrix lighting layout against physical layout.
- [ ] Test simple recorded macro playback.

### Phase 6: ISP Flasher (High Risk - Requires Explicit Human Approval)
- [ ] Perform full pre-flight check on target firmware binary.
- [ ] Issue software trigger to enter bootloader mode: confirm device re-enumerates as `0C45:82AA`.
- [ ] Query chip ID and Mask ROM version (`opcode 0x2A`).
- [ ] Read back flash and generate binary backup.
- [ ] Perform full erase, page write, and byte-for-byte verify cycle.
- [ ] Execute hardware recovery test: verify grounding Pin `P2.2` (Pin 3 of LQFP64) forces bootloader mode.

---

## 3. Test Execution Log

| Date | Phase | Tester | Command / Action | Result | Notes |
|---|---|---|---|---|---|
| 2026-10-31 | Phase 1 | Antigravity / Syafiq | `rk855 probe` | PASS | Physical RK855 detected (`0C45:8018`). Primary config channel confirmed at Interface 0 (`MI_00`). |
| 2026-10-31 | Phase 1 | Antigravity / Syafiq | `rk855 firmware check` | PASS | SP `0x200015B0`, Reset `0x000066D1`, Marker `0xAAAA5555`, Apple VID/PID & ISP trigger verified. |
| 2026-10-31 | Phase 1 | Antigravity / Syafiq | `pcap_decode.py` | PASS | Decoded 6 test capture traces from `pcap test/`. Verified `04 18`, `04 13`, `04 02`, `04 F0` opcodes and ~10ms timing. |
