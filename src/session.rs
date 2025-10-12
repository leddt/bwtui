use crate::error::{BwError, Result};
use std::fs;
use std::path::PathBuf;

/// Session token manager with platform-specific encryption
pub struct SessionManager {
    /// Path to the encrypted session file
    session_file: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let session_file = Self::get_session_file_path()?;
        Ok(Self { session_file })
    }

    /// Get the path to the session file
    fn get_session_file_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| BwError::CommandFailed("Could not determine home directory".to_string()))?;

        let config_dir = home_dir.join(".bwtui");
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                BwError::CommandFailed(format!("Failed to create config directory: {}", e))
            })?;
        }

        Ok(config_dir.join("session.enc"))
    }

    /// Load session token from encrypted file
    pub fn load_token(&self) -> Result<Option<String>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let encrypted_data = fs::read(&self.session_file).map_err(|e| {
            BwError::CommandFailed(format!("Failed to read session file: {}", e))
        })?;

        if encrypted_data.is_empty() {
            return Ok(None);
        }

        let token = Self::decrypt_data(&encrypted_data)?;
        Ok(Some(token))
    }

    /// Save session token to encrypted file
    pub fn save_token(&self, token: &str) -> Result<()> {
        let encrypted_data = Self::encrypt_data(token)?;
        
        fs::write(&self.session_file, encrypted_data).map_err(|e| {
            BwError::CommandFailed(format!("Failed to write session file: {}", e))
        })?;

        Ok(())
    }

    /// Clear the session token
    #[allow(dead_code)]
    pub fn clear_token(&self) -> Result<()> {
        if self.session_file.exists() {
            fs::remove_file(&self.session_file).map_err(|e| {
                BwError::CommandFailed(format!("Failed to remove session file: {}", e))
            })?;
        }
        Ok(())
    }

    /// Encrypt data using Windows DPAPI
    #[cfg(target_os = "windows")]
    fn encrypt_data(data: &str) -> Result<Vec<u8>> {
        use winapi::um::dpapi::CryptProtectData;
        use winapi::um::wincrypt::CRYPTOAPI_BLOB;
        use std::ptr;

        let data_bytes = data.as_bytes();
        
        let mut data_in = CRYPTOAPI_BLOB {
            cbData: data_bytes.len() as u32,
            pbData: data_bytes.as_ptr() as *mut u8,
        };

        let mut data_out = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        unsafe {
            let result = CryptProtectData(
                &mut data_in,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut data_out,
            );

            if result == 0 {
                return Err(BwError::CommandFailed(
                    "Failed to encrypt data with DPAPI".to_string()
                ));
            }

            // Copy the encrypted data
            let encrypted = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();

            // Free the memory allocated by CryptProtectData
            winapi::um::winbase::LocalFree(data_out.pbData as *mut _);

            Ok(encrypted)
        }
    }

    /// Decrypt data using Windows DPAPI
    #[cfg(target_os = "windows")]
    fn decrypt_data(encrypted_data: &[u8]) -> Result<String> {
        use winapi::um::dpapi::CryptUnprotectData;
        use winapi::um::wincrypt::CRYPTOAPI_BLOB;
        use std::ptr;

        let mut data_in = CRYPTOAPI_BLOB {
            cbData: encrypted_data.len() as u32,
            pbData: encrypted_data.as_ptr() as *mut u8,
        };

        let mut data_out = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        unsafe {
            let result = CryptUnprotectData(
                &mut data_in,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut data_out,
            );

            if result == 0 {
                return Err(BwError::CommandFailed(
                    "Failed to decrypt data with DPAPI".to_string()
                ));
            }

            // Copy the decrypted data
            let decrypted = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();

            // Free the memory allocated by CryptUnprotectData
            winapi::um::winbase::LocalFree(data_out.pbData as *mut _);

            String::from_utf8(decrypted).map_err(|e| {
                BwError::CommandFailed(format!("Failed to decode decrypted data: {}", e))
            })
        }
    }

    /// Encrypt data using keyring (macOS/Linux)
    #[cfg(not(target_os = "windows"))]
    fn encrypt_data(data: &str) -> Result<Vec<u8>> {
        use keyring::Entry;
        
        let username = whoami::username();
        let entry = Entry::new("bwtui-bitwarden", &username)
            .map_err(|e| BwError::CommandFailed(format!("Failed to create keyring entry: {}", e)))?;
        
        entry.set_password(data)
            .map_err(|e| BwError::CommandFailed(format!("Failed to save to keyring: {}", e)))?;
        
        // Return a marker indicating data is in keyring
        Ok(b"KEYRING".to_vec())
    }

    /// Decrypt data using keyring (macOS/Linux)
    #[cfg(not(target_os = "windows"))]
    fn decrypt_data(encrypted_data: &[u8]) -> Result<String> {
        use keyring::Entry;
        
        if encrypted_data == b"KEYRING" {
            let username = whoami::username();
            let entry = Entry::new("bwtui-bitwarden", &username)
                .map_err(|e| BwError::CommandFailed(format!("Failed to create keyring entry: {}", e)))?;
            
            entry.get_password()
                .map_err(|e| BwError::CommandFailed(format!("Failed to load from keyring: {}", e)))
        } else {
            Err(BwError::CommandFailed("Invalid session file format".to_string()))
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SessionManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_load_token_nonexistent() {
        let manager = SessionManager::new().unwrap();
        // Clean up any existing token first
        let _ = manager.clear_token();
        let token = manager.load_token();
        assert!(token.is_ok());
        if let Ok(result) = token {
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_save_and_load_token() {
        let manager = SessionManager::new().unwrap();
        
        // Save a test token
        let test_token = "test_session_token_12345";
        match manager.save_token(test_token) {
            Ok(_) => {
                // Load it back
                let loaded = manager.load_token().unwrap();
                assert!(loaded.is_some());
                assert_eq!(loaded.unwrap(), test_token);
                
                // Clean up
                let _ = manager.clear_token();
            }
            Err(e) => {
                // This might fail in CI/test environments
                eprintln!("Note: Encryption test skipped: {}", e);
            }
        }
    }
}
