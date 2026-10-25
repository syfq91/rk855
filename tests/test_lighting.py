"""Tests for lighting protocol packets and mock device handling."""

import unittest
from rk855_gui.core.constants import (
    CMD_BEGIN_LIGHT,
    CMD_SET_LIGHT,
    CMD_CUSTOM_RGB_CHUNK,
    CMD_COMMIT_FLASH,
    CMD_SYNC_LIVE,
)
from rk855_gui.core.lighting import apply_lighting_mode, apply_custom_matrix
from rk855_gui.core.mock_device import MockRK855Device


class TestLightingProtocol(unittest.TestCase):
    def setUp(self):
        self.device = MockRK855Device()

    def test_apply_lighting_mode(self):
        apply_lighting_mode(
            self.device,
            mode_id=14,
            brightness=4,
            speed=3,
            r=128,
            g=0,
            b=255,
            direction=1,
        )

        # Ensure correct state recorded in mock device
        self.assertEqual(self.device.active_lighting_mode, 14)
        self.assertEqual(self.device.brightness, 4)
        self.assertEqual(self.device.speed, 3)
        self.assertEqual(self.device.color, (128, 0, 255))
        self.assertEqual(self.device.direction, 1)

        # Check sequence of packets sent: Begin(0x18) -> Set(0x11) -> Commit(0x02) -> Sync(0xF0)
        opcodes = [p[1] for p in self.device.sent_packets]
        self.assertEqual(opcodes, [CMD_BEGIN_LIGHT, CMD_SET_LIGHT, CMD_COMMIT_FLASH, CMD_SYNC_LIVE])

    def test_apply_custom_matrix(self):
        colors = [(i * 3, i * 2, i) for i in range(68)]
        apply_custom_matrix(self.device, colors)

        opcodes = [p[1] for p in self.device.sent_packets]
        self.assertEqual(opcodes[0], CMD_BEGIN_LIGHT)
        # Followed by chunk packets (0x16)
        chunk_packets = [p for p in self.device.sent_packets if p[1] == CMD_CUSTOM_RGB_CHUNK]
        self.assertEqual(len(chunk_packets), 4)  # 204 bytes / 60 bytes chunking = 4 chunks
        # Followed by commit and sync
        self.assertEqual(opcodes[-2], CMD_COMMIT_FLASH)
        self.assertEqual(opcodes[-1], CMD_SYNC_LIVE)


if __name__ == "__main__":
    unittest.main()
