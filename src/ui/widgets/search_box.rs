use crate::state::AppState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = if state.vault.filter_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let filter_text = if state.vault.filter_query.is_empty() {
        "Type to search...".to_string()
    } else {
        format!("> {}", state.vault.filter_query)
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(" Search ")
        .border_style(style);

    // Add clear search shortcut on the right when there's text
    if !state.vault.filter_query.is_empty() {
        block = block.title(Line::from("^X:Clear search").alignment(Alignment::Right));
    }

    let paragraph = Paragraph::new(filter_text)
        .style(style)
        .block(block);

    frame.render_widget(paragraph, area);
}

