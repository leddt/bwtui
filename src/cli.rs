use crate::error::{BwError, Result};
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

        // Check for session token in environment
        let session_token = std::env::var("BW_SESSION").ok();

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
}

