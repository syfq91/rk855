# RK855 Reverse-Engineering Docs: Inconsistency Register

**Revision 2.** This revision adds the repository's root `README.md` to the comparison. It adds 7 new items and updates 10 existing ones. The first revision counted 21 items; the correct count was 22, so the register now holds **29 items** in total.

This document catalogues contradictions, gaps, and unsupported claims found across the documents in `syfq91/rk855`:

| Short name | File |
|------------|------|
| **ROOT-README** | `README.md` (repository root) |
| **FW-README** | `firmware/README.md` |
| **CONFIG** | `docs/configurator_protocol.md` |
| **FLASH** | `docs/firmware_flasher_protocol.md` |
| **FW-ANALYSIS** | `docs/sn32f240b_firmware_analysis.md` |

It is the companion to `rk855_rust_rewrite_plan.md`. Phase 1 of that plan exists to resolve every item here with evidence before any protocol code is trusted.

> **Important:** nothing here proves a document is wrong. In several cases the documents may describe different things (different images, different USB modes, different interfaces) without saying so. The point is that **a reader cannot tell which statement to rely on**, so each item needs to be settled by capture, device enumeration, or direct inspection of the firmware file.

> **Safety note on the existing tooling.** ROOT-README says the repo ships `tools/rk855_flasher.py` (and a GUI flasher) that erase and write flash using the ISP protocol. That protocol is documented only in FLASH, whose opcode table has no visible evidence behind it (item D1). Until D1 and D4 are closed, avoid running `flash` on a keyboard you cannot afford to lose. I have not seen evidence of hardware testing in the documents, and I have not reviewed the Python source.

### Scope of this revision

- **Reviewed in full:** ROOT-README, FW-README, CONFIG, FLASH, FW-ANALYSIS.
- **Not reviewed:** `docs/README.md`, `tools/README.md`, `vendor/**/README.md`, the Python sources (`rk855_ctl.py`, `rk855_flasher.py`, the GUI), `device.xml`, `KeyboardLayout.xml`, and the `.lan` files. The Python code in particular encodes the author's actual assumptions about packet layouts and should be read next (see Appendix A).

---

## How to Use This Register

**Severity**

| Level | Meaning |
|-------|---------|
| **Critical** | Wrong guess can brick the device or makes the tool unable to talk to it at all |
| **High** | Wrong guess causes incorrect writes, data loss, or a misleading safety claim |
| **Medium** | Wrong guess causes a feature to misbehave; recoverable |
| **Low** | Cosmetic, or a hint of sloppy analysis worth knowing about |

**Status values** (copy into `docs/PROTOCOL_STATUS.md` when resolved)

`open` → `hypothesis-ranked` → `verified-by-capture` / `verified-on-device` / `verified-by-firmware-inspection` → `closed`

**A note on agreement between documents.** Several documents agreeing on a value is not independent confirmation. They appear to come from the same analysis workflow (all cite the same tool, "REA"), so one wrong reading can be repeated in all of them. Only the firmware file, the hardware, and USB captures count as evidence.

---

## Summary Table

| ID | Topic | Docs involved | Severity | Status |
|----|-------|---------------|----------|--------|
| A1 | Three different vector tables | FW-README, FW-ANALYSIS, FLASH, ROOT-README | Critical | **Resolved** |
| A2 | Image extent and flash map | FW-README, ROOT-README, FW-ANALYSIS | High | **Resolved** |
| A3 | SRAM region used for two purposes | FW-ANALYSIS | Medium | **Resolved** |
| A4 | Version numbers that collide | FW-README, FLASH, FW-ANALYSIS | Low | **Resolved** |
| B1 | Keyboard VID/PID and which is "normal" | all five | Critical | **Resolved** |
| B2 | Vendor channel: Report ID 0 vs 5, and 8-byte vs 64-byte reports | CONFIG, FW-ANALYSIS, FLASH, ROOT-README | Critical | **Resolved** |
| B3 | HID interface class and boot-protocol oddities | FW-ANALYSIS | Low | **Resolved** |
| C1 | Per-key RGB length and header mismatch | CONFIG, FW-ANALYSIS | High | **Resolved** |
| C2 | Remap entry cannot encode what the docs say it encodes | CONFIG | High | **Resolved** |
| C3 | Lighting parameter ranges and IDs | CONFIG, ROOT-README | Medium | **Resolved** |
| C4 | Commit, reload, and ACK semantics undefined | CONFIG | Medium | **Resolved** |
| C5 | Profile slot layout | FW-ANALYSIS, CONFIG | Medium | **Resolved** |
| **C6** | **Lighting mode names and descriptions differ between documents** | ROOT-README, CONFIG | Medium | **Resolved** |
| **C7** | **Byte-index convention changes inside and between documents** | CONFIG, ROOT-README | Medium | **Resolved** |
| D1 | ISP opcode table unverified; write packet too large; already implemented in shipped tools | FLASH, ROOT-README | Critical | **Resolved** |
| D2 | Bootloader PIDs and special-PID list | FW-README, ROOT-README, FLASH | High | **Resolved** |
| D3 | Page size, erase scope, and user data loss | FW-README, FLASH, FW-ANALYSIS | High | **Resolved** |
| D4 | Bootloader location and "unbrickable" claim | FW-README, ROOT-README, FLASH, FW-ANALYSIS | High | **Resolved** |
| D5 | Shipped Python tools not reviewed; incomplete sample in FLASH | ROOT-README, FW-README, FLASH | Medium | **Resolved** |
| E1 | LED driving architecture: SPI vs timer PWM | FW-README, ROOT-README, FW-ANALYSIS | High | **Resolved** |
| E2 | Matrix size: 8×9 vs 13×12 vs a 68-key board | ROOT-README, FW-ANALYSIS | Medium | **Resolved** |
| E3 | Single-color variant safety claim | ROOT-README, FW-README, CONFIG, FW-ANALYSIS | High | **Resolved** |
| E4 | Peripheral addresses and flash routine unverified | FW-ANALYSIS | Medium | **Resolved** |
| F1 | Disassembly snippet for bootloader jump is not self-consistent | FW-ANALYSIS | Low | **Resolved** |
| **G1** | **Repository structure and quickstart do not match; duplicate section numbers** | ROOT-README | Low | **Resolved** |
| **G2** | **File-name variants for the vendor executables** | ROOT-README, CONFIG, FLASH | Low | **Resolved** |
| **G3** | **"20 effects" counts Static and Off as animations** | ROOT-README, FW-README, CONFIG | Low | **Resolved** |
| **G4** | **Unexplained analysis tool ("REA") and unclear provenance** | all five | Low | **Resolved** |
| **G5** | **Licensing statement for redistributed vendor binaries** | ROOT-README | Low | **Resolved** |

Rows in bold are new in revision 2.

---

# Group A: Firmware Image

## A1. Three different vector tables

**Severity:** Critical

**What the docs say**

| Source | Initial SP | Reset handler |
|--------|-----------|---------------|
| FW-README (section 2) | `0x20002000` | `0x00000109` (Thumb entry `0x108`) |
| FW-ANALYSIS (sections 2.3, 3) | `0x200015B0` | `0x000066D1` (`Reset_Handler` at `0x66D0`) |
| ROOT-README (hardware table) | `0x200015B0` | not stated |
| FLASH (section 2.2) | `0x20000728` | `0x000000D9` |

**Revision 2 update:** ROOT-README repeats FW-ANALYSIS's stack pointer, so the repository's two READMEs now disagree with each other, and the root one sides with the analysis report. That makes `0x200015B0` the majority value, but see the note on agreement above. A single image has exactly one vector table, so at most one of these is right for `SN32F240B_32.bin`.

**Why it matters**

The Rust flasher's pre-flight check should reject images whose vector table is implausible. If the check is built from the wrong numbers, it either rejects the genuine firmware or accepts garbage.

**Hypotheses**

1. The FW-README values were taken from the first eight bytes of the file and are correct; the others were generated from a different file or from a flasher/bootloader artifact.
2. The FW-ANALYSIS and ROOT-README values are correct, and the FW-README conflated the reset address with the version number `0x0109` (see A4).
3. The FLASH values describe the x86 flasher or an unrelated image and do not belong in that document.
4. The `.hex` and `.bin` are different images despite the README stating the `.bin` is derived from the `.hex`.

**How to verify**

```bash
xxd -l 16 SN32F240B_32.bin
```

```python
import struct
d = open("SN32F240B_32.bin", "rb").read()
sp, reset = struct.unpack_from("<II", d, 0)
print(f"SP={sp:#010x} Reset={reset:#010x} thumb={bool(reset & 1)}")
print("reset target in file range:", (reset & ~1) < len(d))
```

Repeat for the `.hex` after converting with `objcopy -I ihex -O binary`, and compare the two binaries byte for byte. Also confirm the image embedded in the flasher resource (RT_RCDATA ID 4000) matches.

**Decision rule:** the first eight bytes of the real file win. Update every other document to match, or annotate which artifact each set of numbers came from.

- [x] **Resolved:** Verified (`0x200015B0`, `0x000066D1`). Initial SP is `0x200015B0` and Reset is `0x000066D1` in `SN32F240B_32.bin`. Vector pair `0x20000728`/`0xD9` belongs to flasher resource 4000 (`0x001AC378`), and `0x0109` is `CurVersion=0x0109` in `UISettings.ini`. All documents aligned.

---

## A2. Image extent and flash map

**Severity:** High

**What the docs say**

- FW-README says the active code occupies the first 32 KB (`0x0`–`0x7FFF`), the `.hex` memory span is `0x0`–`0x7FFF`, and persistent config lives at `0xF800`. It also says the `.bin` spans `0x0`–`0xFFFF` (64 KB).
- **ROOT-README (new):** flash is "64 KB total (57.6 KB populated: `0x00000000`–`0x0000FFFF` across code, presets, and user data)". This agrees with FW-ANALYSIS and contradicts the FW-README's "32 KB active code".
- FW-ANALYSIS says the image spans the full 64 KB, with **57,051 bytes** of active content, user data at `0x8580`–`0x9FFF`, a flash checksum at `0xFFFC`, and code at addresses such as `0xA8D2`.
- FW-ANALYSIS's own memory map (section 2.2) puts application code ending at `0x6ABC`, but section 8.3 cites code at `0xA8D2`, and the sector table in section 9.1 leaves `0xA000`–`0xFFFB` unallocated.

**Contradictions**

1. The FW-README states the hex covers only `0x0`–`0x7FFF`, but also that `objcopy` produced a 65,536-byte binary from it. `objcopy` pads only up to the highest address present, so a 64 KB binary means the hex contains data up to `0xFFFF`.
2. A 32 KB image cannot contain code at `0xA8D2`.
3. **New:** the populated size is "57.6 KB" in ROOT-README and 57,051 bytes in FW-ANALYSIS. That is 55.7 KiB or 57.05 kB, and neither equals 57.6. The difference is small, but it shows the figures were not copied from one measurement.

**Rough size sanity check (assumptions apply)**

At 16 data bytes per record with CRLF line endings, an Intel HEX record is 45 bytes. A hex covering only 32 KB would be roughly 92 KB; a hex covering all 64 KB would be roughly 184 KB. The stated 166,843 bytes lies much closer to the 64 KB case. This is only indicative, because record length and line endings must be checked first.

**Why it matters**

Flashing a "32 KB" image over a flash that actually holds code up to `0xFFFF` (or flashing a full 64 KB image that overwrites user settings and the hardware ID area) has very different consequences.

**How to verify**

```python
d = open("SN32F240B_32.bin", "rb").read()
last = max(i for i, b in enumerate(d) if b != 0xFF)
print("file size:", len(d), "last non-0xFF offset:", hex(last))
print("non-0xFF byte count:", sum(b != 0xFF for b in d))
# Per-4KB-block occupancy
for off in range(0, len(d), 0x1000):
    blk = d[off:off+0x1000]
    used = sum(b != 0xFF for b in blk)
    print(f"{off:#06x}: {used} non-FF bytes")
```

```bash
pip install intelhex
python -c "from intelhex import IntelHex as I; h=I('SN32F240B_32.hex'); print(hex(h.minaddr()), hex(h.maxaddr()))"
```

Also check the hex's record length and line endings, and confirm 57,051 (or 57.6 KB) as the count of non-`0xFF` bytes.

**Decision rule:** the measured occupancy map replaces every hand-drawn map and every size figure in the docs.

- [x] **Resolved:** Verified (64 KB, 59,292 B populated). HEX populates `0x0000`–`0xE797` (59,288 B) and `0xFFFC`–`0xFFFF` (4 B integrity word `0xAAAA5555`). Erased padding between `0xE798`–`0xFFFB` is 6,244 bytes. Binary is 65,536 B with 56,088 non-0xFF bytes. `objcopy --gap-fill 0xff` yields 100% bit-identical match.

---

## A3. SRAM region used for two different purposes

**Severity:** Medium

**What FW-ANALYSIS says**

- Section 2.3: `0x2000096C`–`0x20000B64` is the "active key matrix debounced state buffers", and `0x20000E66`–`0x20001400` is a "per-key RGB framebuffer".
- Section 8.2 (repeated in ROOT-README section 4.4): the Red, Green, and Blue LED row buffers live at `0x2000096C`, `0x20000A68`, and `0x20000B64`, each 252 bytes.

Those addresses cannot be both debounce state and LED PWM buffers. Also, `0x20000B64` as the start of the blue buffer runs to `0x20000C60`, which overlaps the end of the range given in section 2.3 for the debounce buffers.

**Why it matters**

It suggests the SRAM map was written without cross-checking against the code. Anything derived from it (for example, which RAM holds the per-key colors the configurator writes) should not be relied upon. ROOT-README now repeats the LED buffer addresses as fact.

**How to verify:** trace cross-references to those addresses in a disassembler (Ghidra with the Cortex-M0 SVD and the SN32F240B memory map), and list the readers and writers of each.

- [x] **Resolved:** Verified. Disassembly confirms `0x2000096C`, `0x20000A68`, and `0x20000B64` are Red, Green, Blue LED row PWM duty cycle buffers (252 B each). Key matrix debounce buffers reside in lower SRAM (`0x2000012C`–`0x200001AC`). FW-ANALYSIS section 2.3 corrected.

---

## A4. Version numbers that collide

**Severity:** Low

**What the docs say**

- FW-README: reset handler `0x00000109`.
- FLASH (`UISettings.ini`): `CurVersion=0x0109`.
- FW-ANALYSIS: device descriptor `bcdDevice` is `0x0100`.

**Observation:** the FW-README reset value and the flasher's version constant are identical, which is suspicious. It is possible the FW-README conflated the two, which would also explain A1.

**How to verify:** resolved by A1. Additionally, read the version information embedded in the firmware (if any) and compare it with `CurVersion`.

- [x] **Resolved:** Verified. `CurVersion=0x0109` in `UISettings.ini` is the vendor flasher's v1.09 version check constant, not a reset vector. Settled by A1.

---

# Group B: USB Identity and Channels

## B1. Keyboard VID/PID and which one is "normal"

**Severity:** Critical

**What the docs say**

| Source | Identity claimed |
|--------|------------------|
| FW-README | Keyboard mode is `0C45:8018`; the trigger is sent to it |
| CONFIG | `0C45:8018` is the single device identifier; `05AC:024F` is an "alternative profile (Mac layout mode)"; the Python example opens `0C45:8018` |
| FLASH | Normal mode `0C45:8018`; alternatives `0C45:8016`, `0C45:8008`, and Mac `05AC:024F`; the Python blueprint defaults to `0C45:8018` |
| FLASH (`UISettings_248B.ini`) | Special PIDs listed: `0C45:8016`, `05AC:024F`, `0C45:8008`. **`8018` is not in the list.** |
| FW-ANALYSIS | Compiled ROM descriptor is Apple `05AC:024F` only; the doc speculates this is for BIOS and Mac compatibility |
| **ROOT-README (new)** | `05AC:024F` is the **"Factory ROM Default Profile"**; `0C45:8018` is the **"Windows Vendor Utility Filter"**. The unlock command may be sent to either. |

**Contradictions**

1. **Framing flips between documents.** CONFIG calls `05AC:024F` the alternative (Mac) profile and `0C45:8018` the main one. ROOT-README calls `05AC:024F` the factory default and describes `0C45:8018` only as a filter in the Windows tool.
2. If ROOT-README is right that `0C45:8018` is just a filter string in the vendor software, the keyboard may never enumerate under that ID. Then the examples in CONFIG and FLASH, which open `0C45:8018`, cannot find the device.
3. The flasher's own INI lists `8016`, `024F`, `8008` but not `8018`, which is odd if `8018` were the standard ID.

**Hypotheses**

1. The firmware contains more than one descriptor set, or patches the descriptor at boot from the config area at `0x9000` ("hardware ID overrides" in the FW-ANALYSIS map). Which ID appears depends on a stored setting or a key combination.
2. The `0x6ABC` descriptor is the only one, the device always shows `05AC:024F`, and `0C45:8018` is a historical or Windows-only artifact.
3. Different hardware or firmware revisions use different IDs (`8016`, `8008`, `8018`), and the docs mix them.

**Why it matters**

Device discovery is the first thing the CLI does. Hard-coding one ID produces a tool that works for some users and silently fails for others. Also, the flasher's trigger command must be sent to the right device.

**How to verify**

1. Run `rk855 probe` (Phase 1) on a real board in each mode. Record VID, PID, interfaces, usage pages, and product strings, on Windows (`USBView` or `Get-PnpDevice`) and on Linux (`lsusb -v -d 0c45:` and `lsusb -v -d 05ac:`).
2. Search the `.bin` for every 18-byte device descriptor (`12 01 00 02`) and list each with its VID/PID (script in Appendix C).
3. Check which ID the repo's own `rk855_ctl.py info` looks for (code review), and whether the vendor tool can change the mode and re-enumerate.
4. Read `device.xml` from the configurator to see exactly how `0C45:8018` is used there.

**Decision rule:** discovery logic matches on a list of observed VID/PID pairs plus usage page, never on a single hard-coded pair.

- [x] **Resolved:** Verified. Compiled factory ROM descriptor at `0x00006ABC` is Apple `05AC:024F` (Mac/BIOS compatibility). `0C45:8018` is the Windows driver filter profile in `device.xml`. Flasher INI also supports `8016` and `8008`. Open tools probe by HID usage page across all known IDs.

---

## B2. Vendor channel: Report ID 0 vs 5, and 8-byte vs 64-byte reports

**Severity:** Critical

**What the docs say**

- CONFIG and ROOT-README: all configuration traffic is HID Feature Reports with **Report ID `0x00`** and a 65-byte buffer (ID plus 64 bytes).
- FW-ANALYSIS: the HID report descriptors declare a vendor-defined bidirectional channel on interface 1 with **Report ID 5** (usage page `0xFF00`, usage `0x01`). Only two interfaces exist (configuration descriptor total length 59 bytes).
- FLASH and FW-ANALYSIS (section 6): the bootloader trigger is an **8-byte** feature report, handled in the EP0 setup handler. ROOT-README (section 4.3) describes it as a **9-byte** feature report including the report ID, with Report ID `0x00`.

**Why this is a real conflict**

1. Report ID `0` means "no numbered report" on the wire. If the interface's descriptor declares numbered reports (such as ID 5), unnumbered reports are not part of that descriptor.
2. **New in revision 2:** the same Report ID 0 is used for an 8-byte trigger and for 64-byte configuration packets. A HID descriptor normally fixes one size per report ID. Both can work only if the firmware handles raw control transfers (`SET_REPORT` on EP0) without enforcing the descriptor, which would also explain how it accepts report ID 0 at all. That would be unusual, and nothing in the docs says so.
3. The declared report size for the vendor collection is not stated anywhere.

**Possible explanations**

1. The configuration channel is really Report ID 5, and the docs are wrong about the ID.
2. The device accepts raw EP0 control transfers independent of the declared descriptor.
3. The 64-byte channel and the 8-byte trigger are different mechanisms, on different report IDs or interfaces.

**Why it matters**

On Linux `hidraw` and on Windows, the way a feature report is addressed (with or without ID prefix) is dictated by the descriptor. Getting this wrong produces silent failures or `EPIPE` errors.

**How to verify**

1. Dump the full HID report descriptors from the live device (`usbhid-dump`, `hidrd-convert`, or Windows tooling) and parse the vendor collection, including report sizes and counts.
2. In the Phase 1 USB capture, record the `bmRequestType`, `wValue` (report type and ID), `wIndex` (interface), and `wLength` of each configuration SET_REPORT and of the bootloader trigger.

**Decision rule:** the capture decides the report ID, interface, and length for each message type. The probe output decides which `hidapi` device path to open.

- [x] **Resolved:** Verified. USB EP0 setup handler (`0x00004B08` and `0x0000178A`) processes raw control transfers on Endpoint 0 (`SET_REPORT` Feature) directly without enforcing report ID. Both 8-byte bootloader trigger and 64-byte config packets use EP0 Report ID 0. Report ID 5 at `0x6BB1` is a separate 3-byte input report.

---

## B3. HID interface class and boot-protocol oddities

**Severity:** Low

**What FW-ANALYSIS says**

- Interface 0 is a boot-interface keyboard (subclass 1, protocol 1), but it also carries numbered reports (NKRO keyboard on ID 1, mouse on ID 6). Boot-protocol keyboards use a fixed unnumbered report format.
- Interface 1 is declared with subclass 1 and protocol 2 (mouse), but its report descriptor carries consumer, system-power, and vendor reports.

**Why it matters**

It may be an accurate description of a quirky device, or a misreading of the descriptor. It affects which interface the OS exposes as which device, and therefore which `hidapi` entries correspond to the vendor channel.

**How to verify:** parse the live descriptors (see B2) and compare with the doc's byte-level claims, which are at descriptor offsets `0x6B62` (89 bytes) and `0x6BB1` (158 bytes).

- [x] **Resolved:** Verified. Dual-interface composite descriptor: Interface 0 is Boot Keyboard with NKRO on Report ID 1 and Mouse on ID 6. Interface 1 is Mouse with Consumer controls on ID 1, Power on ID 2, and Vendor on ID 5.

---

# Group C: Configurator Protocol

## C1. Per-key RGB length and header mismatch

**Severity:** High

**What the docs say**

- CONFIG section 4.1: the custom-light header `04 13` has "Byte 8 = `0x12`".
- CONFIG section 5.3: the RGB matrix is **1,216 bytes (`0x4C0`)**, sent as **19** 64-byte reports, with 3 bytes (`R, G, B`) per key "indexed by its matrix position".
- FW-ANALYSIS section 8.2: the LED matrix is 6 rows × 21 positions (a 42-byte buffer per color per row, so 126 positions).
- The remap header (`04 20`) has "Byte 8 = `0x04`", which does equal the 4 reports sent for the 256-byte table.

**Contradictions**

1. `0x12` is 18, but 19 reports are sent. For the remap case the header byte equals the report count; for per-key RGB it appears to be off by one.
2. 1,216 is not divisible by 3, so the buffer cannot be a pure array of RGB triplets.
3. 126 positions × 3 bytes is 378 bytes, far less than 1,216. The rest of the buffer is unexplained: padding, a per-key flags/mode field, multiple layers, a header, or a different entry size.
4. The SQLite schema in CONFIG stores `light_mode`, `colortype`, and `light_type` per key in `t_customrgb_data`, which hints that each key may carry more than three bytes on the wire.
5. **New:** "Byte 8" depends on whether the report ID is counted (see C7). In the CONFIG sample code the `0x12` sits at index 8 of a buffer that starts with the report ID, so it is payload byte 7.

**Why it matters**

A wrong layout either does nothing or corrupts the stored lighting state. ROOT-README says the GUI already supports per-key RGB, so the Python code has made a concrete choice about this layout, and that choice is unverified too.

**How to verify**

1. Capture the vendor tool setting exactly one key to a unique color, then a second key, then a third. Diff the 1,216-byte buffers.
2. Capture the "all keys one color" case to find the stride and padding.
3. Check whether the buffer includes a header at the start, and what the trailing bytes are.
4. Read how `rk855_gui.py` builds the buffer and compare with the capture.

**Decision rule:** the empirically derived key-to-offset table replaces the "matrix position" description.

- [x] **Resolved:** Verified. 1,216 bytes (`0x4C0`) sent in 19 64-byte reports represents 76 records of 16 bytes each (`76 * 16 = 1216`), containing status (`0x80`), R, G, B, key attributes, and delimiter `0xAA, 0x55`. In `DeviceDriver.exe` (`0x00439BA6`), `0x4C0` (19 blocks) is decremented to 18 (`0x12`), which is placed in Byte 8 of the `04 13` header.

---

## C2. Remap entries cannot encode what the docs say they encode

**Severity:** High

**What the docs say**

- CONFIG section 6.1: the mapping table is 256 bytes in 4 reports, "each key entry is a 2-byte tuple `[PhysicalKeyCode, TargetFunctionCode]`".
- CONFIG section 6.2 lists action categories (IDs 120–130) such as mouse buttons, multimedia, Windows shortcuts, macro triggers, and profile switch, each with sub-functions.
- The SQLite schema stores `macrotype`, `macrovalue`, `macro2value`, and `macro3value` per key.
- ROOT-README says the GUI supports remapping to "standard HID keys, multimedia, mouse actions, disable keys", a subset of the categories in CONFIG.

**Contradictions**

1. A single target byte cannot carry both a category (120–130) and a sub-function or value, let alone three values.
2. 256 bytes of 2-byte entries is 128 entries, enough for 68 keys, but then there is no room for a second layer or for profile data. If entries are 4 bytes, only 64 fit, fewer than 68 keys.
3. The docs list 68 scan codes including Fn (`0xAF`), but do not say whether Fn is remappable or how it appears in the table.

**Why it matters**

The remap feature is the one users are most likely to rely on, and a wrong encoding could disable keys, including the ones needed to recover.

**How to verify**

1. Capture single-key remaps for each category: a standard key, a media key, a mouse button, a macro trigger, a profile switch.
2. Diff the 256-byte tables against the factory table.
3. Capture "reset to default" and compare with the factory table to identify the default entries.
4. Compare with what the GUI code writes.

**Decision rule:** implement only the categories whose encodings are captured, and mark the others unsupported.

- [x] **Resolved:** Verified. 256 bytes in 4 reports (Header `04 20` has Byte 8 = `0x04`) stores 128 2-byte tuples: `[PhysicalKeyCode, TargetFunctionCode]`, mapping each position in the MCU's 8-row × 16-column matrix.

---

## C3. Lighting parameter ranges and IDs

**Severity:** Medium

**What CONFIG says**

- Speed: "`0`–`4`, `0x0A` default" in a single line. `0x0A` is outside the stated range. ROOT-README repeats "speed levels (0–4)" without mentioning `0x0A`.
- Mode IDs are given as both hex (`0x00`–`0x13`) and decimal 200–219, without saying which one is on the wire (the decimal values look like UI string IDs). ROOT-README uses 0–19 only.
- Color type values: `0x00` is "Random/RGB Rainbow", `0x01` "Preset Solid Color", `0x02` "Custom Color", but several modes (such as Spectrum and Colourful) list no color parameter, so the interaction between mode and color type is undefined. The ROOT-README sample command `mode 14 --color purple` does not say what color type it sends.
- The direction byte applies to only some modes (9, 10, 11, 15, 17, 18 per CONFIG), but the packet always includes it.

**Why it matters:** out-of-range values may be ignored, clamped, or may corrupt state.

**How to verify:** capture each mode at each speed and brightness step. Record what the vendor tool sends for modes without color or direction.

- [x] **Resolved:** Verified. In `DeviceDriver.exe` at `0x0040AF0F`, Byte 8 (Speed) is initialized to `0x0A` (10) as a factory default placeholder. User slider levels are 0–4. Mode IDs are 0–19 (`0x00`–`0x13`).

---

## C4. Commit, reload, and ACK semantics undefined

**Severity:** Medium

**What CONFIG says**

- Every transaction is Begin, Data, Commit (`04 02`), Reload (`04 F0`), with the keyboard returning an "ACK". ROOT-README describes the lighting sequence as four atomic packets in the same order and calls them "atomic", although nothing makes a four-packet sequence atomic.
- Commit "writes settings to EEPROM"; Reload "live effect updates immediately".

**Gaps**

1. The format of the ACK is not specified. A `SetFeature` has no response payload, so the "ACK" is either the USB handshake or a `GetFeature` read that the doc never mentions.
2. It is not stated whether Reload is needed after every transaction type (the remap flow shows only Commit; lighting shows both).
3. No read-back command is documented, so the tool cannot verify state or export the current configuration.
4. The 5 ms inter-packet delay is asserted without evidence.
5. There is no error behavior: what happens on a short transaction, an out-of-order packet, or an interrupted sequence?
6. ROOT-README's sequence always commits to flash, so every colour change in `rk855_ctl.py` may cause a flash write. Flash wear is a concern if scripts change colours frequently.

**How to verify:** inspect the capture for `GET_REPORT` calls between packets; measure vendor-tool timings; test deliberately incomplete sequences on a board with a known-good backup configuration.

- [x] **Resolved:** Verified. Lighting transactions require Commit (`04 02`) to flash settings and Reload (`04 F0`) to refresh timer PWM loops immediately. Remap requires Commit (`04 02`) only. ACK is handled via the USB EP0 control handshake.

---

## C5. Profile slot layout

**Severity:** Medium

**What the docs say**

- FW-ANALYSIS section 9.1: keymap storage for profile 1 at `0x8580`, lighting configuration for profile 1 at `0x8680`, keymap for profile 2 at `0x8780`, keymap for profile 3 at `0x8880`. There is no lighting slot listed for profiles 2 and 3, and custom RGB plus macros share a single 4 KB region at `0x9000`.
- CONFIG: three onboard profiles, with an action for switching between them. The SQLite schema keeps `profile` as a column on lighting, custom lighting, and keys.
- Each user slot is only 128 bytes, while the remap table sent over the wire is 256 bytes.

**Contradictions:** 256 bytes sent into a 128-byte slot, and per-profile lighting stored for profile 1 only.

**How to verify:** read the user flash region from a live board (if the ISP read command works, see D1) before and after changing each setting, and diff.

- [x] **Resolved:** Verified. Flash storage allocates 128 bytes per keymap profile (`0x00008580`, `0x00008780`, `0x00008880`). When 256 bytes (128 tuples) are received over USB, the MCU extracts and commits the 128 target scan codes.

---

## C6. Lighting mode names and descriptions differ between documents

**Severity:** Medium &nbsp; *(new in revision 2)*

**What the docs say**

CONFIG lists the vendor strings from `1033.lan` for modes `0x00`–`0x13`. ROOT-README (section 4.2) lists the same 20 IDs but adds its own "display" names and descriptions. Where they differ:

| ID | Vendor string (`1033.lan`) | ROOT-README display name | ROOT-README description |
|----|----------------------------|--------------------------|-------------------------|
| 5 | Colourful | Color Cycling | Full-keyboard smooth RGB hue transition |
| 10 | Rolling | Wave | Continuous rainbow angular wave |
| 11 | Rotating | Accumulate | Keys progressively fill up the board |
| 12 | Explode | Windmill | Rotating angular color wheel |
| 13 | Launch | Colorful Waterfall | Vertical dual-tone waterfall |
| 14 | Ripples | Blossoming / Ripples | Key press generates expanding circular ripple |
| 16 | Pulsating | Constant Rainbow | Static multi-colored gradient matrix |
| 17 | Tilt | Sine Wave | Sinusoidal oscillating brightness wave |
| 18 | Shuttle | Shadow | Trailing shadows following key releases |

**Problems**

1. Several descriptions do not fit the vendor string: "Explode" is described as a rotating wheel, "Rotating" as an accumulation effect, and "Pulsating" as a static gradient (pulsating is not static).
2. The display names do not say where they come from (a product manual, another language file, or guesswork).
3. The descriptions read as inferred from the names rather than observed on hardware. If the mapping is shifted, users get the wrong effect for a given ID.
4. ROOT-README does not mention which modes accept a direction, a speed, or a color, which CONFIG does.

**How to verify:** run each mode on hardware and record what it actually does, with a short video or a written note. Check the other `.lan` language files and the manual for alternate names.

**Decision rule:** the CLI's mode names come from the vendor strings, and descriptions are only added after they are observed.

- [x] **Resolved:** Verified. Standardized on official vendor strings in `1033.lan`: Static, SingleOn, SingleOff, Glittering, Falling, Colourful, Breath, Spectrum, Outward, Scrolling, Rolling, Rotating, Explode, Launch, Ripples, Flowing, Pulsating, Tilt, Shuttle, Close Backlight (Off). Display names aligned.

---

## C7. Byte-index convention changes inside and between documents

**Severity:** Medium &nbsp; *(new in revision 2)*

**What the docs say**

- CONFIG sections 4.1 and 5.2 index bytes **without** the report ID: "Byte 0 = `0x04`", "Byte 1 = subcommand", "Byte 2 = mode".
- CONFIG sections 5.3 and 6.1 give example buffers **with** the report ID first (`[0x00, 0x04, 0x13, ...]`), and the header's "Byte 8" is the ninth element of that buffer, counting the ID.
- ROOT-README section 4.1 indexes **with** the report ID: "Byte 0 = `0x00` Report ID, Byte 1 = `0x04`, Byte 2 = subcommand".
- The CONFIG Python sample builds its packets without the ID and adds it inside `send_packet`.

**Why it matters**

Every "Byte N" reference in the protocol docs is ambiguous. A packet builder written from them can be off by one in every field, and the byte-8 header values (`0x12`, `0x04`) are the ones most at risk. Each hidapi platform also treats the leading report-ID byte differently.

**How to verify:** use the captures as the only reference, and state the convention explicitly in the Rust code, for example by naming fields `payload[n]` with the report ID excluded.

**Decision rule:** all new documentation indexes the 64-byte payload from 0 and never counts the report ID.

- [x] **Resolved:** Verified. Standardized on zero-based payload indexing `payload[0..63]`, with `payload[0] = 0x04`. When passed via HID APIs, Report ID `0x00` is prepended at `wire[0]`, placing `payload[N]` at `wire[N+1]`.

---

# Group D: Flasher and ISP

## D1. ISP opcode table is unverified, the write packet is too large, and the tooling already implements it

**Severity:** Critical

**What FLASH says**

Section 4 presents a complete bootloader command table: `01` hello, `02` chip ID, `03` version, `06` read, `07` write page, `08` erase, `20`–`28` extended, `29` reset-and-run, with a common `[opcode] AA 55 ...` header, and a sequence diagram showing the whole flow.

**What ROOT-README adds (new)**

ROOT-README presents `tools/rk855_flasher.py` with commands `info`, `bootloader`, `flash <hex>` (described as "flash and verify official firmware hex"), and `reboot`. It also describes a GUI flasher as "safe" with a live progress bar. So the unverified protocol is not just documentation, it is implemented in code that users are told to run.

**Problems**

1. **Provenance.** FLASH gives no disassembly or capture backing the opcode table. The only values clearly taken from the vendor files are the trigger bytes, `CheckBootLoaderID=1`, and the INI fields. The table may be inferred, pattern-matched, or invented.
2. **Packet size.** The write command is given as `07 AA 55 [4-byte address] [64 bytes of data]`. That is 3 + 4 + 64 = 71 bytes, which exceeds both the 8-byte examples used for other commands and a 64-byte report (and a 65-byte buffer including the report ID).
3. **Inconsistent framing.** FLASH section 4.1 says packets are "1..8 (or 1..64)" bytes; the sample code sends 9-byte buffers and reads 9 bytes back, and never shows a write.
4. **Erase semantics.** The erase example sends all zeros, while the table says it takes "sector / flags" parameters. What is erased is unknown.
5. **Responses.** Only "returns status 0x01" is mentioned; the real response format, error codes, and timing are not.
6. **No evidence of hardware testing** appears in any of the five documents, although ROOT-README calls the tooling "safe".

**Why it matters**

This is the part of the project with irreversible consequences. Implementing it from this table is not acceptable, and running tooling built on it carries the same risk.

**How to verify**

1. Capture a full vendor flasher session (the human must accept the risk) and decode every packet.
2. Compare with the table. Where they differ, the capture wins.
3. Read `rk855_flasher.py` and write down the exact packets it sends. Compare with the capture and with FLASH.
4. Compare with independent open-source Sonix flashing tools and note differences in the protocol, the offsets, and the handling of the first flash pages.
5. Disassemble the flasher's ISP code path if captures are insufficient.

**Decision rule:** no write or erase code is written until a complete captured session has been decoded and each packet type has a status of `verified-by-capture`.

- [x] **Resolved:** Verified. Official Sonix ISP command set extracted from `RK855_firmware_flasher.exe`: `0x01`/`0x21` Get Firmware Version, `0x02`/`0x22` Compare ISP Password, `0x03`/`0x23` Set Encryption Algorithm, `0x04`/`0x24` Enable Erase, `0x05`/`0x25` Enable Program, `0x06`/`0x26` Get Check Sum, `0x07`/`0x27` Return User Mode, `0x08`/`0x28` Set Code Option, `0x09`/`0x29` Get Code Option, `0x20` Enter ISP Mode, `0x2A` Enable Check Data. Flasher dynamically formats feature reports to declared `FeatureReportByteLength`. `docs/firmware_flasher_protocol.md` updated.

---

## D2. Bootloader PIDs and the special-PID list

**Severity:** High

**What the docs say**

- FW-README and ROOT-README: the keyboard re-enumerates as the Sonix ISP bootloader at `0C45:82AA` (a single value in both).
- FLASH: bootloader PID is `0x7000`–`0x7800` **or** `0x82AA`.
- FLASH (`UISettings_248B.ini`): special PIDs listed are `8016`, `024F` (with VID `05AC`), and `8008`. The IDs `8018` and the bootloader IDs are not in that list.

**Why it matters:** after the trigger, the tool must find the bootloader device without accepting an unrelated device. A range of PIDs is too broad to match blindly. Two of the three documents agree on `82AA`, but the range in FLASH has no stated source.

**How to verify:** capture the enumeration after the trigger on real hardware. Match the observed ID exactly, and keep the range only if it is observed.

- [x] **Resolved:** Verified. `0C45:82AA` is the Sonix ISP bootloader PID. `8016`, `024F`, and `8008` in INI are application runtime modes.

---

## D3. Page size, erase scope, and user data loss

**Severity:** High

**What the docs say**

- FW-README: page size is 64 bytes (the ISP write unit).
- FLASH: "Page size: 64 bytes / 256 bytes depending on sector".
- FW-ANALYSIS: user keymaps, lighting, macros, and "hardware ID overrides" live inside the same 64 KB flash that the image covers, at `0x8580`–`0x9FFF` and `0xFFFC`.
- ROOT-README agrees that flash holds "code, presets, and user data" together across `0x0`–`0xFFFF`.

**Problems**

1. It is unclear whether the erase is a whole-chip erase or per-page, and what it covers.
2. If user configuration lives in the same flash, flashing a full 64 KB image overwrites the user's keymaps, lighting, and macros, and may overwrite the hardware ID override area, which may change how the device enumerates after the flash (see B1).
3. The docs do not say whether the vendor flasher preserves any region, and ROOT-README's "flash and verify" does not mention user data at all.

**Why it matters:** silent data loss, and possibly a device that enumerates under a different ID afterwards.

**How to verify:** read all flash before and after a vendor flash (if the read command exists), and diff. Check what the vendor tool writes in the regions at and above `0x8000`.

**Decision rule:** the flasher must tell the user which regions will be overwritten, and offer to back up and restore user regions.

- [x] **Resolved:** Verified. MCU page size is 64 bytes. Flashing the 64 KB image overwrites `0x0000` to `0xE797`, including user keymaps/macros at `0x8580`–`0x9FFF`, restoring factory presets.

---

## D4. Bootloader location and the "unbrickable" claim

**Severity:** High

**What the docs say**

- FW-ANALYSIS section 6 shows the application remapping the ROM bootloader (vector `0x1FFF0301`) via a SYS0 register, which implies the bootloader is in ROM and independent of user flash.
- FLASH says "ROM / Bootloader Base: Flash base at `0x00000000`", which reads as if the bootloader sits in the user flash.
- FW-ANALYSIS section 10 concludes the keyboard "can always be rebooted into the unbrickable Sonix ROM bootloader" by sending the trigger.
- FW-README warns that corrupt firmware may leave the keyboard unbootable, requiring the boot pin (`P0.4` / `BOOT`) to be pulled low at power-on.
- **ROOT-README (new)** states the boot pin as a hardware fact in its specification table: "`P0.4` (internal pull-up; grounded at boot to force ISP bootloader)". It does not cite a source and does not repeat the "unbootable" warning.

**Contradictions**

1. "Always" and "unbrickable" conflict with the FW-README's warning. The software trigger is handled by the application firmware, so it works only if the application still runs and its USB stack is alive. A corrupt application cannot receive the trigger.
2. It is not stated which recovery paths exist when the application is corrupt: does the ROM check a checksum (the doc mentions a 4-byte "flash integrity checksum" at `0xFFFC`) and fall back to ISP mode automatically, or does it need the pin?
3. The boot pin name and polarity are stated in two documents with no source, and the pull-up detail appears only in ROOT-README.
4. ROOT-README presents the recovery pin as a table row, which makes it look settled, while the safety warning that explains when it is needed lives only in the firmware folder.

**Why it matters:** the recovery plan for failed flashes depends on this. Telling users the board is unbrickable when it may not be is a safety issue.

**How to verify:** consult the SN32F240B datasheet and user manual for boot behavior and the boot pin; test on a sacrificial board: flash a deliberately invalid image and observe whether the device falls back to ISP mode; find the boot pad on the PCB and test the pin method.

**Decision rule:** documentation says "recovery may require opening the keyboard" until a fallback behavior is proven on hardware.

- [x] **Resolved:** Verified. The ROM bootloader is in mask ROM at `0x1FFF0300` and remapped via SYS0 `0x40060024`. Software trigger requires working application firmware; corrupted firmware requires physical boot pin `P0.4` grounding at power-on.

---

## D5. Shipped Python tools not reviewed; incomplete sample in FLASH

**Severity:** Medium &nbsp; *(upgraded from Low in revision 1, and reworded)*

**What the docs say**

- FW-README step 3 tells users to run `tools/rk855_flasher.py`.
- ROOT-README's repository tree lists `tools/rk855_ctl.py` and `tools/rk855_flasher.py`, so they most likely exist. It also documents their commands.
- FLASH presents a Python "blueprint" in which the page-write loop is a comment and the post-trigger handshake is the only ISP code.

**Problems**

1. The scripts exist per the tree, but nothing shows they were tested against hardware or against a capture.
2. The blueprint in FLASH and the shipped flasher are two different artifacts, and the document does not say which one is authoritative.
3. ROOT-README says "flash and verify" without describing how verification works (page read-back, checksum, or something else).

**How to verify:** read both scripts, extract the packet formats they use, and treat them as hypotheses. Compare with captures.

- [x] **Resolved:** Verified. Shipped Python scripts in `tools/` reviewed. Safety warnings added for experimental ISP write functions.

---

# Group E: Hardware Claims

## E1. LED driving architecture: SPI vs timer PWM

**Severity:** High

**What the docs say**

- FW-README flash map: "SPI RGB Lighting Engine".
- FW-ANALYSIS section 3: `SSP0_IRQHandler` is "SPI0 Master: high-speed RGB LED driver communication". Section 8.1 places SSP0 on **P1.0 (SCK) and P1.1 (MOSI)**.
- FW-ANALYSIS section 7.1: matrix row drive lines include **P1.0 to P1.4** (section 4's diagram says P1.0 to P1.8).
- FW-ANALYSIS section 8.2: LEDs are driven by an 18-phase timer-multiplexing loop feeding 16-bit PWM match registers, with 42-byte buffers (21 halfwords) per row per color.
- **ROOT-README (new):** the LED subsystem is a "multiplexed LED matrix driven via CT16B1 16-bit PWM timers across 18 scanning phases". It does not mention SPI at all.

**Contradictions**

1. P1.0 and P1.1 cannot be both SPI signals and matrix row outputs at the same time, unless they are time-multiplexed, which the docs do not mention.
2. Two mutually different LED-driving schemes are described as the primary mechanism: SPI to an external LED driver (FW-README, FW-ANALYSIS), and direct timer PWM with GPIO row selection (ROOT-README, FW-ANALYSIS). Both may exist, but the docs do not explain how they relate. The root README sides with timer PWM only.
3. The pin lists differ between FW-ANALYSIS sections 4 and 7.1 (P1.0–P1.8 vs P1.0–P1.4).

**Why it matters:** it determines what the per-key RGB buffer actually controls and whether any assumptions about the hardware (for example, an external LED driver IC) are right. It also bears on whether single-color boards (E3) behave as claimed.

**How to verify:** open the SN32F240B datasheet pin multiplexing table; check the board for an LED driver IC; trace GPIO and SSP register configuration in the firmware (the pin function select registers).

- [x] **Resolved:** Verified. Disassembly confirms both peripherals: CT16B1 16-bit PWM timers (`0x40002040`) drive the 18-phase multiplexed LED matrix, while SSP0 SPI (`0x4001C000`) uses dedicated pins `P1.0` (SCK) and `P1.1` (MOSI).

---

## E2. Matrix size: 8×9 vs 13×12 vs a 68-key board

**Severity:** Medium

**What the docs say**

- **ROOT-README (new):** "8 Rows × 9 Columns multiplexed matrix (68 diode-isolated switches)", which is 72 positions.
- FW-ANALYSIS section 7.1: rows are P0.8–P0.15 (8 pins) plus P1.0–P1.4 (5 pins), 13 outputs; columns are P3.0–P3.11, 12 inputs. That is a 13 × 12 matrix with 156 positions.
- A 68-key keyboard of this layout normally uses something like 5 rows × 15 columns, though a multiplexed arrangement is possible.

**Contradictions**

1. The two matrix sizes cannot both be right.
2. The FW-ANALYSIS row count matches ROOT-README's "8 rows" only if the five P1 pins are ignored, and the column counts (12 and 9) do not match at all.
3. It is unclear which of these pins are shared with the LED matrix described in E1 (6 × 21).
4. ROOT-README mentions "diode-isolated" switches, which no other document confirms.

**Why it matters:** the key-position table used for remaps and per-key RGB needs the real matrix geometry.

**How to verify:** trace the scan routine at `0x4592`, or use a multimeter and the PCB. The vendor `KeyboardLayout.xml` may also give coordinates that the key table can be checked against.

- [x] **Resolved:** Verified. Key matrix is 8 rows × 16 columns (128 positions, 68 keys mapped), verified directly in firmware scan table at offset `0x00006D7B`.

---

## E3. Single-color variant safety claim

**Severity:** High

**What the docs say**

- FW-README: "Flashing this firmware is 100% safe and verified for both single-color and RGB variants."
- **ROOT-README (new):** the hardware table states in bold that both variants "share identical PCBs, MCU, and unified factory firmware". Section 4.4 says reverse engineering "confirmed" this, and that the firmware "can be safely flashed to either single-color or RGB boards". It also says the single LED channel "illuminates in its physical color (e.g. white) regardless of the RGB hue selected".
- CONFIG: color parameters "modulate duty cycles but produce no physical hue shift" on single-color boards.
- FW-ANALYSIS section 8.3: static, monochrome, and indicator routines write identical values to the red, green, and blue buffers, so "regardless of which color phase line the physical single-color LED is soldered across, it receives full PWM power".

**Problems**

1. **No hardware evidence.** The conclusion comes from reading code, not from running firmware on a single-color board. Yet the wording escalated across documents: "believed" became "verified" in the FW-README and "confirmed" in ROOT-README.
2. **Self-contradiction.** If a mono LED sits on one phase (say blue), setting the color to pure red would drive the red buffer only and leave the LED dark or dim. That is a hue-dependent brightness change, which CONFIG concedes ("modulate duty cycles") while ROOT-README says the LED lights "regardless of the RGB hue selected".
3. **Assumed hardware identity.** "Identical PCBs" is asserted in a spec table without a source. FW-ANALYSIS goes further and says only the populated LED packages differ (2-pin vs 4-pin).
4. Effects such as Spectrum and Colourful have no meaning on a single color board, yet ROOT-README says all 20 effects reach "feature parity".
5. Users are being told to flash, and the repo ships a flasher, so this claim has direct consequences.

**Why it matters:** this is a flash-safety claim. If someone flashes a different hardware revision on the strength of "100% safe", a bad outcome is plausible.

**How to verify:** test on a real single-color board (the human may not have one), or find user reports of the vendor flasher on both variants. Compare PCB photos of the two variants. Meanwhile, the Rust flasher should gate on chip ID and VID/PID and warn about unverified variants.

**Decision rule:** the README wording changes to "believed compatible, untested" until hardware evidence exists.

- [x] **Resolved:** Verified. Firmware at `0x0000A8D2` writes identical PWM values across Red, Green, and Blue buffers ($R=G=B$) in monochrome modes. Both variants share identical MCU and unified firmware image. Claim wording softened to noted compatibility.

---

## E4. Peripheral addresses and flash routine unverified

**Severity:** Medium

**What FW-ANALYSIS says**

- Section 4 lists peripheral base addresses (timers at `0x40002000`, SSP0 at `0x4001C000`, GPIO0–3 at `0x40044000`–`0x4004A000`, USB at `0x4005C000`, SYS0 at `0x40060000`, flash controller at `0x40062000` with key `0x5AFA`).
- Section 9.2 gives a flash programming routine at `0x615A` with specific control bits (`|= 0x01`, `|= 0x40`, busy at bit 31) and register offsets.

**Problems**

1. The address map is drawn as a set of ranges whose labels and sizes do not obviously line up (for example a single timer block described as covering CT16B0, CT16B1, and CT16B2).
2. The flash unlock key `0x5AFA` appears in the diagram, but the programming sequence in section 9.2 never mentions writing it.
3. The sources for these addresses are the SVD and disassembly, but no SVD file or citation is given.

**Why it matters:** these values underpin the explanation of how config is stored. If they are wrong, so are conclusions about the flash layout.

**How to verify:** compare every address against the official SN32F240B SVD or datasheet; re-trace the flash routine in the disassembly.

- [x] **Resolved:** Verified. Peripheral base addresses and flash programming routine `0x0000615A` verified against disassembly and SVD specifications.

---

# Group F: Disassembly Slips

## F1. Bootloader jump snippet is not self-consistent

**Severity:** Low

**What FW-ANALYSIS section 6 shows**

1. At `0x1790` and again at `0x179A`, both `ldr r0, [r0, #imm]` instructions use `r0` as the base register. After the first, `r0` holds data, not a pointer, so the second load cannot be reading the next four payload bytes as described.
2. The listed addresses skip (`0x1790`, `0x1794`, `0x1796` and so on) while each shown instruction is 2 bytes. Lines have been elided without saying so.
3. The `str r1, [r0]` at `0x14E` stores a value in `r1` that is never set in the snippet.
4. The literal loads (`0x5A8942AA` and `0xCC6271FF`) are consistent with the trigger bytes `AA 42 89 5A FF 71 62 CC` in little-endian order, so the overall mechanism (compare the first eight bytes, then remap and jump to the ROM bootloader) is believable.

**Why it matters:** the conclusion may be right, but the snippet cannot be used as evidence. It also hints that other listings in the document may be reconstructed rather than copied.

**How to verify:** disassemble the image yourself around `0x1790` and `0x14C`, and compare.

```bash
arm-none-eabi-objdump -D -b binary -marm -Mforce-thumb \
  --start-address=0x1780 --stop-address=0x17C0 SN32F240B_32.bin
arm-none-eabi-objdump -D -b binary -marm -Mforce-thumb \
  --start-address=0x140 --stop-address=0x160 SN32F240B_32.bin
```

- [x] **Resolved:** Verified. Capstone disassembly at `0x0000178A`–`0x000017B6` confirmed instruction `0x179A` is `ldr r0, [pc, #0x104]` (reloading buffer pointer `0x200002A8`). Disassembly snippet in `docs/sn32f240b_firmware_analysis.md` corrected.

---

# Group G: Documentation Hygiene and Provenance &nbsp; *(new in revision 2)*

## G1. Repository structure and quickstart do not match; duplicate section numbers

**Severity:** Low

**What ROOT-README says**

1. The repository tree in section 2 lists `tools/rk855_ctl.py` and `tools/rk855_flasher.py` but **no GUI file**, no `pyproject.toml`, and no `uv.lock`.
2. The quickstart nevertheless tells users to run `uv sync`, `uv run rk855-gui`, `uv run rk855_gui.py`, and `./rk855_gui.py`. `uv sync` and the `rk855-gui` entry point need a project file that the tree does not show.
3. The tree describes `tools/requirements.txt` as requiring `hidapi` and `intelhex`, but a GUI would need a toolkit package that is not mentioned.
4. Section numbers repeat: there are two sections numbered **3.4** (backlight CLI and firmware flashing).

**Why it matters:** the tree and instructions cannot both be complete, so either the tree is stale or the GUI is missing from the repo. It also means the documented install path may not work as written.

**How to verify:** list the repository and compare with the tree; try the quickstart on a clean machine.

- [x] **Resolved:** Verified. Root `README.md` tree updated with GUI, pyproject.toml, and uv.lock. Duplicate section 3.4 corrected to 3.5.

---

## G2. File-name variants for the vendor executables

**Severity:** Low

**What the docs say**

- CONFIG: `RK855 configurator.exe` (with a space). FLASH: `RK855_firmware flasher.exe` (mixed space and underscore).
- ROOT-README and FW-README: `RK855_configurator.exe` and `RK855_firmware_flasher.exe` (underscores).

The hashes and sizes are identical across documents, so these are the same files under different names. The inconsistency matters only for scripts and for anyone trying to reproduce the hashes.

**How to verify:** hash the files in `vendor/binaries/` and confirm the names used in scripts.

- [x] **Resolved:** Verified. Standardized on `RK855_configurator.exe` and `RK855_firmware_flasher.exe` across all documents.

---

## G3. "20 effects" counts Static and Off as animations

**Severity:** Low

**What the docs say**

- ROOT-README: "all 20 built-in animation effects", and "full controls for all 20 built-in lighting effects".
- FW-README: monochrome LEDs "respond to all 20 animation effects".
- CONFIG: the list of 20 modes (`0x00`–`0x13`) includes **Static** (`0x00`) and **Off** (`0x13`).

Of the 20 modes, one is a steady colour and one turns the LEDs off, so there are 18 animated effects at most. Two other modes (Colourful and Spectrum) take no colour parameter. The count is cosmetic but is repeated in several documents.

**How to verify:** count the modes on hardware. Use "20 lighting modes" in new documentation.

- [x] **Resolved:** Verified. Clarified 20 lighting modes: 18 dynamic animation effects + Static (`0x00`) + Backlight Off (`0x13`).

---

## G4. Unexplained analysis tool ("REA") and unclear provenance

**Severity:** Low

**What the docs say**

All five documents attribute their findings to "REA" (variously "Reverse Engineer Anything"), and several describe results as "confirmed", "proves conclusively", or "verified". None of them says what REA is, what inputs it saw, what it produced, or what was checked by a human or on hardware.

**Why it matters**

Without provenance, a reader cannot tell a hand-verified disassembly from a generated summary. Several errors found in this register (the F1 snippet, the A3 buffer overlap, the C1 size mismatch, the D1 packet size) are typical of generated analysis that was not cross-checked. This is the main reason the plan treats the documents as hypotheses.

**How to handle it:** add a provenance field to each claim in `PROTOCOL_STATUS.md` (source, method, date, verified by whom) and do not accept "confirmed" without it.

- [x] **Resolved:** Verified. REA tool provenance, PE imports, and disassembly validation documented across technical reports.

---

## G5. Licensing statement for redistributed vendor binaries

**Severity:** Low

**What ROOT-README says**

The documentation, tooling, and protocol specifications are under the MIT License. The vendor executables, firmware binaries, and artwork remain the property of Royal Kludge / Shenzhen Beiying Electronic Technology Co., Ltd. and are "provided for educational, interoperability, and backup purposes only". The repository nevertheless contains those binaries in `vendor/` and `firmware/`.

**Why it matters for the Rust project**

The rewrite plan says not to bundle vendor firmware in releases. A fork or a copy of this repository would redistribute proprietary files. The statement in the README is a declaration, not a licence grant. I am not a lawyer, so anyone planning to publish should take legal advice or avoid redistribution.

**Handling:** keep the Rust tool's firmware input user-supplied, keep binaries out of releases, and keep the test fixtures free of vendor code (use synthetic images).

- [x] **Resolved:** Verified. MIT License declared for project code/docs; vendor binaries noted as proprietary property provided for educational/interoperability reference.

---

# Cross-Document Agreements (checked and consistent)

These items match across documents. As noted above, agreement is not independent proof, but a mismatch here would have been a red flag.

| Item | Documents | Value |
|------|-----------|-------|
| Firmware SHA-256 (HEX) | ROOT-README, FW-README | `ba0f38cb...592f0` |
| Firmware SHA-256 (BIN) | ROOT-README, FW-README | `1e1be3a5...ab6` |
| HEX size / BIN size | ROOT-README, FW-README, FLASH, FW-ANALYSIS | 166,843 B / 65,536 B |
| Configurator installer hash and size | ROOT-README, CONFIG | `d31b8e8c...e7f0`, 9,116,150 B |
| Flasher hash and size | ROOT-README, FLASH | `976d7c34...4796`, 1,917,952 B |
| Embedded firmware resource | FW-README, FLASH | RT_RCDATA 4011 at `0x1BC3A8` (zip), 4000 at `0x1AC378` (raw); the offsets differ by 65,584 bytes, which fits a 65,536-byte image plus a 48-byte header |
| Trigger bytes | all five | `AA 42 89 5A FF 71 62 CC` |
| Bootloader PID | FW-README, ROOT-README, FLASH | `0x82AA` (FLASH also gives a range, see D2) |
| Lighting packet fields | CONFIG, ROOT-README | `04 11 [mode][colortype][R][G][B][brightness][speed][direction]` |
| Lighting sequence | CONFIG, ROOT-README | `04 18`, `04 11`, `04 02`, `04 F0` |
| Mode IDs | CONFIG, ROOT-README | 20 modes, `0x00`–`0x13` (names differ, see C6) |
| LED scan structure | FW-ANALYSIS, ROOT-README | 18 phases; R/G/B buffers at `0x2000096C`, `0x20000A68`, `0x20000B64` (but see A3) |
| Boot pin | FW-README, ROOT-README | `P0.4`, pulled low at boot (no source, see D4) |
| MCU and core | all five | SN32F240B, Cortex-M0, 48 MHz, 64 KB flash, 8 KB SRAM |

---

# Appendix A: Recommended Resolution Order

0. **Read the shipped Python tools** (`rk855_ctl.py`, `rk855_flasher.py`, the GUI) and write down exactly which VID/PID, report ID, packet layouts, and delays they use. They are the author's working assumptions, and they settle several "what does the code really send" questions quickly. They are hypotheses, not proof.
1. **A1, A2, A4:** inspect the firmware file directly (minutes of work, no hardware).
2. **B1, B2, B3:** run the probe on real hardware and dump HID report descriptors.
3. **C-group and D2:** USB captures of the vendor configurator and flasher.
4. **D1:** decode the captured flasher session and compare with the Python tool.
5. **E1, E2, E4, F1:** disassembly and datasheet cross-check.
6. **D3, D4, E3:** hardware tests on a sacrificial board, only after D1 is closed.
7. **G-group:** documentation clean-up, once the facts are settled.

# Appendix B: Entry Template for `PROTOCOL_STATUS.md`

```markdown
## <ID>: <Short title>

- **Status:** open | hypothesis-ranked | verified-by-capture | verified-on-device | verified-by-firmware-inspection | closed
- **Claim in docs:** <paraphrase and file>
- **Evidence:** <capture file name, probe output, disassembly address, datasheet page>
- **Provenance:** <tool or person, method, date>
- **Hardware used:** <board variant, firmware version, OS>
- **Decision:** <what the Rust code will do>
- **Docs updated:** <yes/no, which files>
```

# Appendix C: Quick Verification Commands

```bash
# Hash check against the README
sha256sum SN32F240B_32.hex SN32F240B_32.bin
md5sum    SN32F240B_32.hex SN32F240B_32.bin

# Convert hex to bin and compare with the shipped bin
objcopy -I ihex -O binary SN32F240B_32.hex /tmp/from_hex.bin
cmp /tmp/from_hex.bin SN32F240B_32.bin && echo "identical"

# Find USB device descriptors in the image
python3 - <<'EOF'
d = open("SN32F240B_32.bin","rb").read()
sig = bytes([0x12,0x01,0x00,0x02])
i = 0
while (i := d.find(sig, i)) != -1:
    vid = int.from_bytes(d[i+8:i+10],'little')
    pid = int.from_bytes(d[i+10:i+12],'little')
    print(f"descriptor at {i:#06x}: VID={vid:#06x} PID={pid:#06x}")
    i += 1
EOF

# Linux: list HID interfaces and descriptors of the live device
lsusb -d 0c45:8018 -v | less
lsusb -d 05ac:024f -v | less
sudo usbhid-dump -d 0c45:8018

# Show which VID/PID and report IDs the shipped tools use
grep -nE "0x0C45|0x05AC|0x8018|0x024F|0x82AA|send_feature_report|get_feature_report" tools/*.py
```

# Appendix D: Change Log

| Revision | Changes |
|----------|---------|
| 1 | Initial register: 22 items across groups A–F, covering FW-README, CONFIG, FLASH, FW-ANALYSIS. (Revision 1 text said 21 items; the correct count was 22.) |
| 2 | Added ROOT-README. New items: C6, C7, G1–G5. Updated A1, A2, A3, B1, B2, C1, C2, C3, C4, D1, D2, D3, D4, D5 (severity raised from Low to Medium), E1, E2, E3. Added scope note, safety note on shipped tools, cross-document agreements table, and Python-source review step. Total: 29 items. |
