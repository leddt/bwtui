# Implementation Plan

## Phase 1: Foundation (Week 1)

### 1.1 Project Setup
- [ ] Initialize Rust project with Cargo
- [ ] Add dependencies to `Cargo.toml`
- [ ] Set up project structure
- [ ] Configure development environment

**Dependencies:**
```toml
[dependencies]
# TUI
ratatui = "0.24"
crossterm = "0.27"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI integration
tokio-process = "0.2"

# Clipboard
arboard = "3.3"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
chrono = "0.4"
uuid = { version = "1.6", features = ["serde"] }

# Filtering
fuzzy-matcher = "0.3"
rayon = "1.8"

# Configuration
config = "0.13"
directories = "5.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Directory Structure:**
```
bwtui/
├── src/
│   ├── main.rs
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── session.rs
│   │   ├── vault.rs
│   │   └── commands.rs
│   ├── state/
│   │   ├── mod.rs
│   │   ├── filter.rs
│   │   └── cache.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── list.rs
│   │   ├── details.rs
│   │   ├── search.rs
│   │   ├── status.rs
│   │   └── help.rs
│   ├── events/
│   │   ├── mod.rs
│   │   ├── keyboard.rs
│   │   └── actions.rs
│   ├── clipboard/
│   │   ├── mod.rs
│   │   └── timeout.rs
│   ├── config.rs
│   ├── error.rs
│   └── types.rs
├── tests/
│   ├── cli_tests.rs
│   ├── filter_tests.rs
│   └── integration_tests.rs
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md
└── IMPLEMENTATION.md
```

### 1.2 Core Types & Data Models

**File: `src/types.rs`**
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub login: Option<LoginData>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    pub revision_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    #[serde(rename = "1")]
    Login,
    #[serde(rename = "2")]
    SecureNote,
    #[serde(rename = "3")]
    Card,
    #[serde(rename = "4")]
    Identity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub uris: Option<Vec<Uri>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uri {
    #[serde(rename = "uri")]
    pub value: String,
    #[serde(rename = "match")]
    pub match_type: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VaultStatus {
    Locked,
    Unlocked,
    LoggedOut,
}
```

### 1.3 Error Handling

**File: `src/error.rs`**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BwError {
    #[error("Bitwarden CLI not found. Please install 'bw' CLI")]
    CliNotFound,
    
    #[error("Vault is locked. Please unlock with 'bw unlock'")]
    VaultLocked,
    
    #[error("Not logged in. Please run 'bw login'")]
    NotLoggedIn,
    
    #[error("Session expired. Please unlock vault again")]
    SessionExpired,
    
    #[error("Failed to execute bw command: {0}")]
    CommandFailed(String),
    
    #[error("Failed to parse CLI output: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, BwError>;
```

## Phase 2: CLI Integration (Week 1-2)

### 2.1 Bitwarden CLI Wrapper

**File: `src/cli/mod.rs`**
```rust
pub mod commands;
pub mod session;
pub mod vault;

use crate::error::{BwError, Result};
use crate::types::{VaultStatus, VaultItem};
use tokio::process::Command;

pub struct BitwardenCli {
    session_token: Option<String>,
    pub status: VaultStatus,
}

impl BitwardenCli {
    pub async fn new() -> Result<Self> {
        // Check if bw CLI is available
        let output = Command::new("bw")
            .arg("--version")
            .output()
            .await
            .map_err(|_| BwError::CliNotFound)?;
        
        if !output.status.success() {
            return Err(BwError::CliNotFound);
        }
        
        let mut cli = Self {
            session_token: None,
            status: VaultStatus::LoggedOut,
        };
        
        cli.check_status().await?;
        Ok(cli)
    }
    
    pub async fn check_status(&mut self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

**File: `src/cli/vault.rs`**
```rust
impl BitwardenCli {
    pub async fn list_items(&self) -> Result<Vec<VaultItem>> {
        if self.status != VaultStatus::Unlocked {
            return Err(BwError::VaultLocked);
        }
        
        let mut cmd = Command::new("bw");
        cmd.arg("list").arg("items");
        
        if let Some(token) = &self.session_token {
            cmd.env("BW_SESSION", token);
        }
        
        let output = cmd.output().await
            .map_err(|e| BwError::CommandFailed(e.to_string()))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BwError::CommandFailed(stderr.to_string()));
        }
        
        let items: Vec<VaultItem> = serde_json::from_slice(&output.stdout)
            .map_err(|e| BwError::ParseError(e.to_string()))?;
        
        Ok(items)
    }
    
    pub async fn get_totp(&self, id: &str) -> Result<String> {
        // Implementation
        Ok(String::new())
    }
    
    pub async fn sync(&mut self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

**Implementation Tasks:**
- [ ] Implement `BitwardenCli::new()` with CLI detection
- [ ] Implement `check_status()` to detect lock state
- [ ] Implement `list_items()` to fetch vault entries
- [ ] Implement `get_totp()` to retrieve TOTP codes
- [ ] Implement `sync()` to refresh vault data
- [ ] Add session token management
- [ ] Add error handling for common failure cases
- [ ] Write unit tests for CLI wrapper

## Phase 3: State Management (Week 2)

### 3.1 Application State

**File: `src/state/mod.rs`**
```rust
pub mod filter;
pub mod cache;

use crate::types::VaultItem;

#[derive(Debug)]
pub struct AppState {
    pub vault_items: Vec<VaultItem>,
    pub filtered_items: Vec<VaultItem>,
    pub filter_query: String,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub vault_unlocked: bool,
    pub input_mode: InputMode,
    pub status_message: Option<StatusMessage>,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Filtering,
}

#[derive(Debug)]
pub struct StatusMessage {
    pub text: String,
    pub level: MessageLevel,
    pub timestamp: std::time::Instant,
}

#[derive(Debug)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault_items: Vec::new(),
            filtered_items: Vec::new(),
            filter_query: String::new(),
            selected_index: 0,
            scroll_offset: 0,
            vault_unlocked: false,
            input_mode: InputMode::Normal,
            status_message: None,
        }
    }
    
    pub fn load_items(&mut self, items: Vec<VaultItem>) {
        self.vault_items = items;
        self.apply_filter();
    }
    
    pub fn apply_filter(&mut self) {
        // Implementation in filter.rs
    }
    
    pub fn selected_item(&self) -> Option<&VaultItem> {
        self.filtered_items.get(self.selected_index)
    }
    
    pub fn select_next(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
        }
    }
    
    pub fn select_previous(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}
```

### 3.2 Filter Logic

**File: `src/state/filter.rs`**
```rust
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::types::VaultItem;

pub struct Filter {
    matcher: SkimMatcherV2,
    case_sensitive: bool,
    fuzzy_enabled: bool,
}

impl Filter {
    pub fn new(case_sensitive: bool, fuzzy_enabled: bool) -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
            case_sensitive,
            fuzzy_enabled,
        }
    }
    
    pub fn apply(&self, items: &[VaultItem], query: &str) -> Vec<VaultItem> {
        if query.is_empty() {
            return items.to_vec();
        }
        
        let query = if self.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        
        items.iter()
            .filter(|item| self.matches(item, &query))
            .cloned()
            .collect()
    }
    
    fn matches(&self, item: &VaultItem, query: &str) -> bool {
        let searchable_text = self.get_searchable_text(item);
        
        if self.fuzzy_enabled {
            self.matcher.fuzzy_match(&searchable_text, query).is_some()
        } else {
            searchable_text.contains(query)
        }
    }
    
    fn get_searchable_text(&self, item: &VaultItem) -> String {
        let mut text = if self.case_sensitive {
            item.name.clone()
        } else {
            item.name.to_lowercase()
        };
        
        if let Some(login) = &item.login {
            if let Some(username) = &login.username {
                text.push(' ');
                text.push_str(if self.case_sensitive {
                    username
                } else {
                    &username.to_lowercase()
                });
            }
            
            if let Some(uris) = &login.uris {
                for uri in uris {
                    text.push(' ');
                    text.push_str(if self.case_sensitive {
                        &uri.value
                    } else {
                        &uri.value.to_lowercase()
                    });
                }
            }
        }
        
        text
    }
}
```

**Implementation Tasks:**
- [ ] Implement basic filtering logic
- [ ] Add fuzzy matching support
- [ ] Implement case-sensitive/insensitive options
- [ ] Optimize for large vaults (>1000 items)
- [ ] Add multi-field search (name, domain, username)
- [ ] Write comprehensive filter tests

## Phase 4: UI Implementation (Week 2-3)

### 4.1 Main UI Structure

**File: `src/ui/mod.rs`**
```rust
pub mod list;
pub mod details;
pub mod search;
pub mod status;
pub mod help;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::Stdout;
use crate::state::AppState;
use crate::error::Result;

pub struct UI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
    
    pub fn render(&mut self, state: &AppState) -> Result<()> {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Search box
                    Constraint::Min(0),      // Entry list
                    Constraint::Length(2),   // Status bar
                ])
                .split(frame.size());
            
            search::render(frame, chunks[0], state);
            list::render(frame, chunks[1], state);
            status::render(frame, chunks[2], state);
        })?;
        
        Ok(())
    }
    
    pub fn cleanup(&mut self) -> Result<()> {
        // Restore terminal state
        Ok(())
    }
}
```

### 4.2 Entry List Widget

**File: `src/ui/list.rs`**
```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
        .filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let style = if idx == state.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            let username = item.login
                .as_ref()
                .and_then(|l| l.username.as_ref())
                .map(|u| format!(" ({})", u))
                .unwrap_or_default();
            
            let content = format!("{}{}", item.name, username);
            
            ListItem::new(Line::from(vec![
                Span::styled(content, style),
            ]))
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Vault Entries ({}) ", state.filtered_items.len()))
        );
    
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_index));
    
    frame.render_stateful_widget(list, area, &mut list_state);
}
```

### 4.3 Search Widget

**File: `src/ui/search.rs`**
```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::state::{AppState, InputMode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = if state.input_mode == InputMode::Filtering {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let paragraph = Paragraph::new(state.filter_query.as_str())
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Filter (press / to focus) ")
                .border_style(style)
        );
    
    frame.render_widget(paragraph, area);
}
```

**Implementation Tasks:**
- [x] Implement main UI layout
- [x] Create entry list widget with scrolling
- [x] Create search/filter input widget
- [x] Create status bar with keybindings hint
- [ ] Create help overlay (press '?')
- [ ] Add syntax highlighting for domains
- [x] Implement responsive layout
- [ ] Add loading indicators
- [ ] Test UI rendering on different terminal sizes

## Phase 5: Event Handling (Week 3)

### 5.1 Event Loop

**File: `src/events/mod.rs`**
```rust
pub mod keyboard;
pub mod actions;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc;
use std::time::Duration;
use crate::error::Result;

pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap() {
                    if let Ok(CrosstermEvent::Key(key)) = event::read() {
                        tx.send(Event::Key(key)).unwrap();
                    }
                } else {
                    tx.send(Event::Tick).unwrap();
                }
            }
        });
        
        Self { rx }
    }
    
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
```

### 5.2 Keyboard Handling

**File: `src/events/keyboard.rs`**
```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::events::actions::Action;
use crate::state::{AppState, InputMode};

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<Action> {
    match state.input_mode {
        InputMode::Normal => handle_normal_mode(key),
        InputMode::Filtering => handle_filter_mode(key),
    }
}

fn handle_normal_mode(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        // Navigation
        (KeyCode::Up, _) => Some(Action::MoveUp),
        (KeyCode::Down, _) => Some(Action::MoveDown),
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(Action::MoveUp),
        (KeyCode::Char('j'), KeyModifiers::CONTROL) => Some(Action::MoveDown),
        (KeyCode::PageUp, _) => Some(Action::PageUp),
        (KeyCode::PageDown, _) => Some(Action::PageDown),
        (KeyCode::Home, _) => Some(Action::Home),
        (KeyCode::End, _) => Some(Action::End),
        
        // Actions
        (KeyCode::Char('u'), _) => Some(Action::CopyUsername),
        (KeyCode::Char('p'), _) => Some(Action::CopyPassword),
        (KeyCode::Char('t'), _) => Some(Action::CopyTotp),
        (KeyCode::Enter, _) => Some(Action::ViewDetails),
        (KeyCode::Char('r'), _) => Some(Action::Refresh),
        
        // Filter
        (KeyCode::Char('/'), _) => Some(Action::EnterFilterMode),
        (KeyCode::Char('f'), KeyModifiers::CONTROL) => Some(Action::EnterFilterMode),
        
        // Quit
        (KeyCode::Char('q'), _) => Some(Action::Quit),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
        
        _ => None,
    }
}

fn handle_filter_mode(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::ExitFilterMode),
        KeyCode::Enter => Some(Action::ExitFilterMode),
        KeyCode::Char(c) => Some(Action::AppendFilter(c)),
        KeyCode::Backspace => Some(Action::DeleteFilterChar),
        _ => None,
    }
}
```

**Implementation Tasks:**
- [ ] Implement event loop with crossterm
- [ ] Implement keyboard event handling
- [ ] Map keys to actions
- [ ] Handle input modes (normal vs filtering)
- [ ] Add Ctrl+J/K navigation support
- [ ] Test all keyboard shortcuts
- [ ] Add configurable keybindings support

## Phase 6: Clipboard & TOTP (Week 3)

### 6.1 Clipboard Manager

**File: `src/clipboard/mod.rs`**
```rust
pub mod timeout;

use arboard::Clipboard;
use tokio::time::{sleep, Duration};
use crate::error::{BwError, Result};

pub struct ClipboardManager {
    clipboard: Clipboard,
    timeout_seconds: u64,
}

impl ClipboardManager {
    pub fn new(timeout_seconds: u64) -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| BwError::ClipboardError(e.to_string()))?;
        
        Ok(Self {
            clipboard,
            timeout_seconds,
        })
    }
    
    pub async fn copy_with_timeout(&mut self, text: String) -> Result<()> {
        self.clipboard
            .set_text(&text)
            .map_err(|e| BwError::ClipboardError(e.to_string()))?;
        
        if self.timeout_seconds > 0 {
            let timeout = self.timeout_seconds;
            tokio::spawn(async move {
                sleep(Duration::from_secs(timeout)).await;
                if let Ok(mut clip) = Clipboard::new() {
                    let _ = clip.set_text("");
                }
            });
        }
        
        Ok(())
    }
}
```

**Implementation Tasks:**
- [ ] Implement clipboard copy functionality
- [ ] Implement auto-clear timer
- [ ] Add cross-platform clipboard support
- [ ] Handle clipboard errors gracefully
- [ ] Test clipboard operations on Windows, Linux, macOS

## Phase 7: Configuration (Week 4)

### 7.1 Configuration File

**File: `src/config.rs`**
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::error::{BwError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub clipboard_timeout: u64,
    pub auto_lock_minutes: u64,
    pub show_password_strength: bool,
    pub entries_per_page: usize,
    pub case_sensitive: bool,
    pub fuzzy_matching: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clipboard_timeout: 30,
            auto_lock_minutes: 15,
            show_password_strength: true,
            entries_per_page: 20,
            case_sensitive: false,
            fuzzy_matching: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| BwError::ConfigError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| BwError::ConfigError(e.to_string()))
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| BwError::ConfigError(e.to_string()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| BwError::ConfigError(e.to_string()))?;
        
        std::fs::write(&config_path, content)
            .map_err(|e| BwError::ConfigError(e.to_string()))?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        ProjectDirs::from("", "", "bwtui")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .ok_or_else(|| BwError::ConfigError("Could not determine config directory".to_string()))
    }
}
```

**Implementation Tasks:**
- [ ] Implement configuration file loading
- [ ] Create default configuration
- [ ] Add configuration validation
- [ ] Document all configuration options
- [ ] Test configuration on different platforms

## Phase 8: Main Application (Week 4)

### 8.1 Main Entry Point

**File: `src/main.rs`**
```rust
mod cli;
mod state;
mod ui;
mod events;
mod clipboard;
mod config;
mod error;
mod types;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = config::Config::load()?;
    
    // Initialize Bitwarden CLI
    let mut bw_cli = cli::BitwardenCli::new().await?;
    
    // Check if vault is unlocked
    if bw_cli.status != types::VaultStatus::Unlocked {
        eprintln!("Vault is locked. Please unlock it first:");
        eprintln!("  bw unlock");
        eprintln!("Then export the session token:");
        eprintln!("  export BW_SESSION=\"...\"");
        return Ok(());
    }
    
    // Initialize application state
    let mut state = state::AppState::new();
    state.vault_unlocked = true;
    
    // Load vault items
    let items = bw_cli.list_items().await?;
    state.load_items(items);
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    // Initialize UI
    let mut ui = ui::UI::new()?;
    
    // Initialize clipboard
    let mut clipboard = clipboard::ClipboardManager::new(config.clipboard_timeout)?;
    
    // Initialize event handler
    let mut event_handler = events::EventHandler::new();
    
    // Main event loop
    loop {
        ui.render(&state)?;
        
        if let Some(event) = event_handler.next().await {
            if let events::Event::Key(key) = event {
                if let Some(action) = events::keyboard::handle_key(key, &state) {
                    if !handle_action(action, &mut state, &mut bw_cli, &mut clipboard).await? {
                        break;
                    }
                }
            }
        }
    }
    
    // Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    
    Ok(())
}

async fn handle_action(
    action: events::actions::Action,
    state: &mut state::AppState,
    bw_cli: &mut cli::BitwardenCli,
    clipboard: &mut clipboard::ClipboardManager,
) -> Result<bool> {
    use events::actions::Action;
    
    match action {
        Action::Quit => return Ok(false),
        Action::MoveUp => state.select_previous(),
        Action::MoveDown => state.select_next(),
        Action::CopyUsername => {
            if let Some(item) = state.selected_item() {
                if let Some(username) = item.login.as_ref().and_then(|l| l.username.as_ref()) {
                    clipboard.copy_with_timeout(username.clone()).await?;
                    state.set_status("Username copied to clipboard", state::MessageLevel::Success);
                }
            }
        }
        Action::CopyPassword => {
            if let Some(item) = state.selected_item() {
                if let Some(password) = item.login.as_ref().and_then(|l| l.password.as_ref()) {
                    clipboard.copy_with_timeout(password.clone()).await?;
                    state.set_status("Password copied to clipboard", state::MessageLevel::Success);
                }
            }
        }
        Action::CopyTotp => {
            if let Some(item) = state.selected_item() {
                let totp = bw_cli.get_totp(&item.id).await?;
                clipboard.copy_with_timeout(totp).await?;
                state.set_status("TOTP code copied to clipboard", state::MessageLevel::Success);
            }
        }
        Action::Refresh => {
            bw_cli.sync().await?;
            let items = bw_cli.list_items().await?;
            state.load_items(items);
            state.set_status("Vault refreshed", state::MessageLevel::Success);
        }
        _ => { /* Handle other actions */ }
    }
    
    Ok(true)
}
```

**Implementation Tasks:**
- [ ] Implement main application loop
- [ ] Wire up all components
- [ ] Add graceful shutdown handling
- [ ] Implement error display in UI
- [ ] Add status messages
- [ ] Test end-to-end functionality

## Phase 9: Testing & Polish (Week 4-5)

### 9.1 Testing

**Unit Tests:**
- [ ] CLI wrapper tests
- [ ] Filter logic tests
- [ ] State management tests
- [ ] Clipboard tests

**Integration Tests:**
- [ ] Full event handling pipeline
- [ ] UI rendering tests
- [ ] Configuration loading tests

**Manual Testing:**
- [ ] Test on Windows
- [ ] Test on Linux
- [ ] Test on macOS
- [ ] Test with large vaults (1000+ entries)
- [ ] Test with empty vault
- [ ] Test with locked vault
- [ ] Test clipboard timeout
- [ ] Test all keyboard shortcuts

### 9.2 Performance Optimization

- [ ] Profile filtering performance
- [ ] Optimize UI rendering for large lists
- [ ] Add virtual scrolling if needed
- [ ] Benchmark startup time
- [ ] Optimize memory usage

### 9.3 Documentation

- [ ] Add code comments
- [ ] Write usage examples
- [ ] Create troubleshooting guide
- [ ] Document all keyboard shortcuts
- [ ] Add contribution guidelines

### 9.4 Polish

- [ ] Add loading spinner
- [ ] Improve error messages
- [ ] Add better status indicators
- [ ] Add entry count display
- [ ] Add vault sync timestamp
- [ ] Add password strength indicator
- [ ] Improve color scheme
- [ ] Add keyboard shortcut help overlay

## Phase 10: Release (Week 5)

### 10.1 Pre-release Checklist

- [ ] All tests passing
- [ ] Documentation complete
- [ ] README with clear installation instructions
- [ ] LICENSE file added
- [ ] CHANGELOG created
- [ ] Version number set in Cargo.toml

### 10.2 Build & Release

- [ ] Build release binaries for:
  - [ ] Windows (x86_64)
  - [ ] Linux (x86_64)
  - [ ] macOS (x86_64)
  - [ ] macOS (aarch64)
- [ ] Create GitHub release
- [ ] Publish to crates.io (optional)
- [ ] Create Homebrew formula (optional)
- [ ] Create AUR package (optional)

### 10.3 Post-release

- [ ] Monitor for issues
- [ ] Gather user feedback
- [ ] Plan future enhancements
- [ ] Set up CI/CD pipeline

## Estimated Timeline

- **Week 1**: Project setup, CLI integration foundation
- **Week 2**: State management, filtering, UI basics
- **Week 3**: Event handling, full UI, clipboard
- **Week 4**: Configuration, main app, testing
- **Week 5**: Polish, documentation, release

**Total**: ~5 weeks for MVP

## Future Enhancements (Post-MVP)

### Version 1.1
- Entry editing
- Create new entries
- Delete entries
- Password generator

### Version 1.2
- Folder navigation
- Tags support
- Advanced search
- Custom themes

### Version 1.3
- Attachment viewing
- Multi-account support
- Import/export
- Backup functionality

### Version 2.0
- Plugin system
- Mouse support
- Split-pane view
- Vim-style commands

