import json
import os

from DAWPY.models import DAWInfo


class ConfigurationService:
    """Service for loading DAW configurations"""

    def __init__(self, config_path: str | None = None) -> None:
        self.config_path = config_path
        self._daws_cache = None

    def load_daw_configurations(self) -> list[DAWInfo]:
        """Load DAW configurations from JSON file"""
        if self._daws_cache is not None:
            return self._daws_cache

        if not os.path.exists(self.config_path):
            raise FileNotFoundError(f"DAW configuration file not found: {self.config_path}")

        try:
            with open(self.config_path, encoding="utf-8") as f:
                data = json.load(f)

            daws = []
            for item in data:
                try:
                    daw_info = DAWInfo(
                        process_name=item["ProcessName"],
                        display_text=item["DisplayText"],
                        title_regex=item["TitleRegex"],
                        client_id=item["ClientID"],
                        hide_version=item.get("HideVersion", False),
                    )
                    daws.append(daw_info)

                except (KeyError, ValueError) as e:
                    print(f"Warning: Invalid DAW configuration: {e}")
                    continue

            self._daws_cache = daws
            return daws

        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid JSON in DAW configuration file: {e}") from e

    def get_daw_by_process_name(self, process_name: str) -> DAWInfo | None:
        """Get DAW configuration by process name"""
        daws = self.load_daw_configurations()
        target_name = process_name.lower().replace(".exe", "")

        for daw in daws:
            if daw.process_name.lower() == target_name:
                return daw

        return None
