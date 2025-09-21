from typing import Optional

from PyQt5.QtCore import QObject, pyqtSignal
from PyQt5.QtWidgets import QAction, QMenu

from DAWPY.models import AppSettings


class TrayContextMenu(QObject):
    """System tray context menu manager"""

    # Signals
    exit_requested = pyqtSignal()
    toggle_project_name_requested = pyqtSignal()
    toggle_system_usage_requested = pyqtSignal()
    update_interval_requested = pyqtSignal()
    show_window_requested = pyqtSignal()

    def __init__(self, app_version: str):
        super().__init__()
        self.app_version = app_version
        self.menu = QMenu()

        # Menu actions
        self.version_action: Optional[QAction] = None
        self.discord_status_action: Optional[QAction] = None
        self.hide_project_action: Optional[QAction] = None
        self.hide_system_action: Optional[QAction] = None

        self._create_menu()

    def _create_menu(self):
        """Create tray context menu"""
        # Version info (disabled)
        self.version_action = QAction(
            f"DAW Discord Rich Presence v{self.app_version}", self
        )
        self.version_action.setEnabled(False)
        self.menu.addAction(self.version_action)

        # Discord status (disabled)
        self.discord_status_action = QAction("Open a DAW to begin displaying RPC", self)
        self.discord_status_action.setEnabled(False)
        self.menu.addAction(self.discord_status_action)

        self.menu.addSeparator()

        # Open Window
        open_action = QAction("Open Window", self)
        open_action.triggered.connect(self.show_window_requested.emit)
        self.menu.addAction(open_action)

        # Settings submenu
        settings_menu = self.menu.addMenu("Settings")

        self.hide_project_action = QAction("[OFF] Hide Project Name", self)
        self.hide_project_action.triggered.connect(
            self.toggle_project_name_requested.emit
        )
        settings_menu.addAction(self.hide_project_action)

        self.hide_system_action = QAction("[OFF] Hide System Usage", self)
        self.hide_system_action.triggered.connect(
            self.toggle_system_usage_requested.emit
        )
        settings_menu.addAction(self.hide_system_action)

        interval_action = QAction("Set Update Interval", self)
        interval_action.triggered.connect(self.update_interval_requested.emit)
        settings_menu.addAction(interval_action)

        self.menu.addSeparator()

        # Exit
        exit_action = QAction("Exit", self)
        exit_action.triggered.connect(self.exit_requested.emit)
        self.menu.addAction(exit_action)

    def update_discord_status(self, message: str):
        """Update Discord status message in menu"""
        if self.discord_status_action:
            self.discord_status_action.setText(message)

    def update_settings_display(self, settings: AppSettings):
        """Update settings menu items"""
        if self.hide_project_action:
            project_text = (
                f"[{'ON' if settings.hide_project_name else 'OFF'}] Hide Project Name"
            )
            self.hide_project_action.setText(project_text)

        if self.hide_system_action:
            system_text = (
                f"[{'ON' if settings.hide_system_usage else 'OFF'}] Hide System Usage"
            )
            self.hide_system_action.setText(system_text)
