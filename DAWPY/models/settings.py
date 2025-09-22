import json
import os
from dataclasses import asdict, dataclass

from DAWPY.utils import ValidationUtils


@dataclass
class AppSettings:
    """Application settings with validation"""

    hide_project_name: bool = False
    hide_system_usage: bool = False
    update_interval: int = 2500

    def __post_init__(self) -> None:
        """Validate settings"""
        if not ValidationUtils.validate_update_interval(self.update_interval):
            raise ValueError("Update interval must be between 1000ms and 100,000,000ms")

    @classmethod
    def load(cls, filepath: str) -> "AppSettings":
        """Load settings from JSON file"""
        if not os.path.exists(filepath):
            # Create default settings file
            settings = cls()
            settings.save(filepath)
            return settings

        with open(filepath) as f:
            data = json.load(f)
            return cls(
                hide_project_name=data.get("HideProjectName", False),
                hide_system_usage=data.get("HideSystemUsage", False),
                update_interval=data.get("UpdateInterval", 2500),
            )

    def save(self, filepath: str) -> None:
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
