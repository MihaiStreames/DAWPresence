from typing import Optional

from PyQt5.QtCore import pyqtSignal
from PyQt5.QtWidgets import QMainWindow, QVBoxLayout, QWidget

from DAWPY.models import AppSettings, DAWStatus
from DAWPY.views.components import AppMenuBar, StatusGroupsWidget
from DAWPY.views.dialogs import IntervalDialog
from DAWPY.views.sys_tray_mann import SystemTrayManager


class MainWindow(QMainWindow):
    """Main application window"""

    # Signals for controller communication
    toggle_project_name_signal = pyqtSignal()
    toggle_system_usage_signal = pyqtSignal()
    update_interval_signal = pyqtSignal(int)
    exit_signal = pyqtSignal()

    def __init__(self, app_version: str = "1.0"):
        super().__init__()
        self.app_version = app_version
        self.exiting = False

        # UI Components
        self.tray_manager: Optional[SystemTrayManager] = None
        self.interval_dialog: Optional[IntervalDialog] = None
        self.menu_bar_manager: Optional[AppMenuBar] = None
        self.status_groups: Optional[StatusGroupsWidget] = None

        self._init_ui()
        self._setup_tray()

    def _init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle(f"DAW Discord Rich Presence v{self.app_version}")
        self.setFixedSize(735, 347)

        # Create central widget and layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(0, 0, 0, 0)

        # Create menu bar manager
        self.menu_bar_manager = AppMenuBar(self.menuBar())
        self._connect_menu_signals()

        # Create status groups
        self.status_groups = StatusGroupsWidget()
        main_layout.addWidget(self.status_groups)

    def _connect_menu_signals(self):
        """Connect menu bar signals"""
        if self.menu_bar_manager:
            self.menu_bar_manager.toggle_project_name_requested.connect(
                self.toggle_project_name_signal.emit
            )
            self.menu_bar_manager.toggle_system_usage_requested.connect(
                self.toggle_system_usage_signal.emit
            )
            self.menu_bar_manager.update_interval_requested.connect(
                self._show_interval_dialog
            )

    def _setup_tray(self):
        """Setup system tray"""
        self.tray_manager = SystemTrayManager(self, self.app_version)
        self.tray_manager.exit_requested.connect(self.exit_signal.emit)
        self.tray_manager.toggle_project_name_requested.connect(
            self.toggle_project_name_signal.emit
        )
        self.tray_manager.toggle_system_usage_requested.connect(
            self.toggle_system_usage_signal.emit
        )
        self.tray_manager.update_interval_requested.connect(self._show_interval_dialog)
        self.tray_manager.show_window_requested.connect(self.show)

    def _show_interval_dialog(self):
        """Show interval setting dialog"""
        if not self.interval_dialog:
            self.interval_dialog = IntervalDialog(self)
            self.interval_dialog.interval_changed.connect(
                self.update_interval_signal.emit
            )

        self.interval_dialog.show_dialog()

    # Public methods for controller interaction

    def update_daw_display(self, daw_status: DAWStatus):
        """Update DAW status display"""
        if self.status_groups:
            self.status_groups.update_daw_display(daw_status)

    def update_settings_display(self, settings: AppSettings):
        """Update menu items to reflect current settings"""
        if self.menu_bar_manager:
            self.menu_bar_manager.update_settings_display(settings)

        if self.tray_manager:
            self.tray_manager.update_settings_display(settings)

    def on_daw_started(self, daw_status: DAWStatus):
        """Handle DAW started event"""
        if self.tray_manager:
            self.tray_manager.show_message(
                "DAW Started", f"{daw_status.display_name} detected!"
            )

    def on_daw_stopped(self, daw_status: DAWStatus):
        """Handle DAW stopped event"""
        if self.tray_manager:
            self.tray_manager.show_message("DAW Stopped", "No DAW detected")

    def on_discord_connected(self):
        """Handle Discord connected event"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(True)
            self.tray_manager.update_discord_status("Connected to Discord")

    def on_discord_disconnected(self):
        """Handle Discord disconnected event"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(False)
            self.tray_manager.update_discord_status(
                "Open a DAW to begin displaying RPC"
            )

    def on_discord_error(self, error: Exception):
        """Handle Discord error"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(False)
            self.tray_manager.update_discord_status(f"Connection failed: {error}")

        from PyQt5.QtWidgets import QMessageBox

        QMessageBox.warning(
            self,
            "Discord Connection Error",
            f"Cannot connect to Discord RPC server.\n{str(error)}",
        )

    def closeEvent(self, event):
        """Handle window close event"""
        if not self.exiting:
            event.ignore()
            self.hide()
            if self.tray_manager:
                self.tray_manager.show_message(
                    "DAWPresence", "Application minimized to tray"
                )
        else:
            event.accept()

    def exit_application(self):
        """Prepare for application exit"""
        self.exiting = True
        self.close()
