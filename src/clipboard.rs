use arboard::Clipboard;
use crate::error::{BwError, Result};

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| {
                let error_msg = format!("Failed to initialize clipboard: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::ClipboardError(e.to_string())
            })?;
        
        crate::logger::Logger::info("Clipboard initialized successfully");
        Ok(Self { clipboard })
    }

    pub fn copy(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .map_err(|e| {
                let error_msg = format!("Failed to copy to clipboard: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::ClipboardError(e.to_string())
            })?;
        
        Ok(())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard manager")
    }
}


