import sys
from collections.abc import Callable
from functools import wraps
from pathlib import Path
from time import time
from typing import Any, TypeVar

from loguru import logger

# Type variable for decorators
F = TypeVar("F", bound=Callable[..., Any])


class LoggingService:
    """Service for application logging using Loguru"""

    def __init__(self, data_dir: str, app_name: str = "DAWPresence") -> None:
        self.data_dir = data_dir
        self.app_name = app_name
        self._setup_logging()

    def _setup_logging(self) -> None:
        """Set up Loguru logging configuration"""
        # Remove default handler
        logger.remove()

        # Create logs directory
        log_dir = Path(self.data_dir) / "logs"
        log_dir.mkdir(parents=True, exist_ok=True)

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
            compression="zip",  # Compress old logs
        )

        logger.info(f"{self.app_name} logging initialized")


def log_performance[F: Callable[..., Any]](func: F) -> F:
    """Log function execution time"""

    @wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
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


def log_errors[F: Callable[..., Any]](func: F) -> F:
    """Log exceptions automatically"""

    @wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        try:
            return func(*args, **kwargs)

        except Exception as e:
            logger.exception(f"Error in {func.__name__}: {e}")
            raise

    return wrapper
