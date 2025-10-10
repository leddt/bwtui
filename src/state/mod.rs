mod vault_state;
mod ui_state;
mod sync_state;
mod status_message;

pub use status_message::{MessageLevel, StatusMessage};
pub use vault_state::VaultState;
pub use ui_state::UIState;
pub use sync_state::SyncState;

use crate::types::VaultItem;
use std::time::Instant;

/// Main application state that composes all sub-states
#[derive(Debug)]
pub struct AppState {
    pub vault: VaultState,
    pub ui: UIState,
    pub sync: SyncState,
    pub status_message: Option<StatusMessage>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: VaultState::new(),
            ui: UIState::new(),
            sync: SyncState::new(),
            status_message: None,
        }
    }

    // Convenience delegates to vault state
    pub fn load_cached_items(&mut self, items: Vec<VaultItem>) {
        self.vault.load_cached_items(items);
    }

    pub fn load_items_with_secrets(&mut self, items: Vec<VaultItem>) {
        self.vault.load_items_with_secrets(items);
    }

    pub fn selected_item(&self) -> Option<&VaultItem> {
        self.vault.selected_item()
    }

    pub fn select_next(&mut self) {
        self.vault.select_next();
    }

    pub fn select_previous(&mut self) {
        self.vault.select_previous();
    }

    pub fn select_index(&mut self, index: usize) {
        self.vault.select_index(index);
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.vault.page_up(page_size);
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.vault.page_down(page_size);
    }

    pub fn jump_to_start(&mut self) {
        self.vault.jump_to_start();
    }

    pub fn jump_to_end(&mut self) {
        self.vault.jump_to_end();
    }

    pub fn append_filter(&mut self, c: char) {
        self.vault.append_filter(c);
    }

    pub fn delete_filter_char(&mut self) {
        self.vault.delete_filter_char();
    }

    pub fn clear_filter(&mut self) {
        self.vault.clear_filter();
    }

    // Convenience delegates to UI state
    pub fn toggle_details_panel(&mut self) {
        self.ui.toggle_details_panel();
    }

    pub fn enter_password_mode(&mut self) {
        self.ui.enter_password_mode();
    }

    pub fn exit_password_mode(&mut self) {
        self.ui.exit_password_mode();
    }

    pub fn append_password_char(&mut self, c: char) {
        self.ui.append_password_char(c);
    }

    pub fn delete_password_char(&mut self) {
        self.ui.delete_password_char();
    }

    pub fn get_password(&self) -> String {
        self.ui.get_password()
    }

    pub fn set_unlock_error(&mut self, error: String) {
        self.ui.set_unlock_error(error);
    }

    pub fn enter_save_token_prompt(&mut self) {
        self.ui.enter_save_token_prompt();
    }

    pub fn set_save_token_response(&mut self, response: bool) {
        self.ui.set_save_token_response(response);
    }

    pub fn exit_save_token_prompt(&mut self) {
        self.ui.exit_save_token_prompt();
    }

    pub fn show_not_logged_in_popup(&mut self) {
        self.ui.show_not_logged_in_popup();
    }

    // Convenience delegates to sync state
    pub fn start_sync(&mut self) {
        self.sync.start();
    }

    pub fn stop_sync(&mut self) {
        self.sync.stop();
    }

    pub fn advance_sync_animation(&mut self) {
        self.sync.advance_animation();
    }

    pub fn sync_spinner(&self) -> &str {
        self.sync.spinner()
    }

    // Status message management
    pub fn set_status(&mut self, text: impl Into<String>, level: MessageLevel) {
        self.status_message = Some(StatusMessage {
            text: text.into(),
            level,
            timestamp: Instant::now(),
        });
    }

    /// Check if status message is older than 3 seconds and clear it
    pub fn expire_old_status(&mut self) {
        if let Some(status) = &self.status_message {
            if status.timestamp.elapsed().as_secs() > 3 {
                self.status_message = None;
            }
        }
    }

    // Convenience accessors for commonly used state
    #[inline]
    pub fn syncing(&self) -> bool {
        self.sync.syncing
    }

    #[inline]
    pub fn password_input_mode(&self) -> bool {
        self.ui.password_input_mode
    }

    #[inline]
    pub fn offer_save_token(&self) -> bool {
        self.ui.offer_save_token
    }

    #[inline]
    pub fn details_panel_visible(&self) -> bool {
        self.ui.details_panel_visible
    }

    #[inline]
    pub fn show_not_logged_in_error(&self) -> bool {
        self.ui.show_not_logged_in_error
    }

    #[inline]
    pub fn secrets_available(&self) -> bool {
        self.vault.secrets_available
    }

    #[inline]
    pub fn initial_load_complete(&self) -> bool {
        self.vault.initial_load_complete
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

