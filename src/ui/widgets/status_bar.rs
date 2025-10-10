use crate::state::{AppState, MessageLevel};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_text = if let Some(status_msg) = &state.status_message {
        let style = match status_msg.level {
            MessageLevel::Info => Style::default().fg(Color::Cyan),
            MessageLevel::Success => Style::default().fg(Color::Green),
            MessageLevel::Warning => Style::default().fg(Color::Yellow),
            MessageLevel::Error => Style::default().fg(Color::Red),
        };

        Paragraph::new(status_msg.text.as_str())
            .style(style)
            .alignment(Alignment::Left)
    } else {
        // Show keybindings with wrapping support
        let bindings = vec![
            "↑↓:Navigate",
            "^U:Username",
            "^P:Password",
            "^T:TOTP",
            "^D:Details",
            "^R:Refresh",
            "ESC:Clear",
            "^C:Quit",
        ];

        let mut spans = Vec::new();
        for (i, binding) in bindings.iter().enumerate() {
            spans.push(Span::styled(*binding, Style::default().fg(Color::DarkGray)));
            if i < bindings.len() - 1 {
                spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            }
        }

        Paragraph::new(Line::from(spans))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(status_text, inner);
}

/// Calculate the height needed for the status bar
pub fn calculate_height(width: u16, state: &AppState) -> u16 {
    // If there's a status message, use fixed height
    if state.status_message.is_some() {
        return 3;
    }
    
    // Calculate height needed for keybindings
    let bindings = vec![
        "↑↓:Navigate",
        "^U:Username",
        "^P:Password",
        "^T:TOTP",
        "^D:Details",
        "^R:Refresh",
        "ESC:Clear",
        "^C:Quit",
    ];
    
    // Account for borders (2 chars) and some padding
    let available_width = width.saturating_sub(4) as usize;
    
    let mut current_line_width = 0;
    let mut lines_needed = 1;
    
    for (i, binding) in bindings.iter().enumerate() {
        let binding_width = binding.chars().count();
        let separator_width = if i < bindings.len() - 1 { 3 } else { 0 }; // " | "
        let total_width = binding_width + separator_width;
        
        if current_line_width + total_width > available_width && current_line_width > 0 {
            lines_needed += 1;
            current_line_width = binding_width + separator_width;
        } else {
            current_line_width += total_width;
        }
    }
    
    // Add 2 for top and bottom borders
    lines_needed as u16 + 2
}

