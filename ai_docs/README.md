# bwtui - Bitwarden Terminal UI

Fast, keyboard-driven terminal interface for Bitwarden password manager.

## Current Status

**Prototype working** (~950 LOC Rust) with mock data. Production CLI integration pending.

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
- **ui.rs** - ratatui rendering (search box, entry list, status bar)
- **clipboard.rs** - Cross-platform clipboard via arboard
- **mock_data.rs** - 10 sample entries for testing

## Tech Stack

- **Rust** - Memory safety, speed, cross-platform single binary
- **ratatui 0.24+** - Modern TUI framework
- **crossterm 0.27+** - Terminal manipulation
- **tokio 1.35+** - Async runtime
- **arboard 3.3+** - Clipboard
- **fuzzy-matcher 0.3+** - Fast fuzzy search
- **serde 1.0+** - JSON parsing for CLI output

## Usage

### Run Prototype
```bash
cargo run --release
```

### Production (when implemented)
```bash
bw login
bwtui
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Type any character | Filter entries |
| Backspace | Delete from filter |
| ESC | Clear filter |
| ↑/↓ or Ctrl+K/J | Navigate entries |
| Page Up/Down | Navigate pages |
| Ctrl+U | Copy username |
| Ctrl+P | Copy password |
| Ctrl+T | Copy TOTP |
| Ctrl+R | Refresh vault |
| Ctrl+C | Quit |

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
- ⏳ Bitwarden CLI integration
- ⏳ Caching layer
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
├── main.rs        # Entry point, main loop (229 lines)
├── types.rs       # Data models (72 lines)
├── state.rs       # State & filtering (196 lines)
├── ui.rs          # UI rendering (191 lines)
├── events.rs      # Event handling (106 lines)
├── clipboard.rs   # Clipboard ops (40 lines)
├── mock_data.rs   # Test data (200 lines)
└── error.rs       # Error types (25 lines)
```

## Documentation

- **README.md** (this file) - Project overview
- **ARCHITECTURE.md** - Technical design
- **IMPLEMENTATION.md** - Build guide
- **TECH_STACK.md** - Technology rationale
- **PERFORMANCE.md** - Optimization guide
- **ROADMAP.md** - Feature planning
- **CONTRIBUTING.md** - Contribution guide

## License

MIT
