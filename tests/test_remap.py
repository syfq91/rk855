"""Unit tests for button/key remapping protocol and encoding."""

import unittest
from rk855_gui.core.constants import (
    CMD_BEGIN_REMAP,
    CMD_COMMIT_FLASH,
    CMD_SYNC_LIVE,
    REMAP_TABLE_SIZE,
    REMAP_ACTIONS,
    ACTION_BY_CODE,
)
from rk855_gui.core.remap import (
    encode_remap_table,
    decode_remap_table,
    apply_remap_table,
)
from rk855_gui.core.mock_device import MockRK855Device


class TestKeyRemap(unittest.TestCase):
    def test_encode_empty_table(self):
        buf = encode_remap_table({})
        self.assertEqual(len(buf), REMAP_TABLE_SIZE)
        self.assertTrue(all(b == 0 for b in buf))

    def test_encode_decode_roundtrip(self):
        mapping = {
            0x39: 0xE0,  # Caps Lock -> Left Ctrl
            0x29: 0x35,  # Esc -> Grave / Tilde
            0xE6: 0xAF,  # Right Alt -> Fn
        }
        encoded = encode_remap_table(mapping)
        self.assertEqual(len(encoded), REMAP_TABLE_SIZE)
        decoded = decode_remap_table(encoded)
        self.assertEqual(decoded, mapping)

    def test_apply_remap_protocol_sequence(self):
        mock_dev = MockRK855Device()
        mapping = {
            0x39: 0xE0,  # Caps Lock -> Left Ctrl
            0xE3: 0xE2,  # Left Win -> Left Alt
        }
        apply_remap_table(mock_dev, mapping)

        # 1 init packet + 4 data chunks + 1 commit packet + 1 sync packet = 7 packets
        self.assertEqual(len(mock_dev.sent_packets), 7)

        # Init packet: starts with 0x04, 0x20
        init_pkt = mock_dev.sent_packets[0]
        self.assertEqual(init_pkt[0], 0x04)
        self.assertEqual(init_pkt[1], CMD_BEGIN_REMAP)

        # Commit packet
        commit_pkt = mock_dev.sent_packets[5]
        self.assertEqual(commit_pkt[0], 0x04)
        self.assertEqual(commit_pkt[1], CMD_COMMIT_FLASH)

        # Sync packet
        sync_pkt = mock_dev.sent_packets[6]
        self.assertEqual(sync_pkt[0], 0x04)
        self.assertEqual(sync_pkt[1], CMD_SYNC_LIVE)

        # Verify mock device decoded the remap table properly
        self.assertEqual(mock_dev.remap_table.get(0x39), 0xE0)
        self.assertEqual(mock_dev.remap_table.get(0xE3), 0xE2)

    def test_remap_actions_definitions(self):
        self.assertGreater(len(REMAP_ACTIONS), 60)
        for act in REMAP_ACTIONS:
            self.assertIn(act.code, ACTION_BY_CODE)
            self.assertTrue(len(act.name) > 0)
            self.assertTrue(len(act.category) > 0)


if __name__ == "__main__":
    unittest.main()
