use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
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

    let paragraph = Paragraph::new(filter_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(style),
        );

    frame.render_widget(paragraph, area);
}

