import os
import sys
from pathlib import Path

from loguru import logger


class LoggingService:
    """Service for application logging using Loguru"""

    def __init__(self, config_dir: str, app_name: str = "DAWPresence"):
        self.config_dir = config_dir
        self.app_name = app_name
        self._setup_logging()

    def _setup_logging(self):
        """Setup Loguru logging configuration"""
        # Remove default handler
        logger.remove()

        # Create logs directory
        log_dir = Path(self.config_dir) / "logs"
        log_dir.mkdir(exist_ok=True)

        # Console handler (warnings and errors only)
        logger.add(
            sys.stderr,
            level="WARNING",
            format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
        )

        # File handler with rotation
        logger.add(
            log_dir / f"{self.app_name.lower()}_{{time:YYYY-MM-DD}}.log",
            level="TRACE",
            format="{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | {name}:{function}:{line} - {message}",
            rotation="1 day",
            retention="7 days",  # Keep logs for 1 week
            compression="zip",   # Compress old logs
        )

        logger.info(f"{self.app_name} logging initialized at: {log_dir}")


# Decorators
from functools import wraps
from time import time


def log_performance(func):
    """Decorator to log function execution time"""

    @wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time()
        try:
            result = func(*args, **kwargs)
            execution_time = (time() - start_time) * 1000  # Convert to ms
            logger.debug(f"{func.__name__} executed in {execution_time:.2f}ms")
            return result
        except Exception as e:
            execution_time = (time() - start_time) * 1000
            logger.error(f"{func.__name__} failed after {execution_time:.2f}ms: {e}")
            raise

    return wrapper


def log_errors(func):
    """Decorator to automatically log exceptions"""

    @wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            logger.exception(f"Error in {func.__name__}: {e}")
            raise

    return wrapper
