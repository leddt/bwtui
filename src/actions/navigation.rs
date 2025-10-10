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

    #[test]
    fn test_navigation_actions() {
        let mut state = AppState::new();
        
        // Should handle navigation actions
        assert!(handle_navigation(&Action::MoveUp, &mut state));
        assert!(handle_navigation(&Action::MoveDown, &mut state));
        
        // Should not handle non-navigation actions
        assert!(!handle_navigation(&Action::Quit, &mut state));
    }
}

