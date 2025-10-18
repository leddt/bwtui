use crate::state::AppState;
use crate::ui::widgets::clickable::{Clickable, is_click_in_area};
use crossterm::event::MouseEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    if let Some(item) = state.selected_item() {
        // Generate all content lines
        let mut lines = Vec::new();
        
        // Title/Name
        lines.push(Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&item.name, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));
        
        // Render type-specific content
        match item.item_type {
            crate::types::ItemType::Login => {
                render_login_details(&mut lines, item, state);
            }
            crate::types::ItemType::SecureNote => {
                render_secure_note_details(&mut lines, item, state);
            }
            crate::types::ItemType::Card => {
                render_card_details(&mut lines, item, state);
            }
            crate::types::ItemType::Identity => {
                render_identity_details(&mut lines, item, state);
            }
        }
        
        // Notes (common to all types)
        if !state.secrets_available() {
            // Show loading spinner when secrets are not yet available
            lines.push(Line::from(vec![
                Span::styled("Notes: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if let Some(notes) = &item.notes {
            if !notes.is_empty() {
                lines.push(Line::from(Span::styled("Notes: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                
                // Split notes by newlines and display all lines
                for line in notes.lines() {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(Color::White))));
                }
            }
        }
        
        // Custom fields (common to all types)
        if !state.secrets_available() {
            // Show loading spinner when secrets are not yet available
                lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Custom Fields: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if let Some(fields) = &item.fields {
            if !fields.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("Custom Fields: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                
                for field in fields.iter() {
                    if let (Some(name), Some(value)) = (&field.name, &field.value) {
                        if !name.is_empty() && !value.is_empty() {
                            lines.push(Line::from(vec![
                                Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                                Span::styled(format!("{}: ", name), Style::default().fg(Color::Cyan)),
                                Span::styled(value, Style::default().fg(Color::White)),
                            ]));
                        }
                    }
                }
            }
        }
        
        // Calculate the actual content height after wrapping
        let available_width = area.width.saturating_sub(2); // Account for borders
        let available_height = area.height.saturating_sub(2); // Account for borders
        
        // Calculate how many lines the content will actually take after wrapping
        let content_height = lines.iter().map(|line| {
            let line_width = line.width() as u16;
            if line_width > available_width {
                (line_width / available_width) + 1
            } else {
                1
            }
        }).sum::<u16>() as usize;
        
        // Create the paragraph
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Details ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });
        
        let max_visible_lines = available_height as usize;
        
        // Calculate maximum scroll position based on actual content height
        // Allow some overscroll to ensure scrollbar reaches the bottom
        let max_scroll = if content_height > max_visible_lines {
            content_height - max_visible_lines + 3
        } else {
            0
        };
        
        // Get current scroll position and clamp it
        let scroll_offset = state.ui.details_panel_scroll.min(max_scroll);
        
        // Apply scrolling to the paragraph
        let scrolled_paragraph = paragraph.scroll((scroll_offset as u16, 0));
        
        // Render the paragraph
        frame.render_widget(scrolled_paragraph, area);
        
        // Render scrollbar if content overflows
        if content_height > max_visible_lines {
            // The scrollbar should represent the total content height
            // With overscroll, we can use the scroll offset directly
            let mut scrollbar_state = ScrollbarState::new(content_height)
                .position(scroll_offset);
            
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("█");
            
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
        
        // Update state with the calculated max scroll after rendering
        state.set_details_max_scroll(max_scroll);
    } else {
        // No item selected
        let paragraph = Paragraph::new("No item selected")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Details ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        
        frame.render_widget(paragraph, area);
    }
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
        
        // Adjust for scroll offset
        let scroll_offset = state.ui.details_panel_scroll;
        let content_line = (relative_y - 1) as usize + scroll_offset;
        
        // Generate the same content structure as the render function to find clickable areas
        let mut lines = Vec::new();
        
        // Title/Name (2 lines: label + blank)
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        
        // Username section
        if login.username.is_some() {
            lines.push(Line::from("")); // Username line
            lines.push(Line::from("")); // Blank line
        } else {
            lines.push(Line::from("")); // Username line (no button)
            lines.push(Line::from("")); // Blank line
        }
        
        // Password section
        if login.password.is_some() {
            lines.push(Line::from("")); // Password line
            lines.push(Line::from("")); // Blank line
        } else {
            lines.push(Line::from("")); // Password line (no button)
            lines.push(Line::from("")); // Blank line
        }
        
        // TOTP section
        if login.totp.is_some() {
            lines.push(Line::from("")); // TOTP line
            lines.push(Line::from("")); // Blank line
        } else {
            lines.push(Line::from("")); // TOTP line (no button)
            lines.push(Line::from("")); // Blank line
        }
        
        // Check if we're clicking on a clickable line
        let mut current_line = 0;
        
        // Name (2 lines: label + blank)
        current_line += 2;
        
        // Username section
        if login.username.is_some() {
            if content_line == current_line {
                // Calculate approximate position of [^U] at end of line
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
                // Check if we have a TOTP code displayed
                if state.current_totp_code().is_some() {
                    // Calculate approximate position of [^T] at end of line
                    let shortcut_start = 19; // After "TOTP: 123456 (Xs) ["
                    let shortcut_end = shortcut_start + 3; // "[^T]" is 4 characters
                    
                    if relative_x >= shortcut_start && relative_x <= shortcut_end {
                        return Some(crate::events::Action::CopyTotp);
                    }
                } else {
                    // No TOTP code displayed, clicking anywhere on the line should fetch it
                    return Some(crate::events::Action::FetchTotp);
                }
            }
        }
        
        None
    }
}

/// Render login-specific details
fn render_login_details<'a>(lines: &mut Vec<Line<'a>>, item: &'a crate::types::VaultItem, state: &AppState) {
    if let Some(login) = &item.login {
        // Username
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
        
        // Password (masked or loading)
        if !state.secrets_available() {
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
        
        // TOTP (or loading)
        if !state.secrets_available() {
            lines.push(Line::from(vec![
                Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if let Some(_totp_secret) = &login.totp {
            if state.totp_loading() {
                lines.push(Line::from(vec![
                    Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
                ]));
            } else if let Some(code) = state.current_totp_code() {
                if let Some(remaining) = state.totp_remaining_seconds() {
                    lines.push(Line::from(vec![
                        Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(code.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({}s)", remaining), Style::default().fg(Color::DarkGray)),
                        Span::styled(" [^T]", Style::default().fg(Color::DarkGray)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(code.clone(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(" [^T]", Style::default().fg(Color::DarkGray)),
                    ]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("TOTP: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("(click to load)", Style::default().fg(Color::DarkGray)),
                ]));
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
                for uri in uris.iter() {
                    lines.push(Line::from(vec![
                        Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                        Span::styled(&uri.uri, Style::default().fg(Color::Blue)),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }
    }
}

/// Render secure note-specific details
fn render_secure_note_details<'a>(_lines: &mut Vec<Line<'a>>, _item: &'a crate::types::VaultItem, _state: &AppState) {
    // Secure notes only have name and notes, which are handled in the common section
    // No additional fields needed
}

/// Render card-specific details
fn render_card_details<'a>(lines: &mut Vec<Line<'a>>, item: &'a crate::types::VaultItem, state: &AppState) {
    if let Some(card) = &item.card {
        // Brand
        if let Some(brand) = &card.brand {
            lines.push(Line::from(vec![
                Span::styled("Brand: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(brand, Style::default().fg(Color::White)),
            ]));
        }
        
        // Cardholder Name
        if let Some(name) = &card.card_holder_name {
            lines.push(Line::from(vec![
                Span::styled("Cardholder: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(name, Style::default().fg(Color::White)),
            ]));
        }
        
        // Card Number (masked or loading)
        if !state.secrets_available() {
            lines.push(Line::from(vec![
                Span::styled("Number: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if card.number.is_some() {
            lines.push(Line::from(vec![
                Span::styled("Number: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("••••-••••-••••-••••", Style::default().fg(Color::Yellow)),
                Span::styled(" [^N]", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Number: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("(none)", Style::default().fg(Color::DarkGray)),
            ]));
        }
        
        // Expiry
        if let (Some(month), Some(year)) = (&card.exp_month, &card.exp_year) {
            lines.push(Line::from(vec![
                Span::styled("Expiry: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}/{}", month, year), Style::default().fg(Color::White)),
            ]));
        }
        
        // CVV (masked or loading)
        if !state.secrets_available() {
            lines.push(Line::from(vec![
                Span::styled("CVV: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} Loading...", state.sync_spinner()), Style::default().fg(Color::Yellow)),
            ]));
        } else if card.code.is_some() {
            lines.push(Line::from(vec![
                Span::styled("CVV: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("•••", Style::default().fg(Color::Yellow)),
                Span::styled(" [^M]", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("CVV: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("(none)", Style::default().fg(Color::DarkGray)),
            ]));
        }
        lines.push(Line::from(""));
    }
}

/// Render identity-specific details
fn render_identity_details<'a>(lines: &mut Vec<Line<'a>>, item: &'a crate::types::VaultItem, _state: &AppState) {
    if let Some(identity) = &item.identity {
        // Name section
        let mut name_parts = Vec::new();
        if let Some(title) = &identity.title {
            name_parts.push(title.clone());
        }
        if let Some(first) = &identity.first_name {
            name_parts.push(first.clone());
        }
        if let Some(middle) = &identity.middle_name {
            name_parts.push(middle.clone());
        }
        if let Some(last) = &identity.last_name {
            name_parts.push(last.clone());
        }
        
        if !name_parts.is_empty() {
            lines.push(Line::from(Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled(name_parts.join(" "), Style::default().fg(Color::White))));
            lines.push(Line::from(""));
        }
        
        // Address section
        let mut address_parts = Vec::new();
        if let Some(addr1) = &identity.address1 {
            address_parts.push(addr1.clone());
        }
        if let Some(addr2) = &identity.address2 {
            address_parts.push(addr2.clone());
        }
        if let Some(addr3) = &identity.address3 {
            address_parts.push(addr3.clone());
        }
        if let Some(city) = &identity.city {
            address_parts.push(city.clone());
        }
        if let Some(state) = &identity.state {
            address_parts.push(state.clone());
        }
        if let Some(postal) = &identity.postal_code {
            address_parts.push(postal.clone());
        }
        if let Some(country) = &identity.country {
            address_parts.push(country.clone());
        }
        
        if !address_parts.is_empty() {
            lines.push(Line::from(Span::styled("Address: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled(address_parts.join(", "), Style::default().fg(Color::White))));
            lines.push(Line::from(""));
        }
        
        // Contact section
        if let Some(phone) = &identity.phone {
            lines.push(Line::from(vec![
                Span::styled("Phone: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(phone, Style::default().fg(Color::White)),
            ]));
        }
        if let Some(email) = &identity.email {
            lines.push(Line::from(vec![
                Span::styled("Email: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(email, Style::default().fg(Color::White)),
            ]));
        }
        if let Some(username) = &identity.username {
            lines.push(Line::from(vec![
                Span::styled("Username: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(username, Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(""));
        
        // ID section
        if let Some(ssn) = &identity.ssn {
            lines.push(Line::from(vec![
                Span::styled("SSN: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(ssn, Style::default().fg(Color::White)),
            ]));
        }
        if let Some(license) = &identity.license_number {
            lines.push(Line::from(vec![
                Span::styled("License: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(license, Style::default().fg(Color::White)),
            ]));
        }
        if let Some(passport) = &identity.passport_number {
            lines.push(Line::from(vec![
                Span::styled("Passport: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(passport, Style::default().fg(Color::White)),
            ]));
        }
        lines.push(Line::from(""));
    }
}

