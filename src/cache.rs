use crate::error::{BwError, Result};
use crate::types::VaultItem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Cache data structure - stores only non-sensitive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVaultData {
    /// Timestamp when the cache was created
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// Cached items (without passwords, TOTP secrets, and notes)
    pub items: Vec<CachedVaultItem>,
}

/// Cached vault item without sensitive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedVaultItem {
    pub id: String,
    pub name: String,
    pub item_type: crate::types::ItemType,
    pub favorite: bool,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    pub revision_date: chrono::DateTime<chrono::Utc>,
    /// Login data without password and TOTP secret
    pub login: Option<CachedLoginData>,
    /// Card data without sensitive fields
    pub card: Option<CachedCardData>,
    /// Identity data (not sensitive, all can be cached)
    pub identity: Option<CachedIdentityData>,
}

/// Simplified URI for caching (without match_type which contains serde_json::Value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUri {
    pub uri: String,
}

/// Login data without sensitive fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLoginData {
    pub username: Option<String>,
    pub uris: Option<Vec<CachedUri>>,
    /// Indicates that a password exists (but don't store the password itself)
    pub has_password: bool,
    /// Indicates that a TOTP secret exists (but don't store the secret itself)
    pub has_totp: bool,
}

/// Card data without sensitive fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCardData {
    pub brand: Option<String>,
    pub card_holder_name: Option<String>,
    /// Indicates that a card number exists (but don't store the number itself)
    pub has_number: bool,
    /// Indicates that a CVV exists (but don't store the CVV itself)
    pub has_cvv: bool,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
}

/// Identity data (not sensitive, all can be cached)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedIdentityData {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub address3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub ssn: Option<String>,
    pub license_number: Option<String>,
    pub passport_number: Option<String>,
    pub username: Option<String>,
}

impl CachedVaultData {
    /// Create cache data from vault items
    pub fn from_vault_items(items: &[VaultItem]) -> Self {
        let cached_items: Vec<CachedVaultItem> = items
            .iter()
            .map(|item| CachedVaultItem {
                id: item.id.clone(),
                name: item.name.clone(),
                item_type: item.item_type.clone(),
                favorite: item.favorite,
                folder_id: item.folder_id.clone(),
                organization_id: item.organization_id.clone(),
                revision_date: item.revision_date,
                login: item.login.as_ref().map(|login| CachedLoginData {
                    username: login.username.clone(),
                    uris: login.uris.as_ref().map(|uris| {
                        uris.iter().map(|uri| CachedUri {
                            uri: uri.uri.clone(),
                        }).collect()
                    }),
                    has_password: login.password.is_some(),
                    has_totp: login.totp.is_some(),
                }),
                card: item.card.as_ref().map(|card| CachedCardData {
                    brand: card.brand.clone(),
                    card_holder_name: card.card_holder_name.clone(),
                    has_number: card.number.is_some(),
                    has_cvv: card.code.is_some(),
                    exp_month: card.exp_month.clone(),
                    exp_year: card.exp_year.clone(),
                }),
                identity: item.identity.as_ref().map(|identity| CachedIdentityData {
                    title: identity.title.clone(),
                    first_name: identity.first_name.clone(),
                    middle_name: identity.middle_name.clone(),
                    last_name: identity.last_name.clone(),
                    address1: identity.address1.clone(),
                    address2: identity.address2.clone(),
                    address3: identity.address3.clone(),
                    city: identity.city.clone(),
                    state: identity.state.clone(),
                    postal_code: identity.postal_code.clone(),
                    country: identity.country.clone(),
                    phone: identity.phone.clone(),
                    email: identity.email.clone(),
                    ssn: identity.ssn.clone(),
                    license_number: identity.license_number.clone(),
                    passport_number: identity.passport_number.clone(),
                    username: identity.username.clone(),
                }),
            })
            .collect();

        Self {
            cached_at: chrono::Utc::now(),
            items: cached_items,
        }
    }

    /// Convert cached items to VaultItems (with placeholders for secrets)
    pub fn to_vault_items(&self) -> Vec<VaultItem> {
        self.items
            .iter()
            .map(|cached| VaultItem {
                id: cached.id.clone(),
                name: cached.name.clone(),
                item_type: cached.item_type.clone(),
                favorite: cached.favorite,
                folder_id: cached.folder_id.clone(),
                organization_id: cached.organization_id.clone(),
                revision_date: cached.revision_date,
                login: cached.login.as_ref().map(|login| crate::types::LoginData {
                    username: login.username.clone(),
                    password: None, // Don't store passwords in cache
                    totp: None,     // Don't store TOTP secrets in cache
                    uris: login.uris.as_ref().map(|uris| {
                        uris.iter().map(|cached_uri| crate::types::Uri {
                            uri: cached_uri.uri.clone(),
                            match_type: None, // Don't store match_type in cache
                        }).collect()
                    }),
                    password_revision_date: None,
                }),
                card: cached.card.as_ref().map(|card| crate::types::CardData {
                    brand: card.brand.clone(),
                    card_holder_name: card.card_holder_name.clone(),
                    number: None, // Don't store card number in cache
                    exp_month: card.exp_month.clone(),
                    exp_year: card.exp_year.clone(),
                    code: None, // Don't store CVV in cache
                }),
                identity: cached.identity.as_ref().map(|identity| crate::types::IdentityData {
                    title: identity.title.clone(),
                    first_name: identity.first_name.clone(),
                    middle_name: identity.middle_name.clone(),
                    last_name: identity.last_name.clone(),
                    address1: identity.address1.clone(),
                    address2: identity.address2.clone(),
                    address3: identity.address3.clone(),
                    city: identity.city.clone(),
                    state: identity.state.clone(),
                    postal_code: identity.postal_code.clone(),
                    country: identity.country.clone(),
                    phone: identity.phone.clone(),
                    email: identity.email.clone(),
                    ssn: identity.ssn.clone(),
                    license_number: identity.license_number.clone(),
                    passport_number: identity.passport_number.clone(),
                    username: identity.username.clone(),
                }),
                notes: None, // Don't store notes in cache
                fields: None, // Don't store custom fields in cache (treat as sensitive)
                object: None,
                creation_date: None,
                deleted_date: None,
                password_history: None,
                attachments: None,
                collection_ids: None,
                reprompt: None,
            })
            .collect()
    }
}

/// Get the cache file path
fn get_cache_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| BwError::CommandFailed("Could not determine home directory".to_string()))?;

    let cache_dir = home_dir.join(".bwtui");
    
    // Create directory if it doesn't exist
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| {
            BwError::CommandFailed(format!("Failed to create cache directory: {}", e))
        })?;
    }

    Ok(cache_dir.join("vault_cache.bin"))
}

/// Load cache from disk
pub fn load_cache() -> Result<Option<CachedVaultData>> {
    let cache_path = get_cache_path()?;

    if !cache_path.exists() {
        return Ok(None);
    }

    let data = fs::read(&cache_path).map_err(|e| {
        BwError::CommandFailed(format!("Failed to read cache file: {}", e))
    })?;

    match bincode::deserialize::<CachedVaultData>(&data) {
        Ok(cached_data) => {
            Ok(Some(cached_data))
        }
        Err(_e) => {
            // If deserialization fails, delete the corrupted cache and return None
            // This handles format changes or corrupted files gracefully
            let _ = fs::remove_file(&cache_path);
            Ok(None)
        }
    }
}

/// Save cache to disk
pub fn save_cache(data: &CachedVaultData) -> Result<()> {
    let cache_path = get_cache_path()?;

    let encoded = bincode::serialize(data).map_err(|e| {
        BwError::CommandFailed(format!("Failed to serialize cache: {}", e))
    })?;

    fs::write(&cache_path, encoded).map_err(|e| {
        BwError::CommandFailed(format!("Failed to write cache file: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_data_creation() {
        let items = vec![];
        let cache = CachedVaultData::from_vault_items(&items);
        assert!(cache.items.is_empty());
    }
}

