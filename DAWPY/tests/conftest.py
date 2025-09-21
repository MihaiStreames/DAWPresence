import json
import os
import tempfile
from unittest.mock import Mock

import pytest

from DAWPY.models import DAWInfo
from DAWPY.services.process_monitor import ProcessInfo


@pytest.fixture
def sample_daw_info():
    """Sample DAW configuration for testing"""
    return DAWInfo(
        process_name="FL64",
        display_text="FL Studio",
        title_regex=r"^(.*?)(?= - FL Studio)",
        client_id="908331713032241153",
        hide_version=True
    )


@pytest.fixture
def sample_process_info():
    """Sample process information for testing"""
    return ProcessInfo(
        pid=1234,
        name="FL64.exe",
        exe_path="C:/Program Files/FL Studio/FL64.exe",
        cpu_percent=15.5,
        memory_mb=512,
        window_title="My Cool Beat - FL Studio"
    )


@pytest.fixture
def temp_config_file():
    """Temporary configuration file for testing"""
    config_data = [
        {
            "ProcessName": "FL64",
            "DisplayText": "FL Studio",
            "TitleRegex": "^(.*?)(?= - FL Studio)",
            "ClientID": "908331713032241153",
            "HideVersion": True
        },
        {
            "ProcessName": "reaper",
            "DisplayText": "REAPER",
            "TitleRegex": "^(.*?)(?= - REAPER v)",
            "ClientID": "909424276208255067",
            "HideVersion": False
        }
    ]

    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(config_data, f)
        temp_path = f.name

    yield temp_path

    # Cleanup
    os.unlink(temp_path)


@pytest.fixture
def mock_discord_client():
    """Mock Discord client for testing"""
    mock_client = Mock()
    mock_client.connect.return_value = None
    mock_client.update.return_value = None
    mock_client.clear.return_value = None
    mock_client.close.return_value = None
    return mock_client
