import os

from PyQt5.QtGui import QIcon
from PyQt5.QtWidgets import QSystemTrayIcon


class TrayIconManager:
    """Manages the system tray icon appearance"""

    def __init__(self, tray_icon: QSystemTrayIcon):
        self.tray_icon = tray_icon
        self._set_icon_color("red")  # Default to disconnected state

    def _set_icon_color(self, color: str):
        """Set tray icon color"""
        icon_path = os.path.join(os.path.dirname(__file__), f"../../assets/{color}.ico")
        self.tray_icon.setIcon(QIcon(icon_path))

    def set_connected_status(self, connected: bool):
        """Update tray icon based on connection status"""
        color = "green" if connected else "red"
        self._set_icon_color(color)
