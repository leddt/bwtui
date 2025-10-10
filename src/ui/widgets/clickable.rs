use crate::state::AppState;
use crossterm::event::MouseEvent;
use ratatui::layout::Rect;

/// Trait for UI widgets that can handle mouse clicks
pub trait Clickable {
    /// Handle a mouse click within the widget's area
    /// Returns Some(Action) if the click was handled, None otherwise
    fn handle_click(&self, mouse: MouseEvent, state: &AppState, area: Rect) -> Option<crate::events::Action>;
}

/// Helper function to check if a mouse event is within a given area
pub fn is_click_in_area(mouse: MouseEvent, area: Rect) -> bool {
    mouse.column >= area.x 
        && mouse.column < area.x + area.width
        && mouse.row >= area.y
        && mouse.row < area.y + area.height
}
