use crate::events::Action;
use crate::state::AppState;

/// Handle UI actions (details panel, etc.)
pub fn handle_ui(action: &Action, state: &mut AppState) -> bool {
    match action {
        Action::ToggleDetailsPanel => {
            state.toggle_details_panel();
        }
        Action::OpenDetailsPanel => {
            if !state.details_panel_visible() {
                state.toggle_details_panel();
            }
        }
        Action::ScrollDetailsUp => {
            state.scroll_details_up();
        }
        Action::ScrollDetailsDown => {
            state.scroll_details_down();
        }
        Action::CloseDetailsPanel => {
            // Close details panel if it's open
            if state.details_panel_visible() {
                state.toggle_details_panel();
            }
        }
        Action::SelectItemTypeTab(filter) => {
            state.set_item_type_filter(*filter);
        }
        Action::CycleNextTab => {
            state.cycle_next_tab();
        }
        Action::CyclePreviousTab => {
            state.cycle_previous_tab();
        }
        _ => {
            return false; // Not a UI action
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
    fn test_ui_actions() {
        let mut state = AppState::new();
        
        // Should handle UI actions
        assert!(handle_ui(&Action::ToggleDetailsPanel, &mut state));
        assert!(handle_ui(&Action::OpenDetailsPanel, &mut state));
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        
        // Should not handle non-UI actions
        assert!(!handle_ui(&Action::Quit, &mut state));
    }

    #[test]
    fn test_details_panel_toggle() {
        let mut state = AppState::new();
        
        // Initially details panel is not visible
        assert!(!state.details_panel_visible());
        
        // Toggle to open
        handle_ui(&Action::ToggleDetailsPanel, &mut state);
        assert!(state.details_panel_visible());
        
        // Toggle to close
        handle_ui(&Action::ToggleDetailsPanel, &mut state);
        assert!(!state.details_panel_visible());
    }

    #[test]
    fn test_open_details_panel_only_when_closed() {
        let mut state = AppState::new();
        
        // Initially closed
        assert!(!state.details_panel_visible());
        
        // OpenDetailsPanel should open it
        handle_ui(&Action::OpenDetailsPanel, &mut state);
        assert!(state.details_panel_visible());
        
        // OpenDetailsPanel when already open should not change state
        handle_ui(&Action::OpenDetailsPanel, &mut state);
        assert!(state.details_panel_visible());
    }

    #[test]
    fn test_close_details_panel_only_when_open() {
        let mut state = AppState::new();
        
        // Initially closed
        assert!(!state.details_panel_visible());
        
        // CloseDetailsPanel when closed should not change state
        handle_ui(&Action::CloseDetailsPanel, &mut state);
        assert!(!state.details_panel_visible());
        
        // Open it first
        handle_ui(&Action::ToggleDetailsPanel, &mut state);
        assert!(state.details_panel_visible());
        
        // Now close it
        handle_ui(&Action::CloseDetailsPanel, &mut state);
        assert!(!state.details_panel_visible());
    }

    #[test]
    fn test_tab_filtering_functionality() {
        let mut state = AppState::new();
        
        // Add items of different types
        let items = vec![
            create_test_item("1", "GitHub", ItemType::Login),
            create_test_item("2", "Bank Note", ItemType::SecureNote),
            create_test_item("3", "Visa Card", ItemType::Card),
            create_test_item("4", "My Identity", ItemType::Identity),
        ];
        state.load_items_with_secrets(items);
        
        // Initially all items should be visible
        assert_eq!(state.vault.filtered_items.len(), 4);
        
        // Filter to Login items
        handle_ui(&Action::SelectItemTypeTab(Some(ItemType::Login)), &mut state);
        assert_eq!(state.vault.filtered_items.len(), 1);
        assert_eq!(state.vault.filtered_items[0].id, "1");
        
        // Filter to Card items
        handle_ui(&Action::SelectItemTypeTab(Some(ItemType::Card)), &mut state);
        assert_eq!(state.vault.filtered_items.len(), 1);
        assert_eq!(state.vault.filtered_items[0].id, "3");
        
        // Filter to show all
        handle_ui(&Action::SelectItemTypeTab(None), &mut state);
        assert_eq!(state.vault.filtered_items.len(), 4);
    }

    #[test]
    fn test_tab_cycling_changes_filter() {
        let mut state = AppState::new();
        
        let items = vec![
            create_test_item("1", "GitHub", ItemType::Login),
            create_test_item("2", "Note", ItemType::SecureNote),
            create_test_item("3", "Card", ItemType::Card),
        ];
        state.load_items_with_secrets(items);
        
        // Initially all items visible
        assert_eq!(state.vault.filtered_items.len(), 3);
        
        // Cycle to Login tab
        handle_ui(&Action::CycleNextTab, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 1);
        assert_eq!(state.vault.filtered_items[0].item_type, ItemType::Login);
        
        // Cycle to SecureNote tab
        handle_ui(&Action::CycleNextTab, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 1);
        assert_eq!(state.vault.filtered_items[0].item_type, ItemType::SecureNote);
        
        // Cycle back to show all
        handle_ui(&Action::CycleNextTab, &mut state);
        handle_ui(&Action::CycleNextTab, &mut state);
        handle_ui(&Action::CycleNextTab, &mut state);
        assert_eq!(state.vault.filtered_items.len(), 3);
    }
}

