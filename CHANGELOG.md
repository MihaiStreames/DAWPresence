<a id="changelog-top"></a>

<div align="center">
  <h1>Changelog</h1>

  <h3>All notable changes to DAWPresence</h3>

</div>

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#v220--architecture-rewrite">v2.2.0</a></li>
    <li><a href="#v210--multi-process-daws">v2.1.0</a></li>
    <li><a href="#v201--size-and-memory-optimizations">v2.0.1</a></li>
    <li><a href="#v200--first-functional-rust-release">v2.0.0</a></li>
    <li><a href="#v102--bugfixes">v1.0.2</a></li>
    <li><a href="#v101--rust-rewrite">v1.0.1</a></li>
    <li><a href="#v100--initial-release">v1.0.0</a></li>
  </ol>
</details>

## v2.2.0 - Architecture rewrite

Complete architecture rewrite with direct Win32 APIs, event-driven process monitoring, and a Windows installer.

**New stuff:**

- Windows installer (Inno Setup) with start menu shortcut, desktop shortcut, auto-start option, and uninstaller
- Direct Win32 process monitoring via `CreateToolhelp32Snapshot` (replaces `sysinfo` crate)
- Event-driven process exit detection via `RegisterWaitForSingleObject` (NT kernel threadpool, zero CPU idle)
- Typed error handling via `thiserror` (replaces string errors)
- Compiled regex cache - patterns compiled once, reused every tick
- Pre-normalized DAW configs - process names lowercased and stripped at startup
- 28 unit tests covering config parsing, regex extraction, process matching, status formatting

**Changed:**

- Stable Rust toolchain (dropped nightly requirement)
- Split monolithic modules into focused files (20+ files, all under 250 lines)
- Discord IPC uses single `Mutex<DiscordState>` with poison recovery (was three separate Mutexes)
- Icon decoding cached via `LazyLock` (was re-decoded on every state change)
- Uninstaller removes settings and logs from `%APPDATA%`

**Removed:**

- `sysinfo` dependency (replaced by direct Win32 APIs)
- Nightly Rust requirement (`generic_const_exprs`)
- Platform stubs (`unsupported.rs`)

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v2.1.0 - Multi-process DAWs

Fixes Bitwig Studio (and other multi-process DAWs) showing incorrect stats.

**New stuff:**

- `AdditionalProcessNames` config field for multi-process DAWs (prefix matching)
- Versioned `daws.json` format - auto-updates local config when a new version ships
- Window icon shows Discord connection state (red/green)

**Fixed:**

- Bitwig Studio now aggregates CPU/RAM across all processes (main UI, audio engine, plugin hosts)

**Changed:**

- Extracted `app.rs` from `main.rs` for cleaner MVU separation
- Stricter clippy linting (`pub` to `pub(crate)`, unsafe blocks in unsafe fns)

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v2.0.1 - Size and memory optimizations

Leaned hard on size and idle RAM reductions for Windows builds.

**Changed:**

- Release builds strip logging and favor smaller binaries
- Process monitoring refreshes only the data we use

**Performance:**

- Binary size: ~36 MB -> ~17 MB -> ~4 MB
- RAM usage: ~70 MB+ -> ~7 MB

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v2.0.0 - First functional Rust release

The Rust rewrite is now fully functional. All DAW project names are detected correctly on Windows 10 and 11.

**Fixed:**

- Project name detection now works on all Windows versions (fixes #2)
- Switched from `regex` to `fancy-regex` crate (the old crate doesn't support lookaheads)

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.2 - Bugfixes

Bug fixes for tray icon responsiveness and window title detection.

**Fixed:**

- Tray icon stays responsive by pumping Windows messages in the event loop
- Window title detection no longer requires `IsWindowEnabled` check
- Dynamic buffer size for window titles via `GetWindowTextLengthW`
- Skip windows with empty titles early

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.1 - Rust rewrite

Complete rewrite from Python to Rust. Same functionality, faster, smaller, no runtime dependencies.

**New stuff:**

- Single standalone `.exe` - no Python or dependencies needed
- Native Windows GUI using iced
- System tray with status indicator
- Persistent settings via `confy` (stored in `%APPDATA%`)
- Cross-compilation support from Linux

**Removed:**

- Python codebase, PyInstaller build system, all Python dependencies

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

## v1.0.0 - Initial release

Python version. A rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC).

**Features:**

- Discord Rich Presence for FL Studio, Ableton Live, REAPER, Bitwig Studio, Studio One, LMMS, and Cubase
- Automatic DAW detection via process monitoring
- Project name extraction from window titles using regex
- System tray icon with status indicator
- Configurable refresh interval

<p align="right">(<a href="#changelog-top">back to top</a>)</p>

---

<div align="center">
  <p>Back to <a href="README.md">README</a>?</p>
</div>
