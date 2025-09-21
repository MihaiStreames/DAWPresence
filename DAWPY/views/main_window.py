from typing import Optional

from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QFont
from PyQt5.QtWidgets import (
    QMainWindow,
    QWidget,
    QVBoxLayout,
    QGroupBox,
    QLabel,
    QGridLayout,
    QAction
)

from DAWPY.models import DAWStatus, AppSettings
from DAWPY.views.dialogs import IntervalDialog
from DAWPY.views.tray_icon import SystemTrayManager


class MainWindow(QMainWindow):
    """Main application window"""

    # Signals for controller communication
    toggle_project_name_signal = pyqtSignal()
    toggle_system_usage_signal = pyqtSignal()
    update_interval_signal = pyqtSignal(int)
    exit_signal = pyqtSignal()

    def __init__(self, app_version: str = "2.0"):
        super().__init__()
        self.app_version = app_version
        self.exiting = False

        # UI Components
        self.tray_manager: Optional[SystemTrayManager] = None
        self.interval_dialog: Optional[IntervalDialog] = None

        # Status labels
        self.daw_name_label: Optional[QLabel] = None
        self.project_name_label: Optional[QLabel] = None
        self.cpu_usage_label: Optional[QLabel] = None
        self.ram_usage_label: Optional[QLabel] = None

        # Menu actions
        self.hide_project_action: Optional[QAction] = None
        self.hide_system_action: Optional[QAction] = None

        self._init_ui()
        self._setup_tray()

    def _init_ui(self) -> None:
        """Initialize the user interface"""
        self.setWindowTitle(f"DAW Discord Rich Presence v{self.app_version}")
        self.setFixedSize(735, 347)

        # Create central widget and layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(0, 0, 0, 0)

        # Create menu bar
        self._create_menu_bar()

        # Create content area
        content_widget = QWidget()
        grid_layout = QGridLayout(content_widget)
        grid_layout.setContentsMargins(10, 10, 10, 10)
        grid_layout.setSpacing(10)

        # Create status groups
        self._create_status_groups(grid_layout)

        # Set layout properties
        grid_layout.setColumnStretch(0, 1)
        grid_layout.setColumnStretch(1, 1)
        grid_layout.setRowStretch(0, 1)
        grid_layout.setRowStretch(1, 1)

        main_layout.addWidget(content_widget)

    def _create_menu_bar(self) -> None:
        """Create the application menu bar"""
        menubar = self.menuBar()

        # Hide Project Name action
        self.hide_project_action = QAction("[OFF] Hide Project Name", self)
        self.hide_project_action.triggered.connect(self.toggle_project_name_signal.emit)
        menubar.addAction(self.hide_project_action)

        # Hide System Usage action
        self.hide_system_action = QAction("[OFF] Hide System Usage", self)
        self.hide_system_action.triggered.connect(self.toggle_system_usage_signal.emit)
        menubar.addAction(self.hide_system_action)

        # Set Update Interval action
        update_interval_action = QAction("Set Update Interval", self)
        update_interval_action.triggered.connect(self._show_interval_dialog)
        menubar.addAction(update_interval_action)

    def _create_status_groups(self, layout: QGridLayout) -> None:
        """Create status display groups"""
        # DAW Name group
        daw_group = QGroupBox("Current Digital Audio Workstation")
        daw_layout = QVBoxLayout(daw_group)
        self.daw_name_label = QLabel("None")
        self.daw_name_label.setAlignment(Qt.AlignCenter)
        self.daw_name_label.setFont(QFont("Tahoma", 12))
        daw_layout.addWidget(self.daw_name_label)

        # Project Name group
        project_group = QGroupBox("Opening Project")
        project_layout = QVBoxLayout(project_group)
        self.project_name_label = QLabel("None")
        self.project_name_label.setAlignment(Qt.AlignCenter)
        self.project_name_label.setFont(QFont("Tahoma", 12))
        project_layout.addWidget(self.project_name_label)

        # CPU Usage group
        cpu_group = QGroupBox("CPU Usage")
        cpu_layout = QVBoxLayout(cpu_group)
        self.cpu_usage_label = QLabel("Undefined")
        self.cpu_usage_label.setAlignment(Qt.AlignCenter)
        self.cpu_usage_label.setFont(QFont("Consolas", 20))
        cpu_layout.addWidget(self.cpu_usage_label)

        # RAM Usage group
        ram_group = QGroupBox("RAM Usage")
        ram_layout = QVBoxLayout(ram_group)
        self.ram_usage_label = QLabel("Undefined")
        self.ram_usage_label.setAlignment(Qt.AlignCenter)
        self.ram_usage_label.setFont(QFont("Consolas", 20))
        ram_layout.addWidget(self.ram_usage_label)

        # Add to grid
        layout.addWidget(daw_group, 0, 0)
        layout.addWidget(project_group, 0, 1)
        layout.addWidget(cpu_group, 1, 0)
        layout.addWidget(ram_group, 1, 1)

    def _setup_tray(self) -> None:
        """Setup system tray"""
        self.tray_manager = SystemTrayManager(self, self.app_version)
        self.tray_manager.exit_requested.connect(self.exit_signal.emit)
        self.tray_manager.toggle_project_name_requested.connect(self.toggle_project_name_signal.emit)
        self.tray_manager.toggle_system_usage_requested.connect(self.toggle_system_usage_signal.emit)
        self.tray_manager.update_interval_requested.connect(self._show_interval_dialog)
        self.tray_manager.show_window_requested.connect(self.show)

    def _show_interval_dialog(self) -> None:
        """Show interval setting dialog"""
        if not self.interval_dialog:
            self.interval_dialog = IntervalDialog(self)
            self.interval_dialog.interval_changed.connect(self.update_interval_signal.emit)

        self.interval_dialog.show_dialog()

    # Public methods for controller interaction

    def update_daw_display(self, daw_status: DAWStatus) -> None:
        """Update DAW status display"""
        self.daw_name_label.setText(daw_status.display_name)
        self.project_name_label.setText(daw_status.project_name)
        self.cpu_usage_label.setText(daw_status.cpu_usage_str)
        self.ram_usage_label.setText(daw_status.ram_usage_str)

    def update_settings_display(self, settings: AppSettings) -> None:
        """Update menu items to reflect current settings"""
        project_text = f"[{'ON' if settings.hide_project_name else 'OFF'}] Hide Project Name"
        system_text = f"[{'ON' if settings.hide_system_usage else 'OFF'}] Hide System Usage"

        self.hide_project_action.setText(project_text)
        self.hide_system_action.setText(system_text)

        if self.tray_manager:
            self.tray_manager.update_settings_display(settings)

    def on_daw_started(self, daw_status: DAWStatus) -> None:
        """Handle DAW started event"""
        if self.tray_manager:
            self.tray_manager.show_message("DAW Started", f"{daw_status.display_name} detected!")

    def on_daw_stopped(self, daw_status: DAWStatus) -> None:
        """Handle DAW stopped event"""
        if self.tray_manager:
            self.tray_manager.show_message("DAW Stopped", "No DAW detected")

    def on_discord_connected(self) -> None:
        """Handle Discord connected event"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(True)
            self.tray_manager.update_discord_status("Connected to Discord")

    def on_discord_disconnected(self) -> None:
        """Handle Discord disconnected event"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(False)
            self.tray_manager.update_discord_status("Open a DAW to begin displaying RPC")

    def on_discord_error(self, error: Exception) -> None:
        """Handle Discord error"""
        if self.tray_manager:
            self.tray_manager.set_connected_status(False)
            self.tray_manager.update_discord_status(f"Connection failed: {error}")

        from PyQt5.QtWidgets import QMessageBox
        QMessageBox.warning(self, "Discord Connection Error", f"Cannot connect to Discord RPC server.\n{str(error)}")

    def closeEvent(self, event) -> None:
        """Handle window close event"""
        if not self.exiting:
            event.ignore()
            self.hide()
            if self.tray_manager:
                self.tray_manager.show_message("DAWPresence", "Application minimized to tray")
        else:
            event.accept()

    def exit_application(self) -> None:
        """Prepare for application exit"""
        self.exiting = True
        self.close()
