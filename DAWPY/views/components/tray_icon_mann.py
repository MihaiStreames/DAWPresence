from PyQt5.QtGui import QIcon
from PyQt5.QtWidgets import QSystemTrayIcon

from DAWPY.utils import IconUtils


class TrayIconManager:
    """Manages the system tray icon appearance"""

    def __init__(self, tray_icon: QSystemTrayIcon) -> None:
        self.tray_icon = tray_icon
        self._set_icon_color("red")  # Default to disconnected state

    def _set_icon_color(self, color: str) -> None:
        """Set tray icon color"""
        icon_path = IconUtils.get_icon_path(color)
        self.tray_icon.setIcon(QIcon(icon_path))

    def set_connected_status(self, connected: bool) -> None:
        """Update tray icon based on connection status"""
        color = "green" if connected else "red"
        self._set_icon_color(color)
