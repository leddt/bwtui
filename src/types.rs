use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub login: Option<LoginData>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub folder_id: Option<String>,
    pub organization_id: Option<String>,
    pub revision_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    #[serde(rename = "1")]
    Login,
    #[serde(rename = "2")]
    SecureNote,
    #[serde(rename = "3")]
    Card,
    #[serde(rename = "4")]
    Identity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub uris: Option<Vec<Uri>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uri {
    #[serde(rename = "uri")]
    pub value: String,
    #[serde(rename = "match")]
    pub match_type: Option<u8>,
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
                uri.value
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap_or(&uri.value)
                    .to_string()
            })
    }
}

