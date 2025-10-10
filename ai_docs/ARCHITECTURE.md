# Architecture

## Overview

bwtui is designed as a lightweight wrapper around the official Bitwarden CLI, providing a fast and intuitive terminal interface for vault management.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         bwtui                                │
│                                                              │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │    CLI     │  │     TUI      │  │   State Manager   │   │
│  │  Handler   │  │   Renderer   │  │                   │   │
│  └─────┬──────┘  └──────┬───────┘  └─────────┬────────┘   │
│        │                │                     │             │
│        │         ┌──────▼──────┐             │             │
│        │         │   Event     │             │             │
│        │         │   Handler   │             │             │
│        │         └──────┬──────┘             │             │
│        │                │                     │             │
│  ┌─────▼────────────────▼─────────────────────▼────────┐   │
│  │              Application State                       │   │
│  │  - Vault entries (cached)                            │   │
│  │  - Filter state                                      │   │
│  │  - Selected entry                                    │   │
│  │  - UI state (ListState for scrolling, focus)         │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
└───────────────────────────┼──────────────────────────────────┘
                            │
                    ┌───────▼────────┐
                    │  Bitwarden CLI │
                    │      (bw)      │
                    └────────────────┘
```

## Component Breakdown

### 1. CLI Handler (`src/cli/`)

Responsible for all interactions with the Bitwarden CLI.

**Modules:**
- `mod.rs` - Main CLI interface
- `session.rs` - Session management and authentication
- `vault.rs` - Vault operations (list, get, sync)
- `commands.rs` - Command execution wrapper

**Key Responsibilities:**
- Execute `bw` commands
- Parse JSON responses
- Handle session state
- Error handling and recovery

**Design Pattern:**
```rust
pub struct BitwardenCli {
    session_token: Option<String>,
    vault_status: VaultStatus,
}

impl BitwardenCli {
    pub async fn list_items(&self) -> Result<Vec<VaultItem>>;
    pub async fn sync(&mut self) -> Result<()>;
    pub async fn get_totp(&self, id: &str) -> Result<String>;
    pub fn is_locked(&self) -> bool;
}
```

### 2. State Manager (`src/state/`)

Manages application state and business logic.

**Modules:**
- `mod.rs` - State container
- `filter.rs` - Filtering logic
- `cache.rs` - Vault cache management

**Key Responsibilities:**
- Maintain filtered vault entries
- Handle search/filter operations
- Track UI state (selection, scroll)
- Manage vault cache

**Design Pattern:**
```rust
pub struct AppState {
    vault_items: Vec<VaultItem>,
    filtered_items: Vec<VaultItem>,
    filter_query: String,
    selected_index: usize,
    scroll_offset: usize,
    vault_unlocked: bool,
}

impl AppState {
    pub fn apply_filter(&mut self, query: &str);
    pub fn select_next(&mut self);
    pub fn select_previous(&mut self);
    pub fn selected_item(&self) -> Option<&VaultItem>;
}
```

### 3. TUI Renderer (`src/ui/`)

Handles all terminal UI rendering using ratatui.

**Modules:**
- `mod.rs` - Main UI coordinator
- `list.rs` - Entry list widget
- `details.rs` - Entry details view
- `search.rs` - Search/filter input widget
- `status.rs` - Status bar widget
- `help.rs` - Help overlay

**Key Responsibilities:**
- Render all UI components
- Handle layout calculations
- Theme and styling
- Responsive design

**Design Pattern:**
```rust
pub struct UI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn render(&mut self, state: &AppState) -> Result<()>;
    fn render_list(&self, frame: &mut Frame, area: Rect, state: &AppState);
    fn render_search(&self, frame: &mut Frame, area: Rect, filter: &str);
    fn render_status(&self, frame: &mut Frame, area: Rect, status: &str);
}
```

### 4. Event Handler (`src/events/`)

Processes user input and system events.

**Modules:**
- `mod.rs` - Event loop
- `keyboard.rs` - Keyboard input handling
- `actions.rs` - Action definitions and dispatch

**Key Responsibilities:**
- Capture keyboard events
- Map keys to actions
- Dispatch actions to appropriate handlers
- Handle async events (clipboard, TOTP refresh)

**Design Pattern:**
```rust
pub enum Action {
    MoveUp,
    MoveDown,
    CopyUsername,
    CopyPassword,
    CopyTotp,
    UpdateFilter(String),
    Refresh,
    Quit,
}

pub struct EventHandler;

impl EventHandler {
    pub async fn handle(&self, event: Event, state: &mut AppState) -> Result<Action>;
}
```

### 5. Clipboard Manager (`src/clipboard/`)

Handles clipboard operations with auto-clear functionality.

**Modules:**
- `mod.rs` - Clipboard interface
- `timeout.rs` - Auto-clear timer

**Key Responsibilities:**
- Copy to system clipboard
- Auto-clear after timeout
- Cross-platform compatibility

## Data Flow

### Startup Sequence
```
1. main() -> Initialize terminal
2. Check bw CLI availability
3. Check vault status (locked/unlocked)
4. Load vault items (if unlocked)
5. Initialize UI
6. Enter event loop
```

### Event Processing Flow
```
User Input
    ↓
Event Handler (keyboard event)
    ↓
Action Dispatch
    ↓
State Update + CLI Operations (if needed)
    ↓
UI Re-render
```

### Filter Operation Flow
```
User types in search box
    ↓
Filter query updated in state
    ↓
Apply filter algorithm (fuzzy/exact match)
    ↓
Update filtered_items list
    ↓
Reset scroll and selection
    ↓
Re-render UI with filtered results
```

### Copy Operation Flow
```
User presses 'p' (copy password)
    ↓
Get selected entry
    ↓
Extract password from vault item
    ↓
Copy to clipboard
    ↓
Show status message
    ↓
Start auto-clear timer
    ↓
(After timeout) Clear clipboard
```

## Performance Considerations

### Caching Strategy
- Vault items cached in memory after first load
- Filtered results computed on-demand but cached
- Only refresh on explicit user action or TTL expiration

### Filtering Performance
- Use Rayon for parallel filtering on large vaults (>1000 items)
- Implement fuzzy matching with configurable sensitivity
- Debounce filter input (50ms) to avoid excessive re-computation

### UI Rendering
- Only render visible entries (virtual scrolling)
- Diff-based rendering (ratatui handles this)
- Lazy loading for entry details

### Async Operations
- All CLI operations are async (tokio)
- Non-blocking clipboard operations
- Background vault sync

## Security Architecture

### Session Management
- Session token stored only in memory (never written to disk)
- Token obtained from `BW_SESSION` env var or `bw unlock`
- Auto-lock after inactivity timeout

### Clipboard Security
- Clipboard cleared after configurable timeout (default: 30s)
- Option to disable clipboard history in OS settings
- No clipboard persistence

### Data Handling
- All vault data stays in memory only
- No local storage or caching to disk
- TOTP codes are fetched on-demand, never stored

### Process Security
- Minimal privileges required
- No network access (all via bw CLI)
- Clear separation of concerns

## Error Handling

### Error Categories
1. **CLI Errors** - bw command failures
2. **Session Errors** - Authentication/lock issues
3. **UI Errors** - Terminal rendering issues
4. **Clipboard Errors** - Copy operation failures

### Error Recovery Strategy
- Graceful degradation (show error, maintain state)
- Automatic retry for transient failures
- User-friendly error messages
- Detailed logging for debugging

## Testing Strategy

### Unit Tests
- Filter logic
- State management
- CLI command parsing
- Error handling

### Integration Tests
- CLI interaction (mocked)
- Event handling pipeline
- State transitions

### Manual Testing
- Cross-platform compatibility
- Performance with large vaults
- Edge cases (empty vault, no session, etc.)

## Future Enhancements

### Phase 2
- Entry editing
- Create new entries
- Password generator
- Multi-vault support

### Phase 3
- Attachment viewing
- Advanced search (tags, folders)
- Custom themes
- Plugin system

### Phase 4
- Mouse support (optional)
- Split-pane view
- Quick commands (vim-style)
- Integration with other password managers

