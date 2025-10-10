# bwtui - Bitwarden Terminal UI

Fast, keyboard-driven terminal interface for Bitwarden password manager.

## Current Status

**Working with real Bitwarden data** (~1,100 LOC Rust) via CLI integration. Disk caching layer pending.

## Architecture

```
User Input → Event Handler → State Manager → Bitwarden CLI
                ↓                                   ↓
           UI Renderer ←───────────────────────────┘
```

### Core Components

- **types.rs** - Data models (VaultItem, LoginData, Uri)
- **state.rs** - State management, filtering (fuzzy + exact match)
- **events.rs** - Keyboard input → Actions
- **ui.rs** - ratatui rendering (search box, entry list, details panel, status bar)
- **cli.rs** - Bitwarden CLI integration (list, sync, TOTP)
- **totp_util.rs** - TOTP code generation with live countdown
- **clipboard.rs** - Cross-platform clipboard via arboard

## Tech Stack

- **Rust** - Memory safety, speed, cross-platform single binary
- **ratatui 0.24+** - Modern TUI framework
- **crossterm 0.27+** - Terminal manipulation
- **tokio 1.35+** - Async runtime
- **arboard 3.3+** - Clipboard
- **fuzzy-matcher 0.3+** - Fast fuzzy search
- **totp-lite 2.0+** - TOTP code generation
- **base32 0.4+** - Base32 encoding/decoding
- **serde 1.0+** - JSON parsing for CLI output

## Usage

### Setup

```bash
npm install -g @bitwarden/cli
bw login
bw unlock  # Copy the session token
export BW_SESSION="token"  # Windows: $env:BW_SESSION="token"
cargo run --release
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Type any character | Filter entries |
| Backspace | Delete from filter |
| ESC | Clear filter |
| ↑/↓ or ^K/^J | Navigate entries |
| Page Up/Down | Navigate pages |
| ^U | Copy username |
| ^P | Copy password |
| ^T | Copy TOTP |
| ^D | Toggle details panel |
| ^R | Refresh vault |
| ^C | Quit |

## Performance

Target: <50ms startup even with 10s+ CLI load times.

**Strategy:**
1. Disk cache of vault data (JSON)
2. Background refresh via tokio spawn
3. Instant startup from cache
4. TOTP caching (30s refresh)
5. Parallel filtering for large vaults

See PERFORMANCE.md for implementation details.

## Building

### Linux/macOS
```bash
cargo build --release
```

### Windows
Requires Visual Studio Build Tools (C++ components) or use WSL.
```bash
# Install VS Build Tools, then:
cargo build --release
```

## Configuration

`~/.config/bwtui/config.toml`:
```toml
clipboard_timeout = 30        # seconds
auto_lock_minutes = 15
case_sensitive = false
fuzzy_matching = true
```

## Development Roadmap

**v1.0 (MVP)**
- ✅ Terminal UI (prototype done)
- ✅ Bitwarden CLI integration
- ⏳ Disk caching layer
- ⏳ Background refresh

**v2.0+**
- Entry editing
- Password generator
- Multi-account support
- Advanced search syntax
- Custom themes

See ROADMAP.md for full feature planning.

## Security

- Uses official Bitwarden CLI (no direct API access)
- Session tokens handled by bw CLI
- Clipboard auto-clear
- No local vault storage by this app
- Vault unlock delegated to bw CLI

## Project Structure

```
src/
├── main.rs        # Entry point, main loop
├── cli.rs         # Bitwarden CLI integration
├── types.rs       # Data models
├── state.rs       # State & filtering
├── ui.rs          # UI rendering & details panel
├── events.rs      # Event handling
├── clipboard.rs   # Clipboard operations
├── totp_util.rs   # TOTP code generation
└── error.rs       # Error types
```

## Documentation

- **README.md** - Project overview
- **ARCHITECTURE.md** - Technical design
- **CLI_INTEGRATION.md** - Bitwarden CLI integration details
- **IMPLEMENTATION.md** - Build guide
- **TECH_STACK.md** - Technology rationale
- **PERFORMANCE.md** - Optimization guide
- **DETAILS_PANEL.md** - Details panel feature
- **SCROLLING.md** - Scrolling implementation
- **ROADMAP.md** - Feature planning
- **CONTRIBUTING.md** - Contribution guidelines

## License

MIT
