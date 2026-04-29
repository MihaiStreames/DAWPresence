# Changelog

All notable changes to DAWPresence.

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
- Dual renderer builds: tiny-skia (CPU, ~3.4 MB) and wgpu (GPU, ~4.6 MB)

### Changed

- Blue accent color scheme throughout the app
- Smaller binary (trimmed dependencies, switched async executor)
- Cleaner debug logging (app-only by default, no framework noise)

### Fixed

- Unnecessary Discord disconnects when no DAW running

## [2.2.0] - skipped

v2.2.0 was not released due to installer build issues. All changes are included in v3.0.0.

### Added

- Windows installer (Inno Setup) with start menu shortcut, desktop shortcut, auto-start option, and uninstaller
- Direct Win32 process monitoring via `CreateToolhelp32Snapshot` (replaces `sysinfo` crate)
- Event-driven process exit detection via `RegisterWaitForSingleObject` (NT kernel threadpool, zero CPU idle)
- Typed error handling via `thiserror` (replaces string errors)
- Compiled regex cache - patterns compiled once, reused every tick
- Pre-normalized DAW configs - process names lowercased and stripped at startup
- 28 unit tests covering config parsing, regex extraction, process matching, status formatting

### Changed

- Stable Rust toolchain (dropped nightly requirement)
- Split monolithic modules into focused files (20+ files, all under 250 lines)
- Discord IPC uses single mutex with poison recovery (was three separate mutexes)
- Icon decoding cached (was re-decoded on every state change)
- Tray and status icons are now circles with multi-size support for high-DPI displays
- Interval modal shrinks when no validation error is shown
- Click outside the interval modal to dismiss (warns if you have unsaved changes)
- Faster DAW discovery when scanning running processes
- Uninstaller removes settings and logs from `%APPDATA%`

### Fixed

- "Minimize to tray" defaulting to off when upgrading from older config files
- Process exit detection could race with cleanup in rare timing conditions
- RAM display logic had an unreachable code path
- Tray icon thread not cleaning up properly on shutdown
- Settings load errors were silently ignored (now logs a warning)

### Removed

- `sysinfo` dependency (replaced by direct Win32 APIs)
- Nightly Rust requirement (`generic_const_exprs`)
- Platform stubs (`unsupported.rs`)

## [2.1.0] - 2026-04-11

### Added

- `AdditionalProcessNames` config field for multi-process DAWs (prefix matching)
- Versioned `daws.json` format - auto-updates local config when a new version ships
- Window icon shows Discord connection state (red/green)

### Changed

- Extracted `app.rs` from `main.rs` for cleaner MVU separation
- Stricter clippy linting (`pub` to `pub(crate)`, unsafe blocks in unsafe fns)

### Fixed

- Bitwig Studio now aggregates CPU/RAM across all processes (main UI, audio engine, plugin hosts)

## [2.0.1] - 2026-01-01

### Changed

- Release builds strip logging and favor smaller binaries
- Process monitoring refreshes only the data we use

### Performance

- Binary size: ~36 MB -> ~17 MB -> ~4 MB
- RAM usage: ~70 MB+ -> ~7 MB

## [2.0.0] - 2026-01-01

### Fixed

- Project name detection now works on all Windows versions (fixes #2)
- Switched from `regex` to `fancy-regex` crate (the old crate doesn't support lookaheads)

## [1.0.2] - 2026-01-01

### Fixed

- Tray icon stays responsive by pumping Windows messages in the event loop
- Window title detection no longer requires `IsWindowEnabled` check
- Dynamic buffer size for window titles via `GetWindowTextLengthW`
- Skip windows with empty titles early

## [1.0.1] - 2026-01-01

Complete rewrite from Python to Rust. Same functionality, faster, smaller, no runtime dependencies.

### Added

- Single standalone `.exe` - no Python or dependencies needed
- Native Windows GUI using iced
- System tray with status indicator
- Persistent settings via `confy` (stored in `%APPDATA%`)
- Cross-compilation support from Linux

### Removed

- Python codebase, PyInstaller build system, all Python dependencies

## [1.0.0] - 2025-09-22

Initial Python release. A rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC).

### Added

- Discord Rich Presence for FL Studio, Ableton Live, REAPER, Bitwig Studio, Studio One, LMMS, and Cubase
- Automatic DAW detection via process monitoring
- Project name extraction from window titles using regex
- System tray icon with status indicator
- Configurable refresh interval
