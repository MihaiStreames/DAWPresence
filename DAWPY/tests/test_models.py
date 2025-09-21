import pytest

from DAWPY.models import DAWInfo, DAWStatus, AppSettings, DiscordPresence


class TestDAWInfo:
    """Test DAWInfo model"""

    def test_valid_daw_info_creation(self):
        """Test creating valid DAW info"""
        daw = DAWInfo(
            process_name="FL64",
            display_text="FL Studio",
            title_regex=r"^(.*?)(?= - FL Studio)",
            client_id="123456789"
        )

        assert daw.process_name == "FL64"
        assert daw.display_text == "FL Studio"
        assert daw.client_id == "123456789"
        assert daw.hide_version is False  # Default value

    def test_empty_process_name_raises_error(self):
        """Test that empty process name raises ValueError"""
        with pytest.raises(ValueError, match="Process name cannot be empty"):
            DAWInfo(
                process_name="",
                display_text="FL Studio",
                title_regex=r"test",
                client_id="123"
            )

    def test_invalid_regex_raises_error(self):
        """Test that invalid regex raises ValueError"""
        with pytest.raises(ValueError, match="Invalid title regex"):
            DAWInfo(
                process_name="FL64",
                display_text="FL Studio",
                title_regex="[invalid regex",  # Missing closing bracket
                client_id="123"
            )


class TestDAWStatus:
    """Test DAWStatus model"""

    def test_default_daw_status(self):
        """Test default DAW status"""
        status = DAWStatus()

        assert not status.is_running
        assert status.cpu_usage == 0.0
        assert status.ram_usage_mb == 0
        assert status.project_name == "None"
        assert status.display_name == "None"

    def test_cpu_usage_formatting(self):
        """Test CPU usage string formatting"""
        status = DAWStatus(is_running=True, cpu_usage=15.555)
        assert status.cpu_usage_str == "15.56%"

        status_not_running = DAWStatus(is_running=False)
        assert status_not_running.cpu_usage_str == "Undefined"

    def test_ram_usage_formatting(self):
        """Test RAM usage string formatting"""
        status = DAWStatus(is_running=True, ram_usage_mb=512)
        assert status.ram_usage_str == "512MB"

        status_not_running = DAWStatus(is_running=False)
        assert status_not_running.ram_usage_str == "Undefined"

    def test_project_name_extraction(self, sample_daw_info):
        """Test project name extraction from window title"""
        status = DAWStatus(
            daw_info=sample_daw_info,
            window_title="My Cool Beat - FL Studio"
        )

        project = status.extract_project_name()
        assert project == "My Cool Beat"

    def test_project_name_extraction_no_match(self, sample_daw_info):
        """Test project name extraction when regex doesn't match"""
        status = DAWStatus(
            daw_info=sample_daw_info,
            window_title="Some other window title"
        )

        project = status.extract_project_name()
        assert project == "None"


class TestAppSettings:
    """Test AppSettings model"""

    def test_default_settings(self):
        """Test default settings values"""
        settings = AppSettings()

        assert not settings.hide_project_name
        assert not settings.hide_system_usage
        assert settings.update_interval == 2500

    def test_invalid_update_interval_too_low(self):
        """Test that low update interval raises error"""
        with pytest.raises(ValueError, match="Update interval must be at least 1000ms"):
            AppSettings(update_interval=500)

    def test_invalid_update_interval_too_high(self):
        """Test that high update interval raises error"""
        with pytest.raises(ValueError, match="Update interval too large"):
            AppSettings(update_interval=200_000_000)

    def test_settings_update(self):
        """Test settings update method"""
        settings = AppSettings()
        new_settings = settings.update(hide_project_name=True, update_interval=5000)

        # Original unchanged
        assert not settings.hide_project_name
        assert settings.update_interval == 2500

        # New instance updated
        assert new_settings.hide_project_name
        assert new_settings.update_interval == 5000


class TestDiscordPresence:
    """Test DiscordPresence model"""

    def test_default_discord_presence(self):
        """Test default Discord presence"""
        presence = DiscordPresence()

        assert presence.details == ""
        assert presence.state == ""
        assert presence.large_image == "icon"
        assert presence.start_time is not None

    def test_create_for_idle_daw(self):
        """Test creating presence for idle state"""
        daw_status = DAWStatus(is_running=False)
        settings = AppSettings()

        presence = DiscordPresence.create_for_daw(daw_status, settings, "2.0")

        assert presence.details == "Not using any DAW"
        assert presence.state == "Idle"
        assert "DAWRPC v2.0" in presence.large_text

    def test_create_for_running_daw(self, sample_daw_info):
        """Test creating presence for running DAW"""
        daw_status = DAWStatus(
            daw_info=sample_daw_info,
            is_running=True,
            project_name="My Cool Beat",
            cpu_usage=15.5,
            ram_usage_mb=512,
            version="21.2.0"
        )
        settings = AppSettings()

        presence = DiscordPresence.create_for_daw(daw_status, settings, "2.0")

        assert "Opening project: My Cool Beat" in presence.details
        assert "15.50%" in presence.state
        assert "512MB" in presence.state

    def test_to_pypresence_dict(self):
        """Test conversion to pypresence format"""
        presence = DiscordPresence(
            details="Test details",
            state="Test state",
            start_time=1234567890
        )

        result = presence.to_pypresence_dict()

        expected = {
            'details': 'Test details',
            'state': 'Test state',
            'large_image': 'icon',
            'large_text': '',
            'start': 1234567890
        }

        assert result == expected
