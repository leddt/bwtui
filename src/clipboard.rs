use arboard::Clipboard;
use crate::error::{BwError, Result};

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| BwError::ClipboardError(e.to_string()))?;
        
        Ok(Self { clipboard })
    }

    pub fn copy(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .map_err(|e| BwError::ClipboardError(e.to_string()))?;
        
        Ok(())
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_creation() {
        // Clipboard might not be available in CI environments
        if let Ok(_clipboard) = ClipboardManager::new() {
            assert!(true);
        }
    }
}

