# Session Token Encryption

## Overview

The session token is now encrypted using OS-specific encryption APIs and stored in the native OS keyring instead of environment variables. This provides better security while maintaining a seamless user experience.

## Implementation

### Storage Location

The encrypted session token is stored securely using OS-specific methods:

- **Windows**: Encrypted file using DPAPI at `~/.bwtui/session.enc`
- **macOS**: macOS Keychain (with marker file at `~/.bwtui/session.enc`)
- **Linux**: Secret Service API (with marker file at `~/.bwtui/session.enc`)

### Encryption Method

- **Windows**: Direct DPAPI (Data Protection API) calls via `CryptProtectData`/`CryptUnprotectData`
  - Avoids Windows Credential Manager limitations
  - Stores encrypted data directly in a file
  - User-specific encryption using login credentials
  
- **macOS/Linux**: Uses the `keyring` crate for OS-native secure storage
  - macOS: Keychain Services
  - Linux: Secret Service API (libsecret/gnome-keyring)

### How It Works

**Windows:**
1. **Save Token**: Uses `CryptProtectData` to encrypt the token with the user's credentials
2. **Store**: Writes encrypted data to `~/.bwtui/session.enc`
3. **Load**: Reads encrypted data from file
4. **Decrypt**: Uses `CryptUnprotectData` to decrypt using the user's credentials

**macOS/Linux:**
1. **Save Token**: Stores token in OS keyring using `keyring` crate
2. **Marker**: Writes a marker file to `~/.bwtui/session.enc`
3. **Load**: Reads marker, then retrieves token from keyring
4. **Encryption**: All encryption/decryption happens silently using the user's login session credentials

**All Platforms:**
- **No Prompts**: Users are never prompted for passwords - the OS handles authentication transparently
- **User-Specific**: Only the current user can decrypt the token
- **Silent Operation**: Everything happens in the background

### Key Benefits

- **Silent Operation**: No password prompts for the user
- **User-Specific**: Only the current user can decrypt the token
- **OS-Native**: Uses platform-specific secure storage mechanisms
- **Persistent**: Token survives across app restarts
- **Secure**: Even administrators cannot access the encrypted token without the user's credentials

## Code Structure

### SessionManager (`src/session.rs`)

Main API for session token management:

```rust
pub struct SessionManager {
    entry: Entry,  // keyring::Entry
}

impl SessionManager {
    pub fn new() -> Result<Self>
    pub fn load_token(&self) -> Result<Option<String>>
    pub fn save_token(&self, token: &str) -> Result<()>
    pub fn clear_token(&self) -> Result<()>
}
```

The implementation is remarkably simple - just ~50 lines of code with no platform-specific branches!

### BitwardenCli Integration (`src/cli.rs`)

The CLI now automatically loads the encrypted token on initialization:

```rust
pub async fn new() -> Result<Self> {
    // ... check bw CLI availability ...
    
    // Load session token from encrypted storage
    let session_manager = SessionManager::new()?;
    let session_token = session_manager.load_token()?;
    
    Ok(Self { session_token })
}
```

## Migration from Environment Variables

The old implementation used:
- Windows: `setx BW_SESSION`
- macOS: `launchctl setenv BW_SESSION`
- Linux: `~/.profile` with `export BW_SESSION`

The new implementation:
- Uses OS keyring for storage (no files!)
- No modification of system environment variables
- No configuration files to manage

Users' existing environment variables are ignored. They should unlock the vault once to create the encrypted session token in the OS keyring.

