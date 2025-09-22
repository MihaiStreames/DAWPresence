from typing import Any

from PyQt5.QtCore import QObject, pyqtSignal
from PyQt5.QtWidgets import QInputDialog


class IntervalDialog(QObject):
    """Dialog for setting update interval"""

    interval_changed = pyqtSignal(int)

    def __init__(self, parent: Any = None) -> None:
        super().__init__()
        self.parent = parent

    def show_dialog(self, current_interval: int = 2500) -> None:
        """Show interval input dialog"""
        interval, ok = QInputDialog.getInt(
            self.parent,
            "Set Update Interval",
            "Type the presence update interval (in milliseconds):",
            value=current_interval,
            min=1000,
            max=100_000_000,
        )

        if ok:
            self.interval_changed.emit(interval)
