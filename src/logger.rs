use crate::error::{BwError, Result};
use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Static logger instance path
static LOG_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

/// Logger wrapper that handles file logging with sanitization
pub struct Logger;

impl Logger {
    /// Initialize the logger
    /// Creates a timestamped log file and cleans up old logs
    pub fn init() -> Result<()> {
        let log_dir = Self::get_log_directory()?;
        
        // Clean up old log files
        Self::cleanup_old_logs(&log_dir)?;
        
        // Generate timestamped log filename
        let log_filename = Self::generate_log_filename();
        let log_path = log_dir.join(&log_filename);
        
        // Create log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| BwError::CommandFailed(format!("Failed to create log file: {}", e)))?;
        
        // Set file permissions to user-readable only (600 on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()
                .map_err(|e| BwError::CommandFailed(format!("Failed to get log file metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&log_path, perms)
                .map_err(|e| BwError::CommandFailed(format!("Failed to set log file permissions: {}", e)))?;
        }
        
        // Create custom config
        let mut config_builder = ConfigBuilder::default();
        config_builder.set_time_format_rfc3339();
        let _ = config_builder.set_time_offset_to_local(); // Ignore error, use default if it fails
        let config = config_builder.build();
        
        // Initialize simplelog
        WriteLogger::init(
            LevelFilter::Info, // Log ERROR, WARN, and INFO
            config,
            file,
        )
        .map_err(|e| BwError::CommandFailed(format!("Failed to initialize logger: {}", e)))?;
        
        // Store log path
        let log_path_mutex = LOG_PATH.get_or_init(|| Mutex::new(None));
        *log_path_mutex.lock().unwrap() = Some(log_path);
        
        Ok(())
    }
    
    /// Get the log directory path (.bwtui)
    fn get_log_directory() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| BwError::CommandFailed("Could not determine home directory".to_string()))?;
        
        let log_dir = home_dir.join(".bwtui");
        
        // Create directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir).map_err(|e| {
                BwError::CommandFailed(format!("Failed to create log directory: {}", e))
            })?;
        }
        
        Ok(log_dir)
    }
    
    /// Generate timestamped log filename
    fn generate_log_filename() -> String {
        let now = chrono::Utc::now();
        format!("bwtui-{}.log", now.format("%Y-%m-%d-%H-%M-%S"))
    }
    
    /// Clean up old log files, keeping only the 5 most recent
    fn cleanup_old_logs(log_dir: &Path) -> Result<()> {
        // Find all log files matching the pattern
        let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        
        if let Ok(entries) = fs::read_dir(log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("bwtui-") && filename.ends_with(".log") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                log_files.push((path, modified));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Keep only the 5 most recent, delete the rest
        if log_files.len() > 5 {
            for (path, _) in log_files.iter().skip(5) {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Warning: Failed to delete old log file {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Sanitize sensitive data from log messages
    pub fn sanitize_message(message: &str) -> String {
        let mut sanitized = message.to_string();
        
        // Remove session tokens (look for BW_SESSION=... or token-like patterns)
        sanitized = regex::Regex::new(r"BW_SESSION=[^\s]+")
            .unwrap()
            .replace_all(&sanitized, "BW_SESSION=[REDACTED]")
            .to_string();
        
        // Remove token-like strings (long alphanumeric strings)
        sanitized = regex::Regex::new(r"\b[a-zA-Z0-9]{32,}\b")
            .unwrap()
            .replace_all(&sanitized, "[REDACTED]")
            .to_string();
        
        // Remove passwords (look for password: or password = patterns)
        sanitized = regex::Regex::new(r"(?i)password\s*[:=]\s*[^\s]+")
            .unwrap()
            .replace_all(&sanitized, "password=[REDACTED]")
            .to_string();
        
        // Remove TOTP codes (6-digit codes)
        sanitized = regex::Regex::new(r"\b\d{6}\b")
            .unwrap()
            .replace_all(&sanitized, "[REDACTED]")
            .to_string();
        
        // Remove credit card numbers (13-19 digits with optional spaces/dashes)
        sanitized = regex::Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4,7}\b")
            .unwrap()
            .replace_all(&sanitized, "[REDACTED]")
            .to_string();
        
        // Remove CVV codes (3-4 digits)
        sanitized = regex::Regex::new(r"\b(cvv|cvc)\s*[:=]\s*\d{3,4}\b")
            .unwrap()
            .replace_all(&sanitized, "[REDACTED]")
            .to_string();
        
        sanitized
    }
    
    /// Log an error message (sanitized)
    pub fn error(message: &str) {
        let sanitized = Self::sanitize_message(message);
        log::error!("{}", sanitized);
    }
    
    /// Log a warning message (sanitized)
    pub fn warn(message: &str) {
        let sanitized = Self::sanitize_message(message);
        log::warn!("{}", sanitized);
    }
    
    /// Log an info message (sanitized)
    pub fn info(message: &str) {
        let sanitized = Self::sanitize_message(message);
        log::info!("{}", sanitized);
    }
}

