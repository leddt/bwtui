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
    CopyCardNumber,
    CopyCardCvv,
    FetchTotp,
    Refresh,
    ToggleDetailsPanel,
    OpenDetailsPanel,

    // Details panel scrolling
    ScrollDetailsUp,
    ScrollDetailsDown,

    // Password input actions
    SubmitPassword,
    CancelPasswordInput,
    AppendPasswordChar(char),
    DeletePasswordChar,

    // Save token actions
    SaveTokenYes,
    SaveTokenNo,

    // Details panel actions
    CloseDetailsPanel,

    // Tab switching
    SelectItemTypeTab(Option<crate::types::ItemType>),
    SelectTabByIndex(usize),
    CycleNextTab,
    CyclePreviousTab,
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
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
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
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
                _ => None,
            };
        }

        // Handle not logged in error popup
        if state.show_not_logged_in_error() {
            return match (key.code, key.modifiers) {
                (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),
                _ => None,
            };
        }

        // Normal mode
        match (key.code, key.modifiers) {
            // Escape key - close details panel if open, otherwise quit
            (KeyCode::Esc, _) => {
                if state.details_panel_visible() {
                    Some(Action::CloseDetailsPanel)
                } else {
                    Some(Action::Quit)
                }
            }

            // Quit
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Action::Quit),

            // Navigation - Vim style with Ctrl+Shift (details panel scrolling)
            (KeyCode::Char('K'), _) if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::ScrollDetailsUp),
            (KeyCode::Char('J'), _) if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::ScrollDetailsDown),

            // Navigation - Arrow keys with Shift (details panel scrolling)
            (KeyCode::Up, KeyModifiers::SHIFT) => Some(Action::ScrollDetailsUp),
            (KeyCode::Down, KeyModifiers::SHIFT) => Some(Action::ScrollDetailsDown),

            // Navigation - Vim style with Ctrl only (list navigation)
            #[allow(unreachable_patterns)]
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(Action::MoveUp),
            #[allow(unreachable_patterns)]
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => Some(Action::MoveDown),

            // Navigation - Arrow keys (list navigation)
            (KeyCode::Up, _) => Some(Action::MoveUp),
            (KeyCode::Down, _) => Some(Action::MoveDown),

            // Navigation - Page navigation
            (KeyCode::PageUp, _) => Some(Action::PageUp),
            (KeyCode::PageDown, _) => Some(Action::PageDown),
            (KeyCode::Home, _) => Some(Action::Home),
            (KeyCode::End, _) => Some(Action::End),

            // Filter editing
            (KeyCode::Backspace, _) => Some(Action::DeleteFilterChar),
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => Some(Action::ClearFilter),

            // Open details panel (doesn't close if already open)
            (KeyCode::Enter, _) => Some(Action::OpenDetailsPanel),

            // Actions with Ctrl modifier
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => Some(Action::CopyUsername),
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(Action::CopyPassword),
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Action::CopyTotp),
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => Some(Action::CopyCardNumber),
            (KeyCode::Char('m'), KeyModifiers::CONTROL) => Some(Action::CopyCardCvv),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Action::Refresh),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => Some(Action::ToggleDetailsPanel),

            // Tab switching with number keys (Ctrl+number for old behavior, number alone for new)
            (KeyCode::Char('1'), KeyModifiers::CONTROL) => Some(Action::SelectItemTypeTab(None)), // All types
            (KeyCode::Char('2'), KeyModifiers::CONTROL) => Some(Action::SelectItemTypeTab(Some(crate::types::ItemType::Login))),
            (KeyCode::Char('3'), KeyModifiers::CONTROL) => Some(Action::SelectItemTypeTab(Some(crate::types::ItemType::SecureNote))),
            (KeyCode::Char('4'), KeyModifiers::CONTROL) => Some(Action::SelectItemTypeTab(Some(crate::types::ItemType::Card))),
            (KeyCode::Char('5'), KeyModifiers::CONTROL) => Some(Action::SelectItemTypeTab(Some(crate::types::ItemType::Identity))),

            // Tab switching with number keys (direct selection)
            (KeyCode::Char('1'), KeyModifiers::NONE) => Some(Action::SelectTabByIndex(0)), // All types
            (KeyCode::Char('2'), KeyModifiers::NONE) => Some(Action::SelectTabByIndex(1)), // Login
            (KeyCode::Char('3'), KeyModifiers::NONE) => Some(Action::SelectTabByIndex(2)), // SecureNote
            (KeyCode::Char('4'), KeyModifiers::NONE) => Some(Action::SelectTabByIndex(3)), // Card
            (KeyCode::Char('5'), KeyModifiers::NONE) => Some(Action::SelectTabByIndex(4)), // Identity

            // Tab cycling with Tab key
            (KeyCode::Tab, KeyModifiers::SHIFT) => Some(Action::CyclePreviousTab),
            (KeyCode::Tab, _) => Some(Action::CycleNextTab),

            // Tab cycling with Left/Right arrow keys
            (KeyCode::Left, _) => Some(Action::CyclePreviousTab),
            (KeyCode::Right, _) => Some(Action::CycleNextTab),

            // Tab cycling with Ctrl+H/L (Vim-style)
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => Some(Action::CyclePreviousTab),
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => Some(Action::CycleNextTab),

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
