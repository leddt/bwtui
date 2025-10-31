use crate::state::AppState;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[cfg(test)]
mod tests {
    use crate::state::AppState;

    #[test]
    fn test_password_input_functionality() {
        let mut state = AppState::new();
        state.enter_password_mode();
        
        // Test appending characters
        state.append_password_char('t');
        state.append_password_char('e');
        state.append_password_char('s');
        state.append_password_char('t');
        assert_eq!(state.get_password(), "test");
        
        // Test deleting characters
        state.delete_password_char();
        assert_eq!(state.get_password(), "tes");
        
        // Test clearing password
        state.clear_password();
        assert_eq!(state.get_password(), "");
        
        // Verify password mode state
        assert!(state.password_input_mode());
        state.exit_password_mode();
        assert!(!state.password_input_mode());
    }
}

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 40, frame.area());
    
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
    let instruction_text = if state.sync.syncing {
        format!("{} Unlocking vault...", state.sync.spinner())
    } else {
        "Enter your master password to unlock the vault:".to_string()
    };
    let instructions = Paragraph::new(instruction_text)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(instructions, chunks[0]);
    
    // Password input box
    let password_display = "•".repeat(state.ui.password_input.len());
    let password_style = if state.sync.syncing {
        Style::default().fg(Color::DarkGray).bg(Color::Black)
    } else {
        Style::default().fg(Color::Yellow).bg(Color::Black)
    };
    let password_border_style = if state.sync.syncing {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };
    let mut password_block = Block::default()
        .borders(Borders::ALL)
        .border_style(password_border_style)
        .title(" Password ")
        .style(Style::default().bg(Color::Black));

    // Add clear password shortcut on the right when there's text and not syncing
    if !state.ui.password_input.is_empty() && !state.sync.syncing {
        password_block = password_block.title(Line::from(" ^X:Clear ").alignment(Alignment::Right));
    }

    let password_widget = Paragraph::new(password_display)
        .style(password_style)
        .block(password_block);
    frame.render_widget(password_widget, chunks[2]);
    
    // Error message if any
    if let Some(error) = &state.ui.unlock_error {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red).bg(Color::Black))
            .wrap(Wrap { trim: false });
        frame.render_widget(error_widget, chunks[4]);
    }
    
    // Help text
    let help_text = if state.sync.syncing {
        "Please wait while the vault is being unlocked..."
    } else {
        "Press Enter to submit, Esc to cancel"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Black))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[5]);
}

