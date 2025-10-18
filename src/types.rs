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
    #[serde(default)]
    pub card: Option<CardData>,
    #[serde(default)]
    pub identity: Option<IdentityData>,
    pub notes: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<CustomField>>,
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
    pub attachments: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub collection_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    pub reprompt: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardData {
    pub brand: Option<String>,
    #[serde(rename = "cardholderName")]
    pub card_holder_name: Option<String>,
    pub number: Option<String>,
    #[serde(rename = "expMonth")]
    pub exp_month: Option<String>,
    #[serde(rename = "expYear")]
    pub exp_year: Option<String>,
    pub code: Option<String>, // CVV
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityData {
    pub title: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "middleName")]
    pub middle_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub address3: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub ssn: Option<String>,
    #[serde(rename = "licenseNumber")]
    pub license_number: Option<String>,
    #[serde(rename = "passportNumber")]
    pub passport_number: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub name: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub field_type: Option<u8>,
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

    /// Get the card number for display (masked)
    pub fn card_number(&self) -> Option<String> {
        self.card.as_ref().and_then(|c| c.number.as_ref()).map(|num| {
            if num.len() >= 4 {
                format!("****-****-****-{}", &num[num.len()-4..])
            } else {
                "****-****-****-****".to_string()
            }
        })
    }

    /// Get the card brand for display
    pub fn card_brand(&self) -> Option<&str> {
        self.card.as_ref().and_then(|c| c.brand.as_deref())
    }

    /// Get the identity email for display
    pub fn identity_email(&self) -> Option<&str> {
        self.identity.as_ref().and_then(|i| i.email.as_deref())
    }

    /// Get the identity full name for display
    pub fn identity_full_name(&self) -> Option<String> {
        self.identity.as_ref().and_then(|i| {
            let mut parts = Vec::new();
            if let Some(first) = &i.first_name {
                parts.push(first.clone());
            }
            if let Some(middle) = &i.middle_name {
                parts.push(middle.clone());
            }
            if let Some(last) = &i.last_name {
                parts.push(last.clone());
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" "))
            }
        })
    }
}

