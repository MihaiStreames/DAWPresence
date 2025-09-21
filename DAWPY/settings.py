import json
import os


class Settings:
    def __init__(self):
        self.hide_project_name = False
        self.hide_system_usage = False
        self.update_interval = 2500
        self.settings_path = None

    def load(self, filepath):
        """Load settings from JSON file"""
        self.settings_path = filepath

        # Create file if it doesn't exist
        if not os.path.exists(filepath):
            self.save(filepath)
            return

        try:
            with open(filepath, 'r') as f:
                data = json.load(f)
                self.hide_project_name = data.get('HideProjectName', False)
                self.hide_system_usage = data.get('HideSystemUsage', False)
                self.update_interval = data.get('UpdateInterval', 2500)
        except (json.JSONDecodeError, FileNotFoundError):
            # If file is corrupted or not found, save defaults
            self.save(filepath)

    def save(self, filepath=None):
        """Save settings to JSON file"""
        if filepath:
            self.settings_path = filepath

        if not self.settings_path:
            return

        data = {
            'HideProjectName': self.hide_project_name,
            'HideSystemUsage': self.hide_system_usage,
            'UpdateInterval': self.update_interval
        }

        with open(self.settings_path, 'w') as f:
            json.dump(data, f, indent=4)
