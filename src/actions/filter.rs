use crate::events::Action;
use crate::state::AppState;

/// Handle filter/search actions
pub fn handle_filter(action: &Action, state: &mut AppState) -> bool {
    match action {
        Action::AppendFilter(c) => {
            state.append_filter(*c);
        }
        Action::DeleteFilterChar => {
            state.delete_filter_char();
        }
        Action::ClearFilter => {
            state.clear_filter();
        }
        _ => {
            return false; // Not a filter action
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VaultItem, ItemType};

    fn create_test_item(id: &str, name: &str, item_type: ItemType) -> VaultItem {
        VaultItem {
            id: id.to_string(),
            name: name.to_string(),
            item_type,
            login: None,
            card: None,
            identity: None,
            notes: None,
            fields: None,
            favorite: false,
            folder_id: None,
            organization_id: None,
            revision_date: chrono::Utc::now(),
            object: None,
            creation_date: None,
            deleted_date: None,
            password_history: None,
            attachments: None,
            collection_ids: None,
            reprompt: None,
        }
    }

    #[test]
    fn test_filter_actions() {
        let mut state = AppState::new();
        
        // Should handle filter actions
        assert!(handle_filter(&Action::AppendFilter('a'), &mut state));
        assert!(handle_filter(&Action::DeleteFilterChar, &mut state));
        assert!(handle_filter(&Action::ClearFilter, &mut state));
        
        // Should not handle non-filter actions
        assert!(!handle_filter(&Action::Quit, &mut state));
    }

    #[test]
    fn test_filter_functionality() {
        let mut state = AppState::new();
        
        // Add test items
        let items = vec![
            create_test_item("1", "GitHub", ItemType::Login),
            create_test_item("2", "Gmail", ItemType::Login),
            create_test_item("3", "Amazon", ItemType::Login),
            create_test_item("4", "Bank Note", ItemType::SecureNote),
        ];
        state.load_items_with_secrets(items);
        
        // Initially all items should be visible
        assert_eq!(state.vault.filtered_items.len(), 4);
        
        // Filter by text - use "git" which only matches GitHub
        handle_filter(&Action::AppendFilter('g'), &mut state);
        handle_filter(&Action::AppendFilter('i'), &mut state);
        handle_filter(&Action::AppendFilter('t'), &mut state);
        // Filter should match at least GitHub, might also match others with fuzzy matching
        assert!(state.vault.filtered_items.len() >= 1);
        assert!(state.vault.filtered_items.iter().any(|item| item.name == "GitHub"));
        
        // Clear filter
        handle_filter(&Action::ClearFilter, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 4);
        
        // Test with single character filter that should match multiple items
        handle_filter(&Action::AppendFilter('a'), &mut state);
        assert!(state.vault.filtered_items.len() >= 1); // At least Amazon, might match more with fuzzy
        
        // Test delete filter character
        handle_filter(&Action::DeleteFilterChar, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 4); // Back to all items
    }

    #[test]
    fn test_filter_with_type_filter() {
        let mut state = AppState::new();
        
        let items = vec![
            create_test_item("1", "GitHub", ItemType::Login),
            create_test_item("2", "Bank Note", ItemType::SecureNote),
            create_test_item("3", "Amazon", ItemType::Login),
        ];
        state.load_items_with_secrets(items);
        
        // Filter by type first
        state.set_item_type_filter(Some(ItemType::Login));
        assert_eq!(state.vault.filtered_items.len(), 2); // GitHub, Amazon
        
        // Then filter by text
        handle_filter(&Action::AppendFilter('g'), &mut state);
        assert_eq!(state.vault.filtered_items.len(), 1); // GitHub
        
        // Clear text filter
        handle_filter(&Action::ClearFilter, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 2); // Back to Login items
    }
}

