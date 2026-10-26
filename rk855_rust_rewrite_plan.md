# Plan: Rust Rewrite of the RK855 Configurator and Flasher

Target: a cross-platform (Windows and Linux) Rust CLI, and later a GUI, replacing the vendor configurator and firmware flasher for the Royal Kludge RK855 (RK68) keyboard (SONiX SN32F240B).

This plan is written for an AI agent to execute phase by phase. The central rule is that the existing repo docs (`firmware/README.md`, `docs/configurator_protocol.md`, `docs/firmware_flasher_protocol.md`, `docs/sn32f240b_firmware_analysis.md`) contain contradictions, so **protocol facts must be verified against real hardware and USB captures before they are treated as truth**. The plan front-loads that verification and makes flashing the last, most heavily gated feature.

---

## Table of Contents

1. [Ground Rules for the Agent](#1-ground-rules-for-the-agent)
2. [Known Doc Contradictions to Resolve](#2-known-doc-contradictions-to-resolve)
3. [Architecture](#3-architecture)
4. [Phases](#4-phases)
5. [Testing Strategy](#5-testing-strategy)
6. [Risks and Mitigations](#6-risks-and-mitigations)
7. [Definition of Done](#7-definition-of-done)
8. [Human-in-the-Loop Checklist](#8-human-in-the-loop-checklist)

---

## 1. Ground Rules for the Agent

1. **Treat the existing docs as hypotheses, not specs.** Every protocol claim gets a status in `docs/PROTOCOL_STATUS.md`: `unverified`, `verified-by-capture`, or `verified-on-device`.
2. **No flash writes until Phase 6.** Phases 1–5 are read-only or reversible (lighting and keymap writes only, which can be reset with the vendor tool).
3. **All hardware-touching code goes behind a trait** (`HidTransport`) so everything is testable with a mock.
4. **Every destructive operation requires explicit confirmation** (for example `--yes-i-understand`) and a pre-flight checklist.
5. **Ask the human for hardware tasks the agent cannot do**, such as USB captures, boot-pin recovery tests, and testing on a second keyboard variant. Do not guess.
6. **Commit small, with tests.** Each milestone ends with a passing `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`.
7. **Never reproduce or bundle vendor binaries in releases by default.** The tool accepts a user-supplied firmware image.

---

## 2. Known Doc Contradictions to Resolve

These must be settled in Phase 1 using evidence (device enumeration, USB captures, disassembly of the supplied image).

| # | Topic | Conflict |
|---|-------|----------|
| 1 | Vector table | README: SP `0x20002000`, reset `0x109`. Analysis doc: SP `0x200015B0`, reset `0x66D1`. Flasher doc: SP `0x20000728`, reset `0xD9`. |
| 2 | Image extent | README: 32 KB active code, config at `0xF800`. Analysis doc: code beyond `0x8000` (e.g. `0xA8D2`), user data at `0x8580`–`0x9FFF`, 57,051 bytes used. |
| 3 | USB identity | Firmware descriptor shows Apple `05AC:024F`. Configurator and flasher docs use `0C45:8018`. |
| 4 | Vendor channel | Firmware descriptor shows vendor channel as Report ID **5** (usage page `FF00`). Configurator doc says Report ID **0**. |
| 5 | SPI vs matrix pins | SSP0 listed on P1.0/P1.1, but P1.0–P1.4 are also matrix row pins. LED path described as both SPI-driven and timer-PWM multiplexed. |
| 6 | ISP opcodes | Opcode table (`01`, `02`, `03`, `06`, `07`, `08`, `29`) has no evidence of being traced from the flasher; the sample flasher leaves the write step as a comment. |
| 7 | Flash page size | "64 bytes / 256 bytes depending on sector". Needs a definitive answer. |
| 8 | Bootloader PIDs | `0x82AA` only vs `0x7000`–`0x7800` or `0x82AA`. |
| 9 | Recovery claim | "Unbrickable ROM bootloader" vs README warning that a corrupt image may require grounding a boot pin. |
| 10 | Variant safety | "100% safe for single-color and RGB" is claimed from code reading only, not tested on hardware. |

---

## 3. Architecture

A Cargo workspace:

```
rk855/
├─ crates/
│  ├─ rk855-proto/      # pure, no I/O: packet builders/parsers, enums, layout tables
│  ├─ rk855-hid/        # HidTransport trait + hidapi impl + mock impl
│  ├─ rk855-config/     # high-level API: lighting, remap, per-key RGB, macros, profiles
│  ├─ rk855-isp/        # Sonix ISP bootloader client + firmware image handling (ihex/bin)
│  ├─ rk855-cli/        # `rk855` binary (clap)
│  └─ rk855-gui/        # (later) egui frontend
├─ docs/                # existing docs + PROTOCOL_STATUS.md + CAPTURES.md + HARDWARE_TESTS.md
├─ captures/            # pcapng traces from vendor tools (human-provided)
├─ firmware/            # user-supplied images, NOT redistributed by default
├─ tools/               # pcap decoding and helper scripts
└─ udev/99-rk855.rules
```

### Key dependencies

| Purpose | Crate |
|---------|-------|
| HID access (Windows and Linux) | `hidapi` (hidapi-rs), `linux-static-hidraw` feature to avoid libusb |
| CLI | `clap` |
| Errors | `thiserror` (libraries), `anyhow` (binary) |
| Logging | `tracing`, `tracing-subscriber` |
| Profiles | `serde`, `toml` (and optionally `serde_json`) |
| Intel HEX | `ihex` |
| Checksums | `sha2` |
| Progress bars | `indicatif` |
| Property tests | `proptest` |
| CLI tests | `assert_cmd`, `predicates` |
| GUI (later) | `eframe` / `egui` |

### Design principles

- `rk855-proto` is pure and `no_std`-friendly. Packets are fixed-size `[u8; 64]` and builders return them. This keeps the protocol unit-testable against golden byte vectors from captures.
- `HidTransport` is the only boundary to hardware:

  ```rust
  pub trait HidTransport {
      fn set_feature(&mut self, data: &[u8]) -> Result<()>;
      fn get_feature(&mut self, report_id: u8, len: usize) -> Result<Vec<u8>>;
  }
  ```

- The GUI talks only to `rk855-config`. It never touches raw HID.
- Timing values (inter-packet delay, re-enumeration timeout) are named constants, tuned from captures.

---

## 4. Phases

### Phase 1: Discovery and Verification (no feature code shipped)

**Goal:** resolve the doc contradictions with evidence.

**Tasks**

1. **Write `rk855 probe`.** For every `hidapi` device entry, dump VID, PID, interface number, usage page, usage, product string, and path. The human runs it on their keyboard; the agent interprets the output.
2. **Resolve the VID/PID question.** Does the keyboard enumerate as `0C45:8018`, as `05AC:024F`, or as both depending on mode (for example a Mac-mode switch)? Record the answer.
3. **Resolve the vendor interface and report ID question.** Which interface, usage page, and report ID carries the configuration channel?
4. **Capture vendor tool traffic.** Ask the human to capture the vendor configurator and flasher with USBPcap and Wireshark on Windows, or usbmon on Linux (for example with a Windows VM and USB passthrough). Required scenarios:
   - change lighting mode, brightness, speed, and color (one change per capture)
   - set one custom per-key color
   - remap one key
   - record a simple macro
   - run the flasher end to end (**only if the human accepts the risk**)
5. **Write a pcap decoder** (`tools/pcap_decode.py` or a Rust equivalent) that extracts HID SET_REPORT and GET_REPORT payloads from pcapng files into annotated hex. Store output in `captures/decoded/`.
6. **Analyze the supplied firmware image offline.** Check the vector table, the actual used extent, the USB descriptors, and the bootloader-trigger code path. Resolve contradictions 1, 2, 3, 4, and 7 from the table above.
7. **Diff captures against the docs** and update `docs/PROTOCOL_STATUS.md` for every claim.

**Acceptance:** each claim in the configurator and flasher docs has a status, and each open contradiction is resolved or logged as a blocker with a reason.

---

### Phase 2: `rk855-proto` and `rk855-hid`

**Goal:** a tested protocol layer and a transport abstraction.

**Tasks**

1. Define the `HidTransport` trait (see Architecture).
2. Implement `HidApiTransport`:
   - handle the report-ID prefix difference between Windows and Linux (hidraw includes the report ID byte in the buffer)
   - select the correct interface and usage page, as found in Phase 1
   - open by path, not just by VID/PID, because several HID interfaces share one VID/PID
3. Implement `MockTransport` that records sent packets and replays scripted responses.
4. In `rk855-proto`, implement builders and parsers for:
   - transaction primitives: `begin_light` (`04 18`), `set_light` (`04 11`), `custom_light_header` (`04 13`), `begin_remap` (`04 20`), `begin_macro` (`04 19`), `macro_data` (`04 15`), `commit` (`04 02`), `reload` (`04 F0`)
   - enums: `LightMode` (`0x00`–`0x13`), `ColorType`, `Direction`, `RemapCategory` (120–130)
   - the 68-key scan code table and the key-to-matrix-position mapping
5. **Golden tests:** byte-for-byte equality with captured packets from Phase 1.
6. **Property tests:** encode and decode round trips for every message type.
7. Make the inter-packet delay (docs suggest 5 ms) a configurable constant, tuned from captures.

**Acceptance:** all golden vectors pass against the mock, and `rk855 probe` works on Windows and Linux.

---

### Phase 3: Lighting (first real hardware writes)

**Goal:** set built-in lighting safely.

**Tasks**

1. Implement:

   ```
   rk855 light set --mode breath --color ff00ff --brightness 3 --speed 2 --direction ltr
   rk855 light list-modes
   ```

2. Validate ranges. Brightness and speed are documented as 0–4. The docs also show a `0x0A` speed default that is unexplained, so confirm it from captures and either remove or document it.
3. Confirm whether `04 02` (commit) is required for live changes or only for persistence. If they differ, expose `--live` and `--persist`. Onboard storage writes wear flash, so default to the lowest-wear behavior that works.
4. Verify the single-color variant claim only if the human has such a board. Otherwise keep it `unverified` and do not state it as fact in the README.

**Acceptance:** lighting commands visibly work on hardware, and results are logged in `PROTOCOL_STATUS.md`.

---

### Phase 4: Key Remapping and Profiles

**Goal:** remap keys and manage profiles, with a proven way back to defaults.

**Tasks**

1. Implement:

   ```
   rk855 remap set --key CAPS --to ESC
   rk855 remap dump
   rk855 remap reset
   ```

2. Determine the layout of the 256-byte table. The docs say `[PhysicalKeyCode, TargetFunctionCode]` pairs, but the ordering, padding, and the encoding of action categories 120–130 are not specified. Confirm from captures, especially non-standard actions: mouse buttons, media keys, shortcuts, and profile switch.
3. **Read-back:** check whether the device supports reading the current table. If not, the tool keeps its own profile file as the source of truth.
4. Profile files: `rk855 profile export` and `rk855 profile import` (TOML) covering lighting, remaps, and macros. Include a schema version field.
5. Profile switching (action 130) and the three onboard profiles. Confirm the mapping to the flash profile slots.
6. **Build and prove `remap reset` before declaring this phase done.** It is the user's safety net.

**Acceptance:** remaps apply and persist across replug, and reset-to-default is demonstrated on hardware.

---

### Phase 5: Per-Key RGB and Macros

**Goal:** support custom per-key lighting and macros, limited to what is verified.

**Tasks**

1. **Per-key RGB:** 19 packets of 64 bytes (1,216 bytes). The docs do not specify how scan codes map to positions in that buffer, so derive it empirically:
   - light a single key in the vendor tool and capture it
   - identify which offset changed
   - repeat to build a full 68-key to offset table
2. Implement `rk855 rgb set-key ESC ff0000` and `rk855 rgb load map.json`. Optional: map a small image to keys.
3. **Macros:** the format after the `0xAA 0x55` prefix is undocumented. Reverse it from captures of simple macros (single key, key sequence, with delays). Implement only what is confirmed. Mark the rest unsupported rather than guessing.
4. **Desktop-handled actions** (launch app, open URL, text injection): the docs say the desktop software handles some of these. Decide with the human whether to skip them, since a keyboard with no host software cannot execute them.

**Acceptance:** per-key colors match requests visually, and macros trigger as expected on hardware.

---

### Phase 6: Flasher (`rk855-isp`), highest risk

**Do not start until Phases 1–5 are complete and the human explicitly approves.**

#### Step A: Offline work (safe)

1. Parse the `.hex` and `.bin`. Verify the checksums listed in the README and confirm the hex and bin agree.
2. Settle the image-extent contradiction (32 KB vs 64 KB of content) by analyzing the actual file, and report it.
3. Implement pre-flight validation:
   - file size and format
   - vector table sanity (SP inside SRAM range, reset vector inside flash, Thumb bit set)
   - hash check against a user-maintained known-good list

#### Step B: Protocol verification

4. Use the captured vendor flasher session (the highest-value capture). Compare it with the docs' opcode table. Check packet format, response format, erase semantics, and any **code-security or checksum handling** (`CodeSecurity=1` in `UISettings_248B.ini`, flash checksum at `0xFFFC`).
5. Cross-check against existing open-source Sonix tooling (for example the SonixQMK flasher). Where the vendor flow differs from the docs, the capture wins.

#### Step C: Read-only ISP

6. `rk855 isp enter` sends the trigger (`AA 42 89 5A FF 71 62 CC`), then polls for the bootloader device with a timeout instead of a fixed 2 s sleep.
7. `rk855 isp info` performs the handshake and reads the chip ID and bootloader version.
8. `rk855 isp read --out backup.bin` reads back flash. **Always back up first.** If code security blocks readout, say so clearly and require explicit acknowledgment from the user.

#### Step D: Writing (guarded)

9. `rk855 isp flash firmware.bin` runs this flow:
   - **pre-flight:** image validation, backup exists (or acknowledged as impossible), exactly one matching device connected, power warning for laptops on battery
   - enter bootloader, handshake, confirm chip ID
   - erase
   - write pages
   - **read back and verify every page**
   - reset to the application
   - abort and report on any verify mismatch, and do not reset into a partially written image while a retry is still possible
10. `--dry-run` runs the whole flow against the mock and prints what it would send.
11. Write recovery documentation: how to ground the boot pin and re-enter ROM mode, with a clear note that this requires opening the keyboard.
12. Test first on a **sacrificial or spare board**, with the human's consent and an understanding that bricking is possible.
13. Gate flashing on chip ID and VID/PID, and refuse unknown devices unless `--force` is given.

**Acceptance:** a full flash and verify cycle succeeds on hardware, a deliberately corrupted image is rejected in pre-flight, and the human has completed a recovery test.

---

### Phase 7: Platform Support and Packaging

**Tasks**

1. **Linux:** ship `udev/99-rk855.rules` granting `hidraw` access for both the keyboard and the bootloader VID/PIDs (with `TAG+="uaccess"`). The keyboard re-enumerates during flashing, so the rules must cover both.
2. **Windows:** verify that the bootloader device enumerates as plain HID and needs no WinUSB or Zadig step. If it does, document it prominently.
3. **CI (GitHub Actions):** build and test on `ubuntu-latest` and `windows-latest`, with `cargo fmt --check`, `cargo clippy -- -D warnings`, and the mock-based test suite. Hardware tests are `#[ignore]` and run manually.
4. **Releases:** static Linux binary where hidapi allows it, Windows `.exe`, and SHA-256 checksums. Do not bundle vendor firmware.
5. Documentation: README with a clear verified, unverified, and risks section; a troubleshooting guide for permissions and re-enumeration problems.

---

### Phase 8: GUI (optional, after the CLI is solid)

- An `egui` app built only on `rk855-config`.
- Screens: device status; lighting presets; keyboard layout view (from the scan code table) for remaps and per-key colors; profile import and export; flasher tab.
- Put the flasher behind an "Advanced" toggle with a strong warning gate.
- Reuse the same confirmation rules as the CLI.

---

## 5. Testing Strategy

| Layer | Method |
|-------|--------|
| Packet builders and parsers | Golden vectors from captures, plus `proptest` round trips |
| Transaction sequences | Mock transport asserting exact packet order and timing constants |
| CLI | `assert_cmd` tests against the mock transport (hidden `--mock` flag) |
| Hardware | Manual checklist in `docs/HARDWARE_TESTS.md`, run by the human, results logged |
| Flasher | Mock ISP device simulating erase, write, and verify, with injected failures (verify mismatch, timeout, disconnect mid-write) |
| Cross-platform | CI on Windows and Linux for everything except hardware tests |

---

## 6. Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Doc contradictions lead to wrong protocol | Phase 1 captures settle them; never implement from the docs alone |
| Bricking the keyboard | Gated Phase 6, mandatory backup where possible, per-page verification, documented recovery path, spare board for first test |
| Undocumented formats (macros, per-key layout, remap encoding) | Derive from captures; implement only what is confirmed |
| Firmware licensing | Do not bundle vendor firmware; accept a user-supplied image |
| Other hardware revisions behave differently | Gate on chip ID and VID/PID; refuse unknown devices without `--force` |
| Windows and Linux HID differences (report ID, interface selection) | Test both from Phase 2, not at the end |
| Flash wear from repeated writes | Prefer live changes; persist only on request |
| Unverified safety claims propagate into the README | Only state what has a `verified-*` status in `PROTOCOL_STATUS.md` |

---

## 7. Definition of Done

- The `rk855` CLI works on Windows and Linux for lighting, remaps, per-key RGB, profiles, and (verified) flashing.
- Every protocol feature has a status in `PROTOCOL_STATUS.md` backed by evidence.
- CI is green on both platforms, and the mock-based test suite covers the protocol and flashing logic.
- The README states clearly what is verified, what is unverified, and what the risks are.
- A human has completed at least one successful flash, one deliberate bad-image rejection, and one recovery test.

---

## 8. Human-in-the-Loop Checklist

Tasks the agent must request from the human, because it cannot do them itself:

- [ ] Run `rk855 probe` and share the output.
- [ ] Capture USB traffic for each lighting, remap, per-key RGB, and macro scenario.
- [ ] Capture the vendor flasher session (only if willing to risk it).
- [ ] Provide a spare or sacrificial keyboard for first flash tests.
- [ ] Test on a single-color variant, if available.
- [ ] Perform a boot-pin recovery test.
- [ ] Approve the start of Phase 6 explicitly.
