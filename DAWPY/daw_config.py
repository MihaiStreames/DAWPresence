import ctypes
import re

import psutil
import win32gui
import win32process


class DAWConfig:
    def __init__(self, ProcessName, DisplayText, TitleRegex, ClientID, HideVersion=False):
        self.process_name = ProcessName
        self.display_text = DisplayText
        self.title_regex = TitleRegex
        self.client_id = ClientID
        self.hide_version = HideVersion

        # Runtime variables
        self.is_running = False
        self.cpu_usage = "Undefined"
        self.ram_usage = "Undefined"
        self.project_name = "None"
        self.version = "0.0.0"

        # For CPU calculation
        self.last_time = None
        self.last_cpu_percent = None

    def get_process(self):
        """Find and return the DAW process"""
        for proc in psutil.process_iter(['pid', 'name']):
            try:
                # Check if process name matches (without .exe)
                proc_name = proc.info['name']
                if proc_name.lower().replace('.exe', '') == self.process_name.lower():
                    return proc
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        return None

    def get_window_title(self, process):
        """Get the window title of the given process (Windows only)"""

        def enum_window_callback(hwnd, windows):
            if win32gui.IsWindowVisible(hwnd) and win32gui.IsWindowEnabled(hwnd):
                _, found_pid = win32process.GetWindowThreadProcessId(hwnd)
                if found_pid == process.pid:
                    title = win32gui.GetWindowText(hwnd)
                    if title:
                        windows.append(title)
            return True

        windows = []
        win32gui.EnumWindows(enum_window_callback, windows)
        return windows[0] if windows else ""

    def get_cpu_usage(self, process):
        """Calculate CPU usage for the process"""
        try:
            # Use psutil's cpu_percent method
            cpu_percent = process.cpu_percent(interval=0.1)
            if cpu_percent is not None:
                # Divide by number of CPU cores to get percentage per core
                import os
                cpu_count = os.cpu_count() or 1
                normalized_cpu = cpu_percent / cpu_count
                return f"{normalized_cpu:.2f}"
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
        return "Undefined"

    def get_ram_usage(self, process):
        """Get RAM usage for the process"""
        try:
            # Get memory info in bytes
            mem_info = process.memory_info()
            # Convert to MB
            mem_mb = mem_info.rss / (1024 * 1024)
            return f"{int(mem_mb)}MB"
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
        return "Undefined"

    def get_version(self, process):
        """Get the version of the DAW"""
        exe_path = process.exe()  # get the EXE path from the process
        size = ctypes.windll.version.GetFileVersionInfoSizeW(exe_path, None)
        if not size:
            return ""

        res = ctypes.create_string_buffer(size)
        ctypes.windll.version.GetFileVersionInfoW(exe_path, 0, size, res)

        lptr = ctypes.c_void_p()
        lsize = ctypes.c_uint()
        ctypes.windll.version.VerQueryValueW(res, r"\VarFileInfo\Translation", ctypes.byref(lptr), ctypes.byref(lsize))
        if lsize.value == 0:
            return ""

        lang, codepage = ctypes.cast(lptr, ctypes.POINTER(ctypes.c_ushort * 2)).contents
        str_path = f"\\StringFileInfo\\{lang:04X}{codepage:04X}\\ProductVersion"

        ctypes.windll.version.VerQueryValueW(res, str_path, ctypes.byref(lptr), ctypes.byref(lsize))
        if lsize.value == 0:
            return ""

        return ctypes.wstring_at(lptr, lsize.value).rstrip('\x00')

    def update(self, settings):
        """Update DAW status and information"""
        process = self.get_process()

        if process:
            try:
                # Get window title and extract project name
                if not settings.hide_project_name:
                    title = self.get_window_title(process)
                    if title:
                        match = re.search(self.title_regex, title)
                        self.project_name = match.group(1) if match else "None"
                    else:
                        self.project_name = "None"
                else:
                    self.project_name = "(hidden)"

                # Get system usage
                if not settings.hide_system_usage:
                    self.cpu_usage = self.get_cpu_usage(process) + "%"
                    self.ram_usage = self.get_ram_usage(process)
                else:
                    self.cpu_usage = "Disabled"
                    self.ram_usage = "Disabled"

                # Get version
                self.version = self.get_version(process)
                self.is_running = True

            except (psutil.NoSuchProcess, psutil.AccessDenied):
                self.is_running = False
                self.reset_values()
        else:
            self.is_running = False
            self.reset_values()

    def reset_values(self):
        """Reset all values to defaults"""
        self.cpu_usage = "Undefined"
        self.ram_usage = "Undefined"
        self.project_name = "None"
