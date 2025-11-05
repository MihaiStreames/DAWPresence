from collections.abc import Callable
import time

from loguru import logger
from pypresence import Presence

from DAWPY.exceptions import DiscordConnectionError
from DAWPY.models import DiscordPresence


class DiscordService:
    """Service for managing Discord Rich Presence"""

    def __init__(self) -> None:
        self._client: Presence | None = None
        self._current_client_id: str | None = None
        self._is_connected = False
        self._start_time = int(time.time())

        # Callbacks
        self.on_connected: Callable[[], None] | None = None
        self.on_disconnected: Callable[[], None] | None = None
        self.on_error: Callable[[Exception], None] | None = None

    @property
    def is_connected(self) -> bool:
        """Check if Discord client is connected"""
        return self._is_connected and self._client is not None

    def connect(self, client_id: str) -> None:
        """Connect to Discord with specified client ID"""
        if self.is_connected and self._current_client_id == client_id:
            return
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
                raise error from e

    def disconnect(self) -> None:
        """Disconnect from Discord"""
        if not self.is_connected:
            return

        try:
            if self._client:
                self._client.clear()
                self._client.close()

        except Exception as e:
            logger.debug(f"Error during Discord cleanup: {e}")
            # Ignore errors during cleanup

        finally:
            self._cleanup_client()
            if self.on_disconnected:
                self.on_disconnected()

    def update_presence(self, presence: DiscordPresence) -> None:
        """Update Discord Rich Presence"""
        if not self.is_connected:
            raise DiscordConnectionError("Not connected to Discord")

        try:
            presence_data = presence.to_pypresence_dict()
            if not presence_data.get("start"):
                presence_data["start"] = self._start_time

            self._client.update(**presence_data)

        except Exception as e:
            error = DiscordConnectionError(f"Failed to update presence: {e}")
            if self.on_error:
                self.on_error(error)
            else:
                raise error from e

    def _cleanup_client(self) -> None:
        """Clean up client state"""
        self._client = None
        self._current_client_id = None
        self._is_connected = False

    def __del__(self) -> None:
        """Ensure cleanup on object destruction"""
        self.disconnect()
