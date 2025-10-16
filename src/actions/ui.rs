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
}

