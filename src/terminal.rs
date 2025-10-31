use crate::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::Stdout;

/// Setup the terminal for TUI mode
pub fn setup() -> Result<Stdout> {
    enable_raw_mode().map_err(|e| {
        let error_msg = format!("Failed to enable raw mode: {}", e);
        crate::logger::Logger::error(&error_msg);
        e
    })?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        let error_msg = format!("Failed to setup terminal: {}", e);
        crate::logger::Logger::error(&error_msg);
        e
    })?;
    crate::logger::Logger::info("Terminal setup completed");
    Ok(stdout)
}

/// Restore the terminal to normal mode
pub fn cleanup() -> Result<()> {
    disable_raw_mode().map_err(|e| {
        let error_msg = format!("Failed to disable raw mode: {}", e);
        crate::logger::Logger::error(&error_msg);
        e
    })?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture).map_err(|e| {
        let error_msg = format!("Failed to cleanup terminal: {}", e);
        crate::logger::Logger::error(&error_msg);
        e
    })?;
    crate::logger::Logger::info("Terminal cleanup completed");
    Ok(())
}

/// Ensure terminal is restored (best effort, ignores errors)
pub fn ensure_cleanup() {
    if let Err(e) = disable_raw_mode() {
        crate::logger::Logger::warn(&format!("Failed to disable raw mode during cleanup: {}", e));
    }
    if let Err(e) = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture) {
        crate::logger::Logger::warn(&format!("Failed to cleanup terminal: {}", e));
    }
}

