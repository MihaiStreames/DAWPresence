# <img src="https://github.com/MihaiStreames/DAWPresence/raw/master/assets/app/main.png?raw=true" alt="DAWPresence icon" height="28" width="28"> DAWPresence

Discord Rich Presence for your DAW. Polls running processes, extracts the project name from the window title, and pushes it to Discord. Under 2 MB, no runtime deps, no bloat.

[![Release](https://img.shields.io/github/v/release/MihaiStreames/DAWPresence?label=release)](https://github.com/MihaiStreames/DAWPresence/releases)
[![Platform](https://img.shields.io/badge/platform-windows-0078D6)](https://github.com/MihaiStreames/DAWPresence/issues/1)
[![CI](https://github.com/MihaiStreames/DAWPresence/actions/workflows/ci.yml/badge.svg)](https://github.com/MihaiStreames/DAWPresence/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/MihaiStreames/DAWPresence?label=license)](LICENSE)

<div align="center">
  <table><tr>
    <td>
      <img src="assets/home.png" alt="Home" width="100%" />
      <img src="assets/settings.png" alt="Settings" width="100%" />
    </td>
    <td width="30%">
      <img src="assets/preview.png" alt="Discord Preview" width="100%" />
    </td>
  </tr></table>
</div>

## Origin

Started as a Python rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC). The original is C# and Windows-only, and I wanted to learn how it worked. Once the Python port was solid I picked Rust to learn the language properly and rewrote the whole thing again. DAWPresence is the result, and my first real Rust project.

## Requirements

- Windows 10 or later
- Discord desktop app

Linux support tracked in [issue #1](https://github.com/MihaiStreames/DAWPresence/issues/1).

## Get it running

### Installer (recommended)

Download the setup `.exe` from the [latest release](https://github.com/MihaiStreames/DAWPresence/releases/latest). Comes with a start menu shortcut, optional desktop shortcut, and optional auto-start with Windows.

### Portable

Download `DAWPresence.exe` from the [latest release](https://github.com/MihaiStreames/DAWPresence/releases/latest) and run it. Settings live in `%APPDATA%\dawpresence`.

## Build from source

On Windows:

> [!NOTE]
> Windows is very annoying. You WILL need MSVC (Build Tools with Visual C++). One command handles everything silently:
>
> ```pwsh
> winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended"
> ```
>
> If you use `rust-analyzer`, cargo also needs `LIB` and `PATH` set system-wide (run `vcvars64.bat` or set them permanently). The upside is that no `.cargo/config.toml` is needed since MSVC is the default target.

```pwsh
git clone https://github.com/MihaiStreames/DAWPresence.git
cd DAWPresence
cargo build --release
```

On Linux:

> [!NOTE]
> You need `gcc-mingw-w64-x86-64`:
>
> ```sh
> pacman -S mingw-w64-gcc mingw-w64-binutils # arch
> apt install gcc-mingw-w64-x86-64 # ubuntu
> ```
>
> Add a `.cargo/config.toml` so you don't have to pass `--target` every time:
>
> ```toml
> [build]
> target = "x86_64-pc-windows-gnu"
> ```

```sh
git clone https://github.com/MihaiStreames/DAWPresence.git
cd DAWPresence
cargo build --release # drop --target if you added config.toml above
```

## Supported DAWs

| DAW           | Versions                    |
| ------------- | --------------------------- |
| FL Studio     | 11+                         |
| Ableton Live  | 9-12 (Intro/Standard/Suite) |
| REAPER        | All                         |
| Bitwig Studio | All                         |
| Studio One    | All                         |
| LMMS          | All                         |
| Cubase        | 14                          |

## How it works

On startup, a named Win32 event (`Local\DAWPresence-SingleInstance`) enforces single-instance behavior. If the event already exists, the new instance signals the running one to show its window and exits (no duplicate tray icons, no silent second process).

The scanner polls running processes via `CreateToolhelp32Snapshot` and matches against `daws.json` entries by process name. On match, it opens the process with `PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE` and registers a `RegisterWaitForSingleObject` callback on the NT kernel threadpool (process exit detected instantly, no polling loop required).

Each tick reads CPU time via `GetProcessTimes`, working set via `GetProcessMemoryInfo`, and window title via `EnumWindows`. The project name is extracted using a per-DAW regex (compiled patterns cached across ticks, not rebuilt every tick). DAW version comes from the PE version resource (`VerQueryValueW`). All of this feeds a Discord Rich Presence update over the IPC socket.

## Add a DAW

1. Create a Discord application at the [Discord Developer Portal](https://discord.com/developers/applications).
2. Add a Rich Presence asset named `icon` with the DAW's icon.
3. Add an entry to `daws.json`:

```json
{
  "ProcessName": "YourDAW",
  "DisplayText": "Your DAW Name",
  "TitleRegex": "^(.*?)(?= - Your DAW)",
  "ClientID": "your_discord_client_id",
  "HideVersion": false
}
```

| Field                    | Type       | Description                                     |
| ------------------------ | ---------- | ----------------------------------------------- |
| `ProcessName`            | `string`   | Process name without `.exe`                     |
| `DisplayText`            | `string`   | Name shown in DAWPresence                       |
| `TitleRegex`             | `string`   | Regex to extract project name from window title |
| `ClientID`               | `string`   | Discord application client ID                   |
| `HideVersion`            | `boolean`  | Whether to hide version info                    |
| `AdditionalProcessNames` | `string[]` | Extra process name prefixes to aggregate        |

## Acknowledgments

DAWPresence wouldn't exist without [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC). The DAW detection model, regex-driven project name extraction, `daws.json` schema, and overall shape of the tool all come from there. Thanks to [Serena1432](https://github.com/Serena1432) for building the original.

## License

MIT. See [LICENSE](LICENSE).

<div align="center">
  Made with ❤️
</div>
