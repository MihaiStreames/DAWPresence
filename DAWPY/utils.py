import os
from pathlib import Path
import shutil

import psutil


class PathUtils:
    """Path utilities"""

    @staticmethod
    def get_project_root() -> str:
        """Get project root directory (where DAWPY folder is)"""
        # Go up from DAWPY/utils.py to project root
        return str(Path(__file__).parent.parent)

    @staticmethod
    def get_data_directory() -> str:
        """Get .data directory path"""
        return os.path.join(PathUtils.get_project_root(), ".data")

    @staticmethod
    def get_settings_path() -> str:
        """Get settings file path"""
        return os.path.join(PathUtils.get_data_directory(), "settings.json")

    @staticmethod
    def get_daws_config_path() -> str:
        """Get DAWs configuration file path"""
        return os.path.join(PathUtils.get_data_directory(), "daws.json")

    @staticmethod
    def get_icon_path(icon_name: str) -> str:
        """Get icon file path"""
        return os.path.join(PathUtils.get_project_root(), "assets", f"{icon_name}.ico")

    @staticmethod
    def ensure_daws_config() -> None:
        """Ensure daws.json exists in .data directory"""
        data_config_path = PathUtils.get_daws_config_path()

        if os.path.exists(data_config_path):
            return

        # Copy from DAWPY/daws.json to .data/daws.json
        source_path = os.path.join(Path(__file__).parent, "daws.json")

        if os.path.exists(source_path):
            shutil.copy2(source_path, data_config_path)
        else:
            raise FileNotFoundError(
                f"daws.json not found at {source_path}. Please ensure the file exists in the DAWPY directory or download it from the repository.",
            )


class ProcessUtils:
    """Utilities for process management and detection"""

    @staticmethod
    def is_app_already_running(app_name: str = "DAWPresence") -> bool:
        """Check if another instance of the application is already running"""
        current_pid = os.getpid()

        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            try:
                if proc.info["pid"] == current_pid:
                    continue

                # Check for executable name
                if app_name in proc.info["name"]:
                    return True

                # Check for Python script
                cmdline = proc.info.get("cmdline", [])
                if cmdline and any("main.py" in arg for arg in cmdline):
                    return True

            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        return False

    @staticmethod
    def normalize_process_name(process_name: str) -> str:
        """Normalize process name by removing .exe extension and converting to lowercase"""
        return process_name.lower().replace(".exe", "")


class SystemUtils:
    """System-related utilities"""

    @staticmethod
    def get_cpu_count() -> int:
        """Get CPU core count with fallback"""
        return os.cpu_count() or 1

    @staticmethod
    def format_cpu_usage(cpu_percent: float, normalize: bool = True) -> str:
        """Format CPU usage percentage"""
        if normalize:
            cpu_percent = cpu_percent / SystemUtils.get_cpu_count()
        return f"{cpu_percent:.2f}%"

    @staticmethod
    def format_memory_usage(memory_mb: int) -> str:
        """Format memory usage in MB"""
        return f"{memory_mb}MB"

    @staticmethod
    def bytes_to_mb(memory_bytes: int) -> int:
        """Convert bytes to megabytes"""
        return int(memory_bytes / (1024 * 1024))


class ValidationUtils:
    """Validation utilities"""

    @staticmethod
    def validate_update_interval(interval: int) -> bool:
        """Validate update interval is within acceptable range"""
        return 1000 <= interval <= 100_000_000

    @staticmethod
    def validate_required_string(value: str, field_name: str) -> None:
        """Validate that a string field is not empty"""
        if not value or not value.strip():
            raise ValueError(f"{field_name} cannot be empty")


class IconUtils:
    """Icon and UI utilities"""

    @staticmethod
    def get_icon_path(icon_name: str) -> str | None:
        """Get path to icon file with fallback handling"""
        icon_path = PathUtils.get_icon_path(icon_name)

        if os.path.exists(icon_path):
            return icon_path
        return None
