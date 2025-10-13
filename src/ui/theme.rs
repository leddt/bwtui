use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

pub mod colors {
    use super::*;
    pub const ACCENT: Color = Color::Cyan;
    pub const TEXT: Color = Color::White;
    pub const MUTED: Color = Color::DarkGray;
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const DANGER: Color = Color::Red;

    pub const HIGHLIGHT_BG: Color = Color::Cyan;
    pub const HIGHLIGHT_FG: Color = Color::Black;
}

pub const BORDER_TYPE: BorderType = BorderType::Rounded;

pub fn title_active() -> Style {
    Style::default().fg(colors::ACCENT)
}

pub fn title() -> Style {
    Style::default().fg(colors::TEXT)
}

pub fn placeholder() -> Style {
    Style::default().fg(colors::MUTED)
}

pub fn input_active() -> Style {
    Style::default().fg(colors::WARNING)
}

pub fn list_item_selected() -> Style {
    Style::default()
        .fg(colors::HIGHLIGHT_FG)
        .bg(colors::HIGHLIGHT_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn list_item() -> Style {
    Style::default().fg(colors::TEXT)
}

pub fn label() -> Style {
    Style::default()
        .fg(colors::ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn value() -> Style {
    Style::default().fg(colors::TEXT)
}

pub fn muted() -> Style {
    Style::default().fg(colors::MUTED)
}

pub fn success() -> Style {
    Style::default().fg(colors::SUCCESS)
}

pub fn warning() -> Style {
    Style::default().fg(colors::WARNING)
}

pub fn danger() -> Style {
    Style::default().fg(colors::DANGER)
}
