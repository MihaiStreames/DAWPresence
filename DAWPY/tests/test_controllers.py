from unittest.mock import Mock

from DAWPY.controllers import DAWController, DiscordController
from DAWPY.models import DAWInfo, AppSettings, DAWStatus


class TestDAWController:
    """Test DAW controller"""

    def test_scan_for_daws_none_running(self):
        """Test scanning when no DAWs are running"""
        # Setup mocks
        mock_process_monitor = Mock()
        mock_process_monitor.get_process_by_name.return_value = None

        mock_config_service = Mock()
        mock_config_service.load_daw_configurations.return_value = [
            DAWInfo("FL64", "FL Studio", "test", "123")
        ]

        # Test
        controller = DAWController(mock_process_monitor, mock_config_service)
        settings = AppSettings()

        status = controller.scan_for_daws(settings)

        # Verify
        assert not status.is_running
        assert status.display_name == "None"

    def test_scan_for_daws_found_running(self, sample_daw_info, sample_process_info):
        """Test scanning when DAW is found running"""
        # Setup mocks
        mock_process_monitor = Mock()
        mock_process_monitor.get_process_by_name.return_value = sample_process_info
        mock_process_monitor.get_process_version.return_value = "21.2.0"

        mock_config_service = Mock()
        mock_config_service.load_daw_configurations.return_value = [sample_daw_info]

        # Test
        controller = DAWController(mock_process_monitor, mock_config_service)
        settings = AppSettings()

        status = controller.scan_for_daws(settings)

        # Verify
        assert status.is_running
        assert status.display_name == "FL Studio"
        assert status.cpu_usage == 15.5
        assert status.ram_usage_mb == 512
        assert status.version == "21.2.0"

    def test_callbacks_on_daw_started(self, sample_daw_info, sample_process_info):
        """Test that callbacks are fired when DAW starts"""
        # Setup
        mock_process_monitor = Mock()
        mock_process_monitor.get_process_by_name.return_value = sample_process_info
        mock_process_monitor.get_process_version.return_value = "21.2.0"

        mock_config_service = Mock()
        mock_config_service.load_daw_configurations.return_value = [sample_daw_info]

        controller = DAWController(mock_process_monitor, mock_config_service)

        # Setup callback mock
        callback_mock = Mock()
        controller.on_daw_started = callback_mock

        # First scan (no DAW)
        mock_process_monitor.get_process_by_name.return_value = None
        controller.scan_for_daws(AppSettings())

        # Second scan (DAW started)
        mock_process_monitor.get_process_by_name.return_value = sample_process_info
        controller.scan_for_daws(AppSettings())

        # Verify callback was called
        callback_mock.assert_called_once()


class TestDiscordController:
    """Test Discord controller"""

    def test_update_from_idle_daw_status(self):
        """Test updating from idle DAW status"""
        mock_discord_service = Mock()
        mock_discord_service.is_connected = False

        controller = DiscordController(mock_discord_service, "2.0")

        # Test with idle DAW
        daw_status = DAWStatus(is_running=False)
        settings = AppSettings()

        controller.update_from_daw_status(daw_status, settings)

        # Should disconnect
        mock_discord_service.disconnect.assert_called_once()

    def test_update_from_running_daw_status(self, sample_daw_info):
        """Test updating from running DAW status"""
        mock_discord_service = Mock()
        mock_discord_service.is_connected = True

        controller = DiscordController(mock_discord_service, "2.0")
        controller._current_client_id = sample_daw_info.client_id

        # Test with running DAW
        daw_status = DAWStatus(
            daw_info=sample_daw_info,
            is_running=True,
            project_name="Test Project"
        )
        settings = AppSettings()

        controller.update_from_daw_status(daw_status, settings)

        # Should update presence
        mock_discord_service.update_presence.assert_called_once()
