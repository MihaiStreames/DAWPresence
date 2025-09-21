import time
from typing import Optional, Callable

from pypresence import Presence

from DAWPY.models import DiscordPresence


class DiscordConnectionError(Exception):
    """Custom exception for Discord connection issues"""
    pass


class DiscordService:
    """Service for managing Discord Rich Presence"""

    def __init__(self):
        self._client: Optional[Presence] = None
        self._current_client_id: Optional[str] = None
        self._is_connected = False
        self._start_time = int(time.time())

        # Callbacks
        self.on_connected: Optional[Callable] = None
        self.on_disconnected: Optional[Callable] = None
        self.on_error: Optional[Callable[[Exception], None]] = None

    @property
    def is_connected(self) -> bool:
        """Check if Discord client is connected"""
        return self._is_connected and self._client is not None

    @property
    def current_client_id(self) -> Optional[str]:
        """Get current Discord client ID"""
        return self._current_client_id

    def connect(self, client_id: str) -> None:
        """Connect to Discord with specified client ID"""
        if self.is_connected and self._current_client_id == client_id:
            return  # Already connected with same client ID

        # Disconnect existing client if different ID
        if self.is_connected and self._current_client_id != client_id:
            self.disconnect()

        try:
            self._client = Presence(client_id)
            self._client.connect()
            self._current_client_id = client_id
            self._is_connected = True
            self._start_time = int(time.time())

            if self.on_connected:
                self.on_connected()

        except Exception as e:
            self._cleanup_client()
            error = DiscordConnectionError(f"Failed to connect to Discord: {e}")
            if self.on_error:
                self.on_error(error)
            else:
                raise error

    def disconnect(self) -> None:
        """Disconnect from Discord"""
        if not self.is_connected:
            return

        try:
            if self._client:
                self._client.clear()
                self._client.close()
        except Exception:
            pass  # Ignore errors during cleanup
        finally:
            self._cleanup_client()
            if self.on_disconnected:
                self.on_disconnected()

    def update_presence(self, presence: DiscordPresence) -> None:
        """Update Discord Rich Presence"""
        if not self.is_connected:
            raise DiscordConnectionError("Not connected to Discord")

        try:
            # Use service start time if presence doesn't have one
            presence_data = presence.to_pypresence_dict()
            if not presence_data.get('start'):
                presence_data['start'] = self._start_time

            self._client.update(**presence_data)

        except Exception as e:
            error = DiscordConnectionError(f"Failed to update presence: {e}")
            if self.on_error:
                self.on_error(error)
            else:
                raise error

    def clear_presence(self) -> None:
        """Clear Discord Rich Presence"""
        if not self.is_connected:
            return

        try:
            self._client.clear()
        except Exception as e:
            if self.on_error:
                self.on_error(DiscordConnectionError(f"Failed to clear presence: {e}"))

    def _cleanup_client(self) -> None:
        """Clean up client state"""
        self._client = None
        self._current_client_id = None
        self._is_connected = False

    def __del__(self):
        """Ensure cleanup on object destruction"""
        self.disconnect()
