use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub login: Option<LoginData>,
    pub notes: Option<String>,
    pub favorite: bool,
    #[serde(default)]
    pub folder_id: Option<String>,
    #[serde(default)]
    pub organization_id: Option<String>,
    pub revision_date: DateTime<Utc>,
    
    // Additional fields from CLI that we don't use but need for parsing
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub object: Option<String>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub creation_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub deleted_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub password_history: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub fields: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub attachments: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub collection_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub reprompt: Option<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Login,
    SecureNote,
    Card,
    Identity,
}

impl From<u8> for ItemType {
    fn from(value: u8) -> Self {
        match value {
            1 => ItemType::Login,
            2 => ItemType::SecureNote,
            3 => ItemType::Card,
            4 => ItemType::Identity,
            _ => ItemType::Login, // Default to Login for unknown types
        }
    }
}

impl serde::Serialize for ItemType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            ItemType::Login => 1u8,
            ItemType::SecureNote => 2u8,
            ItemType::Card => 3u8,
            ItemType::Identity => 4u8,
        };
        serializer.serialize_u8(value)
    }
}

impl<'de> serde::Deserialize<'de> for ItemType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(ItemType::from(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub uris: Option<Vec<Uri>>,
    
    // Additional field from CLI
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub password_revision_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uri {
    pub uri: String,
    #[serde(rename = "match")]
    pub match_type: Option<serde_json::Value>,
}

impl VaultItem {
    /// Get the username for display
    pub fn username(&self) -> Option<&str> {
        self.login.as_ref().and_then(|l| l.username.as_deref())
    }

    /// Get the domain from URIs
    pub fn domain(&self) -> Option<String> {
        self.login
            .as_ref()
            .and_then(|l| l.uris.as_ref())
            .and_then(|uris| uris.first())
            .map(|uri| {
                // Extract domain from URI
                uri.uri
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap_or(&uri.uri)
                    .to_string()
            })
    }
}

