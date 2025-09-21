import os
import tempfile
from unittest.mock import Mock, patch

import pytest

from DAWPY.models import DiscordPresence
from DAWPY.services import ConfigurationService, DiscordService
from DAWPY.services.discord_service import DiscordConnectionError


class TestConfigurationService:
    """Test configuration service"""

    def test_load_daw_configurations(self, temp_config_file):
        """Test loading DAW configurations from file"""
        service = ConfigurationService(temp_config_file)
        daws = service.load_daw_configurations()

        assert len(daws) == 2
        assert daws[0].process_name == "FL64"
        assert daws[0].display_text == "FL Studio"
        assert daws[1].process_name == "reaper"
        assert daws[1].display_text == "REAPER"

    def test_get_daw_by_process_name(self, temp_config_file):
        """Test finding DAW by process name"""
        service = ConfigurationService(temp_config_file)

        # Test exact match
        daw = service.get_daw_by_process_name("FL64")
        assert daw is not None
        assert daw.display_text == "FL Studio"

        # Test case insensitive
        daw = service.get_daw_by_process_name("fl64")
        assert daw is not None

        # Test not found
        daw = service.get_daw_by_process_name("nonexistent")
        assert daw is None

    def test_missing_config_file(self):
        """Test handling missing config file"""
        service = ConfigurationService("/nonexistent/path.json")

        with pytest.raises(FileNotFoundError):
            service.load_daw_configurations()

    def test_invalid_json_file(self):
        """Test handling invalid JSON"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write("invalid json content {")
            temp_path = f.name

        try:
            service = ConfigurationService(temp_path)
            with pytest.raises(ValueError, match="Invalid JSON"):
                service.load_daw_configurations()
        finally:
            os.unlink(temp_path)


class TestDiscordService:
    """Test Discord service"""

    def test_initial_state(self):
        """Test initial service state"""
        service = DiscordService()

        assert not service.is_connected
        assert service.current_client_id is None

    @patch('services.discord_service.Presence')
    def test_successful_connection(self, mock_presence_class):
        """Test successful Discord connection"""
        # Setup mock
        mock_client = Mock()
        mock_presence_class.return_value = mock_client

        # Test connection
        service = DiscordService()
        service.connect("123456789")

        # Verify
        assert service.is_connected
        assert service.current_client_id == "123456789"
        mock_presence_class.assert_called_once_with("123456789")
        mock_client.connect.assert_called_once()

    @patch('services.discord_service.Presence')
    def test_connection_failure(self, mock_presence_class):
        """Test Discord connection failure"""
        # Setup mock to raise exception
        mock_client = Mock()
        mock_client.connect.side_effect = Exception("Connection failed")
        mock_presence_class.return_value = mock_client

        # Test connection failure
        service = DiscordService()

        with pytest.raises(DiscordConnectionError, match="Failed to connect to Discord"):
            service.connect("123456789")

        assert not service.is_connected
        assert service.current_client_id is None

    @patch('services.discord_service.Presence')
    def test_update_presence(self, mock_presence_class):
        """Test updating Discord presence"""
        # Setup
        mock_client = Mock()
        mock_presence_class.return_value = mock_client

        service = DiscordService()
        service.connect("123456789")

        # Create presence and update
        presence = DiscordPresence(details="Test", state="Testing")
        service.update_presence(presence)

        # Verify update was called
        mock_client.update.assert_called_once()
        call_args = mock_client.update.call_args[1]  # Get keyword arguments
        assert call_args['details'] == "Test"
        assert call_args['state'] == "Testing"

    def test_update_presence_not_connected(self):
        """Test updating presence when not connected"""
        service = DiscordService()
        presence = DiscordPresence()

        with pytest.raises(DiscordConnectionError, match="Not connected to Discord"):
            service.update_presence(presence)

    @patch('services.discord_service.Presence')
    def test_disconnect(self, mock_presence_class):
        """Test Discord disconnection"""
        # Setup connected service
        mock_client = Mock()
        mock_presence_class.return_value = mock_client

        service = DiscordService()
        service.connect("123456789")

        # Disconnect
        service.disconnect()

        # Verify
        assert not service.is_connected
        assert service.current_client_id is None
        mock_client.clear.assert_called_once()
        mock_client.close.assert_called_once()
