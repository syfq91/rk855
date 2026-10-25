"""Key remapping subsystem for Royal Kludge RK855."""

from typing import Protocol
from .constants import (
    CMD_BEGIN_REMAP,
    CMD_COMMIT_FLASH,
    CMD_SYNC_LIVE,
    REMAP_TABLE_SIZE,
    PHYSICAL_KEYS,
    ACTION_BY_CODE,
)


class DeviceSender(Protocol):
    def send_feature(self, payload: bytes | list[int], delay: float = 0.005) -> None:
        ...


def encode_remap_table(remap_dict: dict[int, int]) -> bytes:
    """Encodes a dictionary of {physical_scancode: target_code} into a 256-byte table.

    Each remapped entry occupies 2 bytes: [physical_scancode, target_code].
    Unmapped entries remain zero-padded up to 256 bytes.
    """
    buf = bytearray(REMAP_TABLE_SIZE)
    offset = 0

    # Sort by physical scancode for deterministic order
    for phys_code, target_code in sorted(remap_dict.items()):
        if offset + 2 > REMAP_TABLE_SIZE:
            break
        buf[offset] = phys_code & 0xFF
        buf[offset + 1] = target_code & 0xFF
        offset += 2

    return bytes(buf)


def decode_remap_table(data: bytes) -> dict[int, int]:
    """Decodes a 256-byte table back into a dictionary of {physical_scancode: target_code}."""
    result = {}
    for i in range(0, min(len(data), REMAP_TABLE_SIZE), 2):
        phys = data[i]
        target = data[i + 1]
        if phys != 0:
            result[phys] = target
    return result


def apply_remap_table(
    device: DeviceSender,
    remap_dict: dict[int, int],
) -> None:
    """Sends the complete atomic key remapping transaction to the keyboard MCU."""
    table_bytes = encode_remap_table(remap_dict)

    # 1. Begin Key Remap Init Packet: [0x04, 0x20, 0, 0, 0, 0, 0, 0, 0x04]
    init_payload = [0x04, CMD_BEGIN_REMAP, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04]
    device.send_feature(init_payload)

    # 2. Transfer Mapping Table: 256 bytes in 4 chunks of 64 bytes
    chunk_size = 64
    for i in range(0, len(table_bytes), chunk_size):
        chunk = table_bytes[i : i + chunk_size]
        device.send_feature(chunk)

    # 3. Commit to Flash (EEPROM): [0x04, 0x02]
    device.send_feature([0x04, CMD_COMMIT_FLASH])

    # 4. Reload and Live Sync: [0x04, 0xF0]
    device.send_feature([0x04, CMD_SYNC_LIVE])
