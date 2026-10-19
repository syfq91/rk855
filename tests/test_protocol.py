"""Unit tests for USB HID Feature Report protocol packet encoder."""

import unittest
from rk68.constants import (
    REPORT_ID,
    REPORT_SIZE,
    LightingMode,
    CommandOpcode,
)
from rk68.protocol import (
    build_feature_report,
    build_lighting_packet,
    build_key_remap_packet,
    build_custom_rgb_frame,
    build_reset_packet,
)


class TestRK68Protocol(unittest.TestCase):

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
        self.assertEqual(packet[1], CommandOpcode.SET_LIGHT_CONFIG)
        self.assertEqual(packet[2], LightingMode.WAVE.value)
        self.assertEqual(packet[3], 4)  # speed
        self.assertEqual(packet[4], 5)  # brightness
        self.assertEqual(packet[5], 1)  # direction
        self.assertEqual(packet[6], 255)  # R
        self.assertEqual(packet[7], 128)  # G
        self.assertEqual(packet[8], 0)    # B
        self.assertEqual(packet[9:], b"\x00" * (65 - 9))

    def test_build_lighting_packet_clamping(self):
        # Out-of-bounds speed, brightness, and colors should be safely clamped
        packet = build_lighting_packet(
            mode=LightingMode.STATIC,
            speed=999,
            brightness=-50,
            color_rgb=(300, -10, 120),
            direction=0,
        )
        self.assertEqual(packet[3], 5)    # clamped speed to max 5
        self.assertEqual(packet[4], 0)    # clamped brightness to min 0
        self.assertEqual(packet[6], 255)  # clamped R to max 255
        self.assertEqual(packet[7], 0)    # clamped G to min 0
        self.assertEqual(packet[8], 120)  # B

    def test_build_key_remap_packet(self):
        # Remap CapsLock (0x39) to Left Control (0xE0)
        packet = build_key_remap_packet(key_code=0x39, target_scancode=0xE0)
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.SET_KEY_REMAP)
        self.assertEqual(packet[2], 0x39)
        self.assertEqual(packet[3], 0xE0)
        self.assertEqual(packet[4], 0x00)

    def test_build_custom_rgb_frame(self):
        # Color WASD keys
        key_colors = {
            0x1A: (255, 0, 0),  # W
            0x04: (0, 255, 0),  # A
            0x16: (0, 0, 255),  # S
            0x07: (255, 255, 0),# D
        }
        packet = build_custom_rgb_frame(key_colors, frame_index=0)
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.SET_CUSTOM_RGB)
        self.assertEqual(packet[2], 0)  # frame index
        # First key: W (0x1A, 255, 0, 0)
        self.assertEqual(packet[3:7], bytes([0x1A, 255, 0, 0]))

    def test_build_reset_packet(self):
        packet = build_reset_packet()
        self.assertEqual(len(packet), 65)
        self.assertEqual(packet[0], REPORT_ID)
        self.assertEqual(packet[1], CommandOpcode.RESET_DEFAULT)
        self.assertEqual(packet[2:], b"\x00" * 63)


if __name__ == "__main__":
    unittest.main()
