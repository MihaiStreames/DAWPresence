import os

from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QIcon, QFont
from PyQt5.QtWidgets import (QMainWindow, QWidget, QVBoxLayout, QGroupBox, QLabel, QMenu, QAction,
                             QSystemTrayIcon, QGridLayout, QInputDialog)


class MainWindow(QMainWindow):
    # Signals
    toggle_project_name_signal = pyqtSignal()
    toggle_system_usage_signal = pyqtSignal()
    update_interval_signal = pyqtSignal(int)

    def __init__(self):
        super().__init__()
        self.exiting = False
        self.init_ui()
        self.setup_tray()

    def init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("DAW Discord Rich Presence")
        self.setFixedSize(735, 347)

        # Create central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)

        # Create main layout
        main_layout = QVBoxLayout(central_widget)
        main_layout.setContentsMargins(0, 0, 0, 0)

        # Create menu bar
        self.create_menu_bar()

        # Create content grid
        content_widget = QWidget()
        grid_layout = QGridLayout(content_widget)
        grid_layout.setContentsMargins(10, 10, 10, 10)
        grid_layout.setSpacing(10)

        # Create group boxes
        # DAW Name group
        daw_group = QGroupBox("Current Digital Audio Workstation")
        daw_layout = QVBoxLayout(daw_group)
        self.daw_name = QLabel("None")
        self.daw_name.setAlignment(Qt.AlignCenter)
        self.daw_name.setFont(QFont("Tahoma", 12))
        daw_layout.addWidget(self.daw_name)

        # Project Name group
        project_group = QGroupBox("Opening Project")
        project_layout = QVBoxLayout(project_group)
        self.project_name = QLabel("None")
        self.project_name.setAlignment(Qt.AlignCenter)
        self.project_name.setFont(QFont("Tahoma", 12))
        project_layout.addWidget(self.project_name)

        # CPU Usage group
        cpu_group = QGroupBox("CPU Usage")
        cpu_layout = QVBoxLayout(cpu_group)
        self.cpu_usage = QLabel("Undefined")
        self.cpu_usage.setAlignment(Qt.AlignCenter)
        self.cpu_usage.setFont(QFont("Consolas", 20))
        cpu_layout.addWidget(self.cpu_usage)

        # RAM Usage group
        ram_group = QGroupBox("RAM Usage")
        ram_layout = QVBoxLayout(ram_group)
        self.ram_usage = QLabel("Undefined")
        self.ram_usage.setAlignment(Qt.AlignCenter)
        self.ram_usage.setFont(QFont("Consolas", 20))
        ram_layout.addWidget(self.ram_usage)

        # Add groups to grid
        grid_layout.addWidget(daw_group, 0, 0)
        grid_layout.addWidget(project_group, 0, 1)
        grid_layout.addWidget(cpu_group, 1, 0)
        grid_layout.addWidget(ram_group, 1, 1)

        # Set equal column stretch
        grid_layout.setColumnStretch(0, 1)
        grid_layout.setColumnStretch(1, 1)
        grid_layout.setRowStretch(0, 1)
        grid_layout.setRowStretch(1, 1)

        main_layout.addWidget(content_widget)

    def create_menu_bar(self):
        """Create the menu bar"""
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
        self.update_interval_action = QAction("Set Update Interval", self)
        self.update_interval_action.triggered.connect(self.show_interval_dialog)
        menubar.addAction(self.update_interval_action)

    def setup_tray(self):
        """Setup system tray icon"""
        self.tray_icon = QSystemTrayIcon(self)
        self.change_notify_icon("red")

        # Create tray menu
        tray_menu = QMenu()

        # Version info
        self.version_action = QAction("DAW Discord Rich Presence", self)
        self.version_action.setEnabled(False)
        tray_menu.addAction(self.version_action)

        # Discord username
        self.discord_username_action = QAction("Open a DAW to begin displaying RPC", self)
        self.discord_username_action.setEnabled(False)
        tray_menu.addAction(self.discord_username_action)

        tray_menu.addSeparator()

        # Open Window
        open_action = QAction("Open Window", self)
        open_action.triggered.connect(self.show)
        tray_menu.addAction(open_action)

        # Settings submenu
        settings_menu = tray_menu.addMenu("Settings")

        self.tray_hide_project = QAction("[OFF] Hide Project Name", self)
        self.tray_hide_project.triggered.connect(self.toggle_project_name_signal.emit)
        settings_menu.addAction(self.tray_hide_project)

        self.tray_hide_system = QAction("[OFF] Hide System Usage", self)
        self.tray_hide_system.triggered.connect(self.toggle_system_usage_signal.emit)
        settings_menu.addAction(self.tray_hide_system)

        tray_interval = QAction("Set Update Interval", self)
        tray_interval.triggered.connect(self.show_interval_dialog)
        settings_menu.addAction(tray_interval)

        tray_menu.addSeparator()

        # Exit
        self.exit_action = QAction("Exit", self)
        tray_menu.addAction(self.exit_action)

        self.tray_icon.setContextMenu(tray_menu)
        self.tray_icon.show()

        # Double click to show window
        self.tray_icon.activated.connect(self.tray_icon_activated)

    def tray_icon_activated(self, reason):
        """Handle tray icon activation"""
        if reason == QSystemTrayIcon.DoubleClick:
            self.show()

    def change_notify_icon(self, color):
        """Change the system tray icon color"""
        icon_path = os.path.join(os.path.dirname(__file__), f"{color}.ico")
        if os.path.exists(icon_path):
            self.tray_icon.setIcon(QIcon(icon_path))
        else:
            # Create a simple colored icon if file doesn't exist
            from PyQt5.QtGui import QPixmap, QPainter, QBrush
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

    def update_setting_display(self, settings):
        """Update the display of settings in menus"""
        project_text = f"[{'ON' if settings.hide_project_name else 'OFF'}] Hide Project Name"
        system_text = f"[{'ON' if settings.hide_system_usage else 'OFF'}] Hide System Usage"

        self.hide_project_action.setText(project_text)
        self.hide_system_action.setText(system_text)
        self.tray_hide_project.setText(project_text)
        self.tray_hide_system.setText(system_text)

    def show_interval_dialog(self):
        """Show dialog to set update interval"""
        interval, ok = QInputDialog.getInt(
            self,
            "Set Update Interval",
            "Type the presence update interval (in milliseconds):",
            value=2500,
            min=1000,
            max=100000000
        )
        if ok:
            self.update_interval_signal.emit(interval)

    def update_display(self, daw, project, cpu, ram):
        """Update the main display"""
        self.daw_name.setText(daw)
        self.project_name.setText(project)
        self.cpu_usage.setText(cpu)
        self.ram_usage.setText(ram)

    def update_discord_status(self, message, connected=False):
        """Update Discord connection status in tray menu"""
        self.discord_username_action.setText(message)

    def set_version(self, version):
        """Set the version display"""
        version_text = f"DAW Discord Rich Presence v{version}"
        self.setWindowTitle(version_text)
        self.version_action.setText(version_text)

    def closeEvent(self, event):
        """Handle window close event"""
        if not self.exiting:
            event.ignore()
            self.hide()
        else:
            event.accept()
