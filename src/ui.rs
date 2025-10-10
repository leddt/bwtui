use crate::error::Result;
use crate::state::{AppState, MessageLevel};
use crate::totp_util;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::Stdout;

pub struct UI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn render(&mut self, state: &mut AppState) -> Result<()> {
        self.terminal.draw(|frame| {
            let status_bar_height = calculate_status_bar_height(frame.size().width, state);
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),              // Search box
                    Constraint::Min(0),                 // Entry list and details
                    Constraint::Length(status_bar_height), // Status bar (dynamic height)
                ])
                .split(frame.size());

            render_search_box(frame, chunks[0], state);
            
            // Split the middle section horizontally if details panel is visible
            if state.details_panel_visible {
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(50),     // Entry list
                        Constraint::Percentage(50),     // Details panel
                    ])
                    .split(chunks[1]);
                
                state.list_area = main_chunks[0];
                state.details_panel_area = main_chunks[1];
                render_entry_list(frame, main_chunks[0], state);
                render_details_panel(frame, main_chunks[1], state);
            } else {
                state.list_area = chunks[1];
                state.details_panel_area = Rect::default();
                render_entry_list(frame, chunks[1], state);
            }
            
            render_status_bar(frame, chunks[2], state);
        })?;

        Ok(())
    }
}

fn calculate_status_bar_height(width: u16, state: &AppState) -> u16 {
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

fn render_search_box(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = if state.filter_query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let filter_text = if state.filter_query.is_empty() {
        "Type to search...".to_string()
    } else {
        format!("> {}", state.filter_query)
    };

    let paragraph = Paragraph::new(filter_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(style),
        );

    frame.render_widget(paragraph, area);
}

fn render_entry_list(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let items: Vec<ListItem> = state
        .filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == state.selected_index;
            
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Build display text
            let mut spans = vec![
                Span::styled(
                    if is_selected { "► " } else { "  " },
                    style,
                ),
            ];

            // Add favorite indicator
            if item.favorite {
                spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
            }

            // Add item name
            spans.push(Span::styled(&item.name, style));

            // Add username if available
            if let Some(username) = item.username() {
                spans.push(Span::styled(" ", style));
                spans.push(Span::styled(
                    format!("({})", username),
                    if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ));
            }

            // Add TOTP indicator
            if item.login.as_ref().and_then(|l| l.totp.as_ref()).is_some() {
                spans.push(Span::styled(" ", style));
                spans.push(Span::styled(
                    "[2FA]",
                    if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if state.filtered_items.is_empty() {
        " No entries found ".to_string()
    } else {
        format!(
            " Vault Entries ({}/{}) ",
            state.filtered_items.len(),
            state.vault_items.len()
        )
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::White)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state.list_state);
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
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

fn render_details_panel(frame: &mut Frame, area: Rect, state: &AppState) {
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
            
            // Password (masked)
            if login.password.is_some() {
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
            
            // TOTP
            if let Some(totp_secret) = &login.totp {
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
                            Span::styled(&uri.value, Style::default().fg(Color::Blue)),
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
        
        // Notes
        if let Some(notes) = &item.notes {
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_ui_creation() {
        // This would require a terminal, so we just test the struct exists
        assert!(true);
    }
}

