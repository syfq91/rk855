"""USB HID Feature Report packet encoder for Sonix RK855 microcontroller.

Implements the reverse-engineered 65-byte protocol extracted directly from
the OEM utility (DeviceDriver.exe, SHA-256 690e9b...):
  - Primary Opcode 0x04 for all configuration reports
  - Sub-opcodes: 0x18 (Preamble), 0x11 (Preset Light), 0x13 (Light Bank),
    0x20 (Key Remap), 0x15 (Custom RGB), 0x02 (Commit), 0xF0 (Flush)
  - 16-byte mode structure with 0x55AA sync magic
  - Multi-packet transaction framing required by Sonix MCU firmware
"""

from typing import Dict, List, Optional, Tuple, Union
from .constants import (
    REPORT_ID,
    REPORT_SIZE,
    REPORT_PAYLOAD_SIZE,
    LightingMode,
    CommandOpcode,
    DEFAULT_BACKLIGHT_RGB,
)


def build_feature_report(payload: bytes) -> bytes:
    """Wrap a payload into a strict 65-byte USB HID Feature Report.

    Structure:
      Byte 0: Report ID (0x00)
      Bytes 1..64: Command payload (padded with 0x00 to exactly 64 bytes)
    """
    if len(payload) > REPORT_PAYLOAD_SIZE:
        raise ValueError(
            f"Payload length {len(payload)} exceeds maximum report payload size {REPORT_PAYLOAD_SIZE}"
        )
    padded = payload.ljust(REPORT_PAYLOAD_SIZE, b"\x00")
    return bytes([REPORT_ID]) + padded


def build_preamble_packet() -> bytes:
    """Build the transaction begin / handshake preamble packet (0x04, 0x18)."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY       # 0x04
    payload[1] = CommandOpcode.SUB_PREAMBLE  # 0x18
    return bytes([REPORT_ID]) + bytes(payload)


def build_commit_packet() -> bytes:
    """Build the commit packet (0x04, 0x02) to flush RAM settings to onboard flash."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY     # 0x04
    payload[1] = CommandOpcode.SUB_COMMIT  # 0x02
    return bytes([REPORT_ID]) + bytes(payload)


def build_flush_packet() -> bytes:
    """Build the session flush / transaction end packet (0x04, 0xF0)."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY    # 0x04
    payload[1] = CommandOpcode.SUB_FLUSH  # 0xF0
    return bytes([REPORT_ID]) + bytes(payload)


def build_lighting_header_packet(num_modes: int = 18) -> bytes:
    """Build the lighting bank descriptor header packet (0x04, 0x13)."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY        # 0x04
    payload[1] = CommandOpcode.SUB_LIGHT_BANK # 0x13
    payload[7] = num_modes & 0xFF             # Byte 8 of 65-byte report
    return bytes([REPORT_ID]) + bytes(payload)


def build_lighting_preset_packet(
    mode: Union[LightingMode, int],
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
    direction: int = 0,
) -> bytes:
    """Build the direct preset lighting configuration packet (0x04, 0x11)."""
    mode_val = mode.value if isinstance(mode, LightingMode) else int(mode)
    speed = max(1, min(5, int(speed)))
    brightness = max(0, min(5, int(brightness)))
    r, g, b = (max(0, min(255, int(c))) for c in color_rgb)
    direction = 1 if direction else 0

    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY           # 0x04
    payload[1] = CommandOpcode.SUB_PRESET_LIGHT  # 0x11
    payload[2] = mode_val & 0xFF
    payload[3] = brightness & 0xFF
    payload[4] = speed & 0xFF
    payload[5] = direction & 0xFF
    payload[7] = 0x0A                            # 10 preset bank indicator
    payload[8] = r
    payload[9] = g
    payload[10] = b
    return bytes([REPORT_ID]) + bytes(payload)


def build_lighting_packet(
    mode: Union[LightingMode, int],
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
    direction: int = 0,
) -> bytes:
    """Build a single 65-byte feature report for preset lighting (backwards-compatible API)."""
    mode_val = mode.value if isinstance(mode, LightingMode) else int(mode)
    speed = max(1, min(5, int(speed)))
    brightness = max(0, min(5, int(brightness)))
    r, g, b = (max(0, min(255, int(c))) for c in color_rgb)
    direction = 1 if direction else 0

    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY           # 0x04
    payload[1] = CommandOpcode.SUB_PRESET_LIGHT  # 0x11
    payload[2] = mode_val & 0xFF
    payload[3] = speed
    payload[4] = brightness
    payload[5] = direction
    payload[6] = r
    payload[7] = g
    payload[8] = b
    return bytes([REPORT_ID]) + bytes(payload)


def build_mode_16byte_block(
    mode_id: int,
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
    direction: int = 0,
    color_type: int = 0,
) -> bytes:
    """Encode a 16-byte lighting mode block with 0x55AA synchronization magic.

    Layout:
      Offset 0:  Mode ID (0..18)
      Offset 1:  Brightness (0..5)
      Offset 2:  Speed (1..5)
      Offset 3:  Direction (0=left, 1=right)
      Offset 4..7: 0x00 Reserved
      Offset 8:  Color Type (0=monochrome/fixed, 1=rainbow/spectrum)
      Offset 9:  R (0..255)
      Offset 10: G (0..255)
      Offset 11: B (0..255)
      Offset 12..13: 0x00 Reserved
      Offset 14..15: 0xAA, 0x55 (Sync magic 0x55AA little-endian)
    """
    block = bytearray(16)
    block[0] = mode_id & 0xFF
    block[1] = max(0, min(5, int(brightness)))
    block[2] = max(1, min(5, int(speed)))
    block[3] = 1 if direction else 0
    # Bytes 4..7 are 0x00
    block[8] = 1 if color_type else 0
    block[9] = max(0, min(255, int(color_rgb[0])))
    block[10] = max(0, min(255, int(color_rgb[1])))
    block[11] = max(0, min(255, int(color_rgb[2])))
    # Bytes 12..13 are 0x00
    block[14] = 0xAA  # 0x55AA low byte
    block[15] = 0x55  # 0x55AA high byte
    return bytes(block)


def build_lighting_bank_packets(
    active_mode: Union[LightingMode, int],
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
    direction: int = 0,
) -> List[bytes]:
    """Generate the 19 raw 65-byte feature report chunks (1216 bytes total) for the lighting bank.

    This matches the exact structure populated at VA 0x00415A00 in DeviceDriver.exe:
      - 19 modes (0 to 18) packed in 16-byte blocks
      - Active selected mode mirrored at offset 1088 (0x440)
      - Each 64-byte payload prefixed with Report ID 0x00
    """
    mode_val = active_mode.value if isinstance(active_mode, LightingMode) else int(active_mode)
    speed = max(1, min(5, int(speed)))
    brightness = max(0, min(5, int(brightness)))
    r, g, b = (max(0, min(255, int(c))) for c in color_rgb)

    # Total buffer size: 1216 bytes (19 chunks of 64 bytes)
    bank_size = 1216
    buffer = bytearray(bank_size)

    # Populate 19 mode definitions
    for m in range(19):
        # Determine if this mode should default to rainbow or fixed color
        is_rainbow = 1 if m in (5, 7) else 0
        m_speed = speed if m == mode_val else 3
        m_bright = brightness if m == mode_val else 3
        m_dir = direction if m == mode_val else 0
        m_color = (r, g, b)

        block = build_mode_16byte_block(
            mode_id=m,
            speed=m_speed,
            brightness=m_bright,
            color_rgb=m_color,
            direction=m_dir,
            color_type=is_rainbow,
        )
        offset = m * 16
        buffer[offset : offset + 16] = block

    # Place active selected mode at offset 1088 (0x440)
    active_block = build_mode_16byte_block(
        mode_id=mode_val,
        speed=speed,
        brightness=brightness,
        color_rgb=(r, g, b),
        direction=direction,
        color_type=1 if mode_val in (5, 7) else 0,
    )
    buffer[1088 : 1088 + 16] = active_block

    # Split into 19 packets of 64 bytes, each prepended with REPORT_ID
    packets = []
    for i in range(0, bank_size, REPORT_PAYLOAD_SIZE):
        chunk = buffer[i : i + REPORT_PAYLOAD_SIZE]
        packets.append(bytes([REPORT_ID]) + bytes(chunk))
    return packets


def build_full_lighting_transaction(
    mode: Union[LightingMode, int],
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = DEFAULT_BACKLIGHT_RGB,
    direction: int = 0,
) -> List[bytes]:
    """Build the complete, verified hardware lighting transaction matching DeviceDriver.exe.

    Sequence:
      1. Preamble report (0x04 0x18)
      2. Lighting bank descriptor header (0x04 0x13 ... 0x12)
      3. Direct preset report (0x04 0x11 ...)
      4. 19 lighting bank chunk reports (1216 bytes, 0x55AA sync)
      5. Commit report (0x04 0x02) -> writes to onboard microcontroller memory
      6. Flush report (0x04 0xF0)  -> activates changes on LED controller
    """
    packets = []
    # 1. Preamble
    packets.append(build_preamble_packet())
    # 2. Header
    packets.append(build_lighting_header_packet(num_modes=18))
    # 3. Direct preset
    packets.append(build_lighting_preset_packet(
        mode=mode,
        speed=speed,
        brightness=brightness,
        color_rgb=color_rgb,
        direction=direction,
    ))
    # 4. Bank chunks
    packets.extend(build_lighting_bank_packets(
        active_mode=mode,
        speed=speed,
        brightness=brightness,
        color_rgb=color_rgb,
        direction=direction,
    ))
    # 5. Commit
    packets.append(build_commit_packet())
    # 6. Flush
    packets.append(build_flush_packet())
    return packets


def build_key_remap_packet(key_code: int, target_scancode: int) -> bytes:
    """Build a single key remap report (0x04, 0x20)."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY        # 0x04
    payload[1] = CommandOpcode.SUB_KEY_REMAP  # 0x20
    payload[2] = key_code & 0xFF
    payload[3] = target_scancode & 0xFF
    payload[4] = (target_scancode >> 8) & 0xFF
    return bytes([REPORT_ID]) + bytes(payload)


def build_key_remap_transaction(remaps: Dict[int, int]) -> List[bytes]:
    """Build the complete key remapping transaction matching DeviceDriver.exe (VA 0x00413250).

    Sequence:
      1. Preamble report (0x04 0x18)
      2. Remap block header (0x04, 0x20, count=4 chunks = 256 bytes)
      3. 4 payload reports containing (key_code, target_scancode) pairs
      4. Commit report (0x04 0x02)
      5. Flush report (0x04 0xF0)
    """
    packets = []
    # 1. Preamble
    packets.append(build_preamble_packet())

    # 2. Header (0x04 0x20 with chunk count 4 at byte 8)
    hdr = bytearray(REPORT_PAYLOAD_SIZE)
    hdr[0] = CommandOpcode.PRIMARY        # 0x04
    hdr[1] = CommandOpcode.SUB_KEY_REMAP  # 0x20
    hdr[7] = 0x04                         # 4 chunks = 256 bytes
    packets.append(bytes([REPORT_ID]) + bytes(hdr))

    # 3. 256 bytes payload buffer
    data_buffer = bytearray(256)
    offset = 0
    for key_code, target_code in remaps.items():
        if offset + 2 > len(data_buffer):
            break
        data_buffer[offset] = key_code & 0xFF
        data_buffer[offset + 1] = target_code & 0xFF
        offset += 2

    # Split into 4 chunks of 64 bytes
    for i in range(0, 256, REPORT_PAYLOAD_SIZE):
        chunk = data_buffer[i : i + REPORT_PAYLOAD_SIZE]
        packets.append(bytes([REPORT_ID]) + bytes(chunk))

    # 4. Commit & Flush
    packets.append(build_commit_packet())
    packets.append(build_flush_packet())
    return packets


def build_custom_rgb_frame(
    key_colors: Dict[int, Tuple[int, int, int]], frame_index: int = 0
) -> bytes:
    """Build a custom per-key RGB lighting packet chunk with 0x55AA sync mark."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY        # 0x04
    payload[1] = CommandOpcode.SUB_CUSTOM_RGB # 0x15
    payload[2] = frame_index & 0xFF

    offset = 3
    for key_code, (r, g, b) in key_colors.items():
        if offset + 4 > REPORT_PAYLOAD_SIZE - 2:
            break
        payload[offset] = key_code & 0xFF
        payload[offset + 1] = max(0, min(255, r))
        payload[offset + 2] = max(0, min(255, g))
        payload[offset + 3] = max(0, min(255, b))
        offset += 4

    # 0x55AA synchronization magic at tail
    payload[62] = 0xAA
    payload[63] = 0x55
    return bytes([REPORT_ID]) + bytes(payload)


def build_reset_packet() -> bytes:
    """Build a reset packet for factory defaults."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.PRIMARY     # 0x04
    payload[1] = CommandOpcode.SUB_COMMIT  # 0x02
    return bytes([REPORT_ID]) + bytes(payload)


def build_reset_transaction() -> List[bytes]:
    """Build a complete reset to factory defaults transaction."""
    packets = []
    packets.append(build_preamble_packet())
    # Send default static blue lighting bank
    packets.extend(build_lighting_bank_packets(
        active_mode=LightingMode.STATIC,
        speed=3,
        brightness=4,
        color_rgb=DEFAULT_BACKLIGHT_RGB,
        direction=0,
    ))
    packets.append(build_commit_packet())
    packets.append(build_flush_packet())
    return packets
