mod cli;
mod clipboard;
mod error;
mod events;
mod state;
mod totp_util;
mod types;
mod ui;

use crossterm::{
    execute,
    event::{EnableMouseCapture, DisableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Result;
use events::{Action, EventHandler};
use state::{AppState, MessageLevel};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Run the application and handle cleanup
    let result = run().await;
    
    // Ensure terminal is restored
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    
    result
}

async fn run() -> Result<()> {
    // Setup terminal FIRST - show UI immediately
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Initialize application state (start empty, will load items async)
    let mut state = AppState::new();

    // Initialize UI
    let mut ui = ui::UI::new()?;

    // Initialize clipboard
    let mut clipboard = match clipboard::ClipboardManager::new() {
        Ok(cb) => Some(cb),
        Err(_) => {
            state.set_status("Warning: Clipboard not available", MessageLevel::Warning);
            None
        }
    };

    // Initialize event handler
    let event_handler = EventHandler::new();

    // Create channel for sync results
    let (sync_tx, mut sync_rx) = mpsc::unbounded_channel::<SyncResult>();
    
    // Create channel for CLI initialization result
    let (cli_tx, mut cli_rx) = mpsc::unbounded_channel::<Result<cli::BitwardenCli>>();

    // Start Bitwarden CLI initialization and vault loading in background
    state.start_sync();
    let sync_tx_clone = sync_tx.clone();
    tokio::spawn(async move {
        // Initialize Bitwarden CLI
        let bw_cli = match cli::BitwardenCli::new().await {
            Ok(cli) => cli,
            Err(error::BwError::CliNotFound) => {
                let _ = sync_tx_clone.send(SyncResult::Error(
                    "Bitwarden CLI not found. Please install: npm install -g @bitwarden/cli".to_string()
                ));
                return;
            }
            Err(e) => {
                let _ = sync_tx_clone.send(SyncResult::Error(format!("CLI error: {}", e)));
                return;
            }
        };

        // Check vault status
        let status = match bw_cli.check_status().await {
            Ok(s) => s,
            Err(e) => {
                let _ = sync_tx_clone.send(SyncResult::Error(format!("Failed to check vault status: {}", e)));
                return;
            }
        };

        // Ensure vault is unlocked
        if status != cli::VaultStatus::Unlocked {
            let error_msg = match status {
                cli::VaultStatus::Unauthenticated => {
                    "Vault not logged in. Please run: bw login"
                }
                cli::VaultStatus::Locked => {
                    "Vault is locked. Please run: bw unlock"
                }
                _ => "Vault is not accessible"
            };
            let _ = sync_tx_clone.send(SyncResult::Error(error_msg.to_string()));
            return;
        }

        // Send CLI instance to main thread
        let _ = cli_tx.send(Ok(bw_cli.clone()));

        // Load vault items
        let result = match bw_cli.list_items().await {
            Ok(items) => SyncResult::Success(items),
            Err(e) => SyncResult::Error(format!("Failed to load vault items: {}", e)),
        };
        let _ = sync_tx_clone.send(result);
    });

    // We'll store the CLI instance once it's initialized
    let mut bw_cli: Option<cli::BitwardenCli> = None;

    // Main event loop
    loop {
        // Clear old status messages
        state.expire_old_status();

        // Advance sync animation
        state.advance_sync_animation();

        // Check for CLI initialization result
        if let Ok(result) = cli_rx.try_recv() {
            match result {
                Ok(cli) => {
                    bw_cli = Some(cli);
                }
                Err(e) => {
                    state.set_status(format!("✗ {}", e), MessageLevel::Error);
                }
            }
        }

        // Check for sync results
        if let Ok(result) = sync_rx.try_recv() {
            handle_sync_result(result, &mut state);
        }

        // Render UI
        ui.render(&mut state)?;

        // Poll for events with 100ms timeout for smoother animation
        // This ensures TOTP countdown and sync animation display refresh smoothly
        if let Ok(Some(action)) = event_handler.poll_event(Duration::from_millis(100), &state) {
            // Always allow Quit
            if matches!(action, Action::Quit) {
                break;
            }
            
            if let Some(ref cli) = bw_cli {
                // CLI is ready, handle all actions
                if !handle_action(action, &mut state, cli, clipboard.as_mut(), sync_tx.clone()).await {
                    break;
                }
            } else {
                // CLI not ready yet - only allow navigation and UI actions
                match action {
                    Action::MoveUp | Action::MoveDown | Action::PageUp | Action::PageDown 
                    | Action::Home | Action::End | Action::SelectIndex(_) 
                    | Action::SelectIndexAndShowDetails(_) | Action::AppendFilter(_) 
                    | Action::DeleteFilterChar | Action::ClearFilter | Action::ToggleDetailsPanel 
                    | Action::OpenDetailsPanel | Action::Tick => {
                        // These actions don't need CLI, handle them directly
                        match action {
                            Action::MoveUp => state.select_previous(),
                            Action::MoveDown => state.select_next(),
                            Action::PageUp => state.page_up(10),
                            Action::PageDown => state.page_down(10),
                            Action::Home => state.jump_to_start(),
                            Action::End => state.jump_to_end(),
                            Action::SelectIndex(idx) => state.select_index(idx),
                            Action::SelectIndexAndShowDetails(idx) => {
                                state.select_index(idx);
                                if !state.details_panel_visible {
                                    state.toggle_details_panel();
                                }
                            }
                            Action::AppendFilter(c) => state.append_filter(c),
                            Action::DeleteFilterChar => state.delete_filter_char(),
                            Action::ClearFilter => state.clear_filter(),
                            Action::ToggleDetailsPanel => state.toggle_details_panel(),
                            Action::OpenDetailsPanel => {
                                if !state.details_panel_visible {
                                    state.toggle_details_panel();
                                }
                            }
                            Action::Tick => {},
                            _ => {}
                        }
                    }
                    _ => {
                        // Actions that need CLI
                        state.set_status("⏳ Please wait, initializing...", MessageLevel::Warning);
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

// Result type for sync operations
enum SyncResult {
    Success(Vec<types::VaultItem>),
    Error(String),
}

fn handle_sync_result(result: SyncResult, state: &mut AppState) {
    state.stop_sync();
    match result {
        SyncResult::Success(items) => {
            state.load_items(items);
            state.set_status("✓ Vault synced successfully", MessageLevel::Success);
        }
        SyncResult::Error(error) => {
            state.set_status(
                format!("✗ Sync failed: {}", error),
                MessageLevel::Error,
            );
        }
    }
}

async fn handle_action(
    action: Action,
    state: &mut AppState,
    bw_cli: &cli::BitwardenCli,
    clipboard: Option<&mut clipboard::ClipboardManager>,
    sync_tx: mpsc::UnboundedSender<SyncResult>,
) -> bool {
    match action {
        Action::Quit => {
            return false;
        }
        Action::Tick => {
            // Periodic tick for UI updates (TOTP countdown, etc.)
            // No action needed, just triggers a render
        }
        
        // Navigation
        Action::MoveUp => {
            state.select_previous();
        }
        Action::MoveDown => {
            state.select_next();
        }
        Action::PageUp => {
            state.page_up(10);
        }
        Action::PageDown => {
            state.page_down(10);
        }
        Action::Home => {
            state.jump_to_start();
        }
        Action::End => {
            state.jump_to_end();
        }
        Action::SelectIndex(index) => {
            state.select_index(index);
        }
        Action::SelectIndexAndShowDetails(index) => {
            state.select_index(index);
            if !state.details_panel_visible {
                state.toggle_details_panel();
            }
        }
        
        // Filter actions
        Action::AppendFilter(c) => {
            state.append_filter(c);
        }
        Action::DeleteFilterChar => {
            state.delete_filter_char();
        }
        Action::ClearFilter => {
            state.clear_filter();
        }
        
        // Copy actions
        Action::CopyUsername => {
            if let Some(item) = state.selected_item() {
                if let Some(username) = item.username() {
                    if let Some(cb) = clipboard {
                        match cb.copy(username) {
                            Ok(_) => {
                                state.set_status(
                                    format!("✓ Username copied: {}", username),
                                    MessageLevel::Success,
                                );
                            }
                            Err(_) => {
                                state.set_status(
                                    "✗ Failed to copy to clipboard",
                                    MessageLevel::Error,
                                );
                            }
                        }
                    } else {
                        state.set_status(
                            "✗ Clipboard not available",
                            MessageLevel::Error,
                        );
                    }
                } else {
                    state.set_status("✗ No username for this entry", MessageLevel::Warning);
                }
            }
        }
        Action::CopyPassword => {
            if let Some(item) = state.selected_item() {
                if let Some(login) = &item.login {
                    if let Some(password) = &login.password {
                        if let Some(cb) = clipboard {
                            match cb.copy(password) {
                                Ok(_) => {
                                    state.set_status(
                                        "✓ Password copied to clipboard (hidden for security)",
                                        MessageLevel::Success,
                                    );
                                }
                                Err(_) => {
                                    state.set_status(
                                        "✗ Failed to copy to clipboard",
                                        MessageLevel::Error,
                                    );
                                }
                            }
                        } else {
                            state.set_status(
                                "✗ Clipboard not available",
                                MessageLevel::Error,
                            );
                        }
                    } else {
                        state.set_status("✗ No password for this entry", MessageLevel::Warning);
                    }
                }
            }
        }
        Action::CopyTotp => {
            if let Some(item) = state.selected_item() {
                if let Some(login) = &item.login {
                    if let Some(totp_secret) = &login.totp {
                        // Generate TOTP locally (much faster than CLI)
                        match totp_util::generate_totp(totp_secret) {
                            Ok((code, _remaining)) => {
                                if let Some(cb) = clipboard {
                                    match cb.copy(&code) {
                                        Ok(_) => {
                                            state.set_status(
                                                format!("✓ TOTP code copied: {}", code),
                                                MessageLevel::Success,
                                            );
                                        }
                                        Err(_) => {
                                            state.set_status(
                                                "✗ Failed to copy to clipboard",
                                                MessageLevel::Error,
                                            );
                                        }
                                    }
                                } else {
                                    state.set_status(
                                        "✗ Clipboard not available",
                                        MessageLevel::Error,
                                    );
                                }
                            }
                            Err(e) => {
                                state.set_status(
                                    format!("✗ Failed to generate TOTP: {}", e),
                                    MessageLevel::Error,
                                );
                            }
                        }
                    } else {
                        state.set_status("✗ No TOTP configured for this entry", MessageLevel::Warning);
                    }
                }
            }
        }
        Action::Refresh => {
            // Don't start a new sync if one is already in progress
            if state.syncing {
                state.set_status("⟳ Sync already in progress...", MessageLevel::Warning);
                return true;
            }
            
            state.start_sync();
            // Don't show status message here - only show the result when done
            
            // Clone what we need for the background task
            let bw_cli_clone = bw_cli.clone();
            let sync_tx_clone = sync_tx.clone();
            
            // Spawn sync operation in background
            tokio::spawn(async move {
                let result = match bw_cli_clone.sync().await {
                    Ok(_) => {
                        match bw_cli_clone.list_items().await {
                            Ok(items) => SyncResult::Success(items),
                            Err(e) => SyncResult::Error(format!("Failed to load items: {}", e)),
                        }
                    }
                    Err(e) => SyncResult::Error(e.to_string()),
                };
                
                // Send result back to main thread
                let _ = sync_tx_clone.send(result);
            });
        }
        Action::ToggleDetailsPanel => {
            state.toggle_details_panel();
        }
        Action::OpenDetailsPanel => {
            if !state.details_panel_visible {
                state.toggle_details_panel();
            }
        }
    }

    true
}

