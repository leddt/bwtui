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
   - Refreshes automatically as countdown reaches zero
   - Displays "(none)" if no TOTP is configured
   - Shows "(invalid secret)" if TOTP secret cannot be decoded
5. **URIs**: 
   - Lists up to 3 URIs associated with the entry
   - Shows "... and N more" if there are more than 3
6. **Notes**: 
   - Displays up to 10 lines of notes
   - Shows "... and N more lines" if notes exceed 10 lines
   - Word-wraps long lines within the panel

## Implementation Details

### State Management
- Added `details_panel_visible: bool` field to `AppState`
- Toggle method: `state.toggle_details_panel()`

### Event Handling
- New action: `Action::ToggleDetailsPanel` - Toggles panel visibility
- New action: `Action::SelectIndexAndShowDetails(usize)` - Selects item and opens panel if hidden
- Key binding: `^D` (Ctrl+D) mapped to toggle action
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

