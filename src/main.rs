mod actions;
mod app;
mod cache;
mod cli;
mod clipboard;
mod error;
mod events;
mod logger;
mod session;
mod state;
mod terminal;
mod types;
mod ui;

use app::App;
use error::Result;
use events::EventHandler;
use session::SessionManager;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger early (before TUI starts)
    // If logger initialization fails, log to stderr but continue execution
    if let Err(e) = logger::Logger::init() {
        eprintln!("Warning: Failed to initialize logger: {}", e);
        eprintln!("Continuing without file logging...");
    } else {
        logger::Logger::info("Application starting");
    }
    
    // Run the application and handle cleanup
    let result = run().await;
    
    // Log shutdown
    logger::Logger::info("Application shutting down");
    
    // Ensure terminal is restored (best effort)
    terminal::ensure_cleanup();
    
    result
}

async fn run() -> Result<()> {
    // Setup terminal
    terminal::setup().map_err(|e| {
        logger::Logger::error(&format!("Failed to setup terminal: {}", e));
        e
    })?;

    // Initialize application
    let mut app = App::new();
    
    // Show clipboard warning if needed
    if app.should_show_clipboard_warning() {
        logger::Logger::warn("Clipboard not available");
        app.state.set_status("Warning: Clipboard not available", state::MessageLevel::Warning);
    }

    // Load cache and start vault initialization
    app.load_from_cache();
    app.start_vault_initialization();

    // Initialize UI, event handler, and session manager
    let mut ui = ui::UI::new().map_err(|e| {
        logger::Logger::error(&format!("Failed to initialize UI: {}", e));
        e
    })?;
    let event_handler = EventHandler::new();
    let session_manager = SessionManager::new().map_err(|e| {
        logger::Logger::error(&format!("Failed to initialize session manager: {}", e));
        e
    })?;

    // Main event loop
    loop {
        // Update app state and render UI
        if let Err(e) = app.update(&mut ui) {
            logger::Logger::error(&format!("Error updating app: {}", e));
            // Continue execution - don't break on update errors
        }

        // Poll for events with 100ms timeout for smooth animation
        match event_handler.poll_event(Duration::from_millis(100), &app.state) {
            Ok(Some(action)) => {
                // Handle the action (returns false if should quit)
                if !app.handle_action(action, &session_manager).await {
                    break;
                }
            }
            Ok(None) => {
                // No event, continue
            }
            Err(e) => {
                logger::Logger::error(&format!("Error polling events: {}", e));
                // Continue execution - don't break on poll errors
            }
        }
    }

    // Cleanup terminal
    terminal::cleanup().map_err(|e| {
        logger::Logger::error(&format!("Failed to cleanup terminal: {}", e));
        e
    })?;

    Ok(())
}
