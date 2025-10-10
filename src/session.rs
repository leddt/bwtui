use crate::error::{BwError, Result};

/// Session token manager with platform-specific persistence
pub struct SessionManager;

impl SessionManager {
    pub fn new() -> Self {
        Self
    }

    /// Load session token from environment variable
    #[allow(dead_code)]
    pub fn load_token(&self) -> Option<String> {
        std::env::var("BW_SESSION").ok()
    }

    /// Save session token to system user environment variable
    pub fn save_token(&self, token: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.save_token_windows(token)
        }
        
        #[cfg(target_os = "macos")]
        {
            self.save_token_macos(token)
        }
        
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            self.save_token_linux(token)
        }
    }

    #[cfg(target_os = "windows")]
    fn save_token_windows(&self, token: &str) -> Result<()> {
        use std::process::Command;
        
        // Use setx to set persistent user environment variable on Windows
        let output = Command::new("setx")
            .arg("BW_SESSION")
            .arg(token)
            .output()
            .map_err(|e| BwError::CommandFailed(format!("Failed to run setx: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BwError::CommandFailed(format!(
                "Failed to set environment variable: {}",
                stderr.trim()
            )));
        }
        
        // Also set in current process so it's available immediately
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    fn save_token_macos(&self, token: &str) -> Result<()> {
        use std::process::Command;
        
        // On macOS, use launchctl to set persistent user environment variable
        let output = Command::new("launchctl")
            .arg("setenv")
            .arg("BW_SESSION")
            .arg(token)
            .output()
            .map_err(|e| BwError::CommandFailed(format!("Failed to run launchctl: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BwError::CommandFailed(format!(
                "Failed to set environment variable: {}",
                stderr.trim()
            )));
        }
        
        // Also set in current process
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }
    
    #[cfg(all(unix, not(target_os = "macos")))]
    fn save_token_linux(&self, token: &str) -> Result<()> {
        use std::process::Command;
        use std::io::Write;
        
        // Try systemd user environment first (most modern Linux systems)
        let systemd_result = Command::new("systemctl")
            .arg("--user")
            .arg("set-environment")
            .arg(format!("BW_SESSION={}", token))
            .output();
        
        if let Ok(output) = systemd_result {
            if output.status.success() {
                // Also set in current process
                std::env::set_var("BW_SESSION", token);
                return Ok(());
            }
        }
        
        // Fallback: Write to ~/.profile which is more standard across shells
        let home = std::env::var("HOME")
            .map_err(|_| BwError::CommandFailed("Could not determine home directory".to_string()))?;
        
        let profile_path = format!("{}/.profile", home);
        
        // Read existing profile
        let mut content = if std::path::Path::new(&profile_path).exists() {
            std::fs::read_to_string(&profile_path)
                .map_err(|e| BwError::IoError(e))?
        } else {
            String::new()
        };
        
        // Check if BW_SESSION is already set
        let bw_session_marker = "# bwtui - Bitwarden session token";
        let has_existing = content.contains(bw_session_marker);
        
        if has_existing {
            // Replace existing BW_SESSION line
            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_next = false;
            
            for line in lines {
                if line.contains(bw_session_marker) {
                    skip_next = true;
                    continue;
                }
                if skip_next && line.trim().starts_with("export BW_SESSION") {
                    skip_next = false;
                    continue;
                }
                new_lines.push(line);
            }
            
            content = new_lines.join("\n");
        }
        
        // Append new BW_SESSION
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("\n{}\nexport BW_SESSION=\"{}\"\n", bw_session_marker, token));
        
        // Write back to profile
        let mut file = std::fs::File::create(&profile_path)
            .map_err(|e| BwError::IoError(e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| BwError::IoError(e))?;
        
        // Also set in current process
        std::env::set_var("BW_SESSION", token);
        
        Ok(())
    }

    /// Clear the session token from the current process environment
    /// Note: This does NOT remove the persistent system environment variable
    #[allow(dead_code)]
    pub fn clear_current_session(&self) {
        std::env::remove_var("BW_SESSION");
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let _manager = SessionManager::new();
        assert!(true);
    }

    #[test]
    fn test_load_token() {
        let manager = SessionManager::new();
        // Just test that it doesn't crash
        let _ = manager.load_token();
    }
}

