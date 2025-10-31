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
            .map_err(|_| {
                crate::logger::Logger::error("Bitwarden CLI not found. Please install: npm install -g @bitwarden/cli");
                BwError::CliNotFound
            })?;

        if !output.status.success() {
            crate::logger::Logger::error("Bitwarden CLI not found or not executable");
            return Err(BwError::CliNotFound);
        }

        crate::logger::Logger::info("Bitwarden CLI found and verified");

        // Load session token from encrypted storage
        let session_manager = SessionManager::new().map_err(|e| {
            crate::logger::Logger::error(&format!("Failed to initialize session manager: {}", e));
            e
        })?;
        let session_token = session_manager.load_token().map_err(|e| {
            crate::logger::Logger::warn(&format!("Failed to load session token: {}", e));
            e
        })?;

        if session_token.is_some() {
            crate::logger::Logger::info("Session token loaded from storage");
        } else {
            crate::logger::Logger::info("No session token found in storage");
        }

        Ok(Self { session_token })
    }

    /// Check the current vault status
    pub async fn check_status(&self) -> Result<VaultStatus> {
        let mut cmd = Command::new("bw");
        cmd.arg("status");

        if let Some(_token) = &self.session_token {
            // Don't log the token, just indicate we're using one
            cmd.env("BW_SESSION", _token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to execute bw status: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::CommandFailed(error_msg)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let sanitized_stderr = crate::logger::Logger::sanitize_message(&stderr);
            let error_msg = format!("bw status failed: {}", sanitized_stderr);
            crate::logger::Logger::error(&error_msg);
            return Err(BwError::CommandFailed(format!("bw status failed: {}", stderr)));
        }

        let status_response: StatusResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| {
                let error_msg = format!("Failed to parse status: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::ParseError(error_msg)
            })?;

        let status = match status_response.status.as_str() {
            "unlocked" => VaultStatus::Unlocked,
            "locked" => VaultStatus::Locked,
            "unauthenticated" => VaultStatus::Unauthenticated,
            _ => VaultStatus::Locked,
        };

        crate::logger::Logger::info(&format!("Vault status: {:?}", status));
        Ok(status)
    }

    /// List all vault items
    pub async fn list_items(&self) -> Result<Vec<VaultItem>> {
        let mut cmd = Command::new("bw");
        cmd.arg("list").arg("items");

        if let Some(_token) = &self.session_token {
            cmd.env("BW_SESSION", _token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to execute bw list: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::CommandFailed(error_msg)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let sanitized_stderr = crate::logger::Logger::sanitize_message(&stderr);
            
            // Check for common error messages
            if stderr.contains("not logged in") {
                crate::logger::Logger::error("Vault is not logged in");
                return Err(BwError::NotLoggedIn);
            } else if stderr.contains("locked") {
                crate::logger::Logger::error("Vault is locked");
                return Err(BwError::VaultLocked);
            }
            
            let error_msg = format!("bw list items failed: {}", sanitized_stderr);
            crate::logger::Logger::error(&error_msg);
            return Err(BwError::CommandFailed(format!(
                "bw list items failed: {}",
                stderr
            )));
        }

        let items: Vec<VaultItem> = serde_json::from_slice(&output.stdout).map_err(|e| {
            let error_msg = format!("Failed to parse vault items: {}", e);
            crate::logger::Logger::error(&error_msg);
            BwError::ParseError(error_msg)
        })?;

        Ok(items)
    }
    /// Sync vault with server
    pub async fn sync(&self) -> Result<()> {
        let mut cmd = Command::new("bw");
        cmd.arg("sync");

        if let Some(_token) = &self.session_token {
            cmd.env("BW_SESSION", _token);
        }

        let output = cmd.output().await.map_err(|e| {
            let error_msg = format!("Failed to execute bw sync: {}", e);
            crate::logger::Logger::error(&error_msg);
            BwError::CommandFailed(error_msg)
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let sanitized_stderr = crate::logger::Logger::sanitize_message(&stderr);
            let error_msg = format!("bw sync failed: {}", sanitized_stderr);
            crate::logger::Logger::error(&error_msg);
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
            .map_err(|e| {
                let error_msg = format!("Failed to execute bw unlock: {}", e);
                crate::logger::Logger::error(&error_msg);
                BwError::CommandFailed(error_msg)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let sanitized_stderr = crate::logger::Logger::sanitize_message(&stderr);
            
            // Check for common error messages
            if stderr.contains("Invalid master password") {
                crate::logger::Logger::error("Invalid master password provided");
                return Err(BwError::CommandFailed("Invalid master password".to_string()));
            } else if stderr.contains("not logged in") {
                crate::logger::Logger::error("Vault is not logged in");
                return Err(BwError::NotLoggedIn);
            }
            
            let error_msg = format!("Failed to unlock vault: {}", sanitized_stderr);
            crate::logger::Logger::error(&error_msg);
            return Err(BwError::CommandFailed(format!(
                "Failed to unlock vault: {}",
                stderr.trim()
            )));
        }

        let session_token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if session_token.is_empty() {
            let error_msg = "Unlock succeeded but no session token was returned";
            crate::logger::Logger::error(error_msg);
            return Err(BwError::CommandFailed(error_msg.to_string()));
        }

        crate::logger::Logger::info("Vault unlocked successfully (session token received)");
        Ok(session_token)
    }

    /// Get TOTP code for a specific item ID
    pub async fn get_totp(&self, item_id: &str) -> Result<String> {
        let mut cmd = Command::new("bw");
        cmd.arg("get")
            .arg("totp")
            .arg(item_id);

        if let Some(_token) = &self.session_token {
            cmd.env("BW_SESSION", _token);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to execute bw get totp for item {}: {}", item_id, e);
                crate::logger::Logger::error(&error_msg);
                BwError::CommandFailed(format!("Failed to execute bw get totp: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let sanitized_stderr = crate::logger::Logger::sanitize_message(&stderr);
            
            // Check for common error messages
            if stderr.contains("not logged in") {
                crate::logger::Logger::error("Vault is not logged in");
                return Err(BwError::NotLoggedIn);
            } else if stderr.contains("locked") {
                crate::logger::Logger::error("Vault is locked");
                return Err(BwError::VaultLocked);
            }
            
            let error_msg = format!("bw get totp failed for item {}: {}", item_id, sanitized_stderr);
            crate::logger::Logger::error(&error_msg);
            return Err(BwError::CommandFailed(format!(
                "bw get totp failed: {}",
                stderr
            )));
        }

        let totp_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if totp_code.is_empty() {
            let error_msg = "TOTP code is empty";
            crate::logger::Logger::error(error_msg);
            return Err(BwError::CommandFailed(error_msg.to_string()));
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

