# Changelog

All notable changes to DAWPresence.

## [3.0.4] - 2026-06-11

### Fixed

- F.Lux (`flux.exe`) no longer falsely detected as FL Studio (#46)
- Project name now extracted from the DAW's main window rather than whichever window has the longest title
- Also tackled issues #41, #42 and #43 (check each for more details)

### Removed

- Per-project timer mode toggle _(the session timer remains)_; proper per-project tracking with persistence is tracked in #48

## [3.0.3] - 2026-05-28

No user-facing changes. Internal code quality pass, closes #38.

## [3.0.2] - 2026-05-24

### Added

- Launching a second instance now brings the existing window to the front instead of opening a duplicate
- Autostart now launches minimized to tray instead of opening the window on login

### Changed

- Binary compressed with UPX: ~3.5 MB -> ~1.5 MB

## [3.0.1] - 2026-04-12

### Fixed

- Removed broken GPU renderer, single build only
- Installer now offers to launch app after install

## [3.0.0] - 2026-04-12

### Added

- Sidebar with Home and Settings pages
- Settings page with togglers and inline interval editor
- Auto-start with Windows toggle
- Timer mode: session time (default) or project time (resets on project change)
- App icon embedded in exe and shown in taskbar/titlebar

### Changed

- Blue accent color scheme throughout the app
- Smaller binary

### Fixed

- Unnecessary Discord disconnects when no DAW running

## [2.2.0] - skipped

v2.2.0 was not released due to installer build issues. All changes are included in v3.0.0.

### Added

- Windows installer with start menu shortcut, desktop shortcut, auto-start option, and uninstaller

### Changed

- Tray and status icons are now circles with high-DPI support
- Interval editor shrinks when no validation error is shown
- Click outside the interval editor to dismiss
- Faster DAW discovery on startup
- Uninstaller removes settings from `%APPDATA%`

### Fixed

- "Minimize to tray" defaulting to off when upgrading from older config files
- Process exit could race with cleanup in rare timing conditions

## [2.1.0] - 2026-04-11

### Added

- `AdditionalProcessNames` config field for multi-process DAWs
- Versioned `daws.json` -- local config auto-updates when a new version ships
- Window icon reflects Discord connection state (red/green)

### Fixed

- Bitwig Studio now aggregates CPU/RAM across all its processes

## [2.0.1] - 2026-01-01

### Changed

- Smaller binary and lower memory usage
- Binary size: ~36 MB -> ~4 MB
- RAM usage: ~70 MB -> ~7 MB

## [2.0.0] - 2026-01-01

### Fixed

- Project name detection now works on all Windows versions (fixes #2)

## [1.0.2] - 2026-01-01

### Fixed

- Tray icon stays responsive while the app is running

## [1.0.1] - 2026-01-01

Complete rewrite from Python to Rust. Same functionality, faster, smaller, no runtime dependencies.

### Added

- Single standalone `.exe` -- no Python or dependencies needed
- Native Windows GUI using iced
- System tray with status indicator
- Persistent settings stored in `%APPDATA%`

### Removed

- Python codebase and all Python dependencies

## [1.0.0] - 2025-09-22

Initial Python release. A rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC).

### Added

- Discord Rich Presence for FL Studio, Ableton Live, REAPER, Bitwig Studio, Studio One, LMMS, and Cubase
- Automatic DAW detection via process monitoring
- Project name extraction from window titles using regex
- System tray icon with status indicator
- Configurable refresh interval
