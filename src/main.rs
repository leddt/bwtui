mod cache;
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

    // Try to load cache immediately for instant UI population
    match cache::load_cache() {
        Ok(Some(cached_data)) => {
            let cached_items = cached_data.to_vault_items();
            state.load_cached_items(cached_items);
            state.set_status(
                format!("✓ Loaded {} items from cache (syncing in background...)", cached_data.items.len()),
                MessageLevel::Info,
            );
        }
        Ok(None) => {
            // No cache available, will load from vault
        }
        Err(_e) => {
            // Failed to load cache, will load from vault
        }
    }

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

    // Create channel for unlock results
    let (unlock_tx, mut unlock_rx) = mpsc::unbounded_channel::<UnlockResult>();

    // Start Bitwarden CLI initialization and vault loading in background
    state.start_sync();
    let sync_tx_clone = sync_tx.clone();
    let unlock_tx_clone = unlock_tx.clone();
    tokio::spawn(async move {
        // Initialize Bitwarden CLI
        let bw_cli = match cli::BitwardenCli::new().await {
            Ok(cli) => cli,
            Err(error::BwError::CliNotFound) => {
                let _ = sync_tx_clone.send(SyncResult::Error(
                    "Bitwarden CLI not found. Please install: npm install-g @bitwarden/cli".to_string()
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

        // Handle vault status
        match status {
            cli::VaultStatus::Unlocked => {
                // Already unlocked, proceed normally
                let _ = cli_tx.send(Ok(bw_cli.clone()));
                let result = match bw_cli.list_items().await {
                    Ok(items) => SyncResult::Success(items),
                    Err(e) => SyncResult::Error(format!("Failed to load vault items: {}", e)),
                };
                let _ = sync_tx_clone.send(result);
            }
            cli::VaultStatus::Locked => {
                // Vault is locked - prompt for password
                let _ = unlock_tx_clone.send(UnlockResult::PasswordRequired(bw_cli));
            }
            cli::VaultStatus::Unauthenticated => {
                // Vault is not logged in - show error popup
                let _ = unlock_tx_clone.send(UnlockResult::NotLoggedIn);
            }
        }
    });

    // We'll store the CLI instance once it's initialized
    let mut bw_cli: Option<cli::BitwardenCli> = None;
    
    // Store session token if user wants to save it
    let mut session_token_to_save: Option<String> = None;

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

        // Check for unlock results
        if let Ok(result) = unlock_rx.try_recv() {
            match result {
                UnlockResult::PasswordRequired(cli) => {
                    // Store the CLI temporarily and prompt for password
                    bw_cli = Some(cli);
                    state.stop_sync();
                    state.enter_password_mode();
                }
                UnlockResult::Success(token, cli) => {
                    // Vault unlocked successfully
                    bw_cli = Some(cli.clone());
                    state.exit_password_mode();
                    
                    // Store token and offer to save it
                    session_token_to_save = Some(token);
                    state.enter_save_token_prompt();
                }
                UnlockResult::Error(error) => {
                    // Unlock failed
                    state.set_unlock_error(error);
                }
                UnlockResult::NotLoggedIn => {
                    // Vault is not logged in - show error popup
                    state.stop_sync();
                    state.show_not_logged_in_popup();
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

            // Handle password input actions
            if state.password_input_mode {
                match action {
                    Action::AppendPasswordChar(c) => {
                        state.append_password_char(c);
                    }
                    Action::DeletePasswordChar => {
                        state.delete_password_char();
                    }
                    Action::SubmitPassword => {
                        // Get the password and attempt unlock
                        let password = state.get_password();
                        if password.is_empty() {
                            state.set_unlock_error("Password cannot be empty".to_string());
                        } else {
                            // Attempt unlock in background
                            if let Some(ref cli) = bw_cli {
                                let cli_clone = cli.clone();
                                let unlock_tx_clone = unlock_tx.clone();
                                tokio::spawn(async move {
                                    match cli_clone.unlock(&password).await {
                                        Ok(token) => {
                                            let new_cli = cli::BitwardenCli::with_session_token(token.clone());
                                            let _ = unlock_tx_clone.send(UnlockResult::Success(token, new_cli));
                                        }
                                        Err(e) => {
                                            let _ = unlock_tx_clone.send(UnlockResult::Error(e.to_string()));
                                        }
                                    }
                                });
                            }
                        }
                    }
                    Action::CancelPasswordInput => {
                        // If user cancels unlock, exit the app since they can't proceed
                        break;
                    }
                    Action::Tick => {},
                    _ => {}
                }
                continue;
            }

            // Handle save token prompt actions
            if state.offer_save_token {
                match action {
                    Action::SaveTokenYes => {
                        state.set_save_token_response(true);
                        state.exit_save_token_prompt();
                        
                        // Save the token
                        if let Some(token) = &session_token_to_save {
                            match save_session_token(token) {
                                Ok(()) => {
                                    state.set_status("✓ Session token saved successfully", MessageLevel::Success);
                                }
                                Err(e) => {
                                    state.set_status(format!("⚠ Failed to save token: {}", e), MessageLevel::Warning);
                                }
                            }
                        }
                        session_token_to_save = None;

                        // Now load vault items
                        if let Some(ref cli) = bw_cli {
                            state.start_sync();
                            let cli_clone = cli.clone();
                            let sync_tx_clone = sync_tx.clone();
                            tokio::spawn(async move {
                                let result = match cli_clone.list_items().await {
                                    Ok(items) => SyncResult::Success(items),
                                    Err(e) => SyncResult::Error(format!("Failed to load vault items: {}", e)),
                                };
                                let _ = sync_tx_clone.send(result);
                            });
                        }
                    }
                    Action::SaveTokenNo => {
                        state.set_save_token_response(false);
                        state.exit_save_token_prompt();
                        state.set_status("Session token not saved", MessageLevel::Info);
                        session_token_to_save = None;

                        // Load vault items anyway
                        if let Some(ref cli) = bw_cli {
                            state.start_sync();
                            let cli_clone = cli.clone();
                            let sync_tx_clone = sync_tx.clone();
                            tokio::spawn(async move {
                                let result = match cli_clone.list_items().await {
                                    Ok(items) => SyncResult::Success(items),
                                    Err(e) => SyncResult::Error(format!("Failed to load vault items: {}", e)),
                                };
                                let _ = sync_tx_clone.send(result);
                            });
                        }
                    }
                    Action::Tick => {},
                    _ => {}
                }
                continue;
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

// Result type for unlock operations
enum UnlockResult {
    PasswordRequired(cli::BitwardenCli),
    Success(String, cli::BitwardenCli), // (session_token, cli_with_token)
    Error(String),
    NotLoggedIn,
}

fn handle_sync_result(result: SyncResult, state: &mut AppState) {
    state.stop_sync();
    match result {
        SyncResult::Success(items) => {
            // Save cache (without secrets)
            let cache_data = cache::CachedVaultData::from_vault_items(&items);
            let _ = cache::save_cache(&cache_data); // Ignore cache save errors

            // Load items with secrets available
            state.load_items_with_secrets(items);
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
            if !state.secrets_available {
                state.set_status(
                    "⏳ Please wait, loading vault secrets...",
                    MessageLevel::Warning,
                );
                return true;
            }
            
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
            if !state.secrets_available {
                state.set_status(
                    "⏳ Please wait, loading vault secrets...",
                    MessageLevel::Warning,
                );
                return true;
            }
            
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

        // Password input actions (should not reach here, but handle for completeness)
        Action::SubmitPassword | Action::CancelPasswordInput 
        | Action::AppendPasswordChar(_) | Action::DeletePasswordChar => {
            // These are handled in the main event loop before calling handle_action
        }

        // Save token actions (should not reach here, but handle for completeness)
        Action::SaveTokenYes | Action::SaveTokenNo => {
            // These are handled in the main event loop before calling handle_action
        }
    }

    true
}

/// Save session token to system user environment variable
fn save_session_token(token: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Use setx to set persistent user environment variable on Windows
        let output = Command::new("setx")
            .arg("BW_SESSION")
            .arg(token)
            .output()
            .map_err(|e| error::BwError::CommandFailed(format!("Failed to run setx: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(error::BwError::CommandFailed(format!(
                "Failed to set environment variable: {}",
                stderr.trim()
            )));
        }
        
        // Also set in current process so it's available immediately
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // On macOS, use launchctl to set persistent user environment variable
        let output = Command::new("launchctl")
            .arg("setenv")
            .arg("BW_SESSION")
            .arg(token)
            .output()
            .map_err(|e| error::BwError::CommandFailed(format!("Failed to run launchctl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(error::BwError::CommandFailed(format!(
                "Failed to set environment variable: {}",
                stderr.trim()
            )));
        }
        
        // Also set in current process
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use std::process::Command;
        
        // Try systemd user environment first (most modern Linux systems)
        let systemd_result = Command::new("systemctl")
            .arg("--user")
            .arg("set-environment")
            .arg(format!("BW_SESSION={}", token))
            .output();
        
        if let Ok(output) = systemd_result {
            if output.status.success() {
                // Also set in current process
                std::env::set_var("BW_SESSION", token);
                return Ok(());
            }
        }
        
        // Fallback: Write to ~/.profile which is more standard across shells
        use std::io::Write;
        
        let home = std::env::var("HOME")
            .map_err(|_| error::BwError::CommandFailed("Could not determine home directory".to_string()))?;
        
        let profile_path = format!("{}/.profile", home);
        
        // Read existing profile
        let mut content = if std::path::Path::new(&profile_path).exists() {
            std::fs::read_to_string(&profile_path)
                .map_err(|e| error::BwError::IoError(e))?
        } else {
            String::new()
        };
        
        // Check if BW_SESSION is already set
        let bw_session_marker = "# bwtui - Bitwarden session token";
        let has_existing = content.contains(bw_session_marker);
        
        if has_existing {
            // Replace existing BW_SESSION line
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_next = false;
            
            for line in lines {
                if line.contains(bw_session_marker) {
                    skip_next = true;
                    continue;
                }
                if skip_next && line.trim().starts_with("export BW_SESSION") {
                    skip_next = false;
                    continue;
                }
                new_lines.push(line);
            }
            
            content = new_lines.join("\n");
        }
        
        // Append new BW_SESSION
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("\n{}\nexport BW_SESSION=\"{}\"\n", bw_session_marker, token));
        
        // Write back to profile
        let mut file = std::fs::File::create(&profile_path)
            .map_err(|e| error::BwError::IoError(e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| error::BwError::IoError(e))?;
        
        // Also set in current process
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }
}

