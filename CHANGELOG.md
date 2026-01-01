<a id="changelog-top"></a>

<div align="center">
  <h1>Changelog</h1>

  <h3>All notable changes to DAWPresence</h3>

</div>

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#v200--first-functional-rust-release">v2.0.0</a></li>
    <li><a href="#v102--bugfixes">v1.0.2</a></li>
    <li><a href="#v101--rust-rewrite">v1.0.1</a></li>
    <li><a href="#v100--initial-release">v1.0.0</a></li>
  </ol>
</details>

## v2.0.0 — First functional Rust release

The Rust rewrite is now fully functional. All DAW project names are detected correctly on Windows 10 and 11.

**Fixed:**

- Project name detection now works on all Windows versions (fixes #2)
- Switched from `regex` to `fancy-regex` crate - the old crate doesn't support lookaheads, which some DAW patterns require

**Changed:**

- Added clippy lints matching Python linting style
- Refactored issue templates and workflows

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.2 — Bugfixes

Bug fixes for tray icon responsiveness and window title detection.

**Fixed:**

- Tray icon now stays responsive by pumping Windows messages in the event loop
- Window title detection no longer requires `IsWindowEnabled` check (was causing some DAW windows to be skipped)
- Fixed buffer size for window titles - now dynamically sized using `GetWindowTextLengthW` instead of fixed 512-char buffer
- Skip windows with empty titles early to avoid unnecessary processing

**New stuff:**

- GitHub Actions release workflow for automated builds
- GitHub issue templates for bug reports and feature requests
- GitHub funding configuration

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.1 — Rust rewrite

Complete rewrite from Python to Rust. Same functionality, but faster, smaller, and no runtime dependencies.

**New stuff:**

- Single standalone `.exe` - no Python or dependencies needed
- Native Windows GUI using [iced](https://iced.rs/)
- System tray with status indicator (green = connected, red = disconnected)
- Desktop notifications via `notify-rust`
- Persistent settings via `confy` (stored in `%APPDATA%`)
- File logging via `tracing-appender`
- Cross-compilation support from Linux using `x86_64-pc-windows-gnu`

**Changed:**

- Replaced PyQt5 with iced for the GUI
- Replaced pypresence with `discord-rich-presence` crate
- Replaced psutil with `sysinfo` crate
- Process monitoring now uses Win32 APIs directly for window titles and version info
- Configuration moved from `config.toml` to platform-specific app data directory

**Removed:**

- Python codebase (`DAWPY/` directory)
- PyInstaller build system
- All Python dependencies

**Technical:**

- ~38MB standalone binary (includes all assets)
- Async runtime via tokio
- Proper shutdown handling with cleanup

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.0 — Initial release

Python version. A rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC) with cleaner architecture.

**Features:**

- Discord Rich Presence for FL Studio, Ableton Live, REAPER, Bitwig Studio, Studio One, LMMS, and Cubase
- Automatic DAW detection via process monitoring
- Project name extraction from window titles using regex
- System tray icon with status indicator
- Configurable refresh interval and start minimized option
- MVC architecture with PyQt5 GUI

**Technical:**

- Python 3.7+ required
- Dependencies: PyQt5, pypresence, psutil
- Windows only

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

---

<div align="center">
  <p>Back to <a href="README.md">README</a>?</p>
</div>
