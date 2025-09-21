from typing import Callable, List, Optional

from DAWPY.models import AppSettings, DAWInfo, DAWStatus
from DAWPY.services import ConfigurationService, ProcessMonitorService


class DAWController:
    """Controller for DAW monitoring and status management"""

    def __init__(
        self,
        process_monitor: ProcessMonitorService,
        config_service: ConfigurationService,
    ):
        self.process_monitor = process_monitor
        self.config_service = config_service
        self._current_status = DAWStatus()

        # Callbacks
        self.on_daw_started: Optional[Callable[[DAWStatus], None]] = None
        self.on_daw_stopped: Optional[Callable[[DAWStatus], None]] = None
        self.on_status_updated: Optional[Callable[[DAWStatus], None]] = None

    @property
    def current_status(self) -> DAWStatus:
        """Get current DAW status"""
        return self._current_status

    def scan_for_daws(self, settings: AppSettings) -> DAWStatus:
        """Scan for running DAWs and update status"""
        previous_status = self._current_status
        new_status = DAWStatus()

        # Get all supported DAW configurations
        daw_configs = self.config_service.load_daw_configurations()

        # Check each DAW
        for daw_info in daw_configs:
            process_info = self.process_monitor.get_process_by_name(
                daw_info.process_name
            )

            if process_info:
                # DAW is running - build status
                new_status = self._build_daw_status(daw_info, process_info, settings)
                break

        # Update current status
        self._current_status = new_status

        # Fire callbacks
        self._handle_status_change(previous_status, new_status)

        if self.on_status_updated:
            self.on_status_updated(new_status)

        return new_status

    def _build_daw_status(
        self, daw_info: DAWInfo, process_info, settings: AppSettings
    ) -> DAWStatus:
        """Build DAWStatus from process information"""
        status = DAWStatus(
            daw_info=daw_info,
            is_running=True,
            cpu_usage=process_info.cpu_percent,
            ram_usage_mb=process_info.memory_mb,
            window_title=process_info.window_title,
            version=self.process_monitor.get_process_version(process_info.exe_path),
        )

        # Extract project name if not hidden
        if not settings.hide_project_name:
            status.project_name = status.extract_project_name()
        else:
            status.project_name = "(hidden)"

        return status

    def _handle_status_change(self, previous: DAWStatus, current: DAWStatus):
        """Handle changes in DAW status"""
        # DAW started
        if not previous.is_running and current.is_running:
            if self.on_daw_started:
                self.on_daw_started(current)

        # DAW stopped
        elif previous.is_running and not current.is_running:
            if self.on_daw_stopped:
                self.on_daw_stopped(current)

    def get_supported_daws(self) -> List[DAWInfo]:
        """Get list of all supported DAWs"""
        return self.config_service.load_daw_configurations()
