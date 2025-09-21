import json
import os
import sys
import time

from PyQt5.QtCore import QTimer
from PyQt5.QtWidgets import QApplication
from pypresence import Presence

from daw_config import DAWConfig
from form import MainWindow
from settings import Settings


class DAWRPCApp:
    def __init__(self):
        self.app = QApplication(sys.argv)
        self.window = MainWindow()

        # Initialize variables
        self.version = "1.0"
        self.current_daw_name = "None"
        self.discord_username_global = ""
        self.error_displayed = False
        self.hide_daw_version = False

        # Settings and paths
        self.settings = Settings()
        self.settings_path = os.path.join(os.path.dirname(__file__), "settings.json")
        self.daw_config_path = os.path.join(os.path.dirname(__file__), "daws.json")

        # Discord RPC client
        self.client = None
        self.presence_data = {
            "large_image": "icon",
            "large_text": f"DAWRPC v{self.version} by MihaiStreames"
        }

        # DAW configurations list
        self.daws = []

        # Setup
        self.setup()

    def setup(self):
        """Initialize the application"""
        # Check for multiple instances
        if self.is_already_running():
            from PyQt5.QtWidgets import QMessageBox
            QMessageBox.critical(None, "Error", "Another instance of DAWRPC is already running.")
            sys.exit(1)

        # Load settings
        self.settings.load(self.settings_path)
        self.window.update_setting_display(self.settings)

        # Check for DAW config file
        if not os.path.exists(self.daw_config_path):
            from PyQt5.QtWidgets import QMessageBox
            QMessageBox.critical(None, "Error",
                                 f'"{self.daw_config_path}" not found.\n'
                                 'Redownload this file from the DAWRPC repository or create one yourself, '
                                 'and reopen this program.')
            sys.exit(1)

        # Load DAW configurations
        with open(self.daw_config_path, 'r') as f:
            daw_data = json.load(f)
            self.daws = [DAWConfig(**daw) for daw in daw_data]

        # Setup window
        self.window.set_version(self.version)

        # Connect signals
        self.window.exit_action.triggered.connect(self.exit_app)
        self.window.toggle_project_name_signal.connect(self.toggle_hide_project_name)
        self.window.toggle_system_usage_signal.connect(self.toggle_hide_system_usage)
        self.window.update_interval_signal.connect(self.set_update_interval)

        # Setup timer
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_presence)
        self.timer.start(self.settings.update_interval)

    def is_already_running(self):
        """Check if another instance is already running"""
        import psutil
        current_pid = os.getpid()
        for proc in psutil.process_iter(['pid', 'name']):
            if proc.info['name'] == 'DAWRPC.exe' or 'python' in proc.info['name']:
                if proc.info['pid'] != current_pid:
                    # Check if it's running our script
                    try:
                        cmdline = proc.cmdline()
                        if any('main.py' in arg for arg in cmdline):
                            return True
                    except:
                        pass
        return False

    def toggle_hide_project_name(self):
        """Toggle hiding project name"""
        self.settings.hide_project_name = not self.settings.hide_project_name
        self.settings.save(self.settings_path)
        self.window.update_setting_display(self.settings)

    def toggle_hide_system_usage(self):
        """Toggle hiding system usage"""
        self.settings.hide_system_usage = not self.settings.hide_system_usage
        self.settings.save(self.settings_path)
        self.window.update_setting_display(self.settings)

    def set_update_interval(self, interval):
        """Set the update interval"""
        if interval and interval >= 1000:
            self.settings.update_interval = interval
            self.settings.save(self.settings_path)
            self.timer.setInterval(interval)
            from PyQt5.QtWidgets import QMessageBox
            QMessageBox.information(None, "Notification",
                                    "Successfully set the presence update interval.")

    def update_presence(self):
        """Update Discord presence"""
        try:
            client_id = ""
            is_running = False

            # Process DAW information
            for daw in self.daws:
                daw.update(self.settings)
                if daw.is_running:
                    self.window.update_display(
                        daw.display_text,
                        daw.project_name,
                        daw.cpu_usage,
                        daw.ram_usage
                    )
                    client_id = daw.client_id
                    self.hide_daw_version = daw.hide_version
                    is_running = True
                    break

            if not is_running:
                self.window.update_display("None", "None", "Undefined", "Undefined")

            # Update Discord RPC
            daw_name = self.window.daw_name.text()
            if daw_name != self.current_daw_name:
                self.setup_discord_client(client_id)
                self.current_daw_name = daw_name

            if daw_name != "None" and self.client:
                self.update_discord_presence()
            elif self.client:
                self.disconnect_client()

        except Exception as e:
            print(f"Error updating presence: {e}")

    def setup_discord_client(self, client_id):
        """Setup Discord RPC client"""
        try:
            if self.client:
                self.disconnect_client()

            if not client_id:
                return

            self.client = Presence(client_id)
            self.client.connect()

            # Update UI
            self.window.update_discord_status("Connected to Discord", True)
            self.window.change_notify_icon("green")
            self.error_displayed = False

            # Set start time
            self.presence_data["start"] = int(time.time())

        except Exception as e:
            self.window.update_discord_status(f"Connection failed: {e}", False)
            self.window.change_notify_icon("red")
            if not self.error_displayed:
                from PyQt5.QtWidgets import QMessageBox
                QMessageBox.warning(None, "DAWRPC Connection Error",
                                    f"Cannot connect to Discord RPC server.\n{str(e)}")
                self.error_displayed = True

    def update_discord_presence(self):
        """Update the Discord presence data"""
        if not self.client:
            return

        try:
            project = self.window.project_name.text()
            cpu = self.window.cpu_usage.text()
            ram = self.window.ram_usage.text()

            if self.settings.hide_system_usage:
                details = "Opening project:"
                state = project
            else:
                details = f"Opening project: {project}"
                if project in ["None", "Untitled"]:
                    details = "Opening an untitled project"
                state = f"{cpu} of CPU usage, {ram} of RAM usage"
                if not self.hide_daw_version:
                    for daw in self.daws:
                        if daw.is_running:
                            version_text = daw.version
                            state = f"v{version_text}, " + state

            self.client.update(
                details=details,
                state=state,
                large_image=self.presence_data["large_image"],
                large_text=self.presence_data["large_text"],
                start=self.presence_data["start"]
            )

        except Exception as e:
            print(f"Error updating Discord presence: {e}")

    def disconnect_client(self):
        """Disconnect Discord client"""
        if self.client:
            try:
                self.client.clear()
                self.client.close()
            except:
                pass
            self.client = None
            self.window.change_notify_icon("red")
            self.window.update_discord_status("Open a DAW to begin displaying RPC", False)

    def exit_app(self):
        """Exit the application"""
        self.disconnect_client()
        self.app.quit()
        sys.exit(0)

    def run(self):
        """Run the application"""
        self.window.show()
        sys.exit(self.app.exec_())


if __name__ == "__main__":
    app = DAWRPCApp()
    app.run()
