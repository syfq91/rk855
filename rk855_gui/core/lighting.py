"""Lighting subsystem control for Royal Kludge RK855."""

from typing import Protocol
from .constants import (
    CMD_BEGIN_LIGHT,
    CMD_SET_LIGHT,
    CMD_CUSTOM_RGB_CHUNK,
    CMD_COMMIT_FLASH,
    CMD_SYNC_LIVE,
)


class DeviceSender(Protocol):
    def send_feature(self, payload: bytes | list[int], delay: float = 0.005) -> None:
        ...


def apply_lighting_mode(
    device: DeviceSender,
    mode_id: int,
    brightness: int = 3,
    speed: int = 2,
    r: int = 255,
    g: int = 0,
    b: int = 255,
    direction: int = 0,
    color_type: int = 1,
) -> None:
    """Sends atomic transaction to apply a built-in lighting effect."""
    # 1. Begin Light Configuration
    device.send_feature([0x04, CMD_BEGIN_LIGHT])

    # 2. Set Light Mode Parameters
    device.send_feature([
        0x04,
        CMD_SET_LIGHT,
        mode_id,
        color_type,
        r,
        g,
        b,
        brightness,
        speed,
        direction,
    ])

    # 3. Commit Changes to Flash
    device.send_feature([0x04, CMD_COMMIT_FLASH])

    # 4. Sync Live Lighting Engine
    device.send_feature([0x04, CMD_SYNC_LIVE])


def apply_custom_matrix(
    device: DeviceSender,
    colors: list[tuple[int, int, int]],
) -> None:
    """Applies per-key RGB coloring for the 68 physical keys."""
    # Ensure exactly 68 RGB values
    padded = list(colors[:68])
    if len(padded) < 68:
        padded.extend([(0, 0, 0)] * (68 - len(padded)))

    # Flatten into byte stream: 68 keys * 3 bytes (R, G, B) = 204 bytes
    flat_bytes = bytearray()
    for r, g, b in padded:
        flat_bytes.extend([max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b))])

    # 1. Begin Configuration
    device.send_feature([0x04, CMD_BEGIN_LIGHT])

    # 2. Upload in chunks of up to 60 bytes each (opcode 0x16)
    offsets = [0, 60, 120, 180, len(flat_bytes)]
    for idx in range(len(offsets) - 1):
        start = offsets[idx]
        end = offsets[idx + 1]
        chunk = flat_bytes[start:end]
        if chunk:
            payload = [0x04, CMD_CUSTOM_RGB_CHUNK, idx, len(chunk)] + list(chunk)
            device.send_feature(payload)

    # 3. Commit to Flash
    device.send_feature([0x04, CMD_COMMIT_FLASH])

    # 4. Sync Live Lighting Engine
    device.send_feature([0x04, CMD_SYNC_LIVE])
