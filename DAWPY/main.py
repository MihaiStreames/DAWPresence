import os
import sys
import traceback

from PyQt5.QtWidgets import QApplication

# Add project root to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from DAWPY.controllers.app_controller import AppController
from DAWPY.views.main_window import MainWindow


def main():
    """Main application entry point"""
    # Application info
    APP_VERSION = "1.0"
    APP_NAME = "DAWPresence"

    # Create Qt application
    app = QApplication(sys.argv)
    app.setApplicationName(APP_NAME)
    app.setApplicationVersion(APP_VERSION)
    app.setQuitOnLastWindowClosed(False)  # Keep running in tray

    try:
        # Create main controller
        controller = AppController(APP_VERSION)

        # Initialize application
        if not controller.initialize():
            return 1

        # Create main window
        main_window = MainWindow(APP_VERSION)

        # Start application
        controller.start(app, main_window)

        # Setup cleanup on exit
        app.aboutToQuit.connect(controller.shutdown)

        # Run application
        return app.exec_()

    except Exception as e:
        print(f"Fatal error: {e}")
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
