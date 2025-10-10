# List Scrolling Implementation

## Overview

The entry list now supports automatic scrolling when navigating beyond the visible area. This is implemented using ratatui's `ListState` widget.

## Implementation Details

### Changes Made

#### 1. State Management (`src/state.rs`)
- Added `list_state: ListState` field to `AppState`
- Added `sync_list_state()` private method to keep `ListState` synchronized with `selected_index`
- Updated all navigation methods to call `sync_list_state()`:
  - `select_next()`
  - `select_previous()`
  - `page_up()`
  - `page_down()`
  - `jump_to_start()`
  - `jump_to_end()`
- Updated `apply_filter()` to sync list state when filtered items change

#### 2. UI Rendering (`src/ui.rs`)
- Changed `render()` method signature to accept `&mut AppState` instead of `&AppState`
- Changed `render_entry_list()` to accept `&mut AppState`
- Updated list rendering to use `frame.render_stateful_widget()` instead of `frame.render_widget()`
- Added `.highlight_style()` to the `List` widget for better visual feedback

#### 3. Main Loop (`src/main.rs`)
- Updated render call to pass `&mut state` instead of `&state`

## How It Works

1. **ListState Tracking**: The `ListState` widget maintains an internal scroll position that automatically adjusts based on the selected index.

2. **Synchronization**: Whenever the selected index changes (via navigation or filtering), the `sync_list_state()` method updates the `ListState` to match:
   ```rust
   fn sync_list_state(&mut self) {
       if self.filtered_items.is_empty() {
           self.list_state.select(None);
       } else {
           self.list_state.select(Some(self.selected_index));
       }
   }
   ```

3. **Automatic Scrolling**: When `render_stateful_widget()` is called with the `ListState`, ratatui automatically:
   - Calculates the visible window based on the area size
   - Scrolls the list to ensure the selected item is visible
   - Handles both upward and downward scrolling

## Benefits

- **Smooth Navigation**: Users can navigate through large lists without losing track of the selected item
- **Efficient**: Only visible items are rendered, maintaining good performance even with large vaults
- **Automatic**: No manual scroll offset calculations needed
- **Consistent**: Behavior matches standard terminal UI expectations

## Testing

To test scrolling:
1. Load a vault with more entries than fit on screen
2. Use arrow keys to navigate down past the visible area - the list scrolls automatically
3. Use Page Up/Down to jump multiple entries - scrolling adjusts appropriately
4. Use Home/End to jump to start/end - view jumps to correct position
5. Filter entries to reduce list size - scroll position resets appropriately

## Edge Cases Handled

- **Empty List**: When `filtered_items` is empty, `ListState` is set to `None` (no selection)
- **Filter Changes**: When filter changes and selected index becomes invalid, it resets to 0
- **Wrapping**: Navigation wraps around (e.g., pressing Up at top goes to bottom)

