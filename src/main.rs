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
    // Initialize Bitwarden CLI
    let bw_cli = match cli::BitwardenCli::new().await {
        Ok(cli) => cli,
        Err(error::BwError::CliNotFound) => {
            eprintln!("❌ Error: Bitwarden CLI not found");
            eprintln!();
            eprintln!("Please install the Bitwarden CLI:");
            eprintln!("  • npm install -g @bitwarden/cli");
            eprintln!("  • Or download from: https://bitwarden.com/help/cli/");
            std::process::exit(1);
        }
        Err(e) => return Err(e),
    };

    // Check vault status
    let status = match bw_cli.check_status().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Error checking vault status: {}", e);
            std::process::exit(1);
        }
    };

    // Ensure vault is unlocked
    if status != cli::VaultStatus::Unlocked {
        eprintln!("❌ Vault is not unlocked");
        eprintln!();
        match status {
            cli::VaultStatus::Unauthenticated => {
                eprintln!("You need to log in first:");
                eprintln!("  bw login");
            }
            cli::VaultStatus::Locked => {
                eprintln!("You need to unlock your vault:");
                eprintln!("  bw unlock");
                eprintln!();
                eprintln!("Then set the session token:");
                eprintln!("  export BW_SESSION=\"your-session-token\"");
            }
            _ => {}
        }
        std::process::exit(1);
    }

    // Load vault items
    let items = match bw_cli.list_items().await {
        Ok(items) => items,
        Err(e) => {
            eprintln!("❌ Error loading vault items: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize application state
    let mut state = AppState::new();
    state.load_items(items);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

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

    // Main event loop
    loop {
        // Clear old status messages
        state.expire_old_status();

        // Render UI
        ui.render(&mut state)?;

        // Poll for events with 250ms timeout for UI updates
        // This ensures TOTP countdown and other time-based displays refresh smoothly
        if let Ok(Some(action)) = event_handler.poll_event(Duration::from_millis(250), &state) {
            if !handle_action(action, &mut state, &bw_cli, clipboard.as_mut()).await {
                break;
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())
}

async fn handle_action(
    action: Action,
    state: &mut AppState,
    bw_cli: &cli::BitwardenCli,
    clipboard: Option<&mut clipboard::ClipboardManager>,
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
            state.set_status("⟳ Syncing with Bitwarden server...", MessageLevel::Info);
            
            match bw_cli.sync().await {
                Ok(_) => {
                    match bw_cli.list_items().await {
                        Ok(items) => {
                            state.load_items(items);
                            state.set_status("✓ Vault synced successfully", MessageLevel::Success);
                        }
                        Err(e) => {
                            state.set_status(
                                format!("✗ Failed to load items: {}", e),
                                MessageLevel::Error,
                            );
                        }
                    }
                }
                Err(e) => {
                    state.set_status(
                        format!("✗ Sync failed: {}", e),
                        MessageLevel::Error,
                    );
                }
            }
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

