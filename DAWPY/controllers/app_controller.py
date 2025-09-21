import os
from typing import Optional

from PyQt5.QtCore import QTimer
from PyQt5.QtWidgets import QApplication, QMessageBox

from DAWPY.controllers.daw_controller import DAWController
from DAWPY.controllers.discord_controller import DiscordController
from DAWPY.models import AppSettings
from DAWPY.services import (ConfigurationService, DiscordService,
                            ProcessMonitorService)


class AppController:
    """Main application controller"""

    def __init__(self, app_version: str):
        self.app_version = app_version
        self.app: Optional[QApplication] = None
        self.main_window = None

        # Paths
        self._config_dir = self._get_config_directory()
        self._settings_path = os.path.join(self._config_dir, "settings.json")
        self._daws_config_path = os.path.join(self._config_dir, "daws.json")

        # Services
        self.process_monitor = ProcessMonitorService()
        self.config_service = ConfigurationService(self._daws_config_path)
        self.discord_service = DiscordService()

        # Controllers
        self.daw_controller = DAWController(self.process_monitor, self.config_service)
        self.discord_controller = DiscordController(self.discord_service, app_version)

        # Settings and state
        self.settings: Optional[AppSettings] = None
        self.update_timer: Optional[QTimer] = None

        # Setup callbacks
        self._setup_callbacks()

    @staticmethod
    def _get_config_directory() -> str:
        """Get configuration directory path"""
        return os.path.join(os.path.dirname(__file__), "..", "config")

    def _setup_callbacks(self):
        """Setup inter-controller communication"""
        # DAW Controller callbacks
        self.daw_controller.on_daw_started = self._on_daw_started
        self.daw_controller.on_daw_stopped = self._on_daw_stopped
        self.daw_controller.on_status_updated = self._on_daw_status_updated

        # Discord Controller callbacks
        self.discord_controller.on_connected = self._on_discord_connected
        self.discord_controller.on_disconnected = self._on_discord_disconnected
        self.discord_controller.on_error = self._on_discord_error

    def initialize(self) -> bool:
        """Initialize the application"""
        try:
            # Check for multiple instances
            if self._is_already_running():
                QMessageBox.critical(
                    None, "Error", "Another instance of DAWPresence is already running."
                )
                return False

            # Ensure config directory exists
            os.makedirs(self._config_dir, exist_ok=True)

            # Copy default daws.json if it doesn't exist
            self._ensure_daws_config()

            # Load settings
            self.settings = AppSettings.load(self._settings_path)

            # Validate DAW configurations
            try:
                self.config_service.load_daw_configurations()
            except (FileNotFoundError, ValueError) as e:
                QMessageBox.critical(
                    None,
                    "Configuration Error",
                    f"Error loading DAW configurations:\n{e}",
                )
                return False

            return True

        except Exception as e:
            QMessageBox.critical(
                None, "Initialization Error", f"Failed to initialize application:\n{e}"
            )
            return False

    def start(self, app: QApplication, main_window):
        """Start the application"""
        self.app = app
        self.main_window = main_window

        # Setup UI callbacks
        self._connect_ui_signals()

        # Setup update timer
        self.update_timer = QTimer()
        self.update_timer.timeout.connect(self._update_cycle)
        self.update_timer.start(self.settings.update_interval)

        # Update UI with current settings
        self._update_ui_settings()

        # Show window
        self.main_window.show()

    def shutdown(self):
        """Shutdown the application"""
        # Stop timer
        if self.update_timer:
            self.update_timer.stop()

        # Disconnect Discord
        self.discord_controller.disconnect()

        # Save settings
        if self.settings:
            self.settings.save(self._settings_path)

    def toggle_hide_project_name(self):
        """Toggle project name visibility"""
        self.settings = self.settings.update(
            hide_project_name=not self.settings.hide_project_name
        )
        self.settings.save(self._settings_path)
        self._update_ui_settings()

    def toggle_hide_system_usage(self):
        """Toggle system usage visibility"""
        self.settings = self.settings.update(
            hide_system_usage=not self.settings.hide_system_usage
        )
        self.settings.save(self._settings_path)
        self._update_ui_settings()

    def set_update_interval(self, interval: int):
        """Set presence update interval"""
        try:
            self.settings = self.settings.update(update_interval=interval)
            self.settings.save(self._settings_path)

            if self.update_timer:
                self.update_timer.setInterval(interval)

            QMessageBox.information(
                None, "Success", "Update interval changed successfully."
            )
        except ValueError as e:
            QMessageBox.warning(None, "Invalid Interval", str(e))

    def _update_cycle(self):
        """Main update cycle"""
        try:
            # Scan for DAWs and update status
            daw_status = self.daw_controller.scan_for_daws(self.settings)

            # Update Discord presence
            self.discord_controller.update_from_daw_status(daw_status, self.settings)

        except Exception as e:
            print(f"Error in update cycle: {e}")

    def _connect_ui_signals(self):
        """Connect UI signals to controller methods"""
        if hasattr(self.main_window, "toggle_project_name_signal"):
            self.main_window.toggle_project_name_signal.connect(
                self.toggle_hide_project_name
            )

        if hasattr(self.main_window, "toggle_system_usage_signal"):
            self.main_window.toggle_system_usage_signal.connect(
                self.toggle_hide_system_usage
            )

        if hasattr(self.main_window, "update_interval_signal"):
            self.main_window.update_interval_signal.connect(self.set_update_interval)

        if hasattr(self.main_window, "exit_signal"):
            self.main_window.exit_signal.connect(self.shutdown)

    def _update_ui_settings(self):
        """Update UI to reflect current settings"""
        if hasattr(self.main_window, "update_settings_display"):
            self.main_window.update_settings_display(self.settings)

    def _on_daw_started(self, daw_status):
        """Handle DAW started event"""
        if hasattr(self.main_window, "on_daw_started"):
            self.main_window.on_daw_started(daw_status)

    def _on_daw_stopped(self, daw_status):
        """Handle DAW stopped event"""
        if hasattr(self.main_window, "on_daw_stopped"):
            self.main_window.on_daw_stopped(daw_status)

    def _on_daw_status_updated(self, daw_status):
        """Handle DAW status update"""
        if hasattr(self.main_window, "update_daw_display"):
            self.main_window.update_daw_display(daw_status)

    def _on_discord_connected(self):
        """Handle Discord connected event"""
        if hasattr(self.main_window, "on_discord_connected"):
            self.main_window.on_discord_connected()

    def _on_discord_disconnected(self):
        """Handle Discord disconnected event"""
        if hasattr(self.main_window, "on_discord_disconnected"):
            self.main_window.on_discord_disconnected()

    def _on_discord_error(self, error: Exception):
        """Handle Discord error"""
        if hasattr(self.main_window, "on_discord_error"):
            self.main_window.on_discord_error(error)

    @staticmethod
    def _is_already_running() -> bool:
        """Check if another instance is already running"""
        import psutil

        current_pid = os.getpid()

        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            try:
                if proc.info["pid"] == current_pid:
                    continue

                # Check for DAWPresence executable
                if "DAWPresence" in proc.info["name"]:
                    return True

                # Check for Python script
                cmdline = proc.info.get("cmdline", [])
                if cmdline and any("main.py" in arg for arg in cmdline):
                    return True

            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        return False

    def _ensure_daws_config(self):
        """Ensure daws.json exists in config directory"""
        if os.path.exists(self._daws_config_path):
            return

        # Copy from package directory
        package_daws_path = os.path.join(os.path.dirname(__file__), "..", "daws.json")

        if os.path.exists(package_daws_path):
            import shutil

            shutil.copy2(package_daws_path, self._daws_config_path)
        else:
            raise FileNotFoundError(
                "daws.json not found in package directory. Please download it from the repository."
            )
