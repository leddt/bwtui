use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;
use crate::state::AppState;
use crate::ui::widgets::{details::DetailsClickHandler, entry_list::EntryListClickHandler, clickable::Clickable};

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Tick, // Periodic update for TOTP countdown and other time-based updates
    
    // Navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    #[allow(dead_code)]
    SelectIndex(usize),
    SelectIndexAndShowDetails(usize),
    
    // Filter
    AppendFilter(char),
    DeleteFilterChar,
    ClearFilter,
    
    // Actions
    CopyUsername,
    CopyPassword,
    CopyTotp,
    Refresh,
    ToggleDetailsPanel,
    OpenDetailsPanel,

    // Password input actions
    SubmitPassword,
    CancelPasswordInput,
    AppendPasswordChar(char),
    DeletePasswordChar,

    // Save token actions
    SaveTokenYes,
    SaveTokenNo,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    /// Poll for next event with timeout
    pub fn poll_event(&self, timeout: Duration, state: &AppState) -> std::io::Result<Option<Action>> {
        if event::poll(timeout)? {
            match event::read()? {
                CrosstermEvent::Key(key) => {
                    // Only process key press events, ignore key release and repeat events
                    if key.kind == KeyEventKind::Press {
                        if let Some(action) = self.handle_key(key, state) {
                            return Ok(Some(action));
                        }
                        // If no action for this key, fall through to Tick
                    }
                }
                CrosstermEvent::Mouse(mouse) => {
                    if let Some(action) = self.handle_mouse(mouse, state) {
                        return Ok(Some(action));
                    }
                    // If no action for this mouse event, fall through to Tick
                }
                _ => {}
            }
        }
        // Return Tick action to ensure UI refreshes periodically
        // This is important for updating TOTP countdown and other time-based displays
        Ok(Some(Action::Tick))
    }

    /// Convert key event to action (unified mode)
    fn handle_key(&self, key: KeyEvent, state: &AppState) -> Option<Action> {
        // Handle password input mode
        if state.password_input_mode() {
            return match (key.code, key.modifiers) {
                // Submit password
                (KeyCode::Enter, _) => Some(Action::SubmitPassword),
                // Cancel
                (KeyCode::Esc, _) => Some(Action::CancelPasswordInput),
                // Delete character
                (KeyCode::Backspace, _) => Some(Action::DeletePasswordChar),
                // Quit application (Ctrl+C always works)
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
                // Any other printable character
                (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                    Some(Action::AppendPasswordChar(c))
                }
                _ => None,
            };
        }

        // Handle save token prompt
        if state.offer_save_token() {
            return match (key.code, key.modifiers) {
                (KeyCode::Char('y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), KeyModifiers::NONE) | (KeyCode::Char('Y'), KeyModifiers::SHIFT) => {
                    Some(Action::SaveTokenYes)
                }
                (KeyCode::Char('n'), KeyModifiers::NONE) | (KeyCode::Char('N'), KeyModifiers::NONE) | (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
                    Some(Action::SaveTokenNo)
                }
                (KeyCode::Esc, _) => Some(Action::SaveTokenNo), // Esc = No
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
                _ => None,
            };
        }

        // Handle not logged in error popup
        if state.show_not_logged_in_error() {
            return match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(Action::Quit),
                _ => None,
            };
        }

        // Normal mode
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
            
            // Open details panel (doesn't close if already open)
            (KeyCode::Enter, _) => Some(Action::OpenDetailsPanel),
            
            // Actions with Ctrl modifier
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Action::CopyUsername),
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(Action::CopyPassword),
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Action::CopyTotp),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Action::Refresh),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => Some(Action::ToggleDetailsPanel),
            
            // Any other printable character updates the filter
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                Some(Action::AppendFilter(c))
            }
            
            _ => None,
        }
    }

    /// Convert mouse event to action
    fn handle_mouse(&self, mouse: MouseEvent, state: &AppState) -> Option<Action> {
        match mouse.kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                // Try details panel first (if visible)
                if state.details_panel_visible() {
                    let details_handler = DetailsClickHandler;
                    if let Some(action) = details_handler.handle_click(mouse, state, state.ui.details_panel_area) {
                        return Some(action);
                    }
                }
                
                // Try entry list
                let list_handler = EntryListClickHandler;
                if let Some(action) = list_handler.handle_click(mouse, state, state.ui.list_area) {
                    return Some(action);
                }
                
                None
            }
            MouseEventKind::ScrollUp => {
                // Scroll up moves selection up
                Some(Action::MoveUp)
            }
            MouseEventKind::ScrollDown => {
                // Scroll down moves selection down
                Some(Action::MoveDown)
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

