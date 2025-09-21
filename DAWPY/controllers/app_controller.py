import os
import sys
from typing import Optional

from loguru import logger
from PyQt5.QtCore import QTimer
from PyQt5.QtWidgets import QApplication, QMessageBox

from DAWPY.controllers.daw_controller import DAWController
from DAWPY.controllers.discord_controller import DiscordController
from DAWPY.models import AppSettings
from DAWPY.services import (ConfigurationService, DiscordService,
                            ProcessMonitorService)
from DAWPY.services.logging_service import LoggingService, log_errors


class AppController:
    """Main application controller"""

    def __init__(self, app_version: str):
        self.app_version = app_version

        # Since it's for Windows only, we use APPDATA
        app_data = os.environ.get('APPDATA', os.path.expanduser("~"))
        self._user_data_dir = os.path.join(app_data, "DAWPresence")
        os.makedirs(self._user_data_dir, exist_ok=True)

        # Bundled files path (read-only configs)
        if getattr(sys, 'frozen', False):
            # PyInstaller bundle
            self._bundled_dir = sys._MEIPASS
        else:
            # Development mode
            self._bundled_dir = os.path.join(os.path.dirname(__file__), "..")

        # Logging
        self.logging_service = LoggingService(self._user_data_dir)
        logger.info(f"DAWPresence v{app_version} starting up")

        self.app: Optional[QApplication] = None
        self.main_window = None

        # File paths
        self._settings_path = os.path.join(self._user_data_dir, "settings.json")
        self._daws_config_path = os.path.join(self._bundled_dir, "daws.json")

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

        # Callbacks
        self._setup_callbacks()

        logger.success("Application controller initialized successfully")

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

    @log_errors
    def initialize(self) -> bool:
        """Initialize the application"""
        try:
            if self._is_already_running():
                logger.warning("Another instance is already running")
                return False

            # Load settings
            self.settings = AppSettings.load(self._settings_path)
            logger.info(f"Settings loaded from: {self._settings_path}")
            logger.info(
                f"Settings loaded: Update interval={self.settings.update_interval}ms"
            )

            # Validate configs
            try:
                daw_configs = self.config_service.load_daw_configurations()
                logger.info(f"Loaded {len(daw_configs)} DAW configurations from {self._daws_config_path}")
            except (FileNotFoundError, ValueError) as e:
                logger.error(f"DAW configuration error: {e}")
                return False

            logger.success("Application initialized successfully")
            return True

        except Exception as e:
            logger.exception(f"Failed to initialize application: {e}")
            return False

    def start(self, app: QApplication, main_window):
        """Start the application"""
        self.app = app
        self.main_window = main_window

        # UI callbacks
        self._connect_ui_signals()

        # Update timer
        self.update_timer = QTimer()
        self.update_timer.timeout.connect(self._update_cycle)
        self.update_timer.start(self.settings.update_interval)

        # Update UI
        self._update_ui_settings()
        self.main_window.show()

    def shutdown(self):
        """Shutdown the application"""
        logger.info("Shutting down DAWPresence...")

        # Stop timer
        if self.update_timer:
            self.update_timer.stop()
            logger.debug("Update timer stopped")

        # Disconnect Discord
        self.discord_controller.disconnect()

        # Save settings
        if self.settings:
            self.settings.save(self._settings_path)
            logger.info("Settings saved")

        logger.success("DAWPresence shutdown complete")

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
        # Disable instance check for PyInstaller builds to avoid parent process confusion
        if getattr(sys, 'frozen', False):
            return False

        import psutil

        current_pid = os.getpid()

        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            try:
                if proc.info["pid"] == current_pid:
                    continue

                # Check for Python script (development mode only)
                cmdline = proc.info.get("cmdline", [])
                if cmdline and any("main.py" in arg for arg in cmdline):
                    return True

            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        return False
