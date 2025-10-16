use crate::actions;
use crate::actions::CopyResult;
use crate::cache;
use crate::cli::{self, BitwardenCli};
use crate::clipboard::ClipboardManager;
use crate::error::Result;
use crate::events::Action;
use crate::state::{AppState, MessageLevel};
use crate::types::VaultItem;
use tokio::sync::mpsc;

/// Result type for sync operations
pub enum SyncResult {
    Success(Vec<VaultItem>),
    Error(String),
}

/// Result type for unlock operations
pub enum UnlockResult {
    PasswordRequired(BitwardenCli),
    Success(String, BitwardenCli), // (session_token, cli_with_token)
    Error(String),
    NotLoggedIn,
}

/// Result type for TOTP operations
pub enum TotpResult {
    Success(String, u64), // (code, expires_at)
    Error(String),
}

/// Main application controller
pub struct App {
    pub state: AppState,
    pub clipboard: Option<ClipboardManager>,
    bw_cli: Option<BitwardenCli>,
    sync_tx: mpsc::UnboundedSender<SyncResult>,
    sync_rx: mpsc::UnboundedReceiver<SyncResult>,
    cli_tx: mpsc::UnboundedSender<Result<BitwardenCli>>,
    cli_rx: mpsc::UnboundedReceiver<Result<BitwardenCli>>,
    unlock_tx: mpsc::UnboundedSender<UnlockResult>,
    unlock_rx: mpsc::UnboundedReceiver<UnlockResult>,
    totp_tx: mpsc::UnboundedSender<TotpResult>,
    totp_rx: mpsc::UnboundedReceiver<TotpResult>,
    session_token_to_save: Option<String>,
}

impl App {
    /// Create a new App instance
    pub fn new() -> Self {
        let state = AppState::new();
        
        // Initialize clipboard
        let clipboard = match ClipboardManager::new() {
            Ok(cb) => Some(cb),
            Err(_) => None,
        };

        // Create channels
        let (sync_tx, sync_rx) = mpsc::unbounded_channel::<SyncResult>();
        let (cli_tx, cli_rx) = mpsc::unbounded_channel::<Result<BitwardenCli>>();
        let (unlock_tx, unlock_rx) = mpsc::unbounded_channel::<UnlockResult>();
        let (totp_tx, totp_rx) = mpsc::unbounded_channel::<TotpResult>();

        Self {
            state,
            clipboard,
            bw_cli: None,
            sync_tx,
            sync_rx,
            cli_tx,
            cli_rx,
            unlock_tx,
            unlock_rx,
            totp_tx,
            totp_rx,
            session_token_to_save: None,
        }
    }

    /// Try to load cached vault data
    pub fn load_from_cache(&mut self) {
        match cache::load_cache() {
            Ok(Some(cached_data)) => {
                let cached_items = cached_data.to_vault_items();
                self.state.load_cached_items(cached_items);
                self.state.set_status(
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
    }

    /// Start background vault initialization and loading
    pub fn start_vault_initialization(&mut self) {
        self.state.start_sync();
        
        let sync_tx_clone = self.sync_tx.clone();
        let cli_tx = self.cli_tx.clone();
        let unlock_tx_clone = self.unlock_tx.clone();
        
        tokio::spawn(async move {
            // Initialize Bitwarden CLI
            let bw_cli = match BitwardenCli::new().await {
                Ok(cli) => cli,
                Err(crate::error::BwError::CliNotFound) => {
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
    }

    /// Check for and handle incoming messages from background tasks
    pub fn process_background_messages(&mut self) {
        // Check for CLI initialization result
        if let Ok(result) = self.cli_rx.try_recv() {
            match result {
                Ok(cli) => {
                    self.bw_cli = Some(cli);
                }
                Err(e) => {
                    self.state.set_status(format!("✗ {}", e), MessageLevel::Error);
                }
            }
        }

        // Check for unlock results
        if let Ok(result) = self.unlock_rx.try_recv() {
            self.handle_unlock_result(result);
        }

        // Check for sync results
        if let Ok(result) = self.sync_rx.try_recv() {
            self.handle_sync_result(result);
        }

        // Check for TOTP results
        if let Ok(result) = self.totp_rx.try_recv() {
            self.handle_totp_result(result);
        }
    }

    /// Handle unlock result from background task
    fn handle_unlock_result(&mut self, result: UnlockResult) {
        match result {
            UnlockResult::PasswordRequired(cli) => {
                // Store the CLI temporarily and prompt for password
                self.bw_cli = Some(cli);
                self.state.stop_sync();
                self.state.enter_password_mode();
            }
            UnlockResult::Success(token, cli) => {
                // Vault unlocked successfully
                self.bw_cli = Some(cli);
                self.state.exit_password_mode();
                
                // Store token and offer to save it
                self.session_token_to_save = Some(token);
                self.state.enter_save_token_prompt();
            }
            UnlockResult::Error(error) => {
                // Unlock failed
                self.state.set_unlock_error(error);
            }
            UnlockResult::NotLoggedIn => {
                // Vault is not logged in - show error popup
                self.state.stop_sync();
                self.state.show_not_logged_in_popup();
            }
        }
    }

    /// Handle TOTP result from background task
    fn handle_totp_result(&mut self, result: TotpResult) {
        self.state.set_totp_loading(false);
        match result {
            TotpResult::Success(code, expires_at) => {
                // Get the current item ID to associate the TOTP code with it
                let item_id = self.state.selected_item()
                    .map(|item| item.id.clone())
                    .unwrap_or_default();
                
                // Check if we were copying TOTP before setting the code (which clears the flag)
                let was_copying = self.state.ui.totp_copy_pending;
                
                self.state.set_totp_code(code.clone(), expires_at, item_id);
                
                // If we were copying TOTP, copy it now
                if was_copying {
                    if let Some(cb) = self.clipboard.as_mut() {
                        match cb.copy(&code) {
                            Ok(_) => {
                                self.state.set_status(
                                    format!("✓ TOTP code copied: {}", code),
                                    MessageLevel::Success,
                                );
                            }
                            Err(_) => {
                                self.state.set_status(
                                    "✗ Failed to copy to clipboard",
                                    MessageLevel::Error,
                                );
                            }
                        }
                    } else {
                        self.state.set_status("✗ Clipboard not available", MessageLevel::Error);
                    }
                }
                // No message when just loading for display purposes
            }
            TotpResult::Error(error) => {
                self.state.set_status(
                    format!("✗ Failed to fetch TOTP: {}", error),
                    MessageLevel::Error,
                );
            }
        }
    }

    /// Handle sync result from background task
    fn handle_sync_result(&mut self, result: SyncResult) {
        self.state.stop_sync();
        match result {
            SyncResult::Success(items) => {
                // Save cache (without secrets)
                let cache_data = cache::CachedVaultData::from_vault_items(&items);
                let _ = cache::save_cache(&cache_data); // Ignore cache save errors

                // Load items with secrets available
                self.state.load_items_with_secrets(items);
                self.state.set_status("✓ Vault synced successfully", MessageLevel::Success);
            }
            SyncResult::Error(error) => {
                self.state.set_status(
                    format!("✗ Sync failed: {}", error),
                    MessageLevel::Error,
                );
            }
        }
    }

    /// Attempt to unlock the vault with a password
    pub fn unlock_with_password(&mut self, password: String) {
        if password.is_empty() {
            self.state.set_unlock_error("Password cannot be empty".to_string());
            return;
        }

        // Attempt unlock in background
        if let Some(ref cli) = self.bw_cli {
            let cli_clone = cli.clone();
            let unlock_tx_clone = self.unlock_tx.clone();
            tokio::spawn(async move {
                match cli_clone.unlock(&password).await {
                    Ok(token) => {
                        let new_cli = BitwardenCli::with_session_token(token.clone());
                        let _ = unlock_tx_clone.send(UnlockResult::Success(token, new_cli));
                    }
                    Err(e) => {
                        let _ = unlock_tx_clone.send(UnlockResult::Error(e.to_string()));
                    }
                }
            });
        }
    }

    /// Handle save token response (yes/no)
    pub fn handle_save_token_response(&mut self, save: bool, session_manager: &crate::session::SessionManager) {
        self.state.set_save_token_response(save);
        self.state.exit_save_token_prompt();
        
        if save {
            // Save the token
            if let Some(token) = &self.session_token_to_save {
                match session_manager.save_token(token) {
                    Ok(()) => {
                        self.state.set_status("✓ Session token saved successfully", MessageLevel::Success);
                    }
                    Err(e) => {
                        self.state.set_status(format!("⚠ Failed to save token: {}", e), MessageLevel::Warning);
                    }
                }
            }
        } else {
            self.state.set_status("Session token not saved", MessageLevel::Info);
        }
        
        self.session_token_to_save = None;

        // Now load vault items
        self.load_vault_items();
    }

    /// Start loading vault items from the CLI
    fn load_vault_items(&mut self) {
        if let Some(ref cli) = self.bw_cli {
            self.state.start_sync();
            let cli_clone = cli.clone();
            let sync_tx_clone = self.sync_tx.clone();
            tokio::spawn(async move {
                let result = match cli_clone.list_items().await {
                    Ok(items) => SyncResult::Success(items),
                    Err(e) => SyncResult::Error(format!("Failed to load vault items: {}", e)),
                };
                let _ = sync_tx_clone.send(result);
            });
        }
    }

    /// Fetch TOTP code for the currently selected item
    pub fn fetch_totp_code(&mut self) {
        if !self.state.secrets_available() {
            self.state.set_status(
                "⏳ Please wait, loading vault secrets...",
                MessageLevel::Warning,
            );
            return;
        }

        if let Some(item) = self.state.selected_item() {
            if let Some(login) = &item.login {
                if login.totp.is_some() {
                    if let Some(ref cli) = self.bw_cli {
                        let item_id = item.id.clone();
                        self.state.set_totp_loading(true);
                        // Record the timestamp when we start fetching
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        self.state.set_last_totp_fetch(now);
                        let cli_clone = cli.clone();
                        let totp_tx_clone = self.totp_tx.clone();
                        
                        tokio::spawn(async move {
                            let result = match cli_clone.get_totp(&item_id).await {
                                Ok(code) => {
                                    // Calculate expiration time (TOTP codes are valid for 30 seconds)
                                    let now = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    let expires_at = ((now / 30) + 1) * 30; // Next 30-second boundary
                                    TotpResult::Success(code, expires_at)
                                }
                                Err(e) => TotpResult::Error(e.to_string()),
                            };
                            let _ = totp_tx_clone.send(result);
                        });
                    } else {
                        self.state.set_status(
                            "✗ Bitwarden CLI not available",
                            MessageLevel::Error,
                        );
                    }
                } else {
                    self.state.set_status(
                        "✗ No TOTP configured for this entry",
                        MessageLevel::Warning,
                    );
                }
            }
        }
    }

    /// Trigger a vault refresh/sync
    pub fn refresh_vault(&mut self) {
        // Don't start a new sync if one is already in progress
        if self.state.syncing() {
            self.state.set_status("⟳ Sync already in progress...", MessageLevel::Warning);
            return;
        }

        if let Some(ref bw_cli) = self.bw_cli {
            self.state.start_sync();
            
            let bw_cli_clone = bw_cli.clone();
            let sync_tx_clone = self.sync_tx.clone();
            
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
                
                let _ = sync_tx_clone.send(result);
            });
        }
    }

    /// Handle an action - returns false if app should quit
    pub async fn handle_action(&mut self, action: Action, session_manager: &crate::session::SessionManager) -> bool {
        // Handle quit action
        if matches!(action, Action::Quit) {
            return false;
        }

        // Handle tick action (periodic UI updates)
        if matches!(action, Action::Tick) {
            // Check if we need to refresh TOTP code
            if self.state.details_panel_visible() {
                if let Some(item) = self.state.selected_item() {
                    if let Some(login) = &item.login {
                        if login.totp.is_some() {
                            // Only fetch TOTP if we're not already loading one and enough time has passed
                            if !self.state.totp_loading() && self.state.can_fetch_totp() {
                                // If we have a TOTP code but it's expired, refresh it
                                if self.state.current_totp_code().is_some() && self.state.is_totp_expired() {
                                    self.fetch_totp_code();
                                }
                                // If we don't have a TOTP code yet, fetch it
                                else if self.state.current_totp_code().is_none() {
                                    self.fetch_totp_code();
                                }
                            }
                        }
                    }
                }
            }
            return true;
        }

        // Handle password input modal actions
        if self.state.password_input_mode() {
            return self.handle_password_input_action(action);
        }

        // Handle save token prompt actions
        if self.state.offer_save_token() {
            return self.handle_save_token_action(action, session_manager);
        }

        // Try each action handler in order
        if actions::handle_navigation(&action, &mut self.state) {
            return true;
        }

        if actions::handle_filter(&action, &mut self.state) {
            return true;
        }

        if actions::handle_ui(&action, &mut self.state) {
            return true;
        }

        match actions::handle_copy(&action, &mut self.state, self.clipboard.as_mut(), self.bw_cli.as_ref()) {
            CopyResult::Handled => {
                return true;
            }
            CopyResult::NeedTotpFetch => {
                // Trigger TOTP fetch for copy operation
                self.fetch_totp_code();
                return true;
            }
            CopyResult::NotHandled => {
                // Continue to other action handlers
            }
        }

        // Handle TOTP fetching
        if matches!(action, Action::FetchTotp) {
            self.fetch_totp_code();
            return true;
        }

        // Handle refresh action
        if matches!(action, Action::Refresh) {
            self.refresh_vault();
            return true;
        }

        true
    }

    /// Handle password input modal actions
    fn handle_password_input_action(&mut self, action: Action) -> bool {
        match action {
            Action::AppendPasswordChar(c) => {
                self.state.append_password_char(c);
            }
            Action::DeletePasswordChar => {
                self.state.delete_password_char();
            }
            Action::SubmitPassword => {
                let password = self.state.get_password();
                self.unlock_with_password(password);
            }
            Action::CancelPasswordInput => {
                // If user cancels unlock, exit the app
                return false;
            }
            Action::Tick => {}
            _ => {}
        }
        true
    }

    /// Handle save token prompt actions
    fn handle_save_token_action(&mut self, action: Action, session_manager: &crate::session::SessionManager) -> bool {
        match action {
            Action::SaveTokenYes => {
                self.handle_save_token_response(true, session_manager);
            }
            Action::SaveTokenNo => {
                self.handle_save_token_response(false, session_manager);
            }
            Action::Tick => {}
            _ => {}
        }
        true
    }

    /// Check if clipboard warning should be shown
    pub fn should_show_clipboard_warning(&self) -> bool {
        self.clipboard.is_none()
    }

    /// Update app state and render UI
    pub fn update(&mut self, ui: &mut crate::ui::UI) -> crate::error::Result<()> {
        // Clear old status messages
        self.state.expire_old_status();

        // Advance sync animation
        self.state.advance_sync_animation();

        // Process any incoming messages from background tasks
        self.process_background_messages();

        // Render UI
        ui.render(&mut self.state)?;

        Ok(())
    }
}

