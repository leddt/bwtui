use crate::state::AppState;
use crate::totp_util;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let selected_item = state.selected_item();
    
    let content = if let Some(item) = selected_item {
        let mut lines = Vec::new();
        
        // Title/Name
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&item.name, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));
        
        // Username
        if let Some(login) = &item.login {
            if let Some(username) = &login.username {
                lines.push(Line::from(vec![
                    Span::styled("Username: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(username, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("[ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Copy ^U", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
                    Span::styled(" ]", Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Username: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("(none)", Style::default().fg(Color::DarkGray)),
                ]));
            }
            lines.push(Line::from(""));
            
            // Password (masked or loading)
            if !state.secrets_available() {
                // Show loading spinner when secrets are not yet available
                lines.push(Line::from(vec![
                    Span::styled("Password: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
                ]));
            } else if login.password.is_some() {
                lines.push(Line::from(vec![
                    Span::styled("Password: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("••••••••", Style::default().fg(Color::Yellow)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled("[ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Copy ^P", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
                    Span::styled(" ]", Style::default().fg(Color::DarkGray)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("Password: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("(none)", Style::default().fg(Color::DarkGray)),
                ]));
            }
            lines.push(Line::from(""));
            
            // TOTP (or loading)
            if !state.secrets_available() {
                // Show loading spinner when secrets are not yet available
                lines.push(Line::from(vec![
                    Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
                ]));
            } else if let Some(totp_secret) = &login.totp {
                match totp_util::generate_totp(totp_secret) {
                    Ok((code, remaining)) => {
                        lines.push(Line::from(vec![
                            Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled(code, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::styled(format!(" ({}s)", remaining), Style::default().fg(Color::DarkGray)),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("  ", Style::default()),
                            Span::styled("[ ", Style::default().fg(Color::DarkGray)),
                            Span::styled("Copy ^T", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
                            Span::styled(" ]", Style::default().fg(Color::DarkGray)),
                        ]));
                    }
                    Err(_) => {
                        lines.push(Line::from(vec![
                            Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::styled("(invalid secret)", Style::default().fg(Color::Red)),
                        ]));
                    }
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("(none)", Style::default().fg(Color::DarkGray)),
                ]));
            }
            lines.push(Line::from(""));
            
            // URIs
            if let Some(uris) = &login.uris {
                if !uris.is_empty() {
                    lines.push(Line::from(Span::styled("URIs: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                    for uri in uris.iter().take(3) {
                        lines.push(Line::from(vec![
                            Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                            Span::styled(&uri.uri, Style::default().fg(Color::Blue)),
                        ]));
                    }
                    if uris.len() > 3 {
                        lines.push(Line::from(Span::styled(
                            format!("  ... and {} more", uris.len() - 3),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                    lines.push(Line::from(""));
                }
            }
        }
        
        // Notes (or loading)
        if !state.secrets_available() {
            // Show loading spinner when secrets are not yet available
            lines.push(Line::from(vec![
                Span::styled("Notes: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if let Some(notes) = &item.notes {
            if !notes.is_empty() {
                lines.push(Line::from(Span::styled("Notes: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                lines.push(Line::from(""));
                
                // Split notes by newlines and display
                for line in notes.lines().take(10) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(Color::White))));
                }
                
                let note_lines = notes.lines().count();
                if note_lines > 10 {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("... and {} more lines", note_lines - 10),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }
        
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Details ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false })
    } else {
        Paragraph::new("No item selected")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Details ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
    };
    
    frame.render_widget(content, area);
}

