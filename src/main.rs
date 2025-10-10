mod clipboard;
mod error;
mod events;
mod mock_data;
mod state;
mod types;
mod ui;

use crossterm::{
    execute,
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
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    
    result
}

async fn run() -> Result<()> {
    // Initialize application state with mock data
    let mut state = AppState::new();
    let mock_items = mock_data::generate_mock_data();
    state.load_items(mock_items);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

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

        // Poll for events with 100ms timeout for UI updates
        if let Ok(Some(key)) = event_handler.poll_event(Duration::from_millis(100)) {
            if let Some(action) = event_handler.handle_key(key, &state) {
                if !handle_action(action, &mut state, clipboard.as_mut()).await {
                    break;
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
    action: Action,
    state: &mut AppState,
    clipboard: Option<&mut clipboard::ClipboardManager>,
) -> bool {
    match action {
        Action::Quit => {
            return false;
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
                    if login.totp.is_some() {
                        // In a real implementation, this would generate the TOTP code
                        // For the prototype, we'll show a simulated code
                        let simulated_totp = "123456";
                        
                        if let Some(cb) = clipboard {
                            match cb.copy(simulated_totp) {
                                Ok(_) => {
                                    state.set_status(
                                        format!("✓ TOTP code copied: {}", simulated_totp),
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
                        state.set_status("✗ No TOTP configured for this entry", MessageLevel::Warning);
                    }
                }
            }
        }
        Action::Refresh => {
            // In a real implementation, this would sync with Bitwarden
            // For the prototype, we'll just show a message
            state.set_status("✓ Vault refreshed (mock data)", MessageLevel::Success);
        }
    }

    true
}

