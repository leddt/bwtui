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
    fn test_close_details_panel_action() {
        let mut state = AppState::new();
        
        // Initially details panel is not visible
        assert!(!state.details_panel_visible());
        
        // Open details panel
        state.toggle_details_panel();
        assert!(state.details_panel_visible());
        
        // CloseDetailsPanel should close the details panel
        assert!(handle_ui(&Action::CloseDetailsPanel, &mut state));
        assert!(!state.details_panel_visible());
        
        // CloseDetailsPanel when details panel is already closed should still return true (handled)
        assert!(handle_ui(&Action::CloseDetailsPanel, &mut state));
        assert!(!state.details_panel_visible());
    }

    #[test]
    fn test_tab_cycling() {
        let mut state = AppState::new();
        
        // Initially should be on "All" tab (None)
        assert!(state.ui.get_active_filter().is_none());
        
        // Cycle through tabs
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Login));
        
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::SecureNote));
        
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Card));
        
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Identity));
        
        // Should cycle back to "All"
        assert!(handle_ui(&Action::CycleNextTab, &mut state));
        assert!(state.ui.get_active_filter().is_none());
    }

    #[test]
    fn test_backward_tab_cycling() {
        let mut state = AppState::new();
        
        // Initially should be on "All" tab (None)
        assert!(state.ui.get_active_filter().is_none());
        
        // Cycle backwards through tabs
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Identity));
        
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Card));
        
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::SecureNote));
        
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Login));
        
        // Should cycle back to "All"
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state));
        assert!(state.ui.get_active_filter().is_none());
    }

    #[test]
    fn test_forward_and_backward_cycling_consistency() {
        let mut state = AppState::new();
        
        // Start at "All"
        assert!(state.ui.get_active_filter().is_none());
        
        // Go forward 3 steps
        assert!(handle_ui(&Action::CycleNextTab, &mut state)); // Login
        assert!(handle_ui(&Action::CycleNextTab, &mut state)); // Note
        assert!(handle_ui(&Action::CycleNextTab, &mut state)); // Card
        assert_eq!(state.ui.get_active_filter(), Some(crate::types::ItemType::Card));
        
        // Go backward 3 steps should return to "All"
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state)); // Note
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state)); // Login
        assert!(handle_ui(&Action::CyclePreviousTab, &mut state)); // All
        assert!(state.ui.get_active_filter().is_none());
    }
}

