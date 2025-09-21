import json
import os
from dataclasses import asdict, dataclass


@dataclass
class AppSettings:
    """Application settings with validation"""

    hide_project_name: bool = False
    hide_system_usage: bool = False
    update_interval: int = 2500

    def __post_init__(self):
        """Validate settings"""
        if self.update_interval < 1000:
            raise ValueError("Update interval must be at least 1000ms")
        if self.update_interval > 100_000_000:
            raise ValueError("Update interval too large")

    @classmethod
    def load(cls, filepath: str) -> "AppSettings":
        """Load settings from JSON file"""
        if not os.path.exists(filepath):
            # Create default settings file
            settings = cls()
            settings.save(filepath)
            return settings

        try:
            with open(filepath, "r") as f:
                data = json.load(f)
                return cls(
                    hide_project_name=data.get("HideProjectName", False),
                    hide_system_usage=data.get("HideSystemUsage", False),
                    update_interval=data.get("UpdateInterval", 2500),
                )
        except (json.JSONDecodeError, KeyError, ValueError) as e:
            # If file is corrupted, create new one with defaults
            settings = cls()
            settings.save(filepath)
            return settings

    def save(self, filepath: str):
        """Save settings to JSON file"""
        data = {
            "HideProjectName": self.hide_project_name,
            "HideSystemUsage": self.hide_system_usage,
            "UpdateInterval": self.update_interval,
        }

        os.makedirs(os.path.dirname(filepath), exist_ok=True)
        with open(filepath, "w") as f:
            json.dump(data, f, indent=4)

    def update(self, **kwargs) -> "AppSettings":
        """Create new settings instance with updated values"""
        current_data = asdict(self)
        current_data.update(kwargs)
        return AppSettings(**current_data)
