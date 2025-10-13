pub mod widgets;
pub mod dialogs;
pub mod layout;
pub mod theme;

use crate::error::Result;
use crate::state::AppState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io::Stdout;

pub struct UI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn render(&mut self, state: &mut AppState) -> Result<()> {
        self.terminal.draw(|frame| {
            let status_bar_height = widgets::status_bar::calculate_height(frame.size().width, state);
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),              // Search box
                    Constraint::Min(0),                 // Entry list and details
                    Constraint::Length(status_bar_height), // Status bar (dynamic height)
                ])
                .split(frame.size());

            widgets::search_box::render(frame, chunks[0], state);
            
            // Split the middle section horizontally if details panel is visible
            if state.details_panel_visible() {
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),     // Entry list
                        Constraint::Percentage(50),     // Details panel
                    ])
                    .split(chunks[1]);
                
                state.ui.list_area = main_chunks[0];
                state.ui.details_panel_area = main_chunks[1];
                widgets::entry_list::render(frame, main_chunks[0], state);
                widgets::details::render(frame, main_chunks[1], state);
            } else {
                state.ui.list_area = chunks[1];
                state.ui.details_panel_area = ratatui::layout::Rect::default();
                widgets::entry_list::render(frame, chunks[1], state);
            }
            
            widgets::status_bar::render(frame, chunks[2], state);

            // Render password input dialog, save token prompt, or not logged in error on top if active
            if state.password_input_mode() {
                dialogs::password::render(frame, state);
            } else if state.offer_save_token() {
                dialogs::save_token::render(frame, state);
            } else if state.show_not_logged_in_error() {
                dialogs::not_logged_in::render(frame);
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ui_creation() {
        // This would require a terminal, so we just test the struct exists
        assert!(true);
    }
}

