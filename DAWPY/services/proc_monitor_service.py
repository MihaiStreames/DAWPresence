import ctypes
from dataclasses import dataclass

import psutil
import win32gui
import win32process
from loguru import logger

from DAWPY.services.logging_service import log_errors, log_performance
from DAWPY.utils import ProcessUtils, SystemUtils


@dataclass
class ProcessInfo:
    """Process information data"""

    pid: int
    name: str
    exe_path: str
    cpu_percent: float
    memory_mb: int
    window_title: str = ""


class ProcessMonitorService:
    """Service for monitoring system processes"""

    def __init__(self) -> None:
        self._processes_cache = {}
        logger.info("Process Monitor Service initialized")

    @log_performance
    def get_process_by_name(self, process_name: str) -> ProcessInfo | None:
        """Find process by name (case-insensitive)"""
        target_name = ProcessUtils.normalize_process_name(process_name)

        try:
            for proc in psutil.process_iter(["pid", "name"]):
                try:
                    proc_name = ProcessUtils.normalize_process_name(proc.info["name"])
                    if proc_name == target_name:
                        process_info = self._create_process_info(proc)
                        if process_info:
                            logger.trace(f"Found process: {process_name} (PID: {process_info.pid})")
                            return process_info

                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue

        except Exception as e:
            logger.error(f"Error scanning for process {process_name}: {e}")

        return None

    @log_errors
    def _create_process_info(self, proc: psutil.Process) -> ProcessInfo | None:
        """Create ProcessInfo from psutil.Process"""
        try:
            # Get basic info
            proc_info = proc.as_dict(["pid", "name", "exe"])

            # Get CPU and memory usage
            cpu_percent = proc.cpu_percent(interval=0.1)
            memory_info = proc.memory_info()

            # Normalize CPU usage by core count and convert memory to MB
            normalized_cpu = cpu_percent / SystemUtils.get_cpu_count()
            memory_mb = SystemUtils.bytes_to_mb(memory_info.rss)

            # Get window title
            window_title = self._get_window_title(proc.info["pid"])

            logger.trace(
                f"Process info created for {proc_info['name']}: {memory_mb}MB RAM, {normalized_cpu:.1f}% CPU",
            )

            return ProcessInfo(
                pid=proc_info["pid"],
                name=proc_info["name"],
                exe_path=proc_info["exe"] or "",
                cpu_percent=normalized_cpu,
                memory_mb=memory_mb,
                window_title=window_title,
            )

        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess) as e:
            logger.debug(f"Cannot access process info: {e}")
            return None

    @staticmethod
    def _get_window_title(pid: int) -> str:
        """Get window title for a process (Windows only)"""

        def enum_window_callback(hwnd, windows):
            if win32gui.IsWindowVisible(hwnd) and win32gui.IsWindowEnabled(hwnd):
                _, found_pid = win32process.GetWindowThreadProcessId(hwnd)
                if found_pid == pid:
                    title = win32gui.GetWindowText(hwnd)
                    if title.strip():
                        windows.append(title)
            return True

        windows = []

        try:
            win32gui.EnumWindows(enum_window_callback, windows)
            return windows[0] if windows else ""

        except (OSError, AttributeError) as e:
            logger.debug(f"Failed to enumerate windows: {e}")
            return ""

    @staticmethod
    def get_process_version(exe_path: str) -> str:
        """Get version information from executable (Windows only)"""
        if not exe_path:
            return "0.0.0"

        try:
            size = ctypes.windll.version.GetFileVersionInfoSizeW(exe_path, None)
            if not size:
                return "0.0.0"

            res = ctypes.create_string_buffer(size)
            ctypes.windll.version.GetFileVersionInfoW(exe_path, 0, size, res)

            lptr = ctypes.c_void_p()
            lsize = ctypes.c_uint()
            ctypes.windll.version.VerQueryValueW(
                res,
                r"\VarFileInfo\Translation",
                ctypes.byref(lptr),
                ctypes.byref(lsize),
            )

            if lsize.value == 0:
                return "0.0.0"

            lang, codepage = ctypes.cast(lptr, ctypes.POINTER(ctypes.c_ushort * 2)).contents
            str_path = f"\\StringFileInfo\\{lang:04X}{codepage:04X}\\ProductVersion"

            ctypes.windll.version.VerQueryValueW(
                res,
                str_path,
                ctypes.byref(lptr),
                ctypes.byref(lsize),
            )

            if lsize.value == 0:
                return "0.0.0"

            version = ctypes.wstring_at(lptr, lsize.value).rstrip("\x00")
            return version or "0.0.0"

        except Exception:
            return "0.0.0"
