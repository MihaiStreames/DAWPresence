from PyQt5.QtCore import QObject, pyqtSignal
from PyQt5.QtWidgets import QSystemTrayIcon

from DAWPY.models import AppSettings
from DAWPY.views.components.tray_icon_mann import TrayIconManager
from DAWPY.views.components.tray_menu import TrayContextMenu


class SystemTrayManager(QObject):
    """Manages system tray icon and menu"""

    # Signals
    exit_requested = pyqtSignal()
    toggle_project_name_requested = pyqtSignal()
    toggle_system_usage_requested = pyqtSignal()
    update_interval_requested = pyqtSignal()
    show_window_requested = pyqtSignal()

    def __init__(self, parent_window, app_version: str):
        super().__init__()
        self.parent_window = parent_window
        self.app_version = app_version

        # Tray components
        self.tray_icon = QSystemTrayIcon(parent_window)
        self.icon_manager = TrayIconManager(self.tray_icon)
        self.context_menu = TrayContextMenu(app_version)

        self._setup_tray()

    def _setup_tray(self):
        """Setup tray icon and menu"""
        # Connect context menu signals
        self.context_menu.exit_requested.connect(self.exit_requested.emit)
        self.context_menu.toggle_project_name_requested.connect(
            self.toggle_project_name_requested.emit,
        )
        self.context_menu.toggle_system_usage_requested.connect(
            self.toggle_system_usage_requested.emit,
        )
        self.context_menu.update_interval_requested.connect(self.update_interval_requested.emit)
        self.context_menu.show_window_requested.connect(self.show_window_requested.emit)

        # Setup tray icon
        self.tray_icon.setContextMenu(self.context_menu.menu)
        self.tray_icon.activated.connect(self._on_tray_activated)
        self.tray_icon.show()

    def _on_tray_activated(self, reason):
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show_window_requested.emit()

    def set_connected_status(self, connected: bool):
        """Update tray icon based on connection status"""
        self.icon_manager.set_connected_status(connected)

    def update_discord_status(self, message: str):
        """Update Discord status message in menu"""
        self.context_menu.update_discord_status(message)

    def update_settings_display(self, settings: AppSettings):
        """Update settings menu items"""
        self.context_menu.update_settings_display(settings)

    def show_message(self, title: str, message: str, duration: int = 3000):
        """Show tray notification"""
        self.tray_icon.showMessage(title, message, QSystemTrayIcon.Information, duration)
