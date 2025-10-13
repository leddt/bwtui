use crate::state::AppState;
use crate::ui::widgets::clickable::{Clickable, is_click_in_area};
use crossterm::event::MouseEvent;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let items: Vec<ListItem> = state
        .vault.filtered_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == state.vault.selected_index;
            
            let style = if is_selected { theme::list_item_selected() } else { theme::list_item() };

            // Build display text
            let mut spans = vec![
                Span::styled(
                    if is_selected { "► " } else { "  " },
                    style,
                ),
            ];

            // Add favorite indicator
            if item.favorite {
                spans.push(Span::styled("★ ", theme::warning()));
            }

            // Add item name
            spans.push(Span::styled(&item.name, style));

            // Add username if available
            if let Some(username) = item.username() {
                spans.push(Span::styled(" ", style));
                spans.push(Span::styled(
                    format!("({})", username),
                    if is_selected { theme::list_item_selected() } else { theme::muted() },
                ));
            }

            // Add TOTP indicator
            if item.login.as_ref().and_then(|l| l.totp.as_ref()).is_some() {
                spans.push(Span::styled(" ", style));
                spans.push(Span::styled(
                    "[2FA]",
                    if is_selected { theme::list_item_selected() } else { theme::success() },
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut title = if !state.initial_load_complete() {
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

    // Add scroll indicators based on current offset and visible rows
    let inner_rows = area.height.saturating_sub(2) as usize; // account for borders
    let offset = state.vault.list_state.offset();
    if !state.vault.filtered_items.is_empty() && inner_rows > 0 {
        let has_more_above = offset > 0;
        let has_more_below = offset + inner_rows < state.vault.filtered_items.len();
        if has_more_above || has_more_below {
            title.push(' ');
            if has_more_above { title.push('⇡'); }
            if has_more_above && has_more_below { title.push(' '); }
            if has_more_below { title.push('⇣'); }
            title.push(' ');
        }
    }

    let title_style = if state.syncing() || !state.initial_load_complete() {
        theme::title_active()
    } else {
        theme::title()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(theme::BORDER_TYPE)
                .title(title)
                .title_style(title_style)
                .border_style(title_style),
        )
        .highlight_style(theme::list_item_selected());

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

