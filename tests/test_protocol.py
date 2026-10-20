"""Unit tests for USB HID Feature Report protocol packet encoder."""

import unittest
from rk855.constants import (
    REPORT_ID,
    REPORT_SIZE,
    REPORT_PAYLOAD_SIZE,
    LightingMode,
    CommandOpcode,
)
from rk855.protocol import (
    build_feature_report,
    build_preamble_packet,
    build_commit_packet,
    build_flush_packet,
    build_lighting_header_packet,
    build_lighting_preset_packet,
    build_lighting_packet,
    build_mode_16byte_block,
    build_lighting_bank_packets,
    build_full_lighting_transaction,
    build_key_remap_packet,
    build_key_remap_matrix_table,
    build_key_remap_transaction,
    build_custom_rgb_frame,
    build_reset_packet,
    build_reset_transaction,
)


class TestRK855Protocol(unittest.TestCase):

    def test_build_feature_report_size_and_id(self):
        payload = b"\x04\x01\x02\x03"
        report = build_feature_report(payload)
        self.assertEqual(len(report), REPORT_SIZE)
        self.assertEqual(report[0], REPORT_ID)
        self.assertEqual(report[1:5], payload)
        self.assertEqual(report[5:], b"\x00" * 60)

    def test_build_feature_report_oversized_payload(self):
        with self.assertRaises(ValueError):
            build_feature_report(b"\x00" * 65)

    def test_preamble_packet(self):
        pkt = build_preamble_packet()
        self.assertEqual(len(pkt), REPORT_SIZE)
        self.assertEqual(pkt[0], REPORT_ID)
        self.assertEqual(pkt[1], CommandOpcode.PRIMARY)       # 0x04
        self.assertEqual(pkt[2], CommandOpcode.SUB_PREAMBLE)  # 0x18
        self.assertEqual(pkt[3:], b"\x00" * 62)

    def test_commit_packet(self):
        pkt = build_commit_packet()
        self.assertEqual(len(pkt), REPORT_SIZE)
        self.assertEqual(pkt[0], REPORT_ID)
        self.assertEqual(pkt[1], CommandOpcode.PRIMARY)     # 0x04
        self.assertEqual(pkt[2], CommandOpcode.SUB_COMMIT)  # 0x02
        self.assertEqual(pkt[3:], b"\x00" * 62)

    def test_flush_packet(self):
        pkt = build_flush_packet()
        self.assertEqual(len(pkt), REPORT_SIZE)
        self.assertEqual(pkt[0], REPORT_ID)
        self.assertEqual(pkt[1], CommandOpcode.PRIMARY)    # 0x04
        self.assertEqual(pkt[2], CommandOpcode.SUB_FLUSH)  # 0xF0
        self.assertEqual(pkt[3:], b"\x00" * 62)

    def test_lighting_header_packet(self):
        pkt = build_lighting_header_packet(num_modes=18)
        self.assertEqual(len(pkt), REPORT_SIZE)
        self.assertEqual(pkt[0], REPORT_ID)
        self.assertEqual(pkt[1], CommandOpcode.PRIMARY)         # 0x04
        self.assertEqual(pkt[2], CommandOpcode.SUB_LIGHT_BANK)  # 0x13
        self.assertEqual(pkt[8], 18)                            # Mode count indicator

    def test_lighting_preset_packet(self):
        pkt = build_lighting_preset_packet(
            mode=LightingMode.SCROLLING,
            speed=4,
            brightness=5,
            color_rgb=(255, 128, 0),
            direction=1,
        )
        self.assertEqual(len(pkt), REPORT_SIZE)
        self.assertEqual(pkt[0], REPORT_ID)
        self.assertEqual(pkt[1], CommandOpcode.PRIMARY)           # 0x04
        self.assertEqual(pkt[2], CommandOpcode.SUB_PRESET_LIGHT)  # 0x11
        self.assertEqual(pkt[3], LightingMode.SCROLLING.value)    # 9
        self.assertEqual(pkt[4], 5)                               # brightness
        self.assertEqual(pkt[5], 4)                               # speed
        self.assertEqual(pkt[6], 1)                               # direction
        self.assertEqual(pkt[8], 0x0A)                            # preset bank count
        self.assertEqual(pkt[9], 255)                             # R
        self.assertEqual(pkt[10], 128)                            # G
        self.assertEqual(pkt[11], 0)                              # B

    def test_build_lighting_packet_structure(self):
        packet = build_lighting_packet(
            mode=LightingMode.WAVE,
            speed=4,
            brightness=5,
            color_rgb=(255, 128, 0),
            direction=1,
        )
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.PRIMARY)
        self.assertEqual(packet[2], CommandOpcode.SUB_PRESET_LIGHT)
        self.assertEqual(packet[3], LightingMode.WAVE.value)
        self.assertEqual(packet[4], 4)    # speed
        self.assertEqual(packet[5], 5)    # brightness
        self.assertEqual(packet[6], 1)    # direction
        self.assertEqual(packet[7], 255)  # R
        self.assertEqual(packet[8], 128)  # G
        self.assertEqual(packet[9], 0)    # B

    def test_build_lighting_packet_clamping(self):
        packet = build_lighting_packet(
            mode=LightingMode.STATIC,
            speed=999,
            brightness=-50,
            color_rgb=(300, -10, 120),
            direction=0,
        )
        self.assertEqual(packet[4], 5)    # clamped speed to max 5
        self.assertEqual(packet[5], 0)    # clamped brightness to min 0
        self.assertEqual(packet[7], 255)  # clamped R to max 255
        self.assertEqual(packet[8], 0)    # clamped G to min 0
        self.assertEqual(packet[9], 120)  # B

    def test_mode_16byte_block_magic(self):
        block = build_mode_16byte_block(
            mode_id=0, speed=3, brightness=4, color_rgb=(0, 168, 255)
        )
        self.assertEqual(len(block), 16)
        self.assertEqual(block[0], 0)      # mode
        self.assertEqual(block[1], 4)      # brightness
        self.assertEqual(block[2], 3)      # speed
        self.assertEqual(block[14], 0xAA)  # 0x55AA sync low byte
        self.assertEqual(block[15], 0x55)  # 0x55AA sync high byte

    def test_lighting_bank_packets(self):
        chunks = build_lighting_bank_packets(
            active_mode=LightingMode.STATIC,
            speed=3,
            brightness=4,
            color_rgb=(0, 168, 255),
        )
        # Must be exactly 19 chunks (1216 bytes total)
        self.assertEqual(len(chunks), 19)
        for chunk in chunks:
            self.assertEqual(len(chunk), REPORT_SIZE)
            self.assertEqual(chunk[0], REPORT_ID)

    def test_full_lighting_transaction(self):
        tx = build_full_lighting_transaction(
            mode=LightingMode.STATIC,
            speed=3,
            brightness=4,
            color_rgb=(0, 168, 255),
        )
        # Preamble(1) + Header(1) + Preset(1) + Bank(19) + Commit(1) + Flush(1) = 24 packets
        self.assertEqual(len(tx), 24)
        for pkt in tx:
            self.assertEqual(len(pkt), REPORT_SIZE)
            self.assertEqual(pkt[0], REPORT_ID)

        # Control packets start with PRIMARY 0x04
        self.assertEqual(tx[0][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[0][2], CommandOpcode.SUB_PREAMBLE)
        self.assertEqual(tx[1][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[1][2], CommandOpcode.SUB_LIGHT_BANK)
        self.assertEqual(tx[2][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[2][2], CommandOpcode.SUB_PRESET_LIGHT)
        self.assertEqual(tx[-2][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[-2][2], CommandOpcode.SUB_COMMIT)
        self.assertEqual(tx[-1][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[-1][2], CommandOpcode.SUB_FLUSH)

    def test_build_key_remap_packet(self):
        packet = build_key_remap_packet(key_code=0x39, target_scancode=0xE0, key_index=30)
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.PRIMARY_KEY_REMAP)  # 0x06
        self.assertEqual(packet[2], CommandOpcode.SUB_KEY_REMAP)      # 0x20
        self.assertEqual(packet[3], 0x39)
        self.assertEqual(packet[4], 0xE0)
        self.assertEqual(packet[5], 0x00)
        self.assertEqual(packet[6], 30)

    def test_build_key_remap_matrix_table(self):
        # Default with no remaps
        table = build_key_remap_matrix_table({})
        self.assertEqual(len(table), 256)
        # Key 0 is ESC (0x29)
        self.assertEqual(table[0], 0x29)
        self.assertEqual(table[1], 0x00)
        # Key 30 is CapsLock (0x39)
        self.assertEqual(table[30 * 2], 0x39)
        self.assertEqual(table[30 * 2 + 1], 0x00)

        # With CapsLock (0x39) remapped to Left Control (0xE0)
        remapped_table = build_key_remap_matrix_table({0x39: 0xE0})
        self.assertEqual(remapped_table[30 * 2], 0xE0)
        self.assertEqual(remapped_table[30 * 2 + 1], 0x00)
        # Other keys remain untouched
        self.assertEqual(remapped_table[0], 0x29)

    def test_build_key_remap_transaction(self):
        remaps = {0x39: 0xE0, 0x29: 0x35}
        tx = build_key_remap_transaction(remaps)
        # Preamble(1) + Header(1) + 4 Chunks(4) + 2 Per-Key Direct Packets(2) + Commit(1) + Flush(1) = 10 packets
        self.assertEqual(len(tx), 10)
        for pkt in tx:
            self.assertEqual(len(pkt), REPORT_SIZE)
            self.assertEqual(pkt[0], REPORT_ID)

        # Preamble uses PRIMARY_KEY_REMAP 0x06
        self.assertEqual(tx[0][1], CommandOpcode.PRIMARY_KEY_REMAP)
        self.assertEqual(tx[0][2], CommandOpcode.SUB_PREAMBLE)
        # Header uses PRIMARY_KEY_REMAP 0x06
        self.assertEqual(tx[1][1], CommandOpcode.PRIMARY_KEY_REMAP)
        self.assertEqual(tx[1][2], CommandOpcode.SUB_KEY_REMAP)
        self.assertEqual(tx[1][8], 0x04)  # 4 chunks = 256 bytes

        # Commit & Flush use PRIMARY_KEY_REMAP 0x06
        self.assertEqual(tx[-2][1], CommandOpcode.PRIMARY_KEY_REMAP)
        self.assertEqual(tx[-2][2], CommandOpcode.SUB_COMMIT)
        self.assertEqual(tx[-1][1], CommandOpcode.PRIMARY_KEY_REMAP)
        self.assertEqual(tx[-1][2], CommandOpcode.SUB_FLUSH)

    def test_build_custom_rgb_frame(self):
        key_colors = {
            0x1A: (255, 0, 0),
            0x04: (0, 255, 0),
        }
        packet = build_custom_rgb_frame(key_colors, frame_index=0)
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.PRIMARY)
        self.assertEqual(packet[2], CommandOpcode.SUB_CUSTOM_RGB)
        # Check 0x55AA sync mark at tail
        self.assertEqual(packet[63], 0xAA)
        self.assertEqual(packet[64], 0x55)

    def test_build_reset_transaction(self):
        tx = build_reset_transaction()
        # Safe lighting reset under universal 0x04 primary opcode
        self.assertEqual(tx[0][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[0][2], CommandOpcode.SUB_PREAMBLE)
        self.assertEqual(tx[-2][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[-2][2], CommandOpcode.SUB_COMMIT)
        self.assertEqual(tx[-1][1], CommandOpcode.PRIMARY)
        self.assertEqual(tx[-1][2], CommandOpcode.SUB_FLUSH)


if __name__ == "__main__":
    unittest.main()
