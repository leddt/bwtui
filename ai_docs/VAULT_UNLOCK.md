# Vault Unlock Flow

## Overview

The application now handles locked vaults gracefully on startup by prompting the user for their master password and offering to save the session token for future sessions.

## Implementation

### Components

1. **Password Input Dialog** (`ui.rs`)
   - Modal dialog overlay that accepts password input
   - Shows masked password (•••) for security
   - Displays unlock errors if authentication fails
   - Keyboard shortcuts: Enter to submit, Esc to cancel

2. **Session Token Management** (`main.rs`)
   - After successful unlock, prompts user to save session token
   - Saves to PowerShell profile on Windows (`$env:BW_SESSION`)
   - Saves to shell profile on Unix (~/.bashrc or ~/.zshrc as `export BW_SESSION`)
   - Automatically updates existing token if already present

3. **CLI Unlock** (`cli.rs`)
   - `unlock()` method calls `bw unlock --raw <password>`
   - Returns session token on success
   - Handles invalid password errors gracefully
   - `with_session_token()` creates CLI instance with explicit token

4. **State Management** (`state.rs`)
   - `password_input_mode`: Active when entering password
   - `password_input`: Stores password being typed
   - `unlock_error`: Displays error message if unlock fails
   - `offer_save_token`: Shows save token prompt
   - `save_token_response`: Tracks user's choice

5. **Event Handling** (`events.rs`)
   - New actions: `SubmitPassword`, `CancelPasswordInput`, `AppendPasswordChar`, `DeletePasswordChar`
   - New actions: `SaveTokenYes`, `SaveTokenNo`
   - Input is routed based on current state mode

### Flow

1. **Startup**
   - Initialize Bitwarden CLI
   - Check vault status
   - If unlocked → proceed to load items
   - If locked → show password dialog
   - If unauthenticated → show "not logged in" error popup

2. **Password Entry**
   - User types password (displayed as •••)
   - Enter submits password
   - Esc exits the app (can't proceed without unlocking)
   - Background task attempts unlock

3. **Unlock Success**
   - Exit password mode
   - Show save token prompt
   - User chooses Y/N
   - If Y: Save to shell profile
   - Load vault items

4. **Unlock Failure**
   - Display error in password dialog
   - User can retry with correct password

5. **Not Logged In**
   - Show error popup with instructions to run `bw login`
   - Only action available is Esc (or Ctrl+C) to exit
   - User must log in via CLI and restart the app

### Session Token Storage

The session token is saved as a persistent user environment variable, making it available across all shells and applications.

**Windows**
- Uses `setx` command to set the `BW_SESSION` user environment variable
- Persists across reboots and all shells (PowerShell, CMD, etc.)
- Also sets in current process for immediate availability

**macOS**
- Uses `launchctl setenv` to set the `BW_SESSION` user environment variable
- Persists across reboots and available to all applications
- Also sets in current process for immediate availability

**Linux**
- First attempts to use `systemctl --user set-environment` (systemd)
- Falls back to `~/.profile` if systemd is not available
- Persists across reboots and shells
- Also sets in current process for immediate availability

## Security Considerations

- Password is never stored or logged
- Password is masked in UI (displayed as bullets)
- Session token is stored in user's shell profile (same as manual `bw unlock`)
- Token is only saved if user explicitly agrees
- Unlock happens in background task to prevent UI blocking

## User Experience

- Seamless unlock on startup if vault is locked
- No need to run `bw unlock` manually
- Optional convenience of staying logged in
- Clear error messages for failed unlocks
- Non-intrusive prompts with clear options

