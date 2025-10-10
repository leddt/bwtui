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
}

