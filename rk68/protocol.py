"""USB HID Feature Report packet encoder for Sonix RK68 microcontroller."""

from typing import Dict, List, Tuple
from .constants import (
    REPORT_ID,
    REPORT_SIZE,
    REPORT_PAYLOAD_SIZE,
    LightingMode,
    CommandOpcode,
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


def build_lighting_packet(
    mode: LightingMode,
    speed: int = 3,
    brightness: int = 3,
    color_rgb: Tuple[int, int, int] = (255, 255, 255),
    direction: int = 0,
) -> bytes:
    """Build a 65-byte feature report for setting lighting mode, speed, brightness, and color.
    
    Parameters:
      mode: LightingMode enum value
      speed: Animation speed 1 (slowest) to 5 (fastest)
      brightness: LED brightness 0 (off) to 5 (max)
      color_rgb: (R, G, B) tuple with 0-255 values
      direction: Animation direction (0 = default/left-to-right, 1 = right-to-left)
    """
    # Clamp parameters
    speed = max(1, min(5, int(speed)))
    brightness = max(0, min(5, int(brightness)))
    r, g, b = (max(0, min(255, int(c))) for c in color_rgb)
    direction = 1 if direction else 0

    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.SET_LIGHT_CONFIG  # 0x04
    payload[1] = mode.value
    payload[2] = speed
    payload[3] = brightness
    payload[4] = direction
    payload[5] = r
    payload[6] = g
    payload[7] = b

    return bytes([REPORT_ID]) + bytes(payload)


def build_key_remap_packet(key_code: int, target_scancode: int) -> bytes:
    """Build a packet to remap a physical key scancode to an alternate scancode."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.SET_KEY_REMAP  # 0x06
    payload[1] = key_code & 0xFF
    payload[2] = target_scancode & 0xFF
    payload[3] = (target_scancode >> 8) & 0xFF

    return bytes([REPORT_ID]) + bytes(payload)


def build_custom_rgb_frame(key_colors: Dict[int, Tuple[int, int, int]], frame_index: int = 0) -> bytes:
    """Build a custom per-key RGB lighting packet chunk.
    
    Each key entry contains [keycode, R, G, B].
    Up to 15 keys fit in a single 64-byte payload (header + 15 * 4 bytes = 61 bytes).
    """
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.SET_CUSTOM_RGB  # 0x05
    payload[1] = frame_index & 0xFF

    # Pack up to 15 key-color entries
    offset = 2
    for key_code, (r, g, b) in key_colors.items():
        if offset + 4 > REPORT_PAYLOAD_SIZE:
            break
        payload[offset] = key_code & 0xFF
        payload[offset + 1] = max(0, min(255, r))
        payload[offset + 2] = max(0, min(255, g))
        payload[offset + 3] = max(0, min(255, b))
        offset += 4

    return bytes([REPORT_ID]) + bytes(payload)


def build_reset_packet() -> bytes:
    """Build a packet to reset the keyboard to factory defaults."""
    payload = bytearray(REPORT_PAYLOAD_SIZE)
    payload[0] = CommandOpcode.RESET_DEFAULT  # 0x07
    return bytes([REPORT_ID]) + bytes(payload)
