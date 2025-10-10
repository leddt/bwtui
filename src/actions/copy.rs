use crate::clipboard::ClipboardManager;
use crate::events::Action;
use crate::state::{AppState, MessageLevel};
use crate::totp_util;

/// Handle copy actions (username, password, TOTP)
pub fn handle_copy(
    action: &Action,
    state: &mut AppState,
    clipboard: Option<&mut ClipboardManager>,
) -> bool {
    match action {
        Action::CopyUsername => {
            copy_username(state, clipboard);
        }
        Action::CopyPassword => {
            copy_password(state, clipboard);
        }
        Action::CopyTotp => {
            copy_totp(state, clipboard);
        }
        _ => {
            return false; // Not a copy action
        }
    }
    true
}

fn copy_username(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if let Some(item) = state.selected_item() {
        if let Some(username) = item.username() {
            if let Some(cb) = clipboard {
                match cb.copy(username) {
                    Ok(_) => {
                        state.set_status(
                            format!("✓ Username copied: {}", username),
                            MessageLevel::Success,
                        );
                    }
                    Err(_) => {
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
                            state.set_status(
                                "✓ Password copied to clipboard (hidden for security)",
                                MessageLevel::Success,
                            );
                        }
                        Err(_) => {
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

fn copy_totp(state: &mut AppState, clipboard: Option<&mut ClipboardManager>) {
    if !state.secrets_available() {
        state.set_status(
            "⏳ Please wait, loading vault secrets...",
            MessageLevel::Warning,
        );
        return;
    }

    if let Some(item) = state.selected_item() {
        if let Some(login) = &item.login {
            if let Some(totp_secret) = &login.totp {
                // Generate TOTP locally (much faster than CLI)
                match totp_util::generate_totp(totp_secret) {
                    Ok((code, _remaining)) => {
                        if let Some(cb) = clipboard {
                            match cb.copy(&code) {
                                Ok(_) => {
                                    state.set_status(
                                        format!("✓ TOTP code copied: {}", code),
                                        MessageLevel::Success,
                                    );
                                }
                                Err(_) => {
                                    state.set_status(
                                        "✗ Failed to copy to clipboard",
                                        MessageLevel::Error,
                                    );
                                }
                            }
                        } else {
                            state.set_status("✗ Clipboard not available", MessageLevel::Error);
                        }
                    }
                    Err(e) => {
                        state.set_status(
                            format!("✗ Failed to generate TOTP: {}", e),
                            MessageLevel::Error,
                        );
                    }
                }
            } else {
                state.set_status(
                    "✗ No TOTP configured for this entry",
                    MessageLevel::Warning,
                );
            }
        }
    }
}

