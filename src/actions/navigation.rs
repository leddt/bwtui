use crate::events::Action;
use crate::state::AppState;

/// Handle navigation actions
pub fn handle_navigation(action: &Action, state: &mut AppState) -> bool {
    match action {
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
            state.select_index(*index);
        }
        Action::SelectIndexAndShowDetails(index) => {
            state.select_index(*index);
            if !state.details_panel_visible() {
                state.toggle_details_panel();
            }
        }
        _ => {
            return false; // Not a navigation action
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
    fn test_navigation_actions() {
        let mut state = AppState::new();
        
        // Should handle navigation actions
        assert!(handle_navigation(&Action::MoveUp, &mut state));
        assert!(handle_navigation(&Action::MoveDown, &mut state));
        
        // Should not handle non-navigation actions
        assert!(!handle_navigation(&Action::Quit, &mut state));
    }

    #[test]
    fn test_navigation_functionality() {
        let mut state = AppState::new();
        
        // Add test items
        let items = vec![
            create_test_item("1", "First", ItemType::Login),
            create_test_item("2", "Second", ItemType::Login),
            create_test_item("3", "Third", ItemType::Login),
            create_test_item("4", "Fourth", ItemType::Login),
            create_test_item("5", "Fifth", ItemType::Login),
        ];
        state.load_items_with_secrets(items);
        
        // After loading, items are sorted alphabetically by name
        // So order is: Fifth, First, Fourth, Second, Third
        // But we need to verify we can navigate correctly
        
        // Ensure we have items
        assert!(!state.vault.filtered_items.is_empty());
        assert_eq!(state.vault.filtered_items.len(), 5);
        
        // Reset to first item for consistent testing
        handle_navigation(&Action::Home, &mut state);
        assert_eq!(state.vault.selected_index, 0);
        
        // Verify we can navigate through items
        let first_item_id = state.selected_item().unwrap().id.clone();
        
        // Move down
        handle_navigation(&Action::MoveDown, &mut state);
        assert_eq!(state.vault.selected_index, 1);
        let second_item_id = state.selected_item().unwrap().id.clone();
        assert_ne!(first_item_id, second_item_id);
        
        // Move up
        handle_navigation(&Action::MoveUp, &mut state);
        assert_eq!(state.vault.selected_index, 0);
        let back_to_first = state.selected_item().unwrap().id.clone();
        assert_eq!(back_to_first, first_item_id);
        
        // Move up when at start should wrap to end (circular navigation)
        handle_navigation(&Action::MoveUp, &mut state);
        assert_eq!(state.vault.selected_index, 4);
        
        // Jump to start
        handle_navigation(&Action::Home, &mut state);
        assert_eq!(state.vault.selected_index, 0);
        
        // Jump to end
        handle_navigation(&Action::End, &mut state);
        assert_eq!(state.vault.selected_index, 4);
        
        // Move down when at end should wrap to start (circular navigation)
        handle_navigation(&Action::MoveDown, &mut state);
        assert_eq!(state.vault.selected_index, 0);
        
        // Page down
        handle_navigation(&Action::PageDown, &mut state);
        assert_eq!(state.vault.selected_index, 10.min(4)); // Min of page_size (10) and items.len()-1
        
        // Page up
        handle_navigation(&Action::PageUp, &mut state);
        assert_eq!(state.vault.selected_index, 0);
    }

    #[test]
    fn test_select_index() {
        let mut state = AppState::new();
        
        let items = vec![
            create_test_item("1", "First", ItemType::Login),
            create_test_item("2", "Second", ItemType::Login),
            create_test_item("3", "Third", ItemType::Login),
        ];
        state.load_items_with_secrets(items);
        
        // Select by index
        handle_navigation(&Action::SelectIndex(2), &mut state);
        assert_eq!(state.vault.selected_index, 2);
        assert_eq!(state.selected_item().unwrap().id, "3");
        
        // Select index out of bounds should select last item
        handle_navigation(&Action::SelectIndex(10), &mut state);
        assert_eq!(state.vault.selected_index, 2); // Should stay at last valid index
    }

    #[test]
    fn test_navigation_with_empty_list() {
        let mut state = AppState::new();
        
        // Navigation should not panic with empty list
        handle_navigation(&Action::MoveDown, &mut state);
        handle_navigation(&Action::MoveUp, &mut state);
        handle_navigation(&Action::PageDown, &mut state);
        handle_navigation(&Action::PageUp, &mut state);
        handle_navigation(&Action::Home, &mut state);
        handle_navigation(&Action::End, &mut state);
        
        assert!(state.selected_item().is_none());
    }
}

