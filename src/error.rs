use thiserror::Error;

#[derive(Error, Debug)]
pub enum BwError {
    #[error("Clipboard error: {0}")]
    ClipboardError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BwError>;

