use crate::state::AppState;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, _state: &AppState) {
    let area = centered_rect(70, 35, frame.size());
    
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
        "Would you like to save the session token to the BW_SESSION",
        "environment variable? This will keep you logged in between",
        "app executions and system restarts.",
        "",
        "Note: The token will be saved as a persistent user environment",
        "variable and will be available in all future sessions.",
    ];
    
    let message = Paragraph::new(message_text.join("\n"))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });
    frame.render_widget(message, chunks[0]);
    
    // Options
    let options = Paragraph::new("Press Y to save, N to skip")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(options, chunks[1]);
}

