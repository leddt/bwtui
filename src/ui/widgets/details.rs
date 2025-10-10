use crate::state::AppState;
use crate::totp_util;
use crate::ui::widgets::clickable::{Clickable, is_click_in_area};
use crossterm::event::MouseEvent;
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
                    Span::styled(" [^U]", Style::default().fg(Color::DarkGray)),
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
                    Span::styled(" [^P]", Style::default().fg(Color::DarkGray)),
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
                            Span::styled(" [^T]", Style::default().fg(Color::DarkGray)),
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

/// Details panel click handler
pub struct DetailsClickHandler;

impl Clickable for DetailsClickHandler {
    fn handle_click(&self, mouse: MouseEvent, state: &AppState, area: Rect) -> Option<crate::events::Action> {
        if !is_click_in_area(mouse, area) {
            return None;
        }

        let selected_item = state.selected_item()?;
        let login = selected_item.login.as_ref()?;
        
        // Calculate relative position within the details panel
        let relative_y = mouse.row.saturating_sub(area.y);
        let relative_x = mouse.column.saturating_sub(area.x);
        
        // Account for border (1 line at top)
        if relative_y == 0 {
            return None;
        }
        
        let content_line = relative_y - 1;
        
        // Layout of details panel (0-indexed from top of content):
        // 0: Name: <name>
        // 1: (blank)
        // 2: Username: <username> [^U]  (if username exists)
        // 3: (blank)
        // 4: Password: •••••••• [^P]    (if password exists)
        // 5: (blank)
        // 6: TOTP: <code> (Xs) [^T]     (if TOTP exists)
        // 7: (blank)
        // ... URIs, notes, etc.
        
        let mut current_line = 0;
        
        // Name (2 lines: label + blank)
        current_line += 2;
        
        // Username section
        if login.username.is_some() {
            if content_line == current_line {
                // Calculate approximate position of [^U] at end of line
                // "Username: " (10) + username length + " [" (2) + "^U" (2) + "]" (1) = 15 + username length
                let username_len = login.username.as_ref().unwrap().len() as u16;
                let shortcut_start = 10 + username_len + 2; // After "Username: " + username + " ["
                let shortcut_end = shortcut_start + 3; // "[^U]" is 4 characters
                
                if relative_x >= shortcut_start && relative_x <= shortcut_end {
                    return Some(crate::events::Action::CopyUsername);
                }
            }
            current_line += 2; // label + blank
        } else {
            current_line += 2; // label + blank (no button)
        }
        
        // Password section
        if login.password.is_some() {
            if content_line == current_line {
                // Calculate approximate position of [^P] at end of line
                // "Password: " (10) + "••••••••" (8) + " [" (2) + "^P" (2) + "]" (1) = 23
                let shortcut_start = 20; // After "Password: •••••••• ["
                let shortcut_end = shortcut_start + 3; // "[^P]" is 4 characters
                
                if relative_x >= shortcut_start && relative_x <= shortcut_end {
                    return Some(crate::events::Action::CopyPassword);
                }
            }
            current_line += 2; // label + blank
        } else {
            current_line += 2; // label + blank (no button)
        }
        
        // TOTP section
        if login.totp.is_some() {
            if content_line == current_line {
                // Calculate approximate position of [^T] at end of line
                // "TOTP: " (6) + "123456" (6) + " (Xs)" (5) + " [" (2) + "^T" (2) + "]" (1) = 22
                let shortcut_start = 19; // After "TOTP: 123456 (Xs) ["
                let shortcut_end = shortcut_start + 3; // "[^T]" is 4 characters
                
                if relative_x >= shortcut_start && relative_x <= shortcut_end {
                    return Some(crate::events::Action::CopyTotp);
                }
            }
        }
        
        None
    }
}

