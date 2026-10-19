"""Main application window for RK68 Linux GUI."""

from typing import Optional
from ..config import ConfigManager, KeyboardProfile
from ..constants import LightingMode
from ..device import RK68Device, MockRK68Device
from ..layout import find_key, KEY_DEFINITIONS
from .keyboard_widget import KeyboardCanvas, PYQT_AVAILABLE
from .key_tester import KeyTesterWidget

if PYQT_AVAILABLE:
    from PyQt6.QtCore import Qt, QEvent
    from PyQt6.QtGui import QColor, QIcon
    from PyQt6.QtWidgets import (
        QColorDialog,
        QComboBox,
        QGroupBox,
        QHBoxLayout,
        QLabel,
        QLineEdit,
        QMainWindow,
        QMessageBox,
        QPushButton,
        QSlider,
        QSpinBox,
        QTabWidget,
        QVBoxLayout,
        QWidget,
    )
else:
    QMainWindow = object


class RK68MainWindow(QMainWindow):
    """Primary desktop interface for Royal Kludge RK68 on Linux."""

    def __init__(
        self,
        device: Optional[RK68Device] = None,
        config_manager: Optional[ConfigManager] = None,
        start_tab: str = "config",
    ):
        if not PYQT_AVAILABLE:
            raise RuntimeError(
                "PyQt6 is required to run the graphical interface. Install with: pip install PyQt6"
            )

        super().__init__()
        self.device = device or RK68Device()
        self.config_manager = config_manager or ConfigManager()
        self.current_profile = self.config_manager.load_profile(
            self.config_manager.get_active_profile_name()
        )
        self.current_color = QColor(self.current_profile.color)

        self.setWindowTitle("Royal Kludge RK68 Linux Configurator")
        self.resize(940, 680)
        self.setStyleSheet("""
            QMainWindow { background-color: #1e1e24; }
            QLabel { color: #e0e0e0; font-size: 13px; }
            QGroupBox { color: #61afef; font-weight: bold; border: 1px solid #3e4451; margin-top: 8px; padding-top: 14px; border-radius: 6px; }
            QPushButton { background-color: #2c313a; color: #ffffff; border: 1px solid #4b5263; border-radius: 4px; padding: 6px 14px; font-weight: bold; }
            QPushButton:hover { background-color: #3e4451; }
            QPushButton#applyBtn { background-color: #98c379; color: #1e1e24; }
            QPushButton#applyBtn:hover { background-color: #b5e890; }
            QComboBox { background-color: #2c313a; color: #ffffff; border: 1px solid #4b5263; border-radius: 4px; padding: 4px 8px; }
            QSlider::groove:horizontal { height: 6px; background: #3e4451; border-radius: 3px; }
            QSlider::handle:horizontal { background: #61afef; width: 16px; margin-top: -5px; margin-bottom: -5px; border-radius: 8px; }
            QTabWidget::pane { border: 1px solid #383c48; border-radius: 6px; background-color: #1e1e24; top: -1px; }
            QTabBar::tab { background: #252830; color: #abb2bf; border: 1px solid #383c48; border-bottom: none; padding: 7px 18px; margin-right: 4px; border-top-left-radius: 6px; border-top-right-radius: 6px; font-weight: bold; font-size: 13px; }
            QTabBar::tab:selected { background: #2c313a; color: #61afef; border-color: #61afef; border-bottom: 2px solid #61afef; }
            QTabBar::tab:hover:!selected { background: #2e333d; color: #ffffff; }
        """)

        self._init_ui()
        self._load_profile_ui(self.current_profile)

        if start_tab in ("test", "tester", "key_tester"):
            self.tabs.setCurrentIndex(1)
            self.key_tester.setFocus()

    def _init_ui(self):
        main_widget = QWidget()
        self.setCentralWidget(main_widget)
        main_layout = QVBoxLayout(main_widget)
        main_layout.setSpacing(14)
        main_layout.setContentsMargins(18, 18, 18, 18)

        # Top Bar: Connection status & Profile selector
        top_layout = QHBoxLayout()
        self.status_label = QLabel("Checking device status...")
        self.status_label.setStyleSheet("font-weight: bold; color: #e5c07b;")
        top_layout.addWidget(self.status_label)
        top_layout.addStretch()

        top_layout.addWidget(QLabel("Profile:"))
        self.profile_combo = QComboBox()
        self._refresh_profiles()
        self.profile_combo.currentTextChanged.connect(self._on_profile_changed)
        top_layout.addWidget(self.profile_combo)

        save_profile_btn = QPushButton("Save Profile")
        save_profile_btn.clicked.connect(self._save_current_profile)
        top_layout.addWidget(save_profile_btn)

        reset_btn = QPushButton("Reset Defaults")
        reset_btn.clicked.connect(self._reset_defaults)
        top_layout.addWidget(reset_btn)

        main_layout.addLayout(top_layout)

        # Tab Navigation Widget
        self.tabs = QTabWidget()
        self.tabs.currentChanged.connect(self._on_tab_changed)

        # Tab 1: Configurator (Lighting & Remapping)
        config_tab = QWidget()
        config_layout = QVBoxLayout(config_tab)
        config_layout.setContentsMargins(4, 8, 4, 4)
        config_layout.setSpacing(10)

        # Keyboard Canvas Widget
        self.keyboard_canvas = KeyboardCanvas(self)
        self.keyboard_canvas.key_selected.connect(self._on_key_selected)
        config_layout.addWidget(self.keyboard_canvas, alignment=Qt.AlignmentFlag.AlignCenter)

        # Controls Section
        bottom_layout = QHBoxLayout()

        # Lighting Box
        light_box = QGroupBox("Backlight & RGB Effects")
        light_layout = QVBoxLayout(light_box)

        # Mode
        mode_row = QHBoxLayout()
        mode_row.addWidget(QLabel("Effect Mode:"))
        self.mode_combo = QComboBox()
        for mode in LightingMode:
            self.mode_combo.addItem(mode.name.replace("_", " ").title(), mode)
        self.mode_combo.currentIndexChanged.connect(self._on_lighting_changed)
        mode_row.addWidget(self.mode_combo)
        light_layout.addLayout(mode_row)

        # Brightness
        bright_row = QHBoxLayout()
        bright_row.addWidget(QLabel("Brightness (0-5):"))
        self.bright_slider = QSlider(Qt.Orientation.Horizontal)
        self.bright_slider.setRange(0, 5)
        self.bright_slider.setValue(4)
        self.bright_slider.valueChanged.connect(self._on_lighting_changed)
        bright_row.addWidget(self.bright_slider)
        self.bright_val_label = QLabel("4")
        bright_row.addWidget(self.bright_val_label)
        light_layout.addLayout(bright_row)

        # Speed
        speed_row = QHBoxLayout()
        speed_row.addWidget(QLabel("Speed (1-5):"))
        self.speed_slider = QSlider(Qt.Orientation.Horizontal)
        self.speed_slider.setRange(1, 5)
        self.speed_slider.setValue(3)
        self.speed_slider.valueChanged.connect(self._on_lighting_changed)
        speed_row.addWidget(self.speed_slider)
        self.speed_val_label = QLabel("3")
        speed_row.addWidget(self.speed_val_label)
        light_layout.addLayout(speed_row)

        # Color
        color_row = QHBoxLayout()
        color_row.addWidget(QLabel("Backlight Color:"))
        self.color_btn = QPushButton()
        self.color_btn.setFixedHeight(26)
        self.color_btn.clicked.connect(self._pick_color)
        self._update_color_button()
        color_row.addWidget(self.color_btn)
        light_layout.addLayout(color_row)

        bottom_layout.addWidget(light_box, stretch=3)

        # Key Mapping & Custom Key Box
        key_box = QGroupBox("Selected Key Details")
        key_layout = QVBoxLayout(key_box)

        self.key_info_label = QLabel("Click any keycap above to view or remap.")
        key_layout.addWidget(self.key_info_label)

        remap_row = QHBoxLayout()
        remap_row.addWidget(QLabel("Remap to Scancode:"))
        self.remap_input = QLineEdit()
        self.remap_input.setPlaceholderText("e.g. 0x14 or Control")
        remap_row.addWidget(self.remap_input)
        key_layout.addLayout(remap_row)

        key_btn_row = QHBoxLayout()
        assign_remap_btn = QPushButton("Remap Key")
        assign_remap_btn.clicked.connect(self._assign_remap)
        key_btn_row.addWidget(assign_remap_btn)

        color_key_btn = QPushButton("Color This Key")
        color_key_btn.clicked.connect(self._color_selected_key)
        key_btn_row.addWidget(color_key_btn)

        key_layout.addLayout(key_btn_row)
        key_layout.addStretch()

        bottom_layout.addWidget(key_box, stretch=2)

        config_layout.addLayout(bottom_layout)

        # Bottom Apply Row
        apply_layout = QHBoxLayout()
        apply_layout.addStretch()
        self.apply_btn = QPushButton("Apply Configuration to RK68")
        self.apply_btn.setObjectName("applyBtn")
        self.apply_btn.setFixedHeight(38)
        self.apply_btn.clicked.connect(self._apply_to_hardware)
        apply_layout.addWidget(self.apply_btn)
        config_layout.addLayout(apply_layout)

        self.tabs.addTab(config_tab, "🎛️ Lighting & Remap")

        # Tab 2: Key Switch Tester
        self.key_tester = KeyTesterWidget(self)
        self.tabs.addTab(self.key_tester, "⌨️ Key Switch Tester")

        main_layout.addWidget(self.tabs)

        self.installEventFilter(self)
        self._check_device_status()

    def _on_tab_changed(self, index: int):
        if index == 1:
            self.key_tester.setFocus()

    def eventFilter(self, watched, event):
        if hasattr(self, "tabs") and self.tabs.currentIndex() == 1:
            if event.type() in (QEvent.Type.KeyPress, QEvent.Type.KeyRelease):
                is_press = (event.type() == QEvent.Type.KeyPress)
                consumed = self.key_tester.handle_key_event(event, is_press)
                if consumed:
                    return True
        return super().eventFilter(watched, event)

    def _check_device_status(self):
        devices = RK68Device.enumerate_devices()
        if devices:
            self.status_label.setText("Device: RK68 Connected (Wired USB)")
            self.status_label.setStyleSheet("font-weight: bold; color: #98c379;")
        else:
            self.status_label.setText("Device: RK68 Not Connected (Plug in USB-C cable)")
            self.status_label.setStyleSheet("font-weight: bold; color: #e06c75;")

    def _refresh_profiles(self):
        self.profile_combo.clear()
        profiles = self.config_manager.list_profiles()
        self.profile_combo.addItems(profiles)
        active = self.config_manager.get_active_profile_name()
        if active in profiles:
            self.profile_combo.setCurrentText(active)

    def _load_profile_ui(self, profile: KeyboardProfile):
        mode = LightingMode.from_string(profile.lighting_mode)
        idx = self.mode_combo.findData(mode)
        if idx >= 0:
            self.mode_combo.setCurrentIndex(idx)

        self.bright_slider.setValue(profile.brightness)
        self.bright_val_label.setText(str(profile.brightness))
        self.speed_slider.setValue(profile.speed)
        self.speed_val_label.setText(str(profile.speed))
        self.current_color = QColor(profile.color)
        self._update_color_button()
        self.keyboard_canvas.set_all_colors(
            self.current_color.red(), self.current_color.green(), self.current_color.blue()
        )

    def _update_color_button(self):
        hex_code = self.current_color.name()
        self.color_btn.setText(hex_code)
        self.color_btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {hex_code};
                color: {'#000000' if self.current_color.lightness() > 140 else '#FFFFFF'};
                font-weight: bold;
                border: 1px solid #ffffff;
            }}
        """)

    def _pick_color(self):
        color = QColorDialog.getColor(self.current_color, self, "Select RK68 Backlight Color")
        if color.isValid():
            self.current_color = color
            self._update_color_button()
            self._on_lighting_changed()

    def _on_lighting_changed(self):
        self.bright_val_label.setText(str(self.bright_slider.value()))
        self.speed_val_label.setText(str(self.speed_slider.value()))
        r, g, b = self.current_color.red(), self.current_color.green(), self.current_color.blue()
        self.keyboard_canvas.set_all_colors(r, g, b)

    def _on_key_selected(self, key_def):
        self.key_info_label.setText(
            f"Key: {key_def.desc} ({key_def.name})\nScancode: 0x{key_def.code:02X}"
        )
        if key_def.name in self.current_profile.key_remaps:
            self.remap_input.setText(self.current_profile.key_remaps[key_def.name])
        else:
            self.remap_input.clear()

    def _assign_remap(self):
        selected = self.keyboard_canvas.selected_key
        if not selected:
            QMessageBox.warning(self, "No Key Selected", "Please select a key on the keyboard canvas first.")
            return
        target = self.remap_input.text().strip()
        if not target:
            return
        self.current_profile.key_remaps[selected.name] = target
        QMessageBox.information(
            self, "Key Remapped", f"Key '{selected.desc}' will be remapped to '{target}' on Apply."
        )

    def _color_selected_key(self):
        selected = self.keyboard_canvas.selected_key
        if not selected:
            QMessageBox.warning(self, "No Key Selected", "Please select a key on the keyboard canvas first.")
            return
        color = QColorDialog.getColor(self.current_color, self, f"Select Color for Key '{selected.desc}'")
        if color.isValid():
            self.keyboard_canvas.set_key_color(selected.code, color.red(), color.green(), color.blue())
            self.current_profile.custom_rgb[selected.name] = color.name()

    def _on_profile_changed(self, profile_name):
        if not profile_name:
            return
        self.current_profile = self.config_manager.load_profile(profile_name)
        self._load_profile_ui(self.current_profile)

    def _save_current_profile(self):
        self.current_profile.lighting_mode = self.mode_combo.currentData().name.lower()
        self.current_profile.brightness = self.bright_slider.value()
        self.current_profile.speed = self.speed_slider.value()
        self.current_profile.color = self.current_color.name()
        self.config_manager.save_profile(self.current_profile)
        QMessageBox.information(self, "Profile Saved", f"Profile '{self.current_profile.name}' saved successfully.")

    def _reset_defaults(self):
        confirm = QMessageBox.question(
            self, "Reset Defaults", "Reset keyboard configuration to factory defaults?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if confirm == QMessageBox.StandardButton.Yes:
            try:
                with self.device:
                    self.device.reset_defaults()
                QMessageBox.information(self, "Reset Complete", "Keyboard reset to factory defaults.")
            except Exception as e:
                QMessageBox.critical(self, "Error", f"Failed to reset device: {e}")

    def _apply_to_hardware(self):
        mode = self.mode_combo.currentData()
        speed = self.speed_slider.value()
        brightness = self.bright_slider.value()
        r, g, b = self.current_color.red(), self.current_color.green(), self.current_color.blue()

        try:
            with self.device:
                self.device.set_lighting(
                    mode=mode,
                    speed=speed,
                    brightness=brightness,
                    color_rgb=(r, g, b),
                    direction=self.current_profile.direction,
                )
                # Remaps
                for k_name, scancode_str in self.current_profile.key_remaps.items():
                    k_def = find_key(k_name)
                    if k_def:
                        try:
                            target_sc = int(scancode_str, 16) if scancode_str.startswith("0x") else int(scancode_str)
                            self.device.remap_key(k_def.code, target_sc)
                        except ValueError:
                            pass
            QMessageBox.information(self, "Success", "Configuration sent successfully to RK68 keyboard!")
        except Exception as e:
            QMessageBox.critical(
                self, "Communication Error",
                f"Could not send configuration to RK68: {e}\n\n"
                "Ensure keyboard is connected via wired USB cable and udev rules are active."
            )
