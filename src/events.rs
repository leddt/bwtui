use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
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
                        return Ok(self.handle_key(key, state));
                    }
                }
                CrosstermEvent::Mouse(mouse) => {
                    return Ok(self.handle_mouse(mouse, state));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Convert key event to action (unified mode)
    fn handle_key(&self, key: KeyEvent, _state: &AppState) -> Option<Action> {
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
                let list_area = state.list_area;
                let details_area = state.details_panel_area;
                
                // Check if click is within the details panel (if visible)
                if state.details_panel_visible 
                    && mouse.column >= details_area.x 
                    && mouse.column < details_area.x + details_area.width
                    && mouse.row >= details_area.y
                    && mouse.row < details_area.y + details_area.height
                {
                    // Check if it's a copy button click
                    if let Some(action) = self.handle_details_panel_click(mouse, state, details_area) {
                        return Some(action);
                    }
                }
                
                // Check if click is within the list area
                if mouse.column >= list_area.x 
                    && mouse.column < list_area.x + list_area.width
                    && mouse.row >= list_area.y
                    && mouse.row < list_area.y + list_area.height
                {
                    // Calculate relative position within the list
                    let relative_y = mouse.row - list_area.y;
                    
                    // Account for the border (1 line at top)
                    if relative_y > 0 {
                        let item_index_in_view = (relative_y - 1) as usize;
                        
                        // Get the current scroll offset from the list state
                        let scroll_offset = state.list_state.offset();
                        
                        // Calculate the absolute index in the filtered list
                        let absolute_index = scroll_offset + item_index_in_view;
                        
                        // Only select if it's a valid item
                        if absolute_index < state.filtered_items.len() {
                            return Some(Action::SelectIndexAndShowDetails(absolute_index));
                        }
                    }
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

    /// Handle mouse clicks in the details panel to detect copy button clicks
    fn handle_details_panel_click(&self, mouse: MouseEvent, state: &AppState, area: ratatui::layout::Rect) -> Option<Action> {
        let selected_item = state.selected_item()?;
        let login = selected_item.login.as_ref()?;
        
        // Calculate relative position within the details panel
        let relative_y = mouse.row.saturating_sub(area.y);
        
        // Account for border (1 line at top)
        if relative_y == 0 {
            return None;
        }
        
        let content_line = relative_y - 1;
        
        // Layout of details panel (0-indexed from top of content):
        // 0: Name: <name>
        // 1: (blank)
        // 2: Username: <username>
        // 3:   [ Copy ^U ] (button)
        // 4: (blank)
        // 5: Password: ••••••••
        // 6:   [ Copy ^P ] (button)
        // 7: (blank)
        // 8: TOTP: <code>
        // 9:   [ Copy ^T ] (button)
        // 10: (blank)
        // ... URIs, notes, etc.
        
        let mut current_line = 0;
        
        // Name (2 lines: label + blank)
        current_line += 2;
        
        // Username section
        if login.username.is_some() {
            if content_line == current_line + 1 {
                // Clicked on username copy button
                return Some(Action::CopyUsername);
            }
            current_line += 3; // label, button, blank
        } else {
            current_line += 2; // label + blank (no button)
        }
        
        // Password section
        if login.password.is_some() {
            if content_line == current_line + 1 {
                // Clicked on password copy button
                return Some(Action::CopyPassword);
            }
            current_line += 3; // label, button, blank
        } else {
            current_line += 2; // label + blank (no button)
        }
        
        // TOTP section
        if login.totp.is_some() {
            if content_line == current_line + 1 {
                // Clicked on TOTP copy button
                return Some(Action::CopyTotp);
            }
        }
        
        None
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

