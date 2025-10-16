use crate::error::{BwError, Result};
use crate::session::SessionManager;
use crate::types::VaultItem;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VaultStatus {
    Locked,
    Unlocked,
    Unauthenticated,
}

#[derive(Debug, Deserialize)]
struct StatusResponse {
    status: String,
}

/// Bitwarden CLI wrapper
#[derive(Clone)]
pub struct BitwardenCli {
    session_token: Option<String>,
}

impl BitwardenCli {
    /// Create a new Bitwarden CLI instance
    pub async fn new() -> Result<Self> {
        // Check if bw CLI is available
        let output = Command::new("bw")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|_| BwError::CliNotFound)?;

        if !output.status.success() {
            return Err(BwError::CliNotFound);
        }

        // Load session token from encrypted storage
        let session_manager = SessionManager::new()?;
        let session_token = session_manager.load_token()?;

        Ok(Self { session_token })
    }

    /// Check the current vault status
    pub async fn check_status(&self) -> Result<VaultStatus> {
        let mut cmd = Command::new("bw");
        cmd.arg("status");

        if let Some(token) = &self.session_token {
            cmd.env("BW_SESSION", token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| BwError::CommandFailed(format!("Failed to execute bw status: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BwError::CommandFailed(format!(
                "bw status failed: {}",
                stderr
            )));
        }

        let status_response: StatusResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| BwError::ParseError(format!("Failed to parse status: {}", e)))?;

        match status_response.status.as_str() {
            "unlocked" => Ok(VaultStatus::Unlocked),
            "locked" => Ok(VaultStatus::Locked),
            "unauthenticated" => Ok(VaultStatus::Unauthenticated),
            _ => Ok(VaultStatus::Locked),
        }
    }

    /// List all vault items
    pub async fn list_items(&self) -> Result<Vec<VaultItem>> {
        let mut cmd = Command::new("bw");
        cmd.arg("list").arg("items");

        if let Some(token) = &self.session_token {
            cmd.env("BW_SESSION", token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| BwError::CommandFailed(format!("Failed to execute bw list: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check for common error messages
            if stderr.contains("not logged in") {
                return Err(BwError::NotLoggedIn);
            } else if stderr.contains("locked") {
                return Err(BwError::VaultLocked);
            }
            
            return Err(BwError::CommandFailed(format!(
                "bw list items failed: {}",
                stderr
            )));
        }

        let items: Vec<VaultItem> = serde_json::from_slice(&output.stdout).map_err(|e| {
            BwError::ParseError(format!("Failed to parse vault items: {}", e))
        })?;

        Ok(items)
    }
    /// Sync vault with server
    pub async fn sync(&self) -> Result<()> {
        let mut cmd = Command::new("bw");
        cmd.arg("sync");

        if let Some(token) = &self.session_token {
            cmd.env("BW_SESSION", token);
        }

        let output = cmd.output().await.map_err(|e| {
            BwError::CommandFailed(format!("Failed to execute bw sync: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BwError::CommandFailed(format!(
                "bw sync failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Check if the CLI is authenticated and unlocked
    #[allow(dead_code)]
    pub async fn is_ready(&self) -> Result<bool> {
        let status = self.check_status().await?;
        Ok(status == VaultStatus::Unlocked)
    }

    /// Unlock vault with password and return session token
    pub async fn unlock(&self, password: &str) -> Result<String> {
        let mut cmd = Command::new("bw");
        cmd.arg("unlock")
            .arg("--raw")
            .arg(password)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| BwError::CommandFailed(format!("Failed to execute bw unlock: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check for common error messages
            if stderr.contains("Invalid master password") {
                return Err(BwError::CommandFailed("Invalid master password".to_string()));
            } else if stderr.contains("not logged in") {
                return Err(BwError::NotLoggedIn);
            }
            
            return Err(BwError::CommandFailed(format!(
                "Failed to unlock vault: {}",
                stderr.trim()
            )));
        }

        let session_token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if session_token.is_empty() {
            return Err(BwError::CommandFailed(
                "Unlock succeeded but no session token was returned".to_string()
            ));
        }

        Ok(session_token)
    }

    /// Get TOTP code for a specific item ID
    pub async fn get_totp(&self, item_id: &str) -> Result<String> {
        let mut cmd = Command::new("bw");
        cmd.arg("get")
            .arg("totp")
            .arg(item_id);

        if let Some(token) = &self.session_token {
            cmd.env("BW_SESSION", token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| BwError::CommandFailed(format!("Failed to execute bw get totp: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // Check for common error messages
            if stderr.contains("not logged in") {
                return Err(BwError::NotLoggedIn);
            } else if stderr.contains("locked") {
                return Err(BwError::VaultLocked);
            }
            
            return Err(BwError::CommandFailed(format!(
                "bw get totp failed: {}",
                stderr
            )));
        }

        let totp_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if totp_code.is_empty() {
            return Err(BwError::CommandFailed(
                "TOTP code is empty".to_string()
            ));
        }

        Ok(totp_code)
    }

    /// Create a new instance with a specific session token
    pub fn with_session_token(token: String) -> Self {
        Self {
            session_token: Some(token),
        }
    }
}

