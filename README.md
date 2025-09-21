# DAWPresence

A simple Python app showing what you're creating in your Digital Audio Workstation on your Discord profile.

![Preview](./preview.png)

## About This Project

This is a complete Python rewrite of [Serena1432's DAWRPC](https://github.com/Serena1432/DAWRPC) with several code
quality enhancements:

- Clean MVC pattern with proper separation of concerns
- Component-based UI and service-oriented backend
- Immutable data models, dependency injection, and SOLID principles
- Comprehensive exception handling and validation

**Special thanks to [Serena1432](https://github.com/Serena1432) for creating the original DAWRPC and providing the
foundation for this enhanced version!**

## Supported DAWs

- **FL Studio** (11 or later)
- **Ableton Live** (9 Intro/Standard/Suite or later)
- **REAPER**
- **Bitwig Studio**
- **Studio One**
- **LMMS**
- **Cubase 14**

## Installation

### Prerequisites

- Python 3.7+
- Windows OS
- Discord Desktop App

### Setup

1. Clone this repository:
   ```bash
   git clone https://github.com/MihaiStreames/DAWPresence.git
   cd DAWPresence
   ```

2. Install dependencies:
   ```bash
   pip install -r DAWPY/requirements.txt
   ```

3. Run the application:
   ```bash
   cd DAWPY
   python main.py
   ```

## Contributing

### Adding New DAW Support

1. Create a new Discord application at [Discord Developer Portal](https://discord.com/developers/applications)
2. Add a new Rich Presence Asset Image (with the DAW's icon) and the Asset Name being  `icon`
3. Edit `daws.json` with the new DAW's details. Below are the required properties:

| Properties    | Type      | Description                                                                                                           |
|---------------|-----------|-----------------------------------------------------------------------------------------------------------------------|
| `ProcessName` | `string`  | DAW's process name without `.exe`                                                                                     |
| `DisplayText` | `string`  | The text to be displayed when detected in DAWPresence                                                                 |
| `TitleRegex`  | `string`  | Regular expression from the DAW's window title. DAWPresence will take the first matched string as the "project name". |
| `ClientID`    | `string`  | Discord Client ID for displaying Rich Presence.                                                                       
| `HideVersion` | `boolean` | Whether to hide the DAW version in DAWPresence.                                                                       |

```json
{
	"ProcessName": "YourDAW",
	"DisplayText": "Your DAW Name",
	"TitleRegex": "^(.*?)(?= - Your DAW)",
	"ClientID": "your_discord_client_id",
	"HideVersion": false
}
```

## Acknowledgments

- **[Serena1432](https://github.com/Serena1432)** - Original DAWRPC creator
- **[qwertyquerty](https://github.com/qwertyquerty/pypresence)** - Python Discord RPC library

## Contact

- Discord: sincoswashere