use crate::state::AppState;
use crate::types::ItemType;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Tabs},
    Frame,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter, PartialEq, Eq)]
enum TabType {
    #[default]
    #[strum(to_string = "^1 All")]
    All,
    #[strum(to_string = "^2 Logins")]
    Login,
    #[strum(to_string = "^3 Notes")]
    SecureNote,
    #[strum(to_string = "^4 Cards")]
    Card,
    #[strum(to_string = "^5 Identities")]
    Identity,
}

impl TabType {
    fn from_item_type(item_type: Option<ItemType>) -> Self {
        match item_type {
            None => TabType::All,
            Some(ItemType::Login) => TabType::Login,
            Some(ItemType::SecureNote) => TabType::SecureNote,
            Some(ItemType::Card) => TabType::Card,
            Some(ItemType::Identity) => TabType::Identity,
        }
    }

    fn get_count(&self, state: &AppState) -> usize {
        match self {
            TabType::All => state.vault.vault_items.len(),
            TabType::Login => state.vault.vault_items.iter()
                .filter(|item| item.item_type == ItemType::Login)
                .count(),
            TabType::SecureNote => state.vault.vault_items.iter()
                .filter(|item| item.item_type == ItemType::SecureNote)
                .count(),
            TabType::Card => state.vault.vault_items.iter()
                .filter(|item| item.item_type == ItemType::Card)
                .count(),
            TabType::Identity => state.vault.vault_items.iter()
                .filter(|item| item.item_type == ItemType::Identity)
                .count(),
        }
    }

    fn title(&self, state: &AppState) -> Line<'static> {
        let count = self.get_count(state);
        format!("{} ({})", self, count)
            .fg(Color::White)
            .into()
    }

    fn highlight_style(&self) -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let active_filter = state.ui.get_active_filter();
    let current_tab = TabType::from_item_type(active_filter);
    
    // Create tab titles with counts
    let titles: Vec<Line> = TabType::iter()
        .map(|tab| tab.title(state))
        .collect();
    
    // Get the selected tab index
    let selected_index = TabType::iter()
        .position(|tab| tab == current_tab)
        .unwrap_or(0);
    
    // Create the Tabs widget
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Item Types ")
        )
        .select(selected_index)
        .highlight_style(current_tab.highlight_style())
        .divider("");
    
    frame.render_widget(tabs, area);
}
