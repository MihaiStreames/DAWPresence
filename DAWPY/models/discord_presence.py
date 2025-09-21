import time
from dataclasses import dataclass
from typing import Optional

from DAWPY.models import AppSettings, DAWStatus


@dataclass
class DiscordPresence:
    """Discord Rich Presence data structure"""
    details: str = ""
    state: str = ""
    large_image: str = "icon"
    large_text: str = ""
    start_time: Optional[int] = None

    def __post_init__(self):
        """Set start time if not provided"""
        if self.start_time is None:
            self.start_time = int(time.time())

    @classmethod
    def create_for_daw(cls, daw_status: DAWStatus, settings: AppSettings, version: str) -> 'DiscordPresence':
        """Create Discord presence from DAW status"""
        if not daw_status.is_running:
            return cls(
                details="Not using any DAW",
                state="Idle",
                large_text=f"DAWPresence v{version}"
            )

        project = daw_status.project_name
        if settings.hide_project_name:
            project = "(hidden)"

        # Build details
        if project in ["None", "Untitled"]:
            details = "Opening an untitled project"
        else:
            details = f"Opening project: {project}"

        # Build state
        if settings.hide_system_usage:
            state = f"Using {daw_status.display_name}"
        else:
            state_parts = []

            # Add version if not hidden
            if daw_status.daw_info and not daw_status.daw_info.hide_version and daw_status.version != "0.0.0":
                state_parts.append(f"v{daw_status.version}")

            # Add system usage
            state_parts.append(f"{daw_status.cpu_usage_str} CPU")
            state_parts.append(f"{daw_status.ram_usage_str} RAM")

            state = ", ".join(state_parts)

        return cls(
            details=details,
            state=state,
            large_text=f"DAWPresence v{version}"
        )

    def to_pypresence_dict(self) -> dict:
        """Convert to pypresence format"""
        return {
            'details': self.details,
            'state': self.state,
            'large_image': self.large_image,
            'large_text': self.large_text,
            'start': self.start_time
        }
