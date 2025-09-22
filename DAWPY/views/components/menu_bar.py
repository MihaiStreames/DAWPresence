from PyQt5.QtCore import QObject, pyqtSignal
from PyQt5.QtWidgets import QAction, QMenuBar

from DAWPY.models import AppSettings


class AppMenuBar(QObject):
    """Application menu bar manager"""

    # Signals
    toggle_project_name_requested = pyqtSignal()
    toggle_system_usage_requested = pyqtSignal()
    update_interval_requested = pyqtSignal()

    def __init__(self, menubar: QMenuBar) -> None:
        super().__init__()
        self.menubar = menubar
        self.hide_project_action: QAction | None = None
        self.hide_system_action: QAction | None = None
        self._create_menu()

    def _create_menu(self) -> None:
        """Create menu actions"""
        # Hide Project Name action
        self.hide_project_action = QAction("[OFF] Hide Project Name", self)
        self.hide_project_action.triggered.connect(self.toggle_project_name_requested.emit)
        self.menubar.addAction(self.hide_project_action)

        # Hide System Usage action
        self.hide_system_action = QAction("[OFF] Hide System Usage", self)
        self.hide_system_action.triggered.connect(self.toggle_system_usage_requested.emit)
        self.menubar.addAction(self.hide_system_action)

        # Set Update Interval action
        update_interval_action = QAction("Set Update Interval", self)
        update_interval_action.triggered.connect(self.update_interval_requested.emit)
        self.menubar.addAction(update_interval_action)

    def update_settings_display(self, settings: AppSettings) -> None:
        """Update menu items to reflect current settings"""
        project_text = f"[{'ON' if settings.hide_project_name else 'OFF'}] Hide Project Name"
        system_text = f"[{'ON' if settings.hide_system_usage else 'OFF'}] Hide System Usage"

        if self.hide_project_action:
            self.hide_project_action.setText(project_text)
        if self.hide_system_action:
            self.hide_system_action.setText(system_text)
