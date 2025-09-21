import re
from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class DAWInfo:
    """Immutable DAW configuration data"""
    process_name: str
    display_text: str
    title_regex: str
    client_id: str
    hide_version: bool = False

    def __post_init__(self):
        """Validate DAW configuration"""
        if not self.process_name:
            raise ValueError("Process name cannot be empty")
        if not self.display_text:
            raise ValueError("Display text cannot be empty")
        if not self.client_id:
            raise ValueError("Client ID cannot be empty")

        # Validate regex
        try:
            re.compile(self.title_regex)
        except re.error as e:
            raise ValueError(f"Invalid title regex: {e}")


@dataclass
class DAWStatus:
    """Current runtime state of a DAW"""
    daw_info: Optional[DAWInfo] = None
    is_running: bool = False
    cpu_usage: float = 0.0
    ram_usage_mb: int = 0
    project_name: str = "None"
    version: str = "0.0.0"
    window_title: str = ""

    @property
    def cpu_usage_str(self) -> str:
        """Format CPU usage as string"""
        if self.is_running:
            return f"{self.cpu_usage:.2f}%"
        return "Undefined"

    @property
    def ram_usage_str(self) -> str:
        """Format RAM usage as string"""
        if self.is_running:
            return f"{self.ram_usage_mb}MB"
        return "Undefined"

    @property
    def display_name(self) -> str:
        """Get display name for UI"""
        if self.daw_info:
            return self.daw_info.display_text
        return "None"

    def extract_project_name(self) -> str:
        """Extract project name from window title using regex"""
        if not self.daw_info or not self.window_title:
            return "None"

        try:
            match = re.search(self.daw_info.title_regex, self.window_title)
            if match:
                project = match.group(1).strip()
                return project if project else "Untitled"
        except (re.error, IndexError):
            pass

        return "None"
