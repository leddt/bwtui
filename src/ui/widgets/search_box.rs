use crate::state::AppState;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = if state.vault.filter_query.is_empty() {
        theme::placeholder()
    } else {
        theme::input_active()
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
                .border_type(theme::BORDER_TYPE)
                .title(" Search ")
                .title_style(theme::title_active())
                .border_style(style),
        );

    frame.render_widget(paragraph, area);
}

