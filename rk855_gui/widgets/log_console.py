"""Color-coded scrolling log console widget."""

from datetime import datetime
from PyQt6.QtCore import Qt
from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QTextEdit,
    QPushButton,
    QApplication,
)


class LogConsole(QWidget):
    """Timestamped log view with syntax highlighting for status levels."""

    def __init__(self, parent: QWidget | None = None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        self.text_edit = QTextEdit()
        self.text_edit.setReadOnly(True)
        self.text_edit.setStyleSheet("""
            QTextEdit {
                background-color: #121418;
                color: #d8dee9;
                font-family: 'Consolas', 'Courier New', monospace;
                font-size: 11px;
                border: 1px solid #2d3139;
                border-radius: 4px;
                padding: 4px;
            }
        """)
        layout.addWidget(self.text_edit)

        # Toolbar
        btn_bar = QHBoxLayout()
        btn_bar.setContentsMargins(0, 0, 0, 0)

        self.btn_clear = QPushButton("Clear Log")
        self.btn_clear.clicked.connect(self.clear)
        self.btn_clear.setStyleSheet("font-size: 11px; padding: 2px 8px;")

        self.btn_copy = QPushButton("Copy All")
        self.btn_copy.clicked.connect(self.copy_all)
        self.btn_copy.setStyleSheet("font-size: 11px; padding: 2px 8px;")

        btn_bar.addStretch()
        btn_bar.addWidget(self.btn_copy)
        btn_bar.addWidget(self.btn_clear)
        layout.addLayout(btn_bar)

    def append_log(self, level: str, message: str) -> None:
        """Appends a color-coded message."""
        now = datetime.now().strftime("%H:%M:%S")
        color = "#abb2bf"  # default info
        prefix = "[INFO]"

        lvl = level.lower()
        if lvl == "success":
            color = "#98c379"
            prefix = "[OK]"
        elif lvl == "warning":
            color = "#e5c07b"
            prefix = "[WARN]"
        elif lvl == "error":
            color = "#e06c75"
            prefix = "[ERR]"
        elif lvl == "debug":
            color = "#5c6370"
            prefix = "[DBG]"

        html = f"<span style='color: #5c6370;'>{now}</span> <span style='color: {color}; font-weight: bold;'>{prefix}</span> <span style='color: #e5e9f0;'>{message}</span>"
        self.text_edit.append(html)
        self.text_edit.verticalScrollBar().setValue(self.text_edit.verticalScrollBar().maximum())

    def clear(self) -> None:
        self.text_edit.clear()

    def copy_all(self) -> None:
        QApplication.clipboard().setText(self.text_edit.toPlainText())
