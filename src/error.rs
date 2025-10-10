use thiserror::Error;

#[derive(Error, Debug)]
pub enum BwError {
    #[error("Bitwarden CLI not found. Please install 'bw' CLI")]
    CliNotFound,

    #[error("Vault is locked. Please unlock with 'bw unlock'")]
    VaultLocked,

    #[error("Not logged in. Please run 'bw login'")]
    NotLoggedIn,

    #[error("Session expired. Please unlock vault again")]
    #[allow(dead_code)]
    SessionExpired,

    #[error("Failed to execute bw command: {0}")]
    CommandFailed(String),

    #[error("Failed to parse CLI output: {0}")]
    ParseError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BwError>;

