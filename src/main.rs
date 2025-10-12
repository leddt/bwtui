mod actions;
mod app;
mod cache;
mod cli;
mod clipboard;
mod error;
mod events;
mod session;
mod state;
mod terminal;
mod totp_util;
mod types;
mod ui;

use app::App;
use error::Result;
use events::EventHandler;
use session::SessionManager;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Run the application and handle cleanup
    let result = run().await;
    
    // Ensure terminal is restored (best effort)
    terminal::ensure_cleanup();
    
    result
}

async fn run() -> Result<()> {
    // Setup terminal
    terminal::setup()?;

    // Initialize application
    let mut app = App::new();
    
    // Show clipboard warning if needed
    if app.should_show_clipboard_warning() {
        app.state.set_status("Warning: Clipboard not available", state::MessageLevel::Warning);
    }

    // Load cache and start vault initialization
    app.load_from_cache();
    app.start_vault_initialization();

    // Initialize UI, event handler, and session manager
    let mut ui = ui::UI::new()?;
    let event_handler = EventHandler::new();
    let session_manager = SessionManager::new()?;

    // Main event loop
    loop {
        // Update app state and render UI
        app.update(&mut ui)?;

        // Poll for events with 100ms timeout for smooth animation
        if let Ok(Some(action)) = event_handler.poll_event(Duration::from_millis(100), &app.state) {
            // Handle the action (returns false if should quit)
            if !app.handle_action(action, &session_manager).await {
                break;
            }
        }
    }

    // Cleanup terminal
    terminal::cleanup()?;

    Ok(())
}
