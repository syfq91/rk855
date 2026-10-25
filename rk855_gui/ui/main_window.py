"""Main Window for Royal Kludge RK855 GUI Configurator & Firmware Flasher."""

import hashlib
import os
from PyQt6.QtCore import Qt, QSize
from PyQt6.QtGui import QIcon, QFont
from PyQt6.QtWidgets import (
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QTabWidget,
    QLabel,
    QComboBox,
    QSlider,
    QPushButton,
    QFileDialog,
    QMessageBox,
    QProgressBar,
    QCheckBox,
    QLineEdit,
    QGroupBox,
    QGridLayout,
    QFormLayout,
    QTextEdit,
    QSplitter,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
    QApplication,
)

from ..core.constants import (
    LIGHTING_MODES,
    KEY_PRESETS,
    PHYSICAL_KEYS,
    REMAP_ACTIONS,
    ACTION_BY_CODE,
    RemapAction,
    KEYBOARD_VID,
    KEYBOARD_PID,
    BOOTLOADER_VID,
    BOOTLOADER_PID,
    FLASH_LIMIT,
)
from ..core.device import RK855Device, DeviceMode, get_udev_rule_text
from ..core.lighting import apply_lighting_mode, apply_custom_matrix
from ..core.remap import apply_remap_table
from ..core.flasher import (
    load_firmware,
    trigger_bootloader_mode,
    reboot_to_normal_mode,
)
from ..core.mock_device import MockRK855Device
from ..widgets.keyboard_view import KeyboardView
from ..widgets.color_picker import ColorPickerWidget
from ..widgets.log_console import LogConsole
from ..widgets.status_banner import StatusBanner
from .workers import DeviceWatcher, FlashWorker


DARK_STYLESHEET = """
QMainWindow {
    background-color: #1e222b;
}
QWidget {
    background-color: #1e222b;
    color: #abb2bf;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 12px;
}
QTabWidget::pane {
    border: 1px solid #2d3139;
    background-color: #1e222b;
    border-radius: 6px;
    top: -1px;
}
QTabBar::tab {
    background-color: #181a1f;
    color: #7f848e;
    padding: 8px 18px;
    margin-right: 3px;
    border-top-left-radius: 6px;
    border-top-right-radius: 6px;
    border: 1px solid #282c34;
    border-bottom: none;
    font-weight: 600;
}
QTabBar::tab:selected {
    background-color: #21252b;
    color: #61afef;
    border-color: #3e4451;
}
QTabBar::tab:hover:!selected {
    background-color: #2c313a;
    color: #d7dae0;
}
QGroupBox {
    border: 1px solid #2d3139;
    border-radius: 6px;
    margin-top: 14px;
    padding-top: 14px;
    font-weight: bold;
    color: #98c379;
}
QGroupBox::title {
    subcontrol-origin: margin;
    subcontrol-position: top left;
    padding: 0 6px;
    left: 10px;
}
QPushButton {
    background-color: #2d3139;
    color: #d7dae0;
    border: 1px solid #3e4451;
    border-radius: 4px;
    padding: 6px 14px;
    font-weight: 500;
}
QPushButton:hover {
    background-color: #3e4451;
    color: #ffffff;
    border-color: #4b5263;
}
QPushButton:pressed {
    background-color: #1e222b;
}
QPushButton:disabled {
    background-color: #1a1c23;
    color: #5c6370;
    border-color: #282c34;
}
QPushButton#primaryBtn {
    background-color: #2c5282;
    color: #ffffff;
    border: 1px solid #3182ce;
    font-weight: bold;
}
QPushButton#primaryBtn:hover {
    background-color: #3182ce;
}
QPushButton#dangerBtn {
    background-color: #742a2a;
    color: #ffffff;
    border: 1px solid #e53e3e;
    font-weight: bold;
}
QPushButton#dangerBtn:hover {
    background-color: #9b2c2c;
}
QComboBox, QLineEdit, QSpinBox {
    background-color: #181a1f;
    color: #abb2bf;
    border: 1px solid #2d3139;
    border-radius: 4px;
    padding: 5px 8px;
}
QComboBox:focus, QLineEdit:focus, QSpinBox:focus {
    border-color: #61afef;
}
QSlider::groove:horizontal {
    height: 6px;
    background: #181a1f;
    border-radius: 3px;
}
QSlider::sub-page:horizontal {
    background: #61afef;
    border-radius: 3px;
}
QSlider::handle:horizontal {
    background: #abb2bf;
    border: 1px solid #5c6370;
    width: 14px;
    margin-top: -4px;
    margin-bottom: -4px;
    border-radius: 7px;
}
QSlider::handle:horizontal:hover {
    background: #ffffff;
}
QProgressBar {
    background-color: #181a1f;
    border: 1px solid #2d3139;
    border-radius: 4px;
    text-align: center;
    color: #ffffff;
    font-weight: bold;
}
QProgressBar::chunk {
    background-color: #98c379;
    border-radius: 3px;
}
"""


class MainWindow(QMainWindow):
    """Main Application Window for RK855 Configurator and Flasher."""

    def __init__(self, mock: bool = False, initial_tab: int = 0):
        super().__init__()
        self.setWindowTitle("Royal Kludge RK855 (RK68) - Open Tooling Suite")
        self.resize(920, 680)
        self.setMinimumSize(840, 580)
        self.setStyleSheet(DARK_STYLESHEET)

        self._is_simulator = mock
        self._current_device_mode = DeviceMode.DISCONNECTED
        self._current_device_info = None
        self._mock_device = MockRK855Device() if mock else None
        self._loaded_firmware_bytes: bytes | None = None
        self._flash_worker: FlashWorker | None = None
        self._remap_dict: dict[int, int] = {}  # {physical_scancode: target_code}
        self._selected_remap_key_index: int = 0  # Esc

        self._setup_ui(initial_tab)
        self._init_watcher()

        if mock:
            self.status_banner.chk_mock.setChecked(True)

    def _setup_ui(self, initial_tab: int) -> None:
        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QVBoxLayout(central)
        main_layout.setContentsMargins(10, 10, 10, 10)
        main_layout.setSpacing(8)

        # Header Banner
        self.status_banner = StatusBanner()
        self.status_banner.refresh_requested.connect(self._on_refresh_device)
        self.status_banner.simulator_toggled.connect(self._on_simulator_toggled)
        main_layout.addWidget(self.status_banner)

        # Tabs
        self.tabs = QTabWidget()
        self.tab_lighting = self._create_lighting_tab()
        self.tab_per_key = self._create_per_key_tab()
        self.tab_remap = self._create_remap_tab()
        self.tab_flasher = self._create_flasher_tab()
        self.tab_info = self._create_info_tab()

        self.tabs.addTab(self.tab_lighting, "💡 Backlight & Effects")
        self.tabs.addTab(self.tab_per_key, "🎨 Per-Key RGB Studio")
        self.tabs.addTab(self.tab_remap, "⌨️ Key Remapping")
        self.tabs.addTab(self.tab_flasher, "⚡ Firmware Flasher")
        self.tabs.addTab(self.tab_info, "ℹ️ Device & udev Help")

        self.tabs.setCurrentIndex(initial_tab)
        main_layout.addWidget(self.tabs)

    # --------------------------------------------------------------------------
    # Tab 1: Lighting Effects
    # --------------------------------------------------------------------------
    def _create_lighting_tab(self) -> QWidget:
        tab = QWidget()
        layout = QHBoxLayout(tab)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(16)

        # Left Column: Effect selection & sliders
        left_box = QWidget()
        left_layout = QVBoxLayout(left_box)
        left_layout.setContentsMargins(0, 0, 0, 0)
        left_layout.setSpacing(12)

        # Mode selector
        mode_group = QGroupBox("Lighting Effect Mode")
        mode_layout = QVBoxLayout(mode_group)
        self.combo_modes = QComboBox()
        for mode_id, info in LIGHTING_MODES.items():
            self.combo_modes.addItem(f"[{mode_id:2d}] {info.name}", mode_id)
        self.combo_modes.currentIndexChanged.connect(self._on_lighting_mode_selected)
        mode_layout.addWidget(self.combo_modes)

        self.lbl_mode_desc = QLabel()
        self.lbl_mode_desc.setWordWrap(True)
        self.lbl_mode_desc.setStyleSheet("color: #7f848e; font-style: italic; padding: 4px;")
        mode_layout.addWidget(self.lbl_mode_desc)
        left_layout.addWidget(mode_group)

        # Effect Parameters
        param_group = QGroupBox("Effect Parameters")
        param_layout = QFormLayout(param_group)
        param_layout.setSpacing(10)

        # Brightness (0-4)
        self.slider_bright = QSlider(Qt.Orientation.Horizontal)
        self.slider_bright.setRange(0, 4)
        self.slider_bright.setValue(3)
        self.lbl_bright_val = QLabel("Level 3 / 4")
        self.slider_bright.valueChanged.connect(lambda v: self.lbl_bright_val.setText(f"Level {v} / 4"))
        param_layout.addRow("Brightness:", self._wrap_slider(self.slider_bright, self.lbl_bright_val))

        # Speed (0-4)
        self.slider_speed = QSlider(Qt.Orientation.Horizontal)
        self.slider_speed.setRange(0, 4)
        self.slider_speed.setValue(2)
        self.lbl_speed_val = QLabel("Level 2 / 4")
        self.slider_speed.valueChanged.connect(lambda v: self.lbl_speed_val.setText(f"Level {v} / 4"))
        param_layout.addRow("Speed:", self._wrap_slider(self.slider_speed, self.lbl_speed_val))

        # Direction (0 or 1)
        self.combo_dir = QComboBox()
        self.combo_dir.addItem("Left-to-Right / Clockwise", 0)
        self.combo_dir.addItem("Right-to-Left / Counter-clockwise", 1)
        param_layout.addRow("Direction:", self.combo_dir)

        left_layout.addWidget(param_group)

        # Quick action buttons
        btn_box = QHBoxLayout()
        self.btn_apply_light = QPushButton("Apply & Save to Keyboard")
        self.btn_apply_light.setObjectName("primaryBtn")
        self.btn_apply_light.clicked.connect(self._on_apply_lighting)

        self.btn_off = QPushButton("Backlight Off")
        self.btn_off.clicked.connect(self._on_turn_off_backlight)

        btn_box.addWidget(self.btn_off)
        btn_box.addWidget(self.btn_apply_light)
        left_layout.addLayout(btn_box)

        left_layout.addStretch()
        layout.addWidget(left_box, 1)

        # Right Column: Color Picker
        right_group = QGroupBox("Color Configuration")
        right_layout = QVBoxLayout(right_group)
        self.light_color_picker = ColorPickerWidget(initial_color=(0, 220, 255))
        right_layout.addWidget(self.light_color_picker)

        lbl_single_color_tip = QLabel(
            "💡 <b>Single-Color / White LED Models:</b> All 20 animation effects, brightness, and speed levels are fully supported via MCU PWM. Color controls illuminate in the physical LED color."
        )
        lbl_single_color_tip.setWordWrap(True)
        lbl_single_color_tip.setStyleSheet("color: #7f848e; font-size: 11px; padding: 6px 4px;")
        right_layout.addWidget(lbl_single_color_tip)

        right_layout.addStretch()
        layout.addWidget(right_group, 1)

        # Trigger initial selection
        self._on_lighting_mode_selected()
        return tab

    def _wrap_slider(self, slider: QSlider, val_label: QLabel) -> QWidget:
        w = QWidget()
        lay = QHBoxLayout(w)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.addWidget(slider)
        lay.addWidget(val_label)
        return w

    def _on_lighting_mode_selected(self) -> None:
        mode_id = self.combo_modes.currentData()
        if mode_id in LIGHTING_MODES:
            info = LIGHTING_MODES[mode_id]
            self.lbl_mode_desc.setText(info.description)
            self.slider_speed.setEnabled(info.has_speed)
            self.combo_dir.setEnabled(info.has_direction)
            self.light_color_picker.setEnabled(info.has_color)

    def _on_apply_lighting(self) -> None:
        mode_id = self.combo_modes.currentData()
        brightness = self.slider_bright.value()
        speed = self.slider_speed.value()
        direction = self.combo_dir.currentData()
        r, g, b = self.light_color_picker.get_color()

        if self._is_simulator or self._mock_device:
            apply_lighting_mode(
                self._mock_device,
                mode_id=mode_id,
                brightness=brightness,
                speed=speed,
                r=r,
                g=g,
                b=b,
                direction=direction,
            )
            QMessageBox.information(self, "Success", f"[Simulator] Lighting mode '{LIGHTING_MODES[mode_id].name}' applied!")
            return

        try:
            dev = RK855Device(KEYBOARD_VID, KEYBOARD_PID)
            with dev:
                apply_lighting_mode(
                    dev,
                    mode_id=mode_id,
                    brightness=brightness,
                    speed=speed,
                    r=r,
                    g=g,
                    b=b,
                    direction=direction,
                )
            QMessageBox.information(self, "Success", f"Lighting mode '{LIGHTING_MODES[mode_id].name}' applied and saved!")
        except Exception as e:
            QMessageBox.critical(self, "Communication Error", f"Failed to apply lighting: {e}")

    def _on_turn_off_backlight(self) -> None:
        idx = self.combo_modes.findData(19)
        if idx >= 0:
            self.combo_modes.setCurrentIndex(idx)
        self.slider_bright.setValue(0)
        self._on_apply_lighting()

    # --------------------------------------------------------------------------
    # Tab 2: Per-Key RGB Studio
    # --------------------------------------------------------------------------
    def _create_per_key_tab(self) -> QWidget:
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(10)

        # Top Control Bar (Key Presets)
        toolbar = QHBoxLayout()
        toolbar.setSpacing(8)

        lbl = QLabel("Quick Select:")
        lbl.setStyleSheet("font-weight: bold;")
        toolbar.addWidget(lbl)

        for preset_name in KEY_PRESETS.keys():
            btn = QPushButton(preset_name)
            btn.setStyleSheet("padding: 4px 10px; font-size: 11px;")
            btn.clicked.connect(lambda _, name=preset_name: self.keyboard_view.select_preset(name))
            toolbar.addWidget(btn)

        btn_clear = QPushButton("Clear Selection")
        btn_clear.setStyleSheet("padding: 4px 10px; font-size: 11px;")
        btn_clear.clicked.connect(lambda: self.keyboard_view.clear_selection())
        toolbar.addWidget(btn_clear)

        toolbar.addStretch()
        layout.addLayout(toolbar)

        # Center Interactive Keyboard
        self.keyboard_view = KeyboardView()
        layout.addWidget(self.keyboard_view, 1)

        # Bottom Color Selection & Upload Bar
        bottom_box = QHBoxLayout()
        bottom_box.setSpacing(12)

        self.perkey_color_picker = ColorPickerWidget(initial_color=(255, 0, 128))
        self.perkey_color_picker.color_changed.connect(self._on_perkey_color_picked)
        bottom_box.addWidget(self.perkey_color_picker)

        action_box = QVBoxLayout()
        action_box.setSpacing(8)

        self.lbl_selected_summary = QLabel("Selected: 0 keys (Click keys or drag box to select)")
        self.lbl_selected_summary.setStyleSheet("color: #61afef; font-weight: bold;")
        self.keyboard_view.selection_changed.connect(self._on_key_selection_changed)
        action_box.addWidget(self.lbl_selected_summary)

        btn_apply_color = QPushButton("Apply Color to Selected")
        btn_apply_color.clicked.connect(lambda: self.keyboard_view.set_selected_keys_color(self.perkey_color_picker.get_color()))
        action_box.addWidget(btn_apply_color)

        self.btn_upload_matrix = QPushButton("Upload Matrix to Keyboard")
        self.btn_upload_matrix.setObjectName("primaryBtn")
        self.btn_upload_matrix.clicked.connect(self._on_upload_matrix)
        action_box.addWidget(self.btn_upload_matrix)

        action_box.addStretch()
        bottom_box.addLayout(action_box)

        layout.addLayout(bottom_box)
        return tab

    def _on_key_selection_changed(self, selected_indices: list[int]) -> None:
        count = len(selected_indices)
        if count == 0:
            self.lbl_selected_summary.setText("Selected: All 68 keys (Click keys to select specific subset)")
        elif count == 1:
            key = PHYSICAL_KEYS[selected_indices[0]]
            self.lbl_selected_summary.setText(f"Selected: Key '{key.label}' (Scan code: 0x{key.scancode:02X})")
        else:
            self.lbl_selected_summary.setText(f"Selected: {count} keys")

    def _on_perkey_color_picked(self, color: tuple[int, int, int]) -> None:
        # If user has specific keys selected, color them immediately
        if self.keyboard_view.get_selected_indices():
            self.keyboard_view.set_selected_keys_color(color)

    def _on_upload_matrix(self) -> None:
        colors = self.keyboard_view.get_key_colors()
        if self._is_simulator or self._mock_device:
            apply_custom_matrix(self._mock_device, colors)
            QMessageBox.information(self, "Success", "[Simulator] 68-key custom RGB matrix uploaded!")
            return

        try:
            dev = RK855Device(KEYBOARD_VID, KEYBOARD_PID)
            with dev:
                apply_custom_matrix(dev, colors)
            QMessageBox.information(self, "Success", "Per-key RGB lighting matrix successfully written to keyboard!")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to upload matrix: {e}")

    # --------------------------------------------------------------------------
    # Tab 3: Key Remapping
    # --------------------------------------------------------------------------
    def _create_remap_tab(self) -> QWidget:
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(10)

        # Header guidance & action buttons
        top_bar = QHBoxLayout()
        lbl_hint = QLabel("Click any key on the visual layout to reassign its function, or review and edit the active remapping table.")
        lbl_hint.setStyleSheet("color: #7f848e; font-style: italic;")
        top_bar.addWidget(lbl_hint)
        top_bar.addStretch()

        self.btn_reset_all_remaps = QPushButton("Clear All Remappings")
        self.btn_reset_all_remaps.clicked.connect(self._on_clear_all_remaps)
        top_bar.addWidget(self.btn_reset_all_remaps)

        self.btn_apply_remaps = QPushButton("Apply & Save Remap to Keyboard")
        self.btn_apply_remaps.setObjectName("primaryBtn")
        self.btn_apply_remaps.clicked.connect(self._on_apply_remap)
        top_bar.addWidget(self.btn_apply_remaps)
        layout.addLayout(top_bar)

        # Interactive Keyboard View configured for Remapping
        self.remap_keyboard_view = KeyboardView()
        self.remap_keyboard_view.set_remap_mode(True)
        self.remap_keyboard_view.key_clicked.connect(self._on_remap_key_clicked)
        layout.addWidget(self.remap_keyboard_view, 1)

        # Bottom section: Editor (Left) & Active Table (Right)
        bottom_box = QHBoxLayout()
        bottom_box.setSpacing(14)

        # Left: Reassign Selected Key
        edit_group = QGroupBox("Reassign Key Function")
        edit_layout = QVBoxLayout(edit_group)
        edit_layout.setSpacing(8)

        self.lbl_selected_remap_key = QLabel("Selected: ESC (Physical Code: 0x29)")
        self.lbl_selected_remap_key.setStyleSheet("font-size: 13px; font-weight: bold; color: #61afef;")
        edit_layout.addWidget(self.lbl_selected_remap_key)

        self.lbl_current_remap_status = QLabel("Current Status: Default Factory Mapping")
        self.lbl_current_remap_status.setStyleSheet("color: #98c379; font-weight: 500;")
        edit_layout.addWidget(self.lbl_current_remap_status)

        form = QFormLayout()
        form.setSpacing(8)

        # Category filter
        self.combo_remap_category = QComboBox()
        categories = ["All", "Letters", "Numbers", "Function Keys", "Navigation & Control", "Modifiers", "Symbols", "Multimedia", "Mouse", "Special"]
        for cat in categories:
            self.combo_remap_category.addItem(cat)
        self.combo_remap_category.currentIndexChanged.connect(self._on_remap_category_changed)
        form.addRow("Category:", self.combo_remap_category)

        # Target action combo
        self.combo_target_action = QComboBox()
        form.addRow("New Action:", self.combo_target_action)
        edit_layout.addLayout(form)

        btn_row = QHBoxLayout()
        self.btn_assign_action = QPushButton("Assign Action")
        self.btn_assign_action.setObjectName("primaryBtn")
        self.btn_assign_action.clicked.connect(self._on_assign_remap_action)

        self.btn_restore_key = QPushButton("Restore Key to Default")
        self.btn_restore_key.clicked.connect(self._on_reset_selected_key_remap)

        btn_row.addWidget(self.btn_assign_action)
        btn_row.addWidget(self.btn_restore_key)
        edit_layout.addLayout(btn_row)

        edit_layout.addStretch()
        bottom_box.addWidget(edit_group, 4)

        # Right: Active Remappings Table
        table_group = QGroupBox("Active Remappings")
        table_layout = QVBoxLayout(table_group)
        table_layout.setSpacing(6)

        self.table_remaps = QTableWidget()
        self.table_remaps.setColumnCount(4)
        self.table_remaps.setHorizontalHeaderLabels(["Key", "Orig Code", "Assigned Action", "Target Code"])
        self.table_remaps.horizontalHeader().setSectionResizeMode(QHeaderView.ResizeMode.Stretch)
        self.table_remaps.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)
        self.table_remaps.setEditTriggers(QTableWidget.EditTrigger.NoEditTriggers)
        self.table_remaps.setStyleSheet("""
            QTableWidget {
                background-color: #16181d;
                border: 1px solid #2d3139;
                gridline-color: #282c34;
            }
            QHeaderView::section {
                background-color: #21252b;
                color: #abb2bf;
                padding: 4px;
                border: 1px solid #282c34;
                font-weight: bold;
            }
        """)
        table_layout.addWidget(self.table_remaps)

        self.lbl_remap_summary = QLabel("Remapped: 0 keys (Max 128)")
        self.lbl_remap_summary.setStyleSheet("color: #7f848e; font-size: 11px;")
        table_layout.addWidget(self.lbl_remap_summary)

        bottom_box.addWidget(table_group, 5)
        layout.addLayout(bottom_box, 1)

        # Initialize combo
        self._populate_remap_actions()
        self._update_selected_key_ui()
        return tab

    def _populate_remap_actions(self) -> None:
        selected_cat = self.combo_remap_category.currentText()
        self.combo_target_action.clear()
        for act in REMAP_ACTIONS:
            if selected_cat == "All" or act.category == selected_cat:
                self.combo_target_action.addItem(f"{act.name} (0x{act.code:02X})", act.code)

    def _on_remap_category_changed(self) -> None:
        self._populate_remap_actions()

    def _on_remap_key_clicked(self, index: int) -> None:
        self._selected_remap_key_index = index
        self._update_selected_key_ui()

    def _update_selected_key_ui(self) -> None:
        key = PHYSICAL_KEYS[self._selected_remap_key_index]
        self.lbl_selected_remap_key.setText(f"Selected: {key.label} (Physical Code: 0x{key.scancode:02X})")
        if key.scancode in self._remap_dict:
            target_code = self._remap_dict[key.scancode]
            act = ACTION_BY_CODE.get(target_code)
            act_name = act.name if act else f"Custom (0x{target_code:02X})"
            self.lbl_current_remap_status.setText(f"Remapped to: {act_name} (0x{target_code:02X})")
            self.lbl_current_remap_status.setStyleSheet("color: #e5c07b; font-weight: bold;")
        else:
            self.lbl_current_remap_status.setText("Current Status: Default Factory Function")
            self.lbl_current_remap_status.setStyleSheet("color: #98c379; font-weight: 500;")

    def _on_assign_remap_action(self) -> None:
        key = PHYSICAL_KEYS[self._selected_remap_key_index]
        target_code = self.combo_target_action.currentData()
        if target_code is not None:
            self._remap_dict[key.scancode] = target_code
            self._update_remap_table_and_view()
            self._update_selected_key_ui()

    def _on_reset_selected_key_remap(self) -> None:
        key = PHYSICAL_KEYS[self._selected_remap_key_index]
        if key.scancode in self._remap_dict:
            del self._remap_dict[key.scancode]
            self._update_remap_table_and_view()
            self._update_selected_key_ui()

    def _on_clear_all_remaps(self) -> None:
        if not self._remap_dict:
            return
        reply = QMessageBox.question(
            self,
            "Clear Remappings",
            "Are you sure you want to restore all keys to default?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
        )
        if reply == QMessageBox.StandardButton.Yes:
            self._remap_dict.clear()
            self._update_remap_table_and_view()
            self._update_selected_key_ui()

    def _update_remap_table_and_view(self) -> None:
        # Build labels dictionary for keyboard view: scancode -> display string
        labels = {}
        for scancode, target_code in self._remap_dict.items():
            act = ACTION_BY_CODE.get(target_code)
            labels[scancode] = act.name if act else f"0x{target_code:02X}"
        self.remap_keyboard_view.set_remap_labels(labels)

        # Update table
        self.table_remaps.setRowCount(len(self._remap_dict))
        scancode_to_key = {k.scancode: k for k in PHYSICAL_KEYS}

        for row, (scancode, target_code) in enumerate(sorted(self._remap_dict.items())):
            key = scancode_to_key.get(scancode)
            key_name = key.label if key else f"0x{scancode:02X}"
            act = ACTION_BY_CODE.get(target_code)
            act_name = act.name if act else f"Code 0x{target_code:02X}"

            self.table_remaps.setItem(row, 0, QTableWidgetItem(key_name))
            self.table_remaps.setItem(row, 1, QTableWidgetItem(f"0x{scancode:02X}"))
            self.table_remaps.setItem(row, 2, QTableWidgetItem(act_name))
            self.table_remaps.setItem(row, 3, QTableWidgetItem(f"0x{target_code:02X}"))

        self.lbl_remap_summary.setText(f"Remapped: {len(self._remap_dict)} keys (Max 128)")

    def _on_apply_remap(self) -> None:
        if self._is_simulator or self._mock_device:
            apply_remap_table(self._mock_device, self._remap_dict)
            QMessageBox.information(
                self,
                "Success",
                f"[Simulator] Applied key remapping ({len(self._remap_dict)} custom keys programmed)!",
            )
            return

        try:
            dev = RK855Device(KEYBOARD_VID, KEYBOARD_PID)
            with dev:
                apply_remap_table(dev, self._remap_dict)
            QMessageBox.information(
                self,
                "Success",
                f"Successfully programmed {len(self._remap_dict)} remapped keys to keyboard flash memory!",
            )
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to apply key remapping: {e}")

    # --------------------------------------------------------------------------
    # Tab 4: Firmware Flasher
    # --------------------------------------------------------------------------
    def _create_flasher_tab(self) -> QWidget:
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(10)

        # Firmware File Selector
        fw_group = QGroupBox("Firmware Image (.hex / .bin)")
        fw_layout = QVBoxLayout(fw_group)

        file_row = QHBoxLayout()
        self.edit_fw_path = QLineEdit()
        self.edit_fw_path.setPlaceholderText("Select official or custom SN32F240B firmware (.hex or .bin)...")
        self.btn_browse = QPushButton("Browse...")
        self.btn_browse.clicked.connect(self._on_browse_firmware)

        file_row.addWidget(self.edit_fw_path)
        file_row.addWidget(self.btn_browse)
        fw_layout.addLayout(file_row)

        self.lbl_fw_meta = QLabel("No firmware file loaded.")
        self.lbl_fw_meta.setStyleSheet("font-family: monospace; color: #7f848e; font-size: 11px;")
        fw_layout.addWidget(self.lbl_fw_meta)
        layout.addWidget(fw_group)

        # Device Bootloader Management
        bl_group = QGroupBox("Operating Mode & Actions")
        bl_layout = QHBoxLayout(bl_group)

        self.btn_trigger_bl = QPushButton("Reboot Keyboard to Bootloader Mode")
        self.btn_trigger_bl.clicked.connect(self._on_trigger_bootloader)
        bl_layout.addWidget(self.btn_trigger_bl)

        self.btn_reboot_normal = QPushButton("Reboot Bootloader to Normal Mode")
        self.btn_reboot_normal.clicked.connect(self._on_reboot_normal)
        bl_layout.addWidget(self.btn_reboot_normal)

        self.chk_verify = QCheckBox("Verify flash contents after write")
        self.chk_verify.setChecked(True)
        bl_layout.addWidget(self.chk_verify)

        layout.addWidget(bl_group)

        # Flashing Execution Box
        exec_box = QHBoxLayout()
        self.btn_flash = QPushButton("Flash Firmware to Microcontroller")
        self.btn_flash.setObjectName("dangerBtn")
        self.btn_flash.setStyleSheet("padding: 10px 24px; font-size: 13px;")
        self.btn_flash.clicked.connect(self._on_start_flashing)
        exec_box.addWidget(self.btn_flash)

        self.btn_cancel_flash = QPushButton("Cancel")
        self.btn_cancel_flash.setEnabled(False)
        self.btn_cancel_flash.clicked.connect(self._on_cancel_flash)
        exec_box.addWidget(self.btn_cancel_flash)

        layout.addLayout(exec_box)

        # Progress bar
        self.progress_bar = QProgressBar()
        self.progress_bar.setRange(0, 100)
        self.progress_bar.setValue(0)
        layout.addWidget(self.progress_bar)

        # Live Console Output
        layout.addWidget(QLabel("ISP Flasher Terminal Log:"))
        self.console = LogConsole()
        layout.addWidget(self.console, 1)

        # Preload repo default firmware if found
        default_hex = os.path.join(os.path.dirname(__file__), "..", "..", "firmware", "SN32F240B_32.hex")
        if os.path.isfile(default_hex):
            self.load_firmware_file(os.path.abspath(default_hex))

        return tab

    def _on_browse_firmware(self) -> None:
        path, _ = QFileDialog.getOpenFileName(
            self,
            "Select Firmware Image",
            "",
            "Firmware Files (*.hex *.bin);;Intel HEX (*.hex);;Binary Images (*.bin);;All Files (*)",
        )
        if path:
            self.load_firmware_file(path)

    def load_firmware_file(self, path: str) -> None:
        try:
            data = load_firmware(path)
            self._loaded_firmware_bytes = data
            self.edit_fw_path.setText(path)

            sha256 = hashlib.sha256(data).hexdigest()
            size_kb = len(data) // 1024
            self.lbl_fw_meta.setText(
                f"Valid SN32F240B Image: {size_kb} KB ({len(data)} bytes) | SHA-256: {sha256[:16]}...{sha256[-8:]}"
            )
            self.console.append_log("info", f"Loaded firmware file: {path} ({size_kb} KB)")
        except Exception as e:
            self.lbl_fw_meta.setText(f"Error loading file: {e}")
            self.console.append_log("error", f"Failed to load firmware: {e}")

    def _on_trigger_bootloader(self) -> None:
        self.console.append_log("info", "Requesting switch to ISP bootloader...")
        if self._is_simulator:
            self.console.append_log("success", "[Simulator] Device switched to ISP bootloader mode.")
            self.status_banner.update_status(DeviceMode.BOOTLOADER, is_simulator=True)
            return

        ok = trigger_bootloader_mode(log_cb=self.console.append_log)
        if ok:
            self.console.append_log("info", "Please wait for device to re-enumerate in bootloader mode (VID: 0x0C45, PID: 0x82AA)...")

    def _on_reboot_normal(self) -> None:
        self.console.append_log("info", "Requesting reboot into user normal firmware...")
        if self._is_simulator:
            self.console.append_log("success", "[Simulator] Rebooted back into normal keyboard mode.")
            self.status_banner.update_status(DeviceMode.NORMAL, is_simulator=True)
            return

        reboot_to_normal_mode(log_cb=self.console.append_log)

    def _on_start_flashing(self) -> None:
        if not self._loaded_firmware_bytes:
            QMessageBox.warning(self, "No Firmware", "Please select a valid .hex or .bin firmware file first.")
            return

        # Confirmation modal
        reply = QMessageBox.question(
            self,
            "Confirm Flash Operation",
            "Warning: Flashing will erase the microcontroller's user flash memory.\n\n"
            "Do NOT unplug the keyboard or power off your computer during this process.\n\n"
            "Do you want to proceed with flashing?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            QMessageBox.StandardButton.No,
        )
        if reply != QMessageBox.StandardButton.Yes:
            return

        self.btn_flash.setEnabled(False)
        self.btn_cancel_flash.setEnabled(True)
        self.progress_bar.setValue(0)

        # Launch FlashWorker
        self._flash_worker = FlashWorker(
            firmware_bytes=self._loaded_firmware_bytes,
            is_simulator=self._is_simulator,
            verify=self.chk_verify.isChecked(),
        )
        self._flash_worker.progress.connect(self._on_flash_progress)
        self._flash_worker.log_message.connect(self.console.append_log)
        self._flash_worker.finished.connect(self._on_flash_finished)
        self._flash_worker.start()

    def _on_cancel_flash(self) -> None:
        if self._flash_worker:
            self._flash_worker.cancel()
            self.console.append_log("warning", "Flashing cancellation requested...")

    def _on_flash_progress(self, percent: int, msg: str) -> None:
        self.progress_bar.setValue(percent)
        self.progress_bar.setFormat(f"{percent}% - {msg}")

    def _on_flash_finished(self, success: bool, msg: str) -> None:
        self.btn_flash.setEnabled(True)
        self.btn_cancel_flash.setEnabled(False)
        if success:
            QMessageBox.information(self, "Flash Complete", msg)
        else:
            QMessageBox.critical(self, "Flash Failed", msg)

    # --------------------------------------------------------------------------
    # Tab 4: Info & Diagnostics
    # --------------------------------------------------------------------------
    def _create_info_tab(self) -> QWidget:
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(14, 14, 14, 14)
        layout.setSpacing(12)

        # Hardware specs
        hw_group = QGroupBox("Hardware Architecture Specifications")
        hw_layout = QGridLayout(hw_group)
        hw_layout.addWidget(QLabel("<b>Keyboard Model:</b> Royal Kludge RK855 / RK68 (65% layout)"), 0, 0)
        hw_layout.addWidget(QLabel("<b>Microcontroller:</b> SONiX SN32F240B (ARM Cortex-M0 @ 48 MHz)"), 0, 1)
        hw_layout.addWidget(QLabel(f"<b>Normal Mode USB:</b> VID: 0x{KEYBOARD_VID:04X}, PID: 0x{KEYBOARD_PID:04X}"), 1, 0)
        hw_layout.addWidget(QLabel(f"<b>ISP Bootloader USB:</b> VID: 0x{BOOTLOADER_VID:04X}, PID: 0x{BOOTLOADER_PID:04X}"), 1, 1)
        hw_layout.addWidget(QLabel("<b>Flash Memory:</b> 64 KB total (32 KB user firmware + ISP bootloader)"), 2, 0)
        hw_layout.addWidget(QLabel("<b>Switch Matrix:</b> 8 Rows × 9 Columns multiplexed matrix (68 keys)"), 2, 1)
        layout.addWidget(hw_group)

        # Linux udev rules section
        udev_group = QGroupBox("Linux USB Permissions (udev)")
        udev_layout = QVBoxLayout(udev_group)
        udev_layout.addWidget(QLabel(
            "On Linux, root permissions or udev rules are required to send raw HID feature reports.\n"
            "To grant user access, save the following rule to <code>/etc/udev/rules.d/99-rk855.rules</code>:"
        ))

        self.edit_udev = QTextEdit()
        self.edit_udev.setReadOnly(True)
        self.edit_udev.setText(get_udev_rule_text())
        self.edit_udev.setFixedHeight(90)
        self.edit_udev.setStyleSheet("font-family: monospace; font-size: 11px;")
        udev_layout.addWidget(self.edit_udev)

        btn_copy_udev = QPushButton("Copy udev Rules to Clipboard")
        btn_copy_udev.clicked.connect(lambda: QApplication.clipboard().setText(self.edit_udev.toPlainText()))
        udev_layout.addWidget(btn_copy_udev)

        layout.addWidget(udev_group)

        # uv tooling tip
        tool_group = QGroupBox("Modern Python Environment (uv)")
        tool_layout = QVBoxLayout(tool_group)
        tool_layout.addWidget(QLabel(
            "This repository uses <b>uv</b> for project and dependency management.<br/>"
            "Run <code>uv sync</code> to install dependencies, and <code>uv run rk855-gui</code> to launch the app."
        ))
        layout.addWidget(tool_group)

        layout.addStretch()
        return tab

    # --------------------------------------------------------------------------
    # Device Watcher & Polling
    # --------------------------------------------------------------------------
    def _init_watcher(self) -> None:
        self.watcher = DeviceWatcher(self)
        self.watcher.set_simulator_mode(self._is_simulator)
        self.watcher.status_changed.connect(self._on_device_status_changed)
        self.watcher.start()

    def _on_device_status_changed(self, mode: DeviceMode, info: dict | None, err: str | None) -> None:
        self._current_device_mode = mode
        self._current_device_info = info
        self.status_banner.update_status(mode, is_simulator=self._is_simulator, details=err)

        # Dynamically enable/disable buttons based on mode
        if self._is_simulator:
            self.btn_apply_light.setEnabled(True)
            self.btn_upload_matrix.setEnabled(True)
            self.btn_apply_remaps.setEnabled(True)
            self.btn_trigger_bl.setEnabled(True)
            self.btn_reboot_normal.setEnabled(True)
            self.btn_flash.setEnabled(True)
        elif mode == DeviceMode.NORMAL:
            self.btn_apply_light.setEnabled(True)
            self.btn_upload_matrix.setEnabled(True)
            self.btn_apply_remaps.setEnabled(True)
            self.btn_trigger_bl.setEnabled(True)
            self.btn_reboot_normal.setEnabled(False)
            self.btn_flash.setEnabled(True)
        elif mode == DeviceMode.BOOTLOADER:
            self.btn_apply_light.setEnabled(False)
            self.btn_upload_matrix.setEnabled(False)
            self.btn_apply_remaps.setEnabled(False)
            self.btn_trigger_bl.setEnabled(False)
            self.btn_reboot_normal.setEnabled(True)
            self.btn_flash.setEnabled(True)
        else:
            self.btn_apply_light.setEnabled(False)
            self.btn_upload_matrix.setEnabled(False)
            self.btn_apply_remaps.setEnabled(False)
            self.btn_trigger_bl.setEnabled(False)
            self.btn_reboot_normal.setEnabled(False)
            self.btn_flash.setEnabled(False)

    def _on_refresh_device(self) -> None:
        if self._is_simulator:
            self.status_banner.update_status(DeviceMode.NORMAL, is_simulator=True)
            return
        mode, info, err = RK855Device.get_status()
        self._on_device_status_changed(mode, info, err)

    def _on_simulator_toggled(self, checked: bool) -> None:
        self._is_simulator = checked
        if checked and self._mock_device is None:
            self._mock_device = MockRK855Device()
        self.watcher.set_simulator_mode(checked)
        self._on_refresh_device()

    def closeEvent(self, event) -> None:
        if self.watcher:
            self.watcher.stop()
        if self._flash_worker and self._flash_worker.isRunning():
            self._flash_worker.cancel()
            self._flash_worker.wait(1000)
        event.accept()
