use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    
    // Navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    
    // Filter
    AppendFilter(char),
    DeleteFilterChar,
    ClearFilter,
    
    // Actions
    CopyUsername,
    CopyPassword,
    CopyTotp,
    Refresh,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Poll for next event with timeout
    pub fn poll_event(&self, timeout: Duration) -> std::io::Result<Option<KeyEvent>> {
        if event::poll(timeout)? {
            if let CrosstermEvent::Key(key) = event::read()? {
                // Only process key press events, ignore key release and repeat events
                if key.kind == KeyEventKind::Press {
                    return Ok(Some(key));
                }
            }
        }
        Ok(None)
    }

    /// Convert key event to action (unified mode)
    pub fn handle_key(&self, key: KeyEvent, _state: &AppState) -> Option<Action> {
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
            (KeyCode::Esc, _) => Some(Action::ClearFilter),
            
            // Navigation - Arrow keys
            (KeyCode::Up, _) => Some(Action::MoveUp),
            (KeyCode::Down, _) => Some(Action::MoveDown),
            
            // Navigation - Vim style with Ctrl
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(Action::MoveUp),
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => Some(Action::MoveDown),
            
            // Navigation - Page navigation
            (KeyCode::PageUp, _) => Some(Action::PageUp),
            (KeyCode::PageDown, _) => Some(Action::PageDown),
            (KeyCode::Home, _) => Some(Action::Home),
            (KeyCode::End, _) => Some(Action::End),
            
            // Filter editing
            (KeyCode::Backspace, _) => Some(Action::DeleteFilterChar),
            
            // Actions with Ctrl modifier
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Action::CopyUsername),
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(Action::CopyPassword),
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Action::CopyTotp),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Action::Refresh),
            
            // Any other printable character updates the filter
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                Some(Action::AppendFilter(c))
            }
            
            _ => None,
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let _handler = EventHandler::new();
        assert!(true);
    }
}

