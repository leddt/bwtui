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
}

