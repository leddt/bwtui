use crate::state::AppState;
use crate::types::ItemType;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let active_filter = state.ui.get_active_filter();
    
    // Count items by type
    let mut counts = std::collections::HashMap::new();
    for item in &state.vault.vault_items {
        let count = counts.entry(&item.item_type).or_insert(0);
        *count += 1;
    }
    
    let total_count = state.vault.vault_items.len();
    let login_count = counts.get(&ItemType::Login).copied().unwrap_or(0);
    let note_count = counts.get(&ItemType::SecureNote).copied().unwrap_or(0);
    let card_count = counts.get(&ItemType::Card).copied().unwrap_or(0);
    let identity_count = counts.get(&ItemType::Identity).copied().unwrap_or(0);
    
    // Build tab spans
    let mut spans = Vec::new();
    
    // All tab
    let all_style = if active_filter.is_none() {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(
        format!("^1 All ({})", total_count),
        all_style,
    ));
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    
    // Login tab
    let login_style = if active_filter == Some(ItemType::Login) {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(
        format!("^2 Logins ({})", login_count),
        login_style,
    ));
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    
    // Notes tab
    let note_style = if active_filter == Some(ItemType::SecureNote) {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(
        format!("^3 Notes ({})", note_count),
        note_style,
    ));
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    
    // Cards tab
    let card_style = if active_filter == Some(ItemType::Card) {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(
        format!("^4 Cards ({})", card_count),
        card_style,
    ));
    spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    
    // Identities tab
    let identity_style = if active_filter == Some(ItemType::Identity) {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    spans.push(Span::styled(
        format!("^5 Identities ({})", identity_count),
        identity_style,
    ));
    
    let paragraph = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Item Types ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    
    frame.render_widget(paragraph, area);
}
