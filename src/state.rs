use crate::types::VaultItem;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::ListState;
use std::time::Instant;

#[derive(Debug)]
pub struct AppState {
    pub vault_items: Vec<VaultItem>,
    pub filtered_items: Vec<VaultItem>,
    pub filter_query: String,
    pub selected_index: usize,
    pub list_state: ListState,
    pub status_message: Option<StatusMessage>,
    fuzzy_enabled: bool,
    case_sensitive: bool,
}

#[derive(Debug)]
pub struct StatusMessage {
    pub text: String,
    pub level: MessageLevel,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl AppState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            vault_items: Vec::new(),
            filtered_items: Vec::new(),
            filter_query: String::new(),
            selected_index: 0,
            list_state,
            status_message: None,
            fuzzy_enabled: true,
            case_sensitive: false,
        }
    }

    pub fn load_items(&mut self, items: Vec<VaultItem>) {
        self.vault_items = items;
        self.apply_filter();
    }

    pub fn apply_filter(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered_items = self.vault_items.clone();
        } else {
            let matcher = SkimMatcherV2::default();
            let query = if self.case_sensitive {
                self.filter_query.clone()
            } else {
                self.filter_query.to_lowercase()
            };

            self.filtered_items = self
                .vault_items
                .iter()
                .filter(|item| {
                    let searchable_text = self.get_searchable_text(item);
                    
                    if self.fuzzy_enabled {
                        matcher.fuzzy_match(&searchable_text, &query).is_some()
                    } else {
                        searchable_text.contains(&query)
                    }
                })
                .cloned()
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_items.len() && !self.filtered_items.is_empty() {
            self.selected_index = 0;
        }
        
        // Sync list state
        self.sync_list_state();
    }

    fn get_searchable_text(&self, item: &VaultItem) -> String {
        let mut text = if self.case_sensitive {
            item.name.clone()
        } else {
            item.name.to_lowercase()
        };

        if let Some(username) = item.username() {
            text.push(' ');
            if self.case_sensitive {
                text.push_str(username);
            } else {
                let lowercase = username.to_lowercase();
                text.push_str(&lowercase);
            }
        }

        if let Some(domain) = item.domain() {
            text.push(' ');
            if self.case_sensitive {
                text.push_str(&domain);
            } else {
                let lowercase = domain.to_lowercase();
                text.push_str(&lowercase);
            }
        }

        text
    }

    pub fn selected_item(&self) -> Option<&VaultItem> {
        self.filtered_items.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
            self.sync_list_state();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.filtered_items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.sync_list_state();
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.selected_index >= page_size {
            self.selected_index -= page_size;
        } else {
            self.selected_index = 0;
        }
        self.sync_list_state();
    }

    pub fn page_down(&mut self, page_size: usize) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + page_size).min(self.filtered_items.len() - 1);
            self.sync_list_state();
        }
    }

    pub fn jump_to_start(&mut self) {
        self.selected_index = 0;
        self.sync_list_state();
    }

    pub fn jump_to_end(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected_index = self.filtered_items.len() - 1;
            self.sync_list_state();
        }
    }
    
    fn sync_list_state(&mut self) {
        if self.filtered_items.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn append_filter(&mut self, c: char) {
        self.filter_query.push(c);
        self.apply_filter();
    }

    pub fn delete_filter_char(&mut self) {
        self.filter_query.pop();
        self.apply_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.apply_filter();
    }

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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

