"""Tests for firmware parsing, flashing simulation, and verification."""

import os
import unittest
from rk855_gui.core.constants import FLASH_LIMIT
from rk855_gui.core.flasher import load_firmware
from rk855_gui.core.mock_device import simulate_flashing


class TestFlasher(unittest.TestCase):
    def setUp(self):
        self.hex_path = os.path.join(os.path.dirname(__file__), "..", "firmware", "SN32F240B_32.hex")
        self.bin_path = os.path.join(os.path.dirname(__file__), "..", "firmware", "SN32F240B_32.bin")

    def test_load_hex_firmware(self):
        if os.path.isfile(self.hex_path):
            data = load_firmware(self.hex_path)
            self.assertEqual(len(data), FLASH_LIMIT)
            # The official firmware SP vector is within SRAM range (0x20000000 - 0x20002000)
            sp_vector = int.from_bytes(data[0:4], byteorder="little")
            self.assertEqual(sp_vector, 0x200015B0)

    def test_load_bin_firmware(self):
        if os.path.isfile(self.bin_path):
            data = load_firmware(self.bin_path)
            self.assertEqual(len(data), FLASH_LIMIT)

    def test_simulate_flashing_lifecycle(self):
        # 64 KB test buffer
        test_fw = bytes([i % 256 for i in range(32768)]).ljust(FLASH_LIMIT, b"\xFF")
        progress_calls = []
        log_calls = []

        success = simulate_flashing(
            test_fw,
            progress_cb=lambda pct, msg: progress_calls.append((pct, msg)),
            log_cb=lambda lvl, msg: log_calls.append((lvl, msg)),
            verify=True,
            step_delay=0.0001,
        )

        self.assertTrue(success)
        self.assertTrue(len(progress_calls) > 5)
        self.assertEqual(progress_calls[-1][0], 100)
        self.assertTrue(any(lvl == "success" for lvl, _ in log_calls))


if __name__ == "__main__":
    unittest.main()
