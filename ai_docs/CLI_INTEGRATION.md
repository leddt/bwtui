# CLI Integration

## Overview
This document describes how `bwtui` integrates with the Bitwarden CLI (`bw`).

## Implementation

bwtui integrates with Bitwarden CLI via `src/cli.rs`.

### Key Points

**Data Flow:**
```
bw CLI → JSON output → serde deserialization → VaultItem structs → UI
```

**CLI Commands:**
- `bw --version` - Check installation
- `bw status` - Get vault status (locked/unlocked/unauthenticated)
- `bw list items` - Load all vault entries (includes encrypted TOTP secrets)
- `bw sync` - Sync with server

**Note:** TOTP codes are generated locally using the `totp-lite` library for instant performance. The TOTP secrets are retrieved from `bw list items` but code generation happens client-side without CLI calls.

**Session Management:**
- Session token from `BW_SESSION` env var
- No disk storage of credentials
- User must `bw login` and `bw unlock` before running

### JSON Parsing Gotchas

Bitwarden CLI returns:
1. **camelCase fields** - Use `#[serde(rename_all = "camelCase")]`
2. **Numeric type field** - `"type": 1` (not `"type": "1"`)
3. **Extra fields** - Add with `#[serde(default, skip_serializing)]`
4. **URI field** - Named `uri` not `value`

Example fix for ItemType:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "u8")]
pub enum ItemType {
    Login,      // type: 1
    SecureNote, // type: 2
    Card,       // type: 3
    Identity,   // type: 4
}
```

### Error Types (src/error.rs)

- `CliNotFound` - bw not installed
- `VaultLocked` - Need to unlock
- `NotLoggedIn` - Need to login
- `CommandFailed` - CLI execution error
- `ParseError` - JSON parse error

### Startup Flow

```
1. Check bw CLI exists
2. Check vault status
3. Verify BW_SESSION env var
4. Load items via `bw list items`
5. Start TUI
```

Exit with helpful error if any step fails.

### User Prerequisites

```bash
npm install -g @bitwarden/cli
bw login
bw unlock
export BW_SESSION="token"  # or $env:BW_SESSION on Windows
cargo run --release
```

### Files Modified

- `src/cli.rs` (new) - CLI wrapper
- `src/error.rs` - Added CLI errors
- `src/main.rs` - Uses CLI instead of mocks
- `src/types.rs` - Fixed JSON deserialization
- `src/ui.rs` - Updated uri.value → uri.uri
- Deleted `src/mock_data.rs`

