use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame) {
    let area = centered_rect(70, 35, frame.size());
    
    // Clear the entire dialog area first
    frame.render_widget(Clear, area);
    
    // Clear the background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Vault Not Logged In ")
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(block.clone(), area);
    
    // Split into content area
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Message
            Constraint::Length(2),  // Help text
        ])
        .split(inner);
    
    // Message
    let message_text = vec![
        "Your Bitwarden vault is not logged in.",
        "",
        "Please run the following command to log in:",
        "",
        "    bw login",
        "",
        "After logging in, restart this application.",
    ];
    
    let message = Paragraph::new(message_text.join("\n"))
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(message, chunks[0]);
    
    // Help text
    let help = Paragraph::new("Press Esc to exit")
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}

