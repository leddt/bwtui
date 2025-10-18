# bwtui

A terminal-based user interface for Bitwarden CLI. This application provides a text-based interface to browse, search, and copy credentials from your Bitwarden vault without leaving the terminal.

## Features

- Browse vault items (logins, secure notes, cards, identities)
- Search and filter vault items
- Copy usernames, passwords, and TOTP codes to clipboard
- View detailed information about vault items
- Automatic vault synchronization
- Caching system to improve startup time
- Session token management for convenience

## Security Considerations

**Important**: This application is vibe-coded and has not undergone security review. Use at your own risk.

- Session tokens are stored using platform-specific secure storage (Windows DPAPI, macOS Keychain, etc.)
- Vault data is cached locally without sensitive information
- Clipboard operations are performed using system clipboard APIs
- No network communication is performed directly by the application (relies on Bitwarden CLI)

## Prerequisites

- [Bitwarden CLI](https://bitwarden.com/help/cli/) installed and configured
- Rust toolchain (for building from source)

## Installation

Pre-built binaries are available as GitHub releases.

### Building from Source

```bash
git clone <repository-url>
cd bwtui
cargo build --release
```

The binary will be available at `target/release/bwtui.exe` (Windows) or `target/release/bwtui` (Unix-like systems).

## Usage

### First Run

1. Ensure you're logged into Bitwarden CLI:
   ```bash
   bw login
   ```

2. Run the application:
   ```bash
   ./bwtui
   ```

3. Enter your master password when prompted

4. Optionally save your session token for future convenience (avoids re-entering password)

### Navigation

- **Up/Down Arrow Keys**: Navigate up/down through vault items 
  - Vim-style **Ctrl+J/K** also works
- **Page Up/Down**: Jump by 10 items
- **Home/End**: Jump to first/last item

### Search and filtering

- Start typing to filter vault items
- **Ctrl+X**: Clear filter
- **Ctrl+1-5**: Select tab (item-type filter)
- **Left/Right Arrow Keys**: Cycle through tabs
  - **Ctrl+H/L** also works

### Copying Credentials

- **Ctrl+U**: Copy username
- **Ctrl+P**: Copy password  
- **Ctrl+T**: Copy TOTP code
- **Ctrl+N**: Copy card number (for card items)
- **Ctrl+M**: Copy card CVV (for card items)

### Details Panel

- **Ctrl+D**: Toggle details panel visibility
- **Shift+Arrow Keys**: Scroll through details when panel is open 
  - **Ctrl+Shift+J/K** also works

### Other Actions

- **Ctrl+R**: Refresh vault (sync with server)
- **Ctrl+Q**: Quit application
- **Ctrl+L**: Lock and quit (clear session token and cache)