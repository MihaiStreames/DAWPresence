<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->

<div align="center">

[![Stars](https://img.shields.io/github/stars/MihaiStreames/DAWPresence?style=social)](https://github.com/MihaiStreames/DAWPresence/stargazers)
[![Release](https://img.shields.io/github/v/release/MihaiStreames/DAWPresence?label=Release)](https://github.com/MihaiStreames/DAWPresence/releases)
[![Rust Edition](https://img.shields.io/badge/Rust-2024-ed7a1f)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6)](https://github.com/MihaiStreames/DAWPresence/issues/1)
[![License](https://img.shields.io/github/license/MihaiStreames/DAWPresence?label=License)](LICENSE)

</div>

<!-- PROJECT LOGO -->

<div align="center">
  <img src="assets/app/main.png" alt="DAWPresence" width="120" />

  <h1>DAWPresence</h1>

  <h3 align="center">Show what you are creating on Discord.</h3>
</div>

<!-- TABLE OF CONTENTS -->

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#supported-daws">Supported DAWs</a></li>
    <li><a href="#how-it-works">How It Works</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## About The Project

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

DAWPresence monitors your running DAW and displays what you're working on in your Discord profile. It detects your project name from the window title and updates your Rich Presence automatically.

Tiny, fast, and stays out of your way. The binary is under 5 MB with zero runtime dependencies.

This is a complete rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC), rebuilt from the ground up in pure Rust.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

- [Rust](https://www.rust-lang.org/)
- [windows-sys](https://crates.io/crates/windows-sys)
- [discord-rich-presence](https://crates.io/crates/discord-rich-presence)
- [iced](https://iced.rs/)
- [tray-icon](https://crates.io/crates/tray-icon)
- [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json)
- [tracing](https://crates.io/crates/tracing)
- [tracing-subscriber](https://crates.io/crates/tracing-subscriber)
- [confy](https://crates.io/crates/confy)
- [crossbeam-channel](https://crates.io/crates/crossbeam-channel)
- [fancy-regex](https://crates.io/crates/fancy-regex)
- [image](https://crates.io/crates/image)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->

## Getting Started

### Prerequisites

- Windows 10 or later
- Discord Desktop App

If you want to help with Linux support, see [issue #1](https://github.com/MihaiStreames/DAWPresence/issues/1).

### Installation

#### Installer (recommended)

Download the setup exe from the [latest release](https://github.com/MihaiStreames/DAWPresence/releases/latest). The installer adds a start menu shortcut, optional desktop shortcut, and optional auto-start with Windows.

> [**Download Installer**](https://github.com/MihaiStreames/DAWPresence/releases/latest)

#### Portable

Download `DAWPresence.exe` from the [latest release](https://github.com/MihaiStreames/DAWPresence/releases/latest) and run it directly. Settings are stored in `%APPDATA%\dawpresence`.

> [**Download Portable**](https://github.com/MihaiStreames/DAWPresence/releases/latest)

#### Building from source

```bash
# clone the repo
git clone https://github.com/MihaiStreames/DAWPresence.git
cd DAWPresence

# build for production
cargo build --release

# run the app (Windows)
./target/release/DAWPresence.exe

# or cross-compile from Linux
cargo build --release --target x86_64-pc-windows-gnu
./target/x86_64-pc-windows-gnu/release/DAWPresence.exe
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- SUPPORTED DAWS -->

## Supported DAWs

| DAW           | Versions                    |
|---------------|-----------------------------|
| FL Studio     | 11+                         |
| Ableton Live  | 9-12 (Intro/Standard/Suite) |
| REAPER        | All                         |
| Bitwig Studio | All                         |
| Studio One    | All                         |
| LMMS          | All                         |
| Cubase        | 14                          |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- HOW IT WORKS -->

## How It Works

DAWPresence uses direct Win32 APIs to detect running DAWs. When a DAW is found, it attaches to the process and monitors CPU, RAM, and window title. Project names are extracted via regex patterns from `daws.json`. Process exit is detected instantly via `RegisterWaitForSingleObject` (NT kernel threadpool). Discord Rich Presence updates automatically.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->

## Contributing

### Adding New DAW Support

1. Create a Discord application at [Discord Developer Portal](https://discord.com/developers/applications)
2. Add a Rich Presence asset named `icon` with the DAW's icon
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
|--------------------------|------------|-------------------------------------------------|
| `ProcessName`            | `string`   | Process name without `.exe`                     |
| `DisplayText`            | `string`   | Name shown in DAWPresence                       |
| `TitleRegex`             | `string`   | Regex to extract project name from window title |
| `ClientID`               | `string`   | Discord application client ID                   |
| `HideVersion`            | `boolean`  | Whether to hide version info                    |
| `AdditionalProcessNames` | `string[]` | Extra process name prefixes to aggregate        |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->

## Acknowledgments

- [Serena1432](https://github.com/Serena1432) - Original DAWRPC creator

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->

## License

MIT. Do whatever you want with it. See [LICENSE](LICENSE) for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

---

<div align="center">

Made with ❤️

</div>
