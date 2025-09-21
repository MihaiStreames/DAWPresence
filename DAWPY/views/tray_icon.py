import os
from typing import Optional

from PyQt5.QtCore import QObject, Qt, pyqtSignal
from PyQt5.QtGui import QBrush, QIcon, QPainter, QPixmap
from PyQt5.QtWidgets import QAction, QMenu, QSystemTrayIcon

from DAWPY.models import AppSettings


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

        # Tray icon and menu
        self.tray_icon = QSystemTrayIcon(parent_window)
        self.tray_menu = QMenu()

        # Menu actions
        self.version_action: Optional[QAction] = None
        self.discord_status_action: Optional[QAction] = None
        self.hide_project_action: Optional[QAction] = None
        self.hide_system_action: Optional[QAction] = None

        self._setup_tray()

    def _setup_tray(self) -> None:
        """Setup tray icon and menu"""
        # Set initial icon (red = disconnected)
        self._set_icon_color("red")

        # Create menu
        self._create_menu()

        # Set menu and show
        self.tray_icon.setContextMenu(self.tray_menu)
        self.tray_icon.activated.connect(self._on_tray_activated)
        self.tray_icon.show()

    def _create_menu(self) -> None:
        """Create tray context menu"""
        # Version info (disabled)
        self.version_action = QAction(
            f"DAW Discord Rich Presence v{self.app_version}", self
        )
        self.version_action.setEnabled(False)
        self.tray_menu.addAction(self.version_action)

        # Discord status (disabled)
        self.discord_status_action = QAction("Open a DAW to begin displaying RPC", self)
        self.discord_status_action.setEnabled(False)
        self.tray_menu.addAction(self.discord_status_action)

        self.tray_menu.addSeparator()

        # Open Window
        open_action = QAction("Open Window", self)
        open_action.triggered.connect(self.show_window_requested.emit)
        self.tray_menu.addAction(open_action)

        # Settings submenu
        settings_menu = self.tray_menu.addMenu("Settings")

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

        self.tray_menu.addSeparator()

        # Exit
        exit_action = QAction("Exit", self)
        exit_action.triggered.connect(self.exit_requested.emit)
        self.tray_menu.addAction(exit_action)

    def _on_tray_activated(self, reason) -> None:
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show_window_requested.emit()

    def _set_icon_color(self, color: str) -> None:
        """Set tray icon color"""
        icon_path = os.path.join(os.path.dirname(__file__), f"../assets/{color}.ico")

        if os.path.exists(icon_path):
            self.tray_icon.setIcon(QIcon(icon_path))
        else:
            # Create simple colored icon
            pixmap = QPixmap(16, 16)
            pixmap.fill(Qt.transparent)
            painter = QPainter(pixmap)

            if color == "green":
                painter.setBrush(QBrush(Qt.green))
            else:
                painter.setBrush(QBrush(Qt.red))

            painter.drawEllipse(0, 0, 16, 16)
            painter.end()
            self.tray_icon.setIcon(QIcon(pixmap))

    def set_connected_status(self, connected: bool) -> None:
        """Update tray icon based on connection status"""
        color = "green" if connected else "red"
        self._set_icon_color(color)

    def update_discord_status(self, message: str) -> None:
        """Update Discord status message in menu"""
        if self.discord_status_action:
            self.discord_status_action.setText(message)

    def update_settings_display(self, settings: AppSettings) -> None:
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

    def show_message(self, title: str, message: str, duration: int = 3000) -> None:
        """Show tray notification"""
        self.tray_icon.showMessage(
            title, message, QSystemTrayIcon.Information, duration
        )
