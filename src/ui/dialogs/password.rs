use crate::state::AppState;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 40, frame.size());
    
    // Clear the entire dialog area first
    frame.render_widget(Clear, area);
    
    // Clear the background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Unlock Vault ")
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(block.clone(), area);
    
    // Split into content area
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Instructions
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Password input
            Constraint::Length(1),  // Spacing
            Constraint::Min(0),     // Error message (if any)
            Constraint::Length(2),  // Help text
        ])
        .split(inner);
    
    // Instructions
    let instructions = Paragraph::new("Enter your master password to unlock the vault:")
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(instructions, chunks[0]);
    
    // Password input box
    let password_display = "•".repeat(state.ui.password_input.len());
    let password_widget = Paragraph::new(password_display)
        .style(Style::default().fg(Color::Yellow).bg(Color::Black))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Password ")
                .style(Style::default().bg(Color::Black)),
        );
    frame.render_widget(password_widget, chunks[2]);
    
    // Error message if any
    if let Some(error) = &state.ui.unlock_error {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red).bg(Color::Black))
            .wrap(Wrap { trim: false });
        frame.render_widget(error_widget, chunks[4]);
    }
    
    // Help text
    let help = Paragraph::new("Press Enter to submit, Esc to cancel")
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

