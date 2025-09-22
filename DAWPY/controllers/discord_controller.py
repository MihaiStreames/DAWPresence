from collections.abc import Callable

from loguru import logger

from DAWPY.exceptions import DiscordConnectionError
from DAWPY.models import AppSettings, DAWStatus, DiscordPresence
from DAWPY.services import DiscordService
from DAWPY.services.logging_service import log_errors


class DiscordController:
    """Controller for Discord Rich Presence management"""

    def __init__(self, discord_service: DiscordService, app_version: str):
        self.discord_service = discord_service
        self.app_version = app_version
        self._current_client_id: str | None = None

        # Setup service callbacks
        self.discord_service.on_connected = self._on_connected
        self.discord_service.on_disconnected = self._on_disconnected
        self.discord_service.on_error = self._on_error

        # Controller callbacks
        self.on_connected: Callable | None = None
        self.on_disconnected: Callable | None = None
        self.on_error: Callable[[Exception], None] | None = None

        logger.info("Discord Controller initialized")

    @property
    def is_connected(self) -> bool:
        """Check if Discord is connected"""
        return self.discord_service.is_connected

    @log_errors
    def update_from_daw_status(self, daw_status: DAWStatus, settings: AppSettings):
        """Update Discord presence based on DAW status"""
        if not daw_status.is_running:
            self.disconnect()
            return

        # Ensure we're connected with correct client ID
        if not self._ensure_connection(daw_status.daw_info.client_id):
            logger.warning("Failed to establish Discord connection")
            return

        # Create and update presence
        presence = DiscordPresence.create_for_daw(daw_status, settings, self.app_version)

        try:
            self.discord_service.update_presence(presence)
            logger.debug(f"Discord presence updated: {presence.details}")
        except DiscordConnectionError as e:
            logger.error(f"Failed to update Discord presence: {e}")
            if self.on_error:
                self.on_error(e)

    def disconnect(self):
        """Disconnect from Discord"""
        self.discord_service.disconnect()
        self._current_client_id = None

    def _ensure_connection(self, client_id: str) -> bool:
        """Ensure connection with specific client ID"""
        if self.is_connected and self._current_client_id == client_id:
            return True

        try:
            self.discord_service.connect(client_id)
            self._current_client_id = client_id
            logger.success(f"Connected to Discord (Client ID: {client_id})")
            return True
        except DiscordConnectionError as e:
            logger.warning(f"Discord connection failed: {e}")
            return False

    def _on_connected(self):
        """Handle Discord connection"""
        logger.info("Discord RPC connected")
        if self.on_connected:
            self.on_connected()

    def _on_disconnected(self):
        """Handle Discord disconnection"""
        logger.info("Discord RPC disconnected")
        self._current_client_id = None
        if self.on_disconnected:
            self.on_disconnected()

    def _on_error(self, error: Exception):
        """Handle Discord errors"""
        if self.on_error:
            self.on_error(error)
