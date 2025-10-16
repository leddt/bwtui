# Details Panel Feature

## Overview
A toggleable details panel that displays comprehensive information about the selected vault entry.

## Usage
- **Toggle**: Press `^D` (Ctrl+D) to show/hide the details panel
- **Mouse Click**: Clicking on an entry with the mouse automatically opens the details panel
- **Default State**: Hidden (panel is not visible when the app starts)
- **Layout**: When visible, splits the screen 50/50 between the entry list and details panel

## Information Displayed

The details panel shows the following information for the selected entry:

1. **Name**: The entry's display name
2. **Username**: The associated username, or "(none)" if not set
   - **Mouse-clickable button**: `[ Copy ^U ]` for quick copying (also works with keyboard shortcut)
3. **Password**: Masked as `••••••••` for security (actual password not displayed)
   - **Mouse-clickable button**: `[ Copy ^P ]` for quick copying (also works with keyboard shortcut)
4. **TOTP**: 
   - Live 6-digit TOTP code if configured
   - Shows countdown timer (e.g., "123456 (15s)")
   - **Mouse-clickable button**: `[ Copy ^T ]` for quick copying (also works with keyboard shortcut)
   - Refreshes automatically every 250ms (countdown updates smoothly, new code appears when previous expires)
   - Displays "(none)" if no TOTP is configured
   - Shows "(invalid secret)" if TOTP secret cannot be decoded
5. **URIs**: 
   - Lists all URIs associated with the entry (no limit)
6. **Notes**: 
   - Displays all lines of notes (no limit)
   - Word-wraps long lines within the panel

## Scrolling

The details panel is scrollable when the content exceeds the available space:

- **Scroll Up**: `SHIFT+Up` or `CTRL+SHIFT+K`
- **Scroll Down**: `SHIFT+Down` or `CTRL+SHIFT+J`
- **Auto-scroll**: Scroll position resets when:
  - Toggling the panel
  - Selecting a different item (navigation, filtering, loading)
  - Changing the selected entry in any way
- **Bounds**: Scrolling is automatically bounded to prevent scrolling beyond content

## Implementation Details

### State Management
- Added `details_panel_visible: bool` field to `AppState`
- Added `details_panel_scroll: usize` field to track scroll position
- Added `details_panel_max_scroll: usize` field to track maximum scroll position
- Toggle method: `state.toggle_details_panel()`
- Scroll methods: `state.scroll_details_up()` and `state.scroll_details_down()`
- Reset method: `state.reset_details_scroll()` - resets scroll to top

### Event Handling
- New action: `Action::ToggleDetailsPanel` - Toggles panel visibility
- New action: `Action::SelectIndexAndShowDetails(usize)` - Selects item and opens panel if hidden
- New actions: `Action::ScrollDetailsUp` and `Action::ScrollDetailsDown` - Scroll the details panel
- Key bindings: 
  - `^D` (Ctrl+D) mapped to toggle action
  - `SHIFT+Up/Down` mapped to scroll actions
  - `CTRL+SHIFT+K/J` mapped to scroll actions
- Mouse click on entry: Automatically opens panel when selecting an item
- Mouse click on copy buttons: Triggers respective copy action (username, password, or TOTP)
- Smart line detection: Calculates which button was clicked based on panel layout and item data

### UI Rendering
- Splits main area horizontally when panel is visible (50/50)
- Details panel has cyan borders and title when active
- TOTP generation uses `totp-lite` and `base32` libraries
- Supports RFC 4648 base32 encoding (with and without padding)
- TOTP parameters: 30-second time step, 6-digit codes, SHA1 algorithm

### Dependencies Added
- `totp-lite = "2.0"` - TOTP code generation
- `base32 = "0.4"` - Base32 secret decoding

## Visual Design
- **Title**: Cyan, bold
- **Labels**: Cyan, bold (Name, Username, Password, TOTP, URIs, Notes)
- **Values**: White for normal text, colors for special elements:
  - Username: White
  - Password: Yellow (masked)
  - TOTP: Green, bold (with gray countdown)
  - URIs: Blue
  - Notes: White
- **Copy Buttons**: 
  - Brackets: Dark gray `[ ]`
  - Button text: Cyan, underlined `Copy ^X` (mouse-clickable)
  - Positioned below their respective fields
  - Only shown when field has a value
- **Empty values**: Dark gray "(none)" text
- **Panel border**: Cyan when active, matches entry list styling

## TOTP Refresh Implementation
- The UI refreshes every 250ms via a periodic `Tick` action
- TOTP codes are regenerated on each render based on current system time
- Countdown timer updates smoothly, showing remaining seconds
- When countdown reaches 0, a new code is automatically generated
- No user interaction required for TOTP refresh

### Technical Details
- Uses `totp_lite::totp_custom` with SHA1 algorithm
- Time step: 30 seconds (standard TOTP period)
- Digits: 6 (standard TOTP length)
- The function receives the raw Unix timestamp and calculates steps internally
- Base32 secret decoding supports both padded and unpadded formats (RFC 4648)

### Bug Fix History
- **Issue**: TOTP codes were not updating after 30 seconds, countdown would reach 0 but code stayed the same
- **Root Cause**: Was incorrectly passing pre-calculated step count (`now / time_step`) to `totp_custom`, but the function expects the raw timestamp and handles step calculation internally
- **Fix**: Changed to pass `now` (Unix timestamp) directly to `totp_custom` instead of `steps`
- **Verification**: Added test `test_totp_changes_across_time_steps` to ensure codes change correctly across time boundaries

### Performance Optimization
- **Issue**: Copying TOTP with `^T` was slow (~1-2 seconds)
- **Root Cause**: Was calling `bw get totp <id>` CLI command which spawns a new process
- **Fix**: Changed to use local `totp_util::generate_totp()` function, same as the display code
- **Result**: Instant TOTP copy, no CLI overhead
- **Removed**: `BitwardenCli::get_totp()` method (no longer needed)

