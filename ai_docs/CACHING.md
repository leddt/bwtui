# Caching System

## Overview

bwtui implements a caching system to provide instant UI population on startup while vault sync happens in the background. This significantly improves perceived performance and user experience.

## Architecture

### Cache Location

Cache is stored in: `~/.bwtui/vault_cache.bin`

### Cache Format

- **Serialization**: bincode (efficient binary format)
- **Size**: Minimal (no secrets, no notes)
- **Metadata stored**:
  - Item ID, name, type
  - Favorite status
  - Folder/organization IDs
  - Revision date
  - Username (non-sensitive)
  - URIs
  - Flags for password/TOTP existence (not the actual secrets)

### What's Excluded from Cache

For security reasons, the following are NEVER cached:
- Passwords
- TOTP secrets
- Notes (may contain sensitive info)

## Flow

### App Startup
1. Load cache immediately (if exists)
2. Populate entry list with cached data
3. Show status: "Loaded N items from cache (syncing in background...)"
4. Start background vault sync
5. When sync completes, replace cached data with full data

### During Cache Mode
- Entry list is fully navigable
- Details panel shows:
  - Name, username, URIs normally
  - Password field: "⠋ Loading..." (spinner)
  - TOTP field: "⠋ Loading..." (spinner)  
  - Notes field: "⠋ Loading..." (spinner)
- Copy password/TOTP shortcuts disabled
  - Attempting shows: "⏳ Please wait, loading vault secrets..."

### After Sync Complete
- All fields populated normally
- Password/TOTP copy shortcuts enabled
- Cache updated with new metadata

## State Tracking

`AppState` tracks cache status via:
- `initial_load_complete`: true when any data loaded (cache or real)
- `secrets_available`: false for cached data, true for real vault data

## Implementation Files

- `src/cache.rs`: Cache data structures and I/O
- `src/state.rs`: State management for cache/secrets tracking
- `src/main.rs`: Load cache on startup, save after sync
- `src/ui.rs`: Show loading spinners for unavailable fields

## Benefits

1. **Instant UI**: No blank screen on startup
2. **Background sync**: User can browse while syncing
3. **Graceful degradation**: Clear indication of unavailable data
4. **Security**: No secrets stored on disk

## Technical Details

### Serialization

The cache uses bincode for efficient binary serialization. Several considerations were made for bincode compatibility:

1. **ItemType enum**: Serialized as u8 (1-4) for both JSON compatibility with Bitwarden CLI and efficient bincode storage.

2. **URI simplification**: The `Uri` type in the original data contains a `serde_json::Value` for `match_type`, which is incompatible with bincode (it uses `deserialize_any`). The cache uses a simplified `CachedUri` that only stores the URI string, which is sufficient for display purposes.

3. **No serde_json::Value types**: All cached types avoid using `serde_json::Value` to ensure bincode compatibility.

### Error Handling

If cache deserialization fails (e.g., format change or corruption), the cache file is automatically deleted and a fresh sync is performed. This ensures the app never gets stuck with a corrupted cache.

