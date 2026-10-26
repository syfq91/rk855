# RK855 Reverse-Engineering Docs: Inconsistency Register

**Revision 3.** This revision reviews the resolutions the repository author recorded in `rk855_doc_inconsistencies_rev2.md`, where all 29 items were marked "Resolved". Each resolution is re-assessed against the evidence it cites and against the other resolutions. The result is a re-graded status for every item, six new items, and an updated list of remaining work.

**Headline result:** 6 items hold up as closed, 7 are mostly resolved, 10 are partly resolved, 5 should be reopened, and 1 is acknowledged with no technical action. Two items could not be confirmed. The most important finding is in **D1**: the new ISP command set contradicts the old opcode table, so the shipped flasher may be sending the wrong commands.

| Short name | File |
|------------|------|
| **ROOT-README** | `README.md` |
| **FW-README** | `firmware/README.md` |
| **CONFIG** | `docs/configurator_protocol.md` |
| **FLASH** | `docs/firmware_flasher_protocol.md` |
| **FW-ANALYSIS** | `docs/sn32f240b_firmware_analysis.md` |
| **REG-2** | `rk855_doc_inconsistencies_rev2.md` (the author's resolved copy of this register) |

This register is the companion to `rk855_rust_rewrite_plan.md`.

---

## 1. Limits of This Review

1. **No independent firmware check.** I did not run any tool on `SN32F240B_32.bin`, the vendor executables, or a keyboard. Every statement about addresses, bytes, and disassembly comes from the text of REG-2.
2. **No capture or hardware evidence anywhere.** None of the 29 resolutions cites a USB capture or a hardware test. At best they are static analysis of the firmware image and the vendor executables. Several items need more than that under their own decision rules.
3. **Doc edits could not be confirmed.** I could not open raw files or commit history (GitHub blocks them). The copy of `docs/firmware_flasher_protocol.md` that I could fetch still shows the old content: the old opcode table, SP `0x20000728` as the "initial vector table", "covers `0x0`–`0x7FFF` (32 KB)", a page size of "64 / 256 bytes", a PID range of `0x7000`–`0x7800`, and the file name with a space. This may be a stale cache or the edits may not be pushed. **Check `git log` for each file that a resolution says was edited.**
4. **Python sources still unreviewed.** `rk855_ctl.py`, `rk855_flasher.py`, and the GUI have not been read by me. REG-2 says they were reviewed but reports no findings (see D5).

---

## 2. Status Legend

**Evidence labels** (what a resolution rests on)

| Label | Meaning |
|-------|---------|
| **FI** | Firmware image inspection (bytes or disassembly of `SN32F240B_32.bin`), as claimed by the author |
| **EXE** | Static analysis of a vendor executable, as claimed by the author |
| **DOC** | Documentation wording change only |
| **CAP** | USB capture of vendor-tool traffic |
| **HW** | Test on a real keyboard |

**Rev 3 status values**

| Status | Meaning |
|--------|---------|
| **Closed** | Resolution is specific, internally consistent, and sufficient for the item as written |
| **Mostly resolved** | Core claim holds; named loose ends remain |
| **Partly resolved** | Some sub-points answered, others not |
| **Reopened** | Resolution is missing, vague, contradicted, or does not meet the item's decision rule |
| **Unconfirmed** | Claims a documentation edit that I could not verify |
| **Acknowledged** | No technical defect to fix; noted for the record |

**A note on agreement between documents.** Several documents agreeing on a value is not independent confirmation, because they appear to come from the same analysis workflow ("REA"). Only the firmware file, the hardware, and USB captures count as evidence.

---

## 3. Summary Table

| ID | Topic | Severity | Evidence cited | Rev 3 status |
|----|-------|----------|----------------|--------------|
| A1 | Three different vector tables | Critical | FI | Mostly resolved |
| A2 | Image extent and flash map | High | FI | **Closed** |
| A3 | SRAM region used for two purposes | Medium | FI | **Closed** |
| A4 | Version numbers that collide | Low | FI | **Closed** |
| B1 | Keyboard VID/PID and which is "normal" | Critical | FI, DOC | Partly resolved |
| B2 | Vendor channel: Report ID 0 vs 5 | Critical | FI | Partly resolved |
| B3 | HID interface class oddities | Low | FI | **Closed** |
| C1 | Per-key RGB length and header | High | EXE | Mostly resolved |
| C2 | Remap entry encoding | High | FI | Partly resolved |
| C3 | Lighting parameter ranges and IDs | Medium | EXE | Partly resolved |
| C4 | Commit, reload, ACK semantics | Medium | none stated | Partly resolved |
| C5 | Profile slot layout | Medium | FI | Partly resolved |
| C6 | Lighting mode names differ | Medium | DOC | Partly resolved |
| C7 | Byte-index convention | Medium | DOC | Mostly resolved |
| D1 | ISP opcode table | Critical | EXE | **Reopened** |
| D2 | Bootloader PIDs | High | none stated | Partly resolved |
| D3 | Page size, erase scope, user data loss | High | FI | Mostly resolved |
| D4 | Bootloader location, "unbrickable" claim | High | FI | Partly resolved |
| D5 | Shipped Python tools | **High** (raised) | none stated | **Reopened** |
| E1 | LED driving: SPI vs timer PWM | High | FI | Partly resolved |
| E2 | Matrix size | Medium | FI | Mostly resolved |
| E3 | Single-color variant safety claim | High | FI | **Reopened** |
| E4 | Peripheral addresses, flash routine | Medium | none stated | **Reopened** |
| F1 | Bootloader jump snippet | Low | FI | **Closed** |
| G1 | Repo structure vs quickstart | Low | DOC | Unconfirmed |
| G2 | Executable file-name variants | Low | DOC | Unconfirmed |
| G3 | "20 effects" wording | Low | DOC | **Closed** |
| G4 | "REA" provenance | Low | DOC | **Reopened** |
| G5 | Licensing statement | Low | DOC | Acknowledged |
| **X1** | **C1 and C2 disagree on the header count rule** | Medium | none | **New** |
| **X2** | **C1 and C7 disagree on byte indexing** | Low | none | **New** |
| **X3** | **A1 and FLASH on what resource 4000 is** | Medium | none | **New** |
| **X4** | **B2 and D1 on report lengths and padding** | Medium | none | **New** |
| **X5** | **E1 and E2 pin conflicts and ROOT-README** | Medium | none | **New** |
| **N1** | **`0xAAAA5555` at `0xFFFC` is a marker, not a checksum** | Medium | FI | **New** |

**Counts:** Closed 6 (A2, A3, A4, B3, F1, G3). Mostly resolved 5 (A1, C1, C7, D3, E2). Partly resolved 10 (B1, B2, C2, C3, C4, C5, C6, D2, D4, E1). Reopened 5 (D1, D5, E3, E4, G4). Unconfirmed 2 (G1, G2). Acknowledged 1 (G5). Original items: 29. New items: 6. Total: 35.

---

## 4. Safety Note on the Existing Tooling

ROOT-README ships `tools/rk855_flasher.py` and a GUI flasher. The ISP protocol they depend on is documented in FLASH, whose opcode table has now been contradicted by the author's own D1 resolution (see D1). Until D1 and D5 are closed:

- Do not run `flash` on a keyboard you cannot afford to lose.
- Do not run any ISP command from the shipped tools that you have not read in the source. In particular, the old table's opcode `0x08` ("erase") is `Set Code Option` in the new command set, and `0x07` ("write page") is `Return User Mode`.
- Back up nothing you cannot restore: a full flash overwrites user keymaps, lighting, and macros (D3).

---

# 5. Group A: Firmware Image

## A1. Three different vector tables

**Severity:** Critical &nbsp;|&nbsp; **Rev 3 status:** Mostly resolved

**Original finding:** README, FW-ANALYSIS, and FLASH gave three different initial SP / Reset pairs.

**REG-2 resolution (FI):** the `.bin` has SP `0x200015B0` and Reset `0x000066D1`. The pair `0x20000728` / `0xD9` belongs to flasher resource 4000 at `0x001AC378`. The value `0x0109` is `CurVersion` in `UISettings.ini`.

**Assessment:** specific and plausible, and it explains the origin of two of the three wrong values. Loose ends:

1. The origin of FW-README's SP `0x20002000` is not explained. It may simply be the top of SRAM, which would make it a guess.
2. If resource 4000 has a different vector table from the `.bin`, it is a different image, which contradicts FLASH 5.2 ("raw binary matching `SN32F240B_32.hex`"). See **X3**.
3. The bytes themselves are not shown in the register.

**Remaining work**
- Show the first 16 bytes of the `.bin` and of resource 4000 in the doc.
- Confirm that FLASH 2.2 no longer presents `0x20000728` / `0xD9` as the target's vector table (the copy I fetched still does).

---

## A2. Image extent and flash map

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Closed

**Original finding:** FW-README said 32 KB active code; ROOT-README and FW-ANALYSIS said 64 KB with 57.6 KB / 57,051 bytes populated.

**REG-2 resolution (FI):** the HEX populates `0x0000`–`0xE797` (59,288 bytes) plus a 4-byte word at `0xFFFC` (`0xAAAA5555`), 59,292 bytes in total. The span `0xE798`–`0xFFFB` (6,244 bytes) is erased padding. The `.bin` is 65,536 bytes with 56,088 non-`0xFF` bytes. `objcopy --gap-fill 0xff` reproduces the `.bin` exactly.

**Assessment:** well specified and cross-consistent. A back-of-envelope check supports it: 59,288 bytes at 16 data bytes per record is about 166.8 KB of Intel HEX (assuming CRLF line endings), against the 166,843-byte file.

**Notes**
- "Populated" now has four meanings in circulation: hex records cover 59,292 bytes, non-`0xFF` bytes are 56,088, FW-ANALYSIS said 57,051, and ROOT-README said 57.6 KB. Docs should use one definition (suggest: "56,088 non-`0xFF` bytes; hex records cover 59,292 bytes").
- FW-README's "config at `0xF800`" falls inside the erased padding, so it is not part of the image (it may be written at run time).
- **Tooling consequence:** a tool that flashes only the first 32 KB would truncate the firmware. The flasher must program `0x0000`–`0xE797` and the word at `0xFFFC`.

**Remaining work:** confirm that "32 KB active image" is gone from FW-README and FLASH, and that the 57,051 / 57.6 KB figures were replaced.

---

## A3. SRAM region used for two different purposes

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Closed

**REG-2 resolution (FI):** the LED row buffers at `0x2000096C`, `0x20000A68`, `0x20000B64` (252 bytes each) are confirmed. The key-matrix debounce buffers are in lower SRAM at `0x2000012C`–`0x200001AC`.

**Assessment:** resolves the overlap. The debounce range is 128 bytes, which matches the 8 × 16 = 128-position matrix found in E2 and C2, a useful independent consistency check.

**Remaining work:** confirm FW-ANALYSIS section 2.3 was edited.

---

## A4. Version numbers that collide

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Closed

**REG-2 resolution (FI):** `CurVersion=0x0109` is the flasher's version-check constant, not a reset vector.

**Assessment:** accepted. It also explains why FW-README's Reset `0x109` looked like the version number. One minor point is unexplained: `bcdDevice` is `0x0100` while `CurVersion` is `0x0109`. It has no effect on the plan.

---

# 6. Group B: USB Identity and Channels

## B1. Keyboard VID/PID and which one is "normal"

**Severity:** Critical &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI, DOC):** the ROM descriptor at `0x6ABC` is Apple `05AC:024F`. `0C45:8018` is the "Windows driver filter profile" in `device.xml`. The flasher INI also supports `8016` and `8008`. The open tools probe by HID usage page across all known IDs.

**Assessment:** this settles what the **firmware descriptor contains**, but not what a board **enumerates as in the field**. Gaps:

1. The register does not say whether the descriptor scan found only one device descriptor in the image.
2. The earlier hypotheses are not eliminated: a stored override in the config area at `0x9000`, multiple descriptor sets, or hardware revisions that differ.
3. If `0C45:8018` is only a filter in the Windows tool, the vendor tool could not find a device that only ever shows `05AC:024F`. A filter that never matches cannot configure anything. The relationship needs explaining.
4. The roles of `8016` and `8008` remain unexplained.
5. CONFIG and FLASH still call `8018` "normal mode", and FLASH's sequence diagram says the keyboard re-enumerates as `8018` after flashing.

**Remaining work:** run the live probe (Phase 1) on Windows and Linux and record the results; post the descriptor scan output and the relevant `device.xml` lines.

---

## B2. Vendor channel: Report ID 0 vs 5, and 8-byte vs 64-byte reports

**Severity:** Critical &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI):** the EP0 setup handlers (`0x00004B08` and `0x0000178A`) process raw `SET_REPORT` Feature control transfers without enforcing the report ID. Both the 8-byte trigger and the 64-byte config packets use EP0 with Report ID 0. Report ID 5 at `0x6BB1` is a separate 3-byte input report.

**Assessment:** plausible, and it matches the register's possible explanation 2 (raw control transfers independent of the descriptor). Gaps:

1. The **declared feature-report size and the target interface** (`wIndex`) are not stated. These decide which `hidapi` path to open.
2. On Windows, `HidD_SetFeature` generally requires the buffer length to match the descriptor's declared feature-report length. D1 says the flasher pads to the declared length, so the "8-byte" trigger may really be padded. See **X4**.
3. B3 describes the Report ID 5 collection as "Vendor", while B2 calls it a 3-byte input report. These should be reconciled in one sentence.

**Remaining work:** dump the live HID descriptors, and record `bmRequestType`, `wValue`, `wIndex`, and `wLength` for the trigger and for configuration writes from a capture.

---

## B3. HID interface class and boot-protocol oddities

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Closed

**REG-2 resolution (FI):** a dual-interface composite descriptor. Interface 0 is a boot keyboard with NKRO on Report ID 1 and mouse on ID 6. Interface 1 is declared as a mouse, with consumer controls on ID 1, power on ID 2, and vendor on ID 5.

**Assessment:** confirms the "quirky but accurate" reading. Accepted.

---

# 7. Group C: Configurator Protocol

## C1. Per-key RGB length and header mismatch

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Mostly resolved

**REG-2 resolution (EXE):** the 1,216 bytes are 76 records of 16 bytes each (status `0x80`, R, G, B, key attributes, delimiter `0xAA 0x55`). In `DeviceDriver.exe` at `0x00439BA6`, the count `0x4C0` (19 blocks) is decremented to 18 (`0x12`) and placed in Byte 8 of the `04 13` header.

**Assessment:** the strongest resolution in group C. It explains both the off-by-one and why 1,216 is not divisible by 3. Gaps:

1. The **field offsets inside the 16-byte record** are not tabulated.
2. The **record-to-key mapping** is not given. 76 records is more than 68 keys and fewer than the 126 LED positions or 128 matrix positions. What are the extra records?
3. "Byte 8" is stated without a convention (see **X2**).
4. The rule "header = block count minus one" does not match the remap header (see **X1**).

**Remaining work:** a table of the 16-byte record, and the key-to-record map derived from captures.

---

## C2. Remap entries cannot encode what the docs say they encode

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI):** 256 bytes in 4 reports hold 128 two-byte tuples `[PhysicalKeyCode, TargetFunctionCode]` over the MCU's 8 × 16 matrix.

**Assessment:** this restates the original claim. It does answer one sub-point well: 128 tuples fits the 8 × 16 = 128 positions, so 68 keys fit with room to spare. It does not answer the main question: how do action categories 120–130 (mouse, media, shortcuts, macros, profile switch) and the three macro value fields fit in a single target byte? C5's resolution implies flash stores only 128 target bytes, which makes the question sharper.

**Remaining work:** capture remaps for each category and decode the target byte; find where macro values are stored.

---

## C3. Lighting parameter ranges and IDs

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (EXE):** at `0x0040AF0F` the vendor tool initializes Byte 8 (speed) to `0x0A` as a factory placeholder. UI sliders are 0–4. Mode IDs are 0–19.

**Assessment:** explains where `0x0A` comes from and settles the mode ID question (hex 0–19, not decimal 200–219). Still open: what the **device** does with `0x0A` if it is sent (ignored, clamped, or accepted); the interplay of color type with modes that take no color; and the direction byte's behavior for modes that ignore it.

**Remaining work:** capture each mode at each speed step and note what the vendor tool actually sends.

---

## C4. Commit, reload, and ACK semantics undefined

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (no evidence stated):** lighting transactions need Commit (`04 02`) to write flash and Reload (`04 F0`) to refresh PWM immediately. Remap needs Commit only. The "ACK" is the USB EP0 control handshake.

**Assessment:** the ACK answer is credible given B2. Not addressed: whether Commit can be **skipped for live-only changes** (the flash-wear concern); any read-back command; the 5 ms delay; and error behavior for incomplete or out-of-order sequences. No addresses or captures are cited.

**Remaining work:** capture a live-only lighting change, and test an interrupted sequence on a backed-up board.

---

## C5. Profile slot layout

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI):** keymap slots are 128 bytes each at `0x8580`, `0x8780`, `0x8880`. When 256 bytes arrive, the MCU extracts and commits the 128 target codes.

**Assessment:** resolves "256 bytes into a 128-byte slot". Still open: lighting slots exist for profile 1 only (`0x8680`); where profile 2 and 3 lighting lives; where custom RGB and macros live per profile; and how profile switching selects a slot.

---

## C6. Lighting mode names and descriptions differ between documents

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (DOC):** standardized on the `1033.lan` vendor strings; display names aligned.

**Assessment:** acceptable as a naming decision. It does not verify the **behavior descriptions** (for example "Explode" described as a rotating wheel, "Pulsating" as a static gradient). The word "Verified" overstates a documentation change.

**Remaining work:** run each mode on hardware and record observed behavior; remove or label any description not observed.

---

## C7. Byte-index convention changes inside and between documents

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Mostly resolved

**REG-2 resolution (DOC):** standardized on zero-based payload indexing `payload[0..63]` with `payload[0] = 0x04`; the report ID is prepended at `wire[0]`, so `payload[N]` is `wire[N+1]`.

**Assessment:** a sound convention. It must be applied everywhere; C1's resolution still says "Byte 8" (see **X2**).

**Remaining work:** search all docs, including the ROOT-README packet diagram, for "Byte N" and convert.

---

# 8. Group D: Flasher and ISP

## D1. ISP opcode table unverified; tooling already implements it

**Severity:** Critical &nbsp;|&nbsp; **Rev 3 status:** **Reopened**

**REG-2 resolution (EXE):** the command set extracted from `RK855_firmware_flasher.exe` is:

| Opcodes | Name |
|---------|------|
| `0x01` / `0x21` | Get Firmware Version |
| `0x02` / `0x22` | Compare ISP Password |
| `0x03` / `0x23` | Set Encryption Algorithm |
| `0x04` / `0x24` | Enable Erase |
| `0x05` / `0x25` | Enable Program |
| `0x06` / `0x26` | Get Check Sum |
| `0x07` / `0x27` | Return User Mode |
| `0x08` / `0x28` | Set Code Option |
| `0x09` / `0x29` | Get Code Option |
| `0x20` | Enter ISP Mode |
| `0x2A` | Enable Check Data |

The flasher formats feature reports to the declared `FeatureReportByteLength`.

**Assessment: this contradicts the old table rather than extending it.**

| Opcode | Old FLASH table | REG-2 resolution |
|--------|-----------------|------------------|
| `0x01` | Handshake / hello | Get Firmware Version |
| `0x02` | Get chip ID | Compare ISP Password |
| `0x03` | Get bootloader version | Set Encryption Algorithm |
| `0x06` | Read / verify flash | Get Check Sum |
| `0x07` | **Write flash page** | **Return User Mode** |
| `0x08` | **Erase flash** | **Set Code Option** |
| `0x29` | **Reset and run app** | **Get Code Option** |

The old table was therefore wrong, not merely unverified. It also had no password or encryption step and no `AA 55` header in the new description. If `rk855_flasher.py` or the GUI were built from the old table, they may send `Set Code Option` where an erase was intended. "Set Code Option" sounds like it writes chip configuration, which could include protection settings; whether that is harmful here is unknown, and nothing in the register says.

**What the resolution still lacks**
1. Packet layouts for every command (offsets, parameters, lengths), and response formats and status codes.
2. The **data path for writing flash**. The list has Enable Erase and Enable Program but no write-data command.
3. The password value (or how it is derived) and the encryption algorithm parameters.
4. The order of the full session.
5. An explanation of the `0x0N` / `0x2N` pairing (command and response? two modes?) and why `0x20` and `0x2A` have no `0x0N` partner.
6. Any **USB capture**. The decision rule requires `verified-by-capture` for every packet type before write or erase code is written.
7. Evidence of where in the executable the list was found (addresses or strings).

**Remaining work:** capture a full vendor flasher session; decode it; reconcile with the shipped Python tools (D5); only then update FLASH. **Decision rule unchanged:** no write or erase code until each packet type is `verified-by-capture`.

---

## D2. Bootloader PIDs and the special-PID list

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (no evidence stated):** `0C45:82AA` is the Sonix ISP bootloader PID. The INI entries `8016`, `024F`, `8008` are application runtime modes.

**Assessment:** `82AA` is probably right (it appears in two other documents), but no source is given and the INI does not list it. The FLASH range `0x7000`–`0x7800` may come from other chips the generic flasher supports, but that is not stated. The meaning of "SpecialPID" and of `CheckBootLoaderID=1` is not explained.

**Remaining work:** observe the enumeration after the trigger on real hardware.

---

## D3. Page size, erase scope, and user data loss

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Mostly resolved

**REG-2 resolution (FI):** the MCU page size is 64 bytes. Flashing the 64 KB image overwrites `0x0000`–`0xE797`, including user keymaps and macros at `0x8580`–`0x9FFF`, which restores factory presets.

**Assessment:** the user-facing consequence is now documented and follows from A2. Gaps: the **source of the 64-byte figure** is not stated (the fetched FLASH still says 64 / 256); the **erase scope** (whole chip or per page) is not stated; the effect on the "hardware ID" area (see B1) is not discussed; and nothing says whether the shipped tools warn or offer backup.

**Remaining work:** state the erase scope; add a pre-flash warning and a backup step to the tools.

---

## D4. Bootloader location and the "unbrickable" claim

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI):** the ROM bootloader is in mask ROM at `0x1FFF0300`, remapped through SYS0 `0x40060024`. The software trigger needs working application firmware. Corrupted firmware requires grounding `P0.4` at power-on.

**Assessment:** the first two statements are backed by firmware inspection. The last is a **hardware behavior claim**, and firmware inspection cannot show it. Nothing identifies `P0.4` as the boot pin or confirms the pull-up. The `0xAAAA5555` word at `0xFFFC` (see **N1**) may be what the ROM checks, in which case a corrupt image might fall back to ISP mode without the pin. The wording should say "may require" until tested.

**Remaining work:** cite the datasheet for the boot pin and boot behavior; test on a sacrificial board.

---

## D5. Shipped Python tools not reviewed

**Severity:** **High** (raised from Medium) &nbsp;|&nbsp; **Rev 3 status:** **Reopened**

**REG-2 resolution (no evidence stated):** scripts in `tools/` reviewed; safety warnings added for the experimental ISP write functions.

**Assessment:** no findings are reported. The point of the item was to extract exactly which packets the scripts send. This matters more now that D1 shows the documented opcodes were wrong.

**Remaining work:** list, for `rk855_ctl.py`, `rk855_flasher.py`, and the GUI: the VID/PID used, report ID and buffer length, every opcode sent, delays, and how "verify" works. Compare with the D1 command set, and in particular check whether `0x07`, `0x08`, or `0x29` are sent as write, erase, or reset. Quote the safety warning text that was added.

---

# 9. Group E: Hardware Claims

## E1. LED driving architecture: SPI vs timer PWM

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** Partly resolved

**REG-2 resolution (FI):** both peripherals are present. CT16B1 PWM timers (`0x40002040`) drive the 18-phase LED matrix, and SSP0 (`0x4001C000`) uses dedicated pins `P1.0` (SCK) and `P1.1` (MOSI).

**Assessment:** the register now accepts both without saying what the SPI connects to. Presence of an interrupt handler does not prove the peripheral is used. Pin function mapping normally needs the datasheet's pin table, not only disassembly. If P1.0 and P1.1 are dedicated to SPI, the earlier pin lists that used them as matrix rows are wrong. See **X5**.

**Remaining work:** state what the SPI drives and where it is used; cite the pin-mux table.

---

## E2. Matrix size: 8×9 vs 13×12 vs a 68-key board

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** Mostly resolved

**REG-2 resolution (FI):** the key matrix is 8 rows × 16 columns (128 positions, 68 keys mapped), from the scan table at `0x00006D7B`.

**Assessment:** a good result, and it agrees with the 128-byte debounce buffer (A3) and 128 remap tuples (C2). It describes a **logical table**, not the pin wiring: 8 × 16 needs 24 lines or some multiplexing, and the pins are not listed. ROOT-README's "8 × 9", FW-ANALYSIS's "13 × 12", and the "diode-isolated" claim are not addressed.

**Remaining work:** list the pin assignments; correct ROOT-README and FW-ANALYSIS; confirm the edits.

---

## E3. Single-color variant safety claim

**Severity:** High &nbsp;|&nbsp; **Rev 3 status:** **Reopened**

**REG-2 resolution (FI):** the code at `0x0000A8D2` writes identical PWM values to the R, G, and B buffers in "monochrome modes". Both variants share an identical MCU and a unified firmware image. The claim wording was softened to "noted compatibility".

**Assessment:** this does not meet the item's decision rule, which was to say "believed compatible, untested" until hardware evidence exists.
1. Code reading is not hardware evidence, yet "identical MCU and unified firmware" is restated as verified.
2. "Monochrome modes" is undefined. The 20-mode list contains no such mode.
3. The self-contradiction is not addressed: a mono LED wired to one color phase would change brightness with the selected color (for example, red-only on a blue-phase LED).
4. "Noted compatibility" is vague and could be read as a safety assurance.

**Remaining work:** change the wording to "believed compatible, untested"; keep the flasher gated on chip ID and VID/PID; compare PCB photos or test a single-color board if one is available.

---

## E4. Peripheral addresses and flash routine unverified

**Severity:** Medium &nbsp;|&nbsp; **Rev 3 status:** **Reopened**

**REG-2 resolution (no evidence stated):** peripheral base addresses and the flash routine at `0x0000615A` "verified against disassembly and SVD specifications".

**Assessment:** a blanket statement. The original item named specific problems: the single timer block labeled as CT16B0, CT16B1, and CT16B2; the unlock key `0x5AFA` that never appears in the programming sequence; and no cited SVD. The resolution reports no discrepancies, no SVD citation, and no answer on `0x5AFA`.

**Remaining work:** list each address checked and its source; say whether `0x5AFA` is written and where; correct the timer labeling.

---

# 10. Group F: Disassembly Slips

## F1. Bootloader jump snippet is not self-consistent

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Closed

**REG-2 resolution (FI):** Capstone at `0x0000178A`–`0x000017B6` shows that the instruction at `0x179A` is `ldr r0, [pc, #0x104]`, reloading the buffer pointer `0x200002A8`. The snippet in FW-ANALYSIS was corrected.

**Assessment:** accepted. It also fits the literal-pool layout, since `(0x179A + 4) & ~3` plus `0x104` lands at `0x18A0`, next to the two key constants at `0x18A4` and `0x18A8` (a consistency check I made, not something the register states). The reasoning for the original error (a register reused after being overwritten) is confirmed.

---

# 11. Group G: Documentation Hygiene and Provenance

## G1. Repository structure and quickstart do not match

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Unconfirmed

**REG-2 resolution (DOC):** the ROOT-README tree was updated with the GUI, `pyproject.toml`, and `uv.lock`; the duplicate section 3.4 became 3.5.

**Assessment:** I could not confirm this. **Remaining work:** confirm that the files exist and run the quickstart on a clean machine.

## G2. File-name variants for the vendor executables

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Unconfirmed

**REG-2 resolution (DOC):** standardized on `RK855_configurator.exe` and `RK855_firmware_flasher.exe`.

**Assessment:** the FLASH copy I fetched still uses `RK855_firmware flasher.exe` in its introduction and in section 2.1. **Remaining work:** check FLASH and CONFIG.

## G3. "20 effects" counts Static and Off as animations

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Closed

**REG-2 resolution (DOC):** clarified as 20 lighting modes: 18 dynamic effects, Static (`0x00`), and Backlight Off (`0x13`). Accepted.

## G4. Unexplained analysis tool ("REA") and unclear provenance

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** **Reopened**

**REG-2 resolution (DOC):** "REA tool provenance, PE imports, and disassembly validation documented across technical reports."

**Assessment:** no location is given. The item asked for a provenance record: what the tool is, what inputs it saw, and what was checked by a human or on hardware.

**Remaining work:** add a short `docs/PROVENANCE.md` (or a section) naming the tool, inputs, and checks. Record that this review found errors in claims from that workflow: the F1 snippet, the A3 overlap, the D1 opcode table, and the C1 size mismatch.

## G5. Licensing statement for redistributed vendor binaries

**Severity:** Low &nbsp;|&nbsp; **Rev 3 status:** Acknowledged

**REG-2 resolution (DOC):** MIT declared for project code and docs; vendor binaries noted as proprietary.

**Assessment:** this restates the existing README text, which is fine because there is no technical defect. Anyone planning to redistribute the repository or its vendor files should consider a `LICENSE` or `NOTICE` file and take their own legal advice. I am not a lawyer. The Rust project keeps firmware input user-supplied and keeps vendor files out of releases.

---

# 12. Group X: Contradictions Between Resolutions &nbsp; *(new in revision 3)*

## X1. C1 and C2 disagree on the header count rule

**Severity:** Medium

- C1: the `04 13` header byte is the block count **minus one**: 19 blocks (`0x4C0`) gives `0x12`.
- C2: the `04 20` header byte is `0x04` for **4** reports, which under the same rule would be `0x03`.

Either the rule differs per command, or the remap transfer has 5 blocks, or the header byte means different things. **Resolve by:** decoding both headers from captures and stating the rule per command.

## X2. C1 and C7 disagree on byte indexing

**Severity:** Low

C7 standardizes on `payload[N]` with the report ID excluded. C1 still says "Byte 8". In the CONFIG sample buffer `[0x00, 0x04, 0x13, 0, 0, 0, 0, 0, 0x12]`, `0x12` is `wire[8]`, which is **`payload[7]`**. **Resolve by:** rewriting C1 and every other "Byte N" reference in the new convention.

## X3. A1 and FLASH on what resource 4000 is

**Severity:** Medium

A1 says the vector pair `0x20000728` / `0xD9` belongs to resource 4000. FLASH 5.2 and the cross-document agreements table treat resource 4000 as the same 64 KB image as `SN32F240B_32.hex`. Both cannot be true. **Resolve by:** comparing the SHA-256 of resource 4000 with the `.bin`, and stating what resource 4000 is (a different image, an ISP stub, or the same image).

## X4. B2 and D1 on report lengths and padding

**Severity:** Medium

B2 says Report ID 0 carries both an 8-byte trigger and 64-byte config packets. D1 says the flasher pads reports to the declared `FeatureReportByteLength`. If so, the trigger may be padded to the declared length, which removes the apparent conflict but leaves the **declared length** unrecorded. **Resolve by:** recording the declared feature-report length and interface from the live descriptor, and the on-wire length of the trigger from a capture. The Rust code should read the declared length, not hard-code 8 or 64.

## X5. E1 and E2: pin conflicts and ROOT-README

**Severity:** Medium

E1 says P1.0 and P1.1 are dedicated SPI pins. E2 gives an 8 × 16 matrix without a pin map. The earlier FW-ANALYSIS pin lists put P1.0 to P1.4 on matrix rows, and ROOT-README describes LED driving by timer PWM only, with an "8 × 9" matrix. **Resolve by:** one table of pins and functions (matrix rows, matrix columns, LED rows, LED PWM channels, SPI) taken from the datasheet pin table and the firmware's pin-configuration code, then correct ROOT-README and FW-ANALYSIS.

---

# 13. Group N: New Findings &nbsp; *(new in revision 3)*

## N1. The word at `0xFFFC` is a constant marker, not a checksum

**Severity:** Medium

A2 reports that the 4-byte word at `0xFFFC` is `0xAAAA5555`. A constant pattern is a validity marker rather than a checksum. FW-ANALYSIS and FLASH call the word a "flash integrity checksum". It matters because:

1. A flasher must write it (it lies outside the main populated range), or the image may be treated as invalid.
2. If the ROM checks it at reset, a corrupt or missing marker might force ISP mode automatically, which would change the recovery picture in D4.
3. The D1 command `Get Check Sum` is a separate mechanism, and the relation between the two is unknown.

**Resolve by:** renaming it "validity marker (unconfirmed)", and testing on a sacrificial board whether the ROM changes behavior when the word is absent or wrong.

---

# 14. Cross-Document Agreements (updated)

| Item | Value | Evidence |
|------|-------|----------|
| Firmware SHA-256 (HEX / BIN) | `ba0f38cb...592f0` / `1e1be3a5...ab6` | Two READMEs agree |
| Sizes | HEX 166,843 B; BIN 65,536 B | All documents agree |
| Hex-to-bin equivalence | `objcopy --gap-fill 0xff` gives a bit-identical file | FI (author) |
| Initial vector pair of the `.bin` | SP `0x200015B0`, Reset `0x000066D1` | FI (author), subject to X3 |
| Populated extent | `0x0000`–`0xE797` plus `0xFFFC` (`0xAAAA5555`) | FI (author); hex size consistent |
| Key matrix tables | 128 positions: scan table `0x6D7B`, debounce buffers `0x2000012C`–`0x200001AC`, 128 remap tuples | FI (author); three independent sizes agree |
| LED row buffers | `0x2000096C`, `0x20000A68`, `0x20000B64`, 252 B each | FI (author) |
| Trigger bytes | `AA 42 89 5A FF 71 62 CC` | All documents; literals match in F1 |
| Bootloader PID | `0x82AA` | Two READMEs; no capture |
| Lighting packet and sequence | `04 18`, `04 11 ...`, `04 02`, `04 F0` | CONFIG and ROOT-README |
| Per-key RGB transfer | 19 × 64-byte reports, 76 × 16-byte records | EXE (author) |
| Boot ROM address | `0x1FFF0300`, remapped via SYS0 `0x40060024` | FI (author) |

---

# 15. Impact on the Rust Rewrite Plan

**Phase 1 (verification)**
- Compare the SHA-256 of flasher resource 4000 with the `.bin` (X3).
- Read the three Python tools and tabulate what they send (D5).
- Record the live descriptor dump with declared feature-report lengths and interface numbers (B2, X4).
- Capture one full vendor flasher session. It is now the single most valuable artifact.

**Phase 2 (protocol layer)**
- Use `payload[0..63]` indexing everywhere (C7). The transport prepends the report ID.
- Read the declared report length from the device instead of hard-coding 8 or 64 (X4).

**Phases 4 and 5 (remap, per-key RGB)**
- Model the per-key RGB buffer as 76 records of 16 bytes, once the record layout and the record-to-key map are captured (C1).
- Do not implement macro, media, or mouse remaps until the target-byte encoding is captured (C2).

**Phase 6 (flasher) must be rewritten around the new command set**
- Session shape from D1: enter ISP, compare password, set encryption algorithm, enable erase, erase, enable program, write data, get check sum, return to user mode. The packet formats, password, and write data path are all **unknown** and must come from the capture.
- Never send `Set Code Option` unless a capture shows the vendor tool doing so with specific values.
- Program the whole of `0x0000`–`0xE797` and the word at `0xFFFC` (A2, N1).
- Warn that flashing resets user keymaps, lighting, and macros, and offer backup (D3).
- Keep the gating on chip ID and VID/PID, and the "believed compatible, untested" wording for single-color boards (E3).
- Keep the recovery text at "may require opening the keyboard" (D4).

---

# 16. Remaining Work, in Priority Order

1. **D1, D5:** capture the vendor flasher session; extract exactly what the shipped scripts send; reconcile.
2. **B1, B2, X4:** live probe and descriptor dump on Windows and Linux; capture `wValue`, `wIndex`, `wLength`.
3. **X3, A1:** hash resource 4000 against the `.bin`; show the vector bytes.
4. **C1, C2, X1, X2:** record layout, record-to-key map, remap target-byte encoding, header rules.
5. **E3, E4:** reword E3; list the addresses checked for E4.
6. **E1, E2, X5:** one authoritative pin table.
7. **D2, D3, D4, N1:** observe enumeration; state erase scope; test marker behavior on a sacrificial board.
8. **C3, C4, C5, C6:** captures for lighting parameters, live vs persisted writes, profile slots, observed mode behavior.
9. **G1, G2, G4, G5 and the stale register header:** confirm edits and add the provenance note.

---

# Appendix A: Updated Entry Template for `PROTOCOL_STATUS.md`

```markdown
## <ID>: <Short title>

- **Status:** open | partly-resolved | mostly-resolved | closed | reopened
- **Evidence type:** FI | EXE | DOC | CAP | HW (list all that apply)
- **Claim in docs:** <paraphrase and file>
- **Evidence:** <addresses, capture file names, probe output, datasheet page>
- **Provenance:** <tool or person, method, date>
- **Hardware used:** <board variant, firmware version, OS>
- **Decision:** <what the Rust code will do>
- **Docs updated:** <yes/no, which files, commit>
```

# Appendix B: Quick Verification Commands

```bash
# Vector table of the image and of the embedded flasher resource 4000 (X3, A1)
xxd -l 16 SN32F240B_32.bin
sha256sum SN32F240B_32.bin resource_4000.bin

# Occupancy (A2)
python3 - <<'EOF'
d = open("SN32F240B_32.bin","rb").read()
print("non-FF bytes:", sum(b != 0xFF for b in d))
print("last populated below 0xFFFC:", hex(max(i for i,b in enumerate(d[:0xFFFC]) if b != 0xFF)))
print("word at 0xFFFC:", d[0xFFFC:0x10000].hex())
EOF

# Hex to bin equivalence (A2)
objcopy -I ihex -O binary --gap-fill 0xff SN32F240B_32.hex /tmp/from_hex.bin
cmp /tmp/from_hex.bin SN32F240B_32.bin && echo identical

# Device descriptors inside the image (B1)
python3 - <<'EOF'
d = open("SN32F240B_32.bin","rb").read()
i = 0
while (i := d.find(bytes([0x12,0x01,0x00,0x02]), i)) != -1:
    print(f"{i:#06x}: VID={int.from_bytes(d[i+8:i+10],'little'):#06x} PID={int.from_bytes(d[i+10:i+12],'little'):#06x}")
    i += 1
EOF

# Live HID descriptors (B2, X4)
sudo usbhid-dump -d 0c45:8018
sudo usbhid-dump -d 05ac:024f

# What do the shipped tools send? (D5)
grep -nE "0x0C45|0x05AC|0x8018|0x024F|0x82AA|0x0[1-9]|0x2[0-9A]|feature_report" tools/*.py
```

# Appendix C: Change Log

| Revision | Changes |
|----------|---------|
| 1 | Initial register: 22 items (groups A–F) over FW-README, CONFIG, FLASH, FW-ANALYSIS. (Revision 1 text said 21; the correct count was 22.) |
| 2 | Added ROOT-README. New items C6, C7, G1–G5. Updated 17 existing items. Total 29. |
| 3 | Reviewed the author's rev 2 resolutions (REG-2). Re-graded all 29 items: 6 closed, 5 mostly resolved, 10 partly resolved, 5 reopened, 2 unconfirmed, 1 acknowledged. Raised D5 to High. Added X1–X5 (contradictions between resolutions) and N1 (marker at `0xFFFC`). Added the old-versus-new opcode comparison under D1, the "limits of this review" section, evidence labels, updated agreements table, a section on the impact on the Rust plan, and a prioritized work list. Total 35. |
