use crate::clipboard::ClipboardManager;
use crate::events::Action;
use crate::state::{AppState, MessageLevel};
use crate::cli::BitwardenCli;

/// Result of copy action handling
pub enum CopyResult {
    Handled,
    NeedTotpFetch,
    NotHandled,
}

/// Handle copy actions (username, password, TOTP)
pub fn handle_copy(
    action: &Action,
    state: &mut AppState,
    clipboard: Option<&mut ClipboardManager>,
    cli: Option<&BitwardenCli>,
) -> CopyResult {
    match action {
        Action::CopyUsername => {
            copy_username(state, clipboard);
            CopyResult::Handled
        }
        Action::CopyPassword => {
            copy_password(state, clipboard);
            CopyResult::Handled
        }
        Action::CopyTotp => {
            copy_totp(state, clipboard, cli)
        }
        Action::CopyCardNumber => {
            copy_card_number(state, clipboard);
            CopyResult::Handled
        }
        Action::CopyCardCvv => {
            copy_card_cvv(state, clipboard);
            CopyResult::Handled
        }
        _ => {
            CopyResult::NotHandled // Not a copy action
        }
    }
}

fn copy_username(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if let Some(item) = state.selected_item() {
        if let Some(username) = item.username() {
            if let Some(cb) = clipboard {
                match cb.copy(username) {
                    Ok(_) => {
                        crate::logger::Logger::info("Username copied to clipboard");
                        state.set_status(
                            format!("✓ Username copied: {}", username),
                            MessageLevel::Success,
                        );
                    }
                    Err(e) => {
                        crate::logger::Logger::error(&format!("Failed to copy username to clipboard: {}", e));
                        state.set_status(
                            "✗ Failed to copy to clipboard",
                            MessageLevel::Error,
                        );
                    }
                }
            } else {
                state.set_status("✗ Clipboard not available", MessageLevel::Error);
            }
        } else {
            state.set_status("✗ No username for this entry", MessageLevel::Warning);
        }
    }
}

fn copy_password(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if !state.secrets_available() {
        state.set_status(
            "⏳ Please wait, loading vault secrets...",
            MessageLevel::Warning,
        );
        return;
    }

    if let Some(item) = state.selected_item() {
        if let Some(login) = &item.login {
            if let Some(password) = &login.password {
                if let Some(cb) = clipboard {
                    match cb.copy(password) {
                        Ok(_) => {
                            crate::logger::Logger::info("Password copied to clipboard");
                            state.set_status(
                                "✓ Password copied to clipboard (hidden for security)",
                                MessageLevel::Success,
                            );
                        }
                        Err(e) => {
                            crate::logger::Logger::error(&format!("Failed to copy password to clipboard: {}", e));
                            state.set_status(
                                "✗ Failed to copy to clipboard",
                                MessageLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_status("✗ Clipboard not available", MessageLevel::Error);
                }
            } else {
                state.set_status("✗ No password for this entry", MessageLevel::Warning);
            }
        }
    }
}

fn copy_totp(state: &mut AppState, clipboard: Option<&mut ClipboardManager>, cli: Option<&BitwardenCli>) -> CopyResult {
    if !state.secrets_available() {
        state.set_status(
            "⏳ Please wait, loading vault secrets...",
            MessageLevel::Warning,
        );
        return CopyResult::Handled;
    }

    if let Some(item) = state.selected_item() {
        if let Some(login) = &item.login {
            if login.totp.is_some() {
                // First, try to use the current TOTP code if it's available and not expired
                if let Some(code) = state.current_totp_code() {
                    if !state.is_totp_expired() && state.totp_belongs_to_item(&item.id) {
                        // Use the existing code
                        if let Some(cb) = clipboard {
                            match cb.copy(code) {
                                Ok(_) => {
                                    crate::logger::Logger::info("TOTP code copied to clipboard");
                                    state.set_status(
                                        format!("✓ TOTP code copied: {}", code),
                                        MessageLevel::Success,
                                    );
                                }
                                Err(e) => {
                                    crate::logger::Logger::error(&format!("Failed to copy TOTP to clipboard: {}", e));
                                    state.set_status(
                                        "✗ Failed to copy to clipboard",
                                        MessageLevel::Error,
                                    );
                                }
                            }
                        } else {
                            state.set_status("✗ Clipboard not available", MessageLevel::Error);
                        }
                        return CopyResult::Handled;
                    }
                }

                // If we don't have a valid TOTP code, fetch it from CLI
                if let Some(_cli) = cli {
                    state.set_status(
                        "⏳ Fetching TOTP code...",
                        MessageLevel::Info,
                    );
                    
                    // Set loading state and copy pending - the actual fetching will be handled by the main loop
                    state.set_totp_loading(true);
                    state.set_totp_copy_pending(true);
                    return CopyResult::NeedTotpFetch;
                } else {
                    state.set_status(
                        "✗ Bitwarden CLI not available",
                        MessageLevel::Error,
                    );
                    return CopyResult::Handled;
                }
            } else {
                state.set_status(
                    "✗ No TOTP configured for this entry",
                    MessageLevel::Warning,
                );
                return CopyResult::Handled;
            }
        }
    }
    CopyResult::Handled
}

fn copy_card_number(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if !state.secrets_available() {
        state.set_status(
            "⏳ Please wait, loading vault secrets...",
            MessageLevel::Warning,
        );
        return;
    }

    if let Some(item) = state.selected_item() {
        if item.item_type != crate::types::ItemType::Card {
            state.set_status("✗ This is not a card entry", MessageLevel::Warning);
            return;
        }

        if let Some(card) = &item.card {
            if let Some(number) = &card.number {
                if let Some(cb) = clipboard {
                    match cb.copy(number) {
                        Ok(_) => {
                            crate::logger::Logger::info("Card number copied to clipboard");
                            state.set_status(
                                "✓ Card number copied to clipboard (hidden for security)",
                                MessageLevel::Success,
                            );
                        }
                        Err(e) => {
                            crate::logger::Logger::error(&format!("Failed to copy card number to clipboard: {}", e));
                            state.set_status(
                                "✗ Failed to copy to clipboard",
                                MessageLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_status("✗ Clipboard not available", MessageLevel::Error);
                }
            } else {
                state.set_status("✗ No card number for this entry", MessageLevel::Warning);
            }
        } else {
            state.set_status("✗ No card data for this entry", MessageLevel::Warning);
        }
    }
}

fn copy_card_cvv(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if !state.secrets_available() {
        state.set_status(
            "⏳ Please wait, loading vault secrets...",
            MessageLevel::Warning,
        );
        return;
    }

    if let Some(item) = state.selected_item() {
        if item.item_type != crate::types::ItemType::Card {
            state.set_status("✗ This is not a card entry", MessageLevel::Warning);
            return;
        }

        if let Some(card) = &item.card {
            if let Some(cvv) = &card.code {
                if let Some(cb) = clipboard {
                    match cb.copy(cvv) {
                        Ok(_) => {
                            crate::logger::Logger::info("CVV copied to clipboard");
                            state.set_status(
                                "✓ CVV copied to clipboard (hidden for security)",
                                MessageLevel::Success,
                            );
                        }
                        Err(e) => {
                            crate::logger::Logger::error(&format!("Failed to copy CVV to clipboard: {}", e));
                            state.set_status(
                                "✗ Failed to copy to clipboard",
                                MessageLevel::Error,
                            );
                        }
                    }
                } else {
                    state.set_status("✗ Clipboard not available", MessageLevel::Error);
                }
            } else {
                state.set_status("✗ No CVV for this entry", MessageLevel::Warning);
            }
        } else {
            state.set_status("✗ No card data for this entry", MessageLevel::Warning);
        }
    }
}

