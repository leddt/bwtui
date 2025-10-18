use crate::state::AppState;
use crate::ui::widgets::clickable::{Clickable, is_click_in_area};
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Alignment, Rect},
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

            // Add type indicator
            let type_indicator = match item.item_type {
                crate::types::ItemType::Login => "🔑",
                crate::types::ItemType::SecureNote => "📝",
                crate::types::ItemType::Card => "💳",
                crate::types::ItemType::Identity => "👤",
            };
            spans.push(Span::styled(type_indicator, Style::default().fg(Color::Yellow)));
            spans.push(Span::styled(" ", style));

            // Add item name
            spans.push(Span::styled(&item.name, style));

            // Add type-specific subtitle
            let subtitle = match item.item_type {
                crate::types::ItemType::Login => {
                    item.username().map(|u| format!("({})", u))
                }
                crate::types::ItemType::SecureNote => {
                    None // No subtitle for notes
                }
                crate::types::ItemType::Card => {
                    item.card_brand().map(|b| format!("({})", b))
                }
                crate::types::ItemType::Identity => {
                    item.identity_email().map(|e| format!("({})", e))
                }
            };

            if let Some(subtitle) = subtitle {
                spans.push(Span::styled(" ", style));
                spans.push(Span::styled(
                    subtitle,
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

    // Create the block with conditional right-aligned syncing indicator
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_bottom(Line::from(" ↑↓:Navigate "))
        .border_style(title_style);

    // Add syncing indicator on the right when syncing (but not during initial load)
    if state.syncing() && state.initial_load_complete() {
        block = block.title(Line::from(format!(" {} Syncing... ", state.sync_spinner())).alignment(Alignment::Right));
    }

    let list = List::new(items).block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state.vault.list_state);
}

/// Entry list click handler
pub struct EntryListClickHandler;

impl Clickable for EntryListClickHandler {
    fn handle_click(&self, mouse: MouseEvent, state: &AppState, area: Rect) -> Option<crate::events::Action> {
        if !is_click_in_area(mouse, area) {
            return None;
        }

        // Calculate relative position within the list
        let relative_y = mouse.row - area.y;
        
        // Account for the border (1 line at top)
        if relative_y > 0 {
            let item_index_in_view = (relative_y - 1) as usize;
            
            // Get the current scroll offset from the list state
            let scroll_offset = state.vault.list_state.offset();
            
            // Calculate the absolute index in the filtered list
            let absolute_index = scroll_offset + item_index_in_view;
            
            // Only select if it's a valid item
            if absolute_index < state.vault.filtered_items.len() {
                return Some(crate::events::Action::SelectIndexAndShowDetails(absolute_index));
            }
        }
        
        None
    }
}

