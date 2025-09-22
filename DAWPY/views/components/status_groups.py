from PyQt5.QtCore import Qt
from PyQt5.QtGui import QFont
from PyQt5.QtWidgets import QGridLayout, QGroupBox, QLabel, QVBoxLayout, QWidget

from DAWPY.models import DAWStatus


class StatusGroupWidget(QGroupBox):
    """Base class for status display groups"""

    def __init__(self, title: str, font_name: str = "Tahoma", font_size: int = 12):
        super().__init__(title)
        self.layout = QVBoxLayout(self)
        self.label = QLabel("None")
        self.label.setAlignment(Qt.AlignCenter)
        self.label.setFont(QFont(font_name, font_size))
        self.layout.addWidget(self.label)

    def update_text(self, text: str):
        """Update the display text"""
        self.label.setText(text)


class DAWNameGroup(StatusGroupWidget):
    """Digital Audio Workstation name display group"""

    def __init__(self):
        super().__init__("Current Digital Audio Workstation")

    def update_daw(self, daw_status: DAWStatus):
        """Update DAW name display"""
        self.update_text(daw_status.display_name)


class ProjectNameGroup(StatusGroupWidget):
    """Project name display group"""

    def __init__(self):
        super().__init__("Opening Project")

    def update_project(self, daw_status: DAWStatus):
        """Update project name display"""
        self.update_text(daw_status.project_name)


class SystemUsageGroup(StatusGroupWidget):
    """Base class for system usage groups"""

    def __init__(self, title: str):
        super().__init__(title, font_name="Consolas", font_size=20)


class CPUUsageGroup(SystemUsageGroup):
    """CPU usage display group"""

    def __init__(self):
        super().__init__("CPU Usage")

    def update_cpu(self, daw_status: DAWStatus):
        """Update CPU usage display"""
        self.update_text(daw_status.cpu_usage_str)


class RAMUsageGroup(SystemUsageGroup):
    """RAM usage display group"""

    def __init__(self):
        super().__init__("RAM Usage")

    def update_ram(self, daw_status: DAWStatus):
        """Update RAM usage display"""
        self.update_text(daw_status.ram_usage_str)


class StatusGroupsWidget(QWidget):
    """Container widget for all status display groups"""

    def __init__(self):
        super().__init__()
        self._init_ui()

    def _init_ui(self):
        """Initialize the status groups layout"""
        self.grid_layout = QGridLayout(self)
        self.grid_layout.setContentsMargins(10, 10, 10, 10)
        self.grid_layout.setSpacing(10)

        # Create status groups
        self.daw_group = DAWNameGroup()
        self.project_group = ProjectNameGroup()
        self.cpu_group = CPUUsageGroup()
        self.ram_group = RAMUsageGroup()

        # Add to grid
        self.grid_layout.addWidget(self.daw_group, 0, 0)
        self.grid_layout.addWidget(self.project_group, 0, 1)
        self.grid_layout.addWidget(self.cpu_group, 1, 0)
        self.grid_layout.addWidget(self.ram_group, 1, 1)

        # Set stretch properties
        self.grid_layout.setColumnStretch(0, 1)
        self.grid_layout.setColumnStretch(1, 1)
        self.grid_layout.setRowStretch(0, 1)
        self.grid_layout.setRowStretch(1, 1)

    def update_daw_display(self, daw_status: DAWStatus):
        """Update all status displays"""
        self.daw_group.update_daw(daw_status)
        self.project_group.update_project(daw_status)
        self.cpu_group.update_cpu(daw_status)
        self.ram_group.update_ram(daw_status)
