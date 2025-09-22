from collections.abc import Callable

from loguru import logger

from DAWPY.models import AppSettings, DAWInfo, DAWStatus
from DAWPY.services import ConfigurationService, ProcessMonitorService
from DAWPY.services.logging_service import log_performance


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
        self.on_daw_started: Callable[[DAWStatus], None] | None = None
        self.on_daw_stopped: Callable[[DAWStatus], None] | None = None
        self.on_status_updated: Callable[[DAWStatus], None] | None = None

        logger.info("DAW Controller initialized")

    @log_performance
    def scan_for_daws(self, settings: AppSettings) -> DAWStatus:
        """Scan for running DAWs and update status"""
        previous_status = self._current_status
        new_status = DAWStatus()

        # Load configs
        try:
            daw_configs = self.config_service.load_daw_configurations()
            logger.debug(f"Loaded {len(daw_configs)} DAW configurations")
        except Exception as e:
            logger.error(f"Failed to load DAW configurations: {e}")
            return new_status

        # Check each DAW
        for daw_info in daw_configs:
            process_info = self.process_monitor.get_process_by_name(daw_info.process_name)

            if process_info:
                new_status = self._build_daw_status(daw_info, process_info, settings)
                logger.info(
                    f"DAW detected: {daw_info.display_text} | Project: {new_status.project_name} | PID: {process_info.pid}",
                )
                break

        # Update status and handle changes
        self._current_status = new_status
        self._handle_status_change(previous_status, new_status)

        if self.on_status_updated:
            self.on_status_updated(new_status)

        return new_status

    def _build_daw_status(
        self,
        daw_info: DAWInfo,
        process_info,
        settings: AppSettings,
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

        # Extract project name
        if not settings.hide_project_name:
            status.project_name = status.extract_project_name()
        else:
            status.project_name = "(hidden)"

        return status

    def _handle_status_change(self, previous: DAWStatus, current: DAWStatus):
        """Handle changes in DAW status"""
        # DAW started
        if not previous.is_running and current.is_running:
            logger.success(f"DAW Started: {current.display_name}")
            if self.on_daw_started:
                self.on_daw_started(current)

        # DAW stopped
        elif previous.is_running and not current.is_running:
            logger.info("No DAW detected")
            if self.on_daw_stopped:
                self.on_daw_stopped(current)
