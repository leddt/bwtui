use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let items: Vec<ListItem> = state
        .vault.filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == state.vault.selected_index;
            
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

    let title = if !state.initial_load_complete() {
        // Show spinner during initial load
        format!(" {} Loading vault... ", state.sync_spinner())
    } else if state.vault.filtered_items.is_empty() {
        " No entries found ".to_string()
    } else if state.syncing() {
        format!(
            " Vault Entries ({}/{}) {} Syncing... ",
            state.vault.filtered_items.len(),
            state.vault.vault_items.len(),
            state.sync_spinner()
        )
    } else {
        format!(
            " Vault Entries ({}/{}) ",
            state.vault.filtered_items.len(),
            state.vault.vault_items.len()
        )
    };

    let title_style = if state.syncing() || !state.initial_load_complete() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(title_style),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state.vault.list_state);
}

