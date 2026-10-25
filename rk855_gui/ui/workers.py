"""Asynchronous worker threads for device polling and firmware flashing."""

import time
from PyQt6.QtCore import QThread, pyqtSignal

from ..core.device import RK855Device, DeviceMode
from ..core.flasher import flash_firmware_bytes
from ..core.mock_device import simulate_flashing


class DeviceWatcher(QThread):
    """Periodically queries attached USB HID devices."""

    status_changed = pyqtSignal(object, object, object)  # (DeviceMode, info_dict, err_msg)

    def __init__(self, parent=None, interval_sec: float = 1.5):
        super().__init__(parent)
        self.interval = interval_sec
        self._running = True
        self._is_simulator = False
        self._last_mode = None

    def set_simulator_mode(self, enabled: bool) -> None:
        self._is_simulator = enabled
        self._last_mode = None  # Force immediate update

    def run(self) -> None:
        while self._running:
            if self._is_simulator:
                self.status_changed.emit(DeviceMode.NORMAL, {"product_string": "RK855 Simulator"}, None)
            else:
                mode, info, err = RK855Device.get_status()
                # Emit update
                self.status_changed.emit(mode, info, err)
                self._last_mode = mode

            # Sleep in small slices for fast responsiveness when stopping
            for _ in range(int(self.interval * 10)):
                if not self._running:
                    break
                time.sleep(0.1)

    def stop(self) -> None:
        self._running = False
        self.wait(1000)


class FlashWorker(QThread):
    """Executes firmware erase, flashing, and verification in a background thread."""

    progress = pyqtSignal(int, str)
    log_message = pyqtSignal(str, str)  # (level, text)
    finished = pyqtSignal(bool, str)    # (success, message)

    def __init__(
        self,
        firmware_bytes: bytes,
        is_simulator: bool = False,
        verify: bool = True,
        parent=None,
    ):
        super().__init__(parent)
        self.firmware_bytes = firmware_bytes
        self.is_simulator = is_simulator
        self.verify = verify
        self._cancelled = False

    def cancel(self) -> None:
        self._cancelled = True

    def run(self) -> None:
        def on_progress(pct: int, msg: str):
            self.progress.emit(pct, msg)

        def on_log(lvl: str, msg: str):
            self.log_message.emit(lvl, msg)

        def is_cancelled() -> bool:
            return self._cancelled

        try:
            if self.is_simulator:
                success = simulate_flashing(
                    self.firmware_bytes,
                    progress_cb=on_progress,
                    log_cb=on_log,
                    cancel_check=is_cancelled,
                    verify=self.verify,
                )
            else:
                success = flash_firmware_bytes(
                    self.firmware_bytes,
                    progress_cb=on_progress,
                    log_cb=on_log,
                    cancel_check=is_cancelled,
                    verify=self.verify,
                )

            if success:
                self.finished.emit(True, "Firmware successfully flashed and verified!")
            else:
                self.finished.emit(False, "Firmware update failed. Check the log for details.")
        except Exception as e:
            self.log_message.emit("error", f"Unhandled exception in flash worker: {e}")
            self.finished.emit(False, str(e))
