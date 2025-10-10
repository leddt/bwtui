use crate::types::VaultItem;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::ListState;

/// State related to vault items, filtering, and selection
#[derive(Debug)]
pub struct VaultState {
    pub vault_items: Vec<VaultItem>,
    pub filtered_items: Vec<VaultItem>,
    pub filter_query: String,
    pub selected_index: usize,
    pub list_state: ListState,
    pub initial_load_complete: bool,
    pub secrets_available: bool,
    fuzzy_enabled: bool,
    case_sensitive: bool,
}

impl VaultState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            vault_items: Vec::new(),
            filtered_items: Vec::new(),
            filter_query: String::new(),
            selected_index: 0,
            list_state,
            initial_load_complete: false,
            secrets_available: false,
            fuzzy_enabled: true,
            case_sensitive: false,
        }
    }

    /// Load items from cache (without secrets)
    pub fn load_cached_items(&mut self, items: Vec<VaultItem>) {
        self.vault_items = items;
        self.apply_filter();
        self.initial_load_complete = true;
        self.secrets_available = false;
    }

    /// Load items with full data including secrets
    pub fn load_items_with_secrets(&mut self, items: Vec<VaultItem>) {
        self.vault_items = items;
        self.apply_filter();
        self.initial_load_complete = true;
        self.secrets_available = true;
    }

    pub fn apply_filter(&mut self) {
        if self.filter_query.is_empty() {
            // When no filter is active, show all items with starred items first
            let mut items = self.vault_items.clone();
            items.sort_by(|a, b| {
                // Sort by favorite status (true before false), then by name
                match (b.favorite, a.favorite) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
            self.filtered_items = items;
        } else {
            let matcher = SkimMatcherV2::default();
            let query = if self.case_sensitive {
                self.filter_query.clone()
            } else {
                self.filter_query.to_lowercase()
            };

            // Collect items with their relevance scores
            let mut items_with_scores: Vec<(VaultItem, i64)> = self
                .vault_items
                .iter()
                .filter_map(|item| {
                    let searchable_text = self.get_searchable_text(item);
                    
                    if self.fuzzy_enabled {
                        matcher.fuzzy_match(&searchable_text, &query)
                            .map(|score| (item.clone(), score))
                    } else {
                        if searchable_text.contains(&query) {
                            // For non-fuzzy matching, use a simple relevance score
                            // Higher score if match is earlier in the string
                            let position = searchable_text.find(&query).unwrap_or(searchable_text.len());
                            let score = 1000 - position as i64;
                            Some((item.clone(), score))
                        } else {
                            None
                        }
                    }
                })
                .collect();

            // Sort by score descending (higher scores = better matches first)
            items_with_scores.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Extract just the items
            self.filtered_items = items_with_scores.into_iter().map(|(item, _)| item).collect();
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

    pub fn select_index(&mut self, index: usize) {
        if index < self.filtered_items.len() {
            self.selected_index = index;
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
}

impl Default for VaultState {
    fn default() -> Self {
        Self::new()
    }
}

