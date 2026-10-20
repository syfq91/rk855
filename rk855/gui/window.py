"""Main application window for RK855 Linux GUI."""

from typing import Optional
from ..config import ConfigManager, KeyboardProfile
from ..constants import (
    LightingMode,
    SINGLE_COLOR_LIGHTING_MODES,
    DEFAULT_BACKLIGHT_COLOR,
    DEFAULT_BACKLIGHT_RGB,
)
from ..device import RK855Device, MockRK855Device, RK68Device, MockRK68Device
from ..layout import find_key, KEY_DEFINITIONS, RemapTarget, REMAP_TARGETS
from .keyboard_widget import KeyboardCanvas, PYQT_AVAILABLE
from .key_tester import KeyTesterWidget
from .remap_dialog import KeyPaletteDialog

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


class RK855MainWindow(QMainWindow):
    """Primary desktop interface for Royal Kludge RK855 on Linux."""

    def __init__(
        self,
        device: Optional[RK855Device] = None,
        config_manager: Optional[ConfigManager] = None,
        start_tab: str = "config",
    ):
        if not PYQT_AVAILABLE:
            raise RuntimeError(
                "PyQt6 is required to run the graphical interface. Install with: pip install PyQt6"
            )

        super().__init__()
        self.device = device or RK855Device()
        self.config_manager = config_manager or ConfigManager()
        self.current_profile = self.config_manager.load_profile(
            self.config_manager.get_active_profile_name()
        )
        self.current_color = QColor(self.current_profile.color)

        self.setWindowTitle("Royal Kludge RK855 Linux Configurator")
        self.resize(940, 720)
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
            self.tabs.setCurrentWidget(self.key_tester)
            self.key_tester.setFocus()
        elif start_tab in ("remap", "remapping"):
            self.tabs.setCurrentWidget(self.remap_tab)

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

        # ==========================================
        # Tab 1: 💡 Backlight Lighting
        # ==========================================
        self.lighting_tab = QWidget()
        lighting_layout = QVBoxLayout(self.lighting_tab)
        lighting_layout.setContentsMargins(12, 12, 12, 12)
        lighting_layout.setSpacing(12)

        # Keyboard Canvas Widget for Lighting Preview
        self.keyboard_canvas = KeyboardCanvas(self)
        self.keyboard_canvas.key_selected.connect(self._on_key_selected)
        lighting_layout.addWidget(self.keyboard_canvas, alignment=Qt.AlignmentFlag.AlignCenter)

        # Lighting Box
        light_box = QGroupBox("Backlight & Blue LED Effects (Monochrome)")
        light_layout = QVBoxLayout(light_box)
        light_layout.setSpacing(10)

        # Mode
        mode_row = QHBoxLayout()
        mode_row.addWidget(QLabel("Effect Mode:"))
        self.mode_combo = QComboBox()

        mode_labels = {
            LightingMode.STATIC: "Static (Solid Blue)",
            LightingMode.BREATH: "Breathing (Pulse)",
            LightingMode.BREATHING: "Breathing (Pulse)",
            LightingMode.SCROLLING: "Wave (Brightness Wave)",
            LightingMode.WAVE: "Wave (Brightness Wave)",
            LightingMode.RIPPLES: "Ripple (Actuation Ripple)",
            LightingMode.RIPPLE: "Ripple (Actuation Ripple)",
            LightingMode.SINGLE_ON: "Reactive (Keypress Glow)",
            LightingMode.REACTIVE: "Reactive (Keypress Glow)",
            LightingMode.OFF: "Backlight Off",
        }
        for mode in SINGLE_COLOR_LIGHTING_MODES:
            label = mode_labels.get(mode, mode.name.replace("_", " ").title())
            self.mode_combo.addItem(label, mode)

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

        # Hardware LED Specification Badge
        led_row = QHBoxLayout()
        led_row.addWidget(QLabel("Hardware LED:"))
        led_badge = QLabel("🔵 Monochrome Ice Blue (0x00A8FF)")
        led_badge.setStyleSheet("""
            QLabel {
                color: #80d8ff;
                font-weight: bold;
                background-color: #102a45;
                border: 1px solid #0091ea;
                border-radius: 4px;
                padding: 4px 8px;
            }
        """)
        led_row.addWidget(led_badge)
        led_row.addStretch()
        light_layout.addLayout(led_row)

        lighting_layout.addWidget(light_box)
        lighting_layout.addStretch()

        # Bottom Apply Row for Lighting
        apply_light_layout = QHBoxLayout()
        apply_light_layout.addStretch()
        self.apply_btn = QPushButton("Apply Lighting to RK855")
        self.apply_btn.setObjectName("applyBtn")
        self.apply_btn.setFixedHeight(38)
        self.apply_btn.clicked.connect(self._apply_to_hardware)
        apply_light_layout.addWidget(self.apply_btn)
        lighting_layout.addLayout(apply_light_layout)

        self.tabs.addTab(self.lighting_tab, "💡 Backlight Lighting")

        # ==========================================
        # Tab 2: 🔄 Key Remapping
        # ==========================================
        self.remap_tab = QWidget()
        remap_layout = QVBoxLayout(self.remap_tab)
        remap_layout.setContentsMargins(12, 12, 12, 12)
        remap_layout.setSpacing(12)

        # Dedicated Keyboard Canvas Widget for Key Remapping Selection
        self.remap_canvas = KeyboardCanvas(self)
        self.remap_canvas.key_selected.connect(self._on_key_selected)
        remap_layout.addWidget(self.remap_canvas, alignment=Qt.AlignmentFlag.AlignCenter)

        # Key Mapping Box
        key_box = QGroupBox("Key Remapping")
        key_layout = QVBoxLayout(key_box)
        key_layout.setSpacing(10)

        self.key_info_label = QLabel("Click any keycap on the keyboard above to view or remap.")
        self.key_info_label.setStyleSheet("color: #e5c07b; font-weight: bold; font-size: 13px;")
        key_layout.addWidget(self.key_info_label)

        # Target Button Selection Row
        target_row = QHBoxLayout()
        target_row.addWidget(QLabel("Remap to:"))

        self.remap_target_combo = QComboBox()
        self.remap_target_combo.setEditable(True)
        self.remap_target_combo.setInsertPolicy(QComboBox.InsertPolicy.NoInsert)
        self._populate_remap_combo()
        target_row.addWidget(self.remap_target_combo, stretch=1)

        palette_btn = QPushButton("Browse Buttons...")
        palette_btn.setStyleSheet("""
            QPushButton {
                background-color: #383c48;
                color: #61afef;
                border: 1px solid #4b5263;
                border-radius: 4px;
                padding: 4px 10px;
                font-weight: bold;
            }
            QPushButton:hover {
                background-color: #4b5263;
                color: #ffffff;
            }
        """)
        palette_btn.clicked.connect(self._open_key_palette)
        target_row.addWidget(palette_btn)
        key_layout.addLayout(target_row)

        key_btn_row = QHBoxLayout()
        assign_remap_btn = QPushButton("Remap Selected Key")
        assign_remap_btn.setStyleSheet("background-color: #98c379; color: #1e1e24; font-weight: bold;")
        assign_remap_btn.clicked.connect(self._assign_remap)
        key_btn_row.addWidget(assign_remap_btn)

        clear_remap_btn = QPushButton("Restore Default")
        clear_remap_btn.clicked.connect(self._clear_remap)
        key_btn_row.addWidget(clear_remap_btn)
        key_layout.addLayout(key_btn_row)

        # Active Remappings Overview
        self.active_remaps_label = QLabel()
        self.active_remaps_label.setWordWrap(True)
        key_layout.addWidget(self.active_remaps_label)

        remap_layout.addWidget(key_box)
        remap_layout.addStretch()

        # Bottom Apply Row for Remapping
        apply_remap_layout = QHBoxLayout()
        apply_remap_layout.addStretch()
        self.apply_remap_btn = QPushButton("Apply Remapping to RK855")
        self.apply_remap_btn.setObjectName("applyBtn")
        self.apply_remap_btn.setFixedHeight(38)
        self.apply_remap_btn.clicked.connect(self._apply_to_hardware)
        apply_remap_layout.addWidget(self.apply_remap_btn)
        remap_layout.addLayout(apply_remap_layout)

        self.tabs.addTab(self.remap_tab, "🔄 Key Remapping")

        # ==========================================
        # Tab 3: ⌨️ Key Switch Tester
        # ==========================================
        self.key_tester = KeyTesterWidget(self)
        self.tabs.addTab(self.key_tester, "⌨️ Key Switch Tester")

        main_layout.addWidget(self.tabs)

        self.installEventFilter(self)
        self._check_device_status()

    def _on_tab_changed(self, index: int):
        if hasattr(self, "key_tester") and self.tabs.currentWidget() == self.key_tester:
            self.key_tester.setFocus()

    def eventFilter(self, watched, event):
        if hasattr(self, "tabs") and hasattr(self, "key_tester") and self.tabs.currentWidget() == self.key_tester:
            if event.type() in (QEvent.Type.KeyPress, QEvent.Type.KeyRelease):
                is_press = (event.type() == QEvent.Type.KeyPress)
                consumed = self.key_tester.handle_key_event(event, is_press)
                if consumed:
                    return True
        return super().eventFilter(watched, event)

    def _check_device_status(self):
        devices = RK855Device.enumerate_devices()
        if devices:
            self.status_label.setText("Device: RK855 Connected (Wired USB)")
            self.status_label.setStyleSheet("font-weight: bold; color: #98c379;")
        else:
            self.status_label.setText("Device: RK855 Not Connected (Plug in USB-C cable)")
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
        self.current_color = QColor(DEFAULT_BACKLIGHT_COLOR)
        self.keyboard_canvas.set_brightness(profile.brightness)
        if hasattr(self, "remap_canvas"):
            self.remap_canvas.set_brightness(profile.brightness)
        self._update_active_remaps_display()

    def _update_color_button(self):
        pass

    def _pick_color(self):
        pass

    def _on_lighting_changed(self):
        bright = self.bright_slider.value()
        self.bright_val_label.setText(str(bright))
        self.speed_val_label.setText(str(self.speed_slider.value()))
        self.keyboard_canvas.set_brightness(bright)
        if hasattr(self, "remap_canvas"):
            self.remap_canvas.set_brightness(bright)

    def _populate_remap_combo(self):
        self.remap_target_combo.clear()
        for t in REMAP_TARGETS:
            self.remap_target_combo.addItem(f"{t.desc} ({t.category}) [0x{t.code:02X}]", t)

    def _open_key_palette(self):
        current_target = self.remap_target_combo.currentData()
        current_code = current_target.code if current_target else None
        dlg = KeyPaletteDialog(self, current_code=current_code)
        if dlg.exec():
            selected = dlg.selected_target
            if selected:
                for i in range(self.remap_target_combo.count()):
                    t = self.remap_target_combo.itemData(i)
                    if t and t.code == selected.code:
                        self.remap_target_combo.setCurrentIndex(i)
                        break

    def _on_key_selected(self, key_def):
        if hasattr(self, "keyboard_canvas"):
            self.keyboard_canvas.selected_key = key_def
            self.keyboard_canvas.update()
        if hasattr(self, "remap_canvas"):
            self.remap_canvas.selected_key = key_def
            self.remap_canvas.update()

        current_map = self.current_profile.key_remaps.get(key_def.name)
        if current_map:
            target_key = find_key(current_map)
            desc_text = target_key.desc if target_key else current_map
            self.key_info_label.setText(
                f"Selected Key: {key_def.desc} ({key_def.name})\nRemapped to: {desc_text}"
            )
            # Pre-select in target combo
            for i in range(self.remap_target_combo.count()):
                t = self.remap_target_combo.itemData(i)
                if t and (t.name.lower() == str(current_map).lower() or hex(t.code).lower() == str(current_map).lower()):
                    self.remap_target_combo.setCurrentIndex(i)
                    break
        else:
            self.key_info_label.setText(
                f"Selected Key: {key_def.desc} ({key_def.name})\nMapping: Factory Default"
            )
            for i in range(self.remap_target_combo.count()):
                t = self.remap_target_combo.itemData(i)
                if t and t.code == key_def.code:
                    self.remap_target_combo.setCurrentIndex(i)
                    break

    def _update_active_remaps_display(self):
        if not hasattr(self, "active_remaps_label"):
            return
        if not self.current_profile or not self.current_profile.key_remaps:
            self.active_remaps_label.setText("Active Remaps: None (all 68 keys at factory default).")
            self.active_remaps_label.setStyleSheet("color: #7f848e; font-style: italic; font-size: 12px; padding-top: 4px;")
        else:
            items = []
            for k_name, scancode_str in self.current_profile.key_remaps.items():
                k_def = find_key(k_name)
                src = k_def.desc if k_def else k_name
                tgt_key = find_key(scancode_str)
                tgt = tgt_key.desc if tgt_key else scancode_str
                items.append(f"<span style='color: #61afef;'>{src}</span> ➔ <span style='color: #98c379;'>{tgt}</span>")
            summary = "<b>Active Remaps:</b> " + " &nbsp;|&nbsp; ".join(items)
            self.active_remaps_label.setText(summary)
            self.active_remaps_label.setStyleSheet("color: #abb2bf; font-size: 12px; padding-top: 4px;")

    def _assign_remap(self):
        selected = (getattr(self, "remap_canvas", None) and self.remap_canvas.selected_key) or self.keyboard_canvas.selected_key
        if not selected:
            QMessageBox.warning(self, "No Key Selected", "Please click a keycap on the keyboard above first.")
            return

        target = self.remap_target_combo.currentData()
        if not target:
            return

        self.current_profile.key_remaps[selected.name] = hex(target.code)
        self.key_info_label.setText(
            f"Selected Key: {selected.desc} ({selected.name})\nRemapped to: {target.desc} (0x{target.code:02X})"
        )
        self._update_active_remaps_display()
        QMessageBox.information(
            self,
            "Key Remapped",
            f"Key '{selected.desc}' will be remapped to '{target.desc}' (0x{target.code:02X}).\n\n"
            "Click 'Apply Remapping to RK855' below to write to keyboard onboard memory."
        )

    def _clear_remap(self):
        selected = (getattr(self, "remap_canvas", None) and self.remap_canvas.selected_key) or self.keyboard_canvas.selected_key
        if not selected:
            QMessageBox.warning(self, "No Key Selected", "Please click a keycap on the keyboard above first.")
            return
        if selected.name in self.current_profile.key_remaps:
            del self.current_profile.key_remaps[selected.name]
            self.key_info_label.setText(
                f"Selected Key: {selected.desc} ({selected.name})\nMapping: Factory Default"
            )
            self._update_active_remaps_display()
            QMessageBox.information(
                self, "Remap Cleared", f"Key '{selected.desc}' restored to factory default."
            )
        else:
            QMessageBox.information(
                self, "Default", f"Key '{selected.desc}' is already at its factory default."
            )

    def _color_selected_key(self):
        pass

    def _on_profile_changed(self, profile_name):
        if not profile_name:
            return
        self.current_profile = self.config_manager.load_profile(profile_name)
        self._load_profile_ui(self.current_profile)

    def _save_current_profile(self):
        self.current_profile.lighting_mode = self.mode_combo.currentData().name.lower()
        self.current_profile.brightness = self.bright_slider.value()
        self.current_profile.speed = self.speed_slider.value()
        self.current_profile.color = DEFAULT_BACKLIGHT_COLOR
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

        try:
            with self.device:
                self.device.set_lighting(
                    mode=mode,
                    speed=speed,
                    brightness=brightness,
                    color_rgb=DEFAULT_BACKLIGHT_RGB,
                    direction=self.current_profile.direction,
                )
                # Remaps
                remap_dict = {}
                for k_name, scancode_str in self.current_profile.key_remaps.items():
                    k_def = find_key(k_name)
                    if k_def:
                        try:
                            target_k = find_key(scancode_str)
                            target_sc = target_k.code if target_k else (
                                int(scancode_str, 16) if scancode_str.startswith("0x") else int(scancode_str)
                            )
                            remap_dict[k_def.code] = target_sc
                        except ValueError:
                            pass
                if remap_dict:
                    self.device.remap_keys(remap_dict)
            QMessageBox.information(self, "Success", "Configuration sent successfully to RK855 keyboard!")
        except Exception as e:
            QMessageBox.critical(
                self, "Communication Error",
                f"Could not send configuration to RK855: {e}\n\n"
                "Ensure keyboard is connected via wired USB cable and udev rules are active."
            )


# Backward compatibility alias
RK68MainWindow = RK855MainWindow
