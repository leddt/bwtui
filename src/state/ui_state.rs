use ratatui::layout::Rect;
use crate::types::ItemType;

use std::time::{SystemTime, UNIX_EPOCH};

/// State related to UI modes, dialogs, and layout
#[derive(Debug)]
pub struct UIState {
    pub details_panel_visible: bool,
    pub details_panel_scroll: usize, // Scroll position for details panel
    pub details_panel_max_scroll: usize, // Maximum scroll position for details panel
    pub password_input_mode: bool,
    pub password_input: String,
    pub unlock_error: Option<String>,
    pub offer_save_token: bool,
    pub save_token_response: Option<bool>,
    pub show_not_logged_in_error: bool,
    pub list_area: Rect,
    pub details_panel_area: Rect,
    // TOTP state
    pub current_totp_code: Option<String>,
    pub totp_expires_at: Option<u64>, // Unix timestamp when current TOTP expires
    pub totp_loading: bool, // Whether we're currently fetching a TOTP code
    pub totp_copy_pending: bool, // Whether we're waiting to copy TOTP after fetch
    pub last_totp_fetch: Option<u64>, // Unix timestamp of last TOTP fetch attempt
    pub totp_item_id: Option<String>, // ID of the item that the current TOTP code belongs to
    // Tab filtering state
    pub active_item_type_filter: Option<ItemType>, // None = all types, Some = specific type
}

impl UIState {
    pub fn new() -> Self {
        Self {
            details_panel_visible: false,
            details_panel_scroll: 0,
            details_panel_max_scroll: 0,
            password_input_mode: false,
            password_input: String::new(),
            unlock_error: None,
            offer_save_token: false,
            save_token_response: None,
            show_not_logged_in_error: false,
            list_area: Rect::default(),
            details_panel_area: Rect::default(),
            current_totp_code: None,
            totp_expires_at: None,
            totp_loading: false,
            totp_copy_pending: false,
            last_totp_fetch: None,
            totp_item_id: None,
            active_item_type_filter: None, // Default to showing all types
        }
    }

    pub fn toggle_details_panel(&mut self) {
        self.details_panel_visible = !self.details_panel_visible;
        // Reset scroll when toggling panel
        self.details_panel_scroll = 0;
    }

    pub fn scroll_details_up(&mut self) {
        if self.details_panel_scroll > 0 {
            self.details_panel_scroll -= 1;
        }
    }

    pub fn scroll_details_down(&mut self) {
        if self.details_panel_scroll < self.details_panel_max_scroll {
            self.details_panel_scroll += 1;
        }
    }

    pub fn set_details_max_scroll(&mut self, max_scroll: usize) {
        self.details_panel_max_scroll = max_scroll;
        // Ensure current scroll doesn't exceed max
        if self.details_panel_scroll > max_scroll {
            self.details_panel_scroll = max_scroll;
        }
    }

    pub fn reset_details_scroll(&mut self) {
        self.details_panel_scroll = 0;
    }

    pub fn enter_password_mode(&mut self) {
        self.password_input_mode = true;
        self.password_input.clear();
        self.unlock_error = None;
    }

    pub fn exit_password_mode(&mut self) {
        self.password_input_mode = false;
        self.password_input.clear();
        self.unlock_error = None;
    }

    pub fn append_password_char(&mut self, c: char) {
        self.password_input.push(c);
    }

    pub fn delete_password_char(&mut self) {
        self.password_input.pop();
    }

    pub fn clear_password(&mut self) {
        self.password_input.clear();
    }

    pub fn get_password(&self) -> String {
        self.password_input.clone()
    }

    pub fn set_unlock_error(&mut self, error: String) {
        self.unlock_error = Some(error);
    }

    pub fn enter_save_token_prompt(&mut self) {
        self.offer_save_token = true;
        self.save_token_response = None;
    }

    pub fn set_save_token_response(&mut self, response: bool) {
        self.save_token_response = Some(response);
    }

    pub fn exit_save_token_prompt(&mut self) {
        self.offer_save_token = false;
        self.save_token_response = None;
    }

    pub fn show_not_logged_in_popup(&mut self) {
        self.show_not_logged_in_error = true;
    }

    /// Set the current TOTP code and its expiration time
    pub fn set_totp_code(&mut self, code: String, expires_at: u64, item_id: String) {
        self.current_totp_code = Some(code);
        self.totp_expires_at = Some(expires_at);
        self.totp_item_id = Some(item_id);
        self.totp_loading = false;
        self.totp_copy_pending = false;
    }

    /// Clear the current TOTP code
    pub fn clear_totp_code(&mut self) {
        self.current_totp_code = None;
        self.totp_expires_at = None;
        self.totp_item_id = None;
        self.totp_loading = false;
        self.totp_copy_pending = false;
    }

    /// Set TOTP loading state
    pub fn set_totp_loading(&mut self, loading: bool) {
        self.totp_loading = loading;
    }

    /// Set TOTP copy pending state
    pub fn set_totp_copy_pending(&mut self, pending: bool) {
        self.totp_copy_pending = pending;
    }

    /// Set last TOTP fetch timestamp
    pub fn set_last_totp_fetch(&mut self, timestamp: u64) {
        self.last_totp_fetch = Some(timestamp);
    }

    /// Check if enough time has passed since last TOTP fetch (minimum 1 second)
    pub fn can_fetch_totp(&self) -> bool {
        if let Some(last_fetch) = self.last_totp_fetch {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now - last_fetch >= 1 // Minimum 1 second between fetches
        } else {
            true // Never fetched before
        }
    }

    /// Check if the current TOTP code belongs to the given item
    pub fn totp_belongs_to_item(&self, item_id: &str) -> bool {
        self.totp_item_id.as_ref().map_or(false, |id| id == item_id)
    }

    /// Check if the current TOTP code is expired
    pub fn is_totp_expired(&self) -> bool {
        if let Some(expires_at) = self.totp_expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= expires_at
        } else {
            true // No TOTP code means it's "expired"
        }
    }

    /// Get remaining seconds for current TOTP code
    pub fn totp_remaining_seconds(&self) -> Option<u64> {
        if let Some(expires_at) = self.totp_expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now < expires_at {
                Some(expires_at - now)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// Set the active item type filter
    pub fn set_item_type_filter(&mut self, filter: Option<ItemType>) {
        self.active_item_type_filter = filter;
    }

    /// Get the active item type filter
    pub fn get_active_filter(&self) -> Option<ItemType> {
        self.active_item_type_filter
    }

    /// Cycle to the next tab in order: All -> Login -> Note -> Card -> Identity -> All
    pub fn cycle_next_tab(&mut self) {
        self.active_item_type_filter = match self.active_item_type_filter {
            None => Some(ItemType::Login),
            Some(ItemType::Login) => Some(ItemType::SecureNote),
            Some(ItemType::SecureNote) => Some(ItemType::Card),
            Some(ItemType::Card) => Some(ItemType::Identity),
            Some(ItemType::Identity) => None, // Cycle back to All
        };
    }

    /// Cycle to the previous tab in order: All <- Login <- Note <- Card <- Identity <- All
    pub fn cycle_previous_tab(&mut self) {
        self.active_item_type_filter = match self.active_item_type_filter {
            None => Some(ItemType::Identity), // Cycle back to Identity
            Some(ItemType::Login) => None,
            Some(ItemType::SecureNote) => Some(ItemType::Login),
            Some(ItemType::Card) => Some(ItemType::SecureNote),
            Some(ItemType::Identity) => Some(ItemType::Card),
        };
    }

}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

