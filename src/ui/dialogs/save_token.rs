use crate::state::AppState;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, _state: &AppState) {
    let area = centered_rect(70, 35, frame.area());
    
    // Clear the entire dialog area first
    frame.render_widget(Clear, area);
    
    // Clear the background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" Save Session Token ")
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(block.clone(), area);
    
    // Split into content area
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Message
            Constraint::Length(2),  // Options
        ])
        .split(inner);
    
    // Message
    let message_text = vec![
        "Vault unlocked successfully!",
        "",
        "Would you like to save the session token securely?",
        "This will keep you logged in between app executions",
        "and system restarts.",
        "",
        "The token will be encrypted using your system's secure",
        "storage. Only you will be able to access it.",
    ];
    
    let message = Paragraph::new(message_text.join("\n"))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(message, chunks[0]);
    
    // Options
    let options = Paragraph::new("Press Y to save, N to skip")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::Black))
        .alignment(Alignment::Center);
    frame.render_widget(options, chunks[1]);
}

