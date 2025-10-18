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
        self.reset_details_scroll();
    }

    pub fn load_items_with_secrets(&mut self, items: Vec<VaultItem>) {
        self.vault.load_items_with_secrets(items);
        self.reset_details_scroll();
    }

    pub fn selected_item(&self) -> Option<&VaultItem> {
        self.vault.selected_item()
    }

    pub fn select_next(&mut self) {
        self.vault.select_next();
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching items
    }

    pub fn select_previous(&mut self) {
        self.vault.select_previous();
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching items
    }

    pub fn select_index(&mut self, index: usize) {
        self.vault.select_index(index);
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching items
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.vault.page_up(page_size);
        self.reset_details_scroll();
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.vault.page_down(page_size);
        self.reset_details_scroll();
    }

    pub fn jump_to_start(&mut self) {
        self.vault.jump_to_start();
        self.reset_details_scroll();
    }

    pub fn jump_to_end(&mut self) {
        self.vault.jump_to_end();
        self.reset_details_scroll();
    }

    pub fn append_filter(&mut self, c: char) {
        let old_selection = self.vault.selected_item().map(|item| item.id.clone());
        self.vault.append_filter(c, self.ui.get_active_filter());
        let new_selection = self.vault.selected_item().map(|item| item.id.clone());
        
        // Clear TOTP if selection changed
        if old_selection != new_selection {
            self.clear_totp_code();
        }
        
        self.reset_details_scroll();
    }

    pub fn delete_filter_char(&mut self) {
        let old_selection = self.vault.selected_item().map(|item| item.id.clone());
        self.vault.delete_filter_char(self.ui.get_active_filter());
        let new_selection = self.vault.selected_item().map(|item| item.id.clone());
        
        // Clear TOTP if selection changed
        if old_selection != new_selection {
            self.clear_totp_code();
        }
        
        self.reset_details_scroll();
    }

    pub fn clear_filter(&mut self) {
        let old_selection = self.vault.selected_item().map(|item| item.id.clone());
        self.vault.clear_filter(self.ui.get_active_filter());
        let new_selection = self.vault.selected_item().map(|item| item.id.clone());
        
        // Clear TOTP if selection changed
        if old_selection != new_selection {
            self.clear_totp_code();
        }
        
        self.reset_details_scroll();
    }

    // Convenience delegates to UI state
    pub fn toggle_details_panel(&mut self) {
        self.ui.toggle_details_panel();
    }

    pub fn scroll_details_up(&mut self) {
        self.ui.scroll_details_up();
    }

    pub fn scroll_details_down(&mut self) {
        self.ui.scroll_details_down();
    }

    pub fn set_details_max_scroll(&mut self, max_scroll: usize) {
        self.ui.set_details_max_scroll(max_scroll);
    }

    pub fn reset_details_scroll(&mut self) {
        self.ui.reset_details_scroll();
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

    // TOTP management
    pub fn set_totp_code(&mut self, code: String, expires_at: u64, item_id: String) {
        self.ui.set_totp_code(code, expires_at, item_id);
    }

    pub fn clear_totp_code(&mut self) {
        self.ui.clear_totp_code();
    }

    pub fn set_totp_loading(&mut self, loading: bool) {
        self.ui.set_totp_loading(loading);
    }

    pub fn set_totp_copy_pending(&mut self, pending: bool) {
        self.ui.set_totp_copy_pending(pending);
    }

    pub fn set_last_totp_fetch(&mut self, timestamp: u64) {
        self.ui.set_last_totp_fetch(timestamp);
    }

    pub fn can_fetch_totp(&self) -> bool {
        self.ui.can_fetch_totp()
    }

    pub fn totp_belongs_to_item(&self, item_id: &str) -> bool {
        self.ui.totp_belongs_to_item(item_id)
    }

    pub fn is_totp_expired(&self) -> bool {
        self.ui.is_totp_expired()
    }

    pub fn totp_remaining_seconds(&self) -> Option<u64> {
        self.ui.totp_remaining_seconds()
    }

    pub fn current_totp_code(&self) -> Option<&String> {
        self.ui.current_totp_code.as_ref()
    }

    pub fn totp_loading(&self) -> bool {
        self.ui.totp_loading
    }

    // Tab filtering
    pub fn set_item_type_filter(&mut self, filter: Option<crate::types::ItemType>) {
        self.ui.set_item_type_filter(filter);
        // Reapply filter with new type filter
        self.vault.apply_filter(filter);
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching tabs
    }

    /// Cycle to the next tab and apply the filter
    pub fn cycle_next_tab(&mut self) {
        self.ui.cycle_next_tab();
        let new_filter = self.ui.get_active_filter();
        // Reapply filter with new type filter
        self.vault.apply_filter(new_filter);
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching tabs
    }

    /// Cycle to the previous tab and apply the filter
    pub fn cycle_previous_tab(&mut self) {
        self.ui.cycle_previous_tab();
        let new_filter = self.ui.get_active_filter();
        // Reapply filter with new type filter
        self.vault.apply_filter(new_filter);
        self.reset_details_scroll();
        self.clear_totp_code(); // Clear TOTP when switching tabs
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

