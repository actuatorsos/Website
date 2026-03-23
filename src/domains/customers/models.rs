//! Client Model
//!
//! نموذج بيانات العملاء

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// Client status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClientStatus {
    /// Active client
    Active,
    /// Inactive client
    Inactive,
    /// Pending approval
    Pending,
}

impl std::fmt::Display for ClientStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientStatus::Active => write!(f, "نشط"),
            ClientStatus::Inactive => write!(f, "غير نشط"),
            ClientStatus::Pending => write!(f, "قيد الانتظار"),
        }
    }
}

/// Client model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Company Name
    pub company_name: String,
    /// Contact Person
    pub contact_person: Option<String>,
    /// Status
    pub status: ClientStatus,
    /// Email
    pub email: Option<String>,
    /// Phone
    pub phone: Option<String>,
    /// Address line
    pub address: Option<String>,
    /// City
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

impl Client {
    /// Get ID as string for template use
    pub fn id_string(&self) -> String {
        self.id
            .as_ref()
            .map(|thing| {
                thing
                    .id
                    .to_string()
                    .replace('"', "")
                    .replace('⟨', "")
                    .replace('⟩', "")
            })
            .unwrap_or_default()
    }

    /// Display email or placeholder
    pub fn email_display(&self) -> &str {
        self.email.as_deref().unwrap_or("-")
    }

    /// Display phone or placeholder
    pub fn phone_display(&self) -> &str {
        self.phone.as_deref().unwrap_or("-")
    }

    /// Display contact person or placeholder
    pub fn contact_display(&self) -> &str {
        self.contact_person.as_deref().unwrap_or("-")
    }

    /// Display full address
    pub fn address_display(&self) -> String {
        match (&self.address, &self.city) {
            (Some(addr), Some(city)) => format!("{}, {}", addr, city),
            (Some(addr), None) => addr.clone(),
            (None, Some(city)) => city.clone(),
            (None, None) => "-".to_string(),
        }
    }

    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            ClientStatus::Active => "bg-green-100 text-green-800",
            ClientStatus::Inactive => "bg-red-100 text-red-800",
            ClientStatus::Pending => "bg-yellow-100 text-yellow-800",
        }
    }

    /// Check if client is active
    pub fn is_active(&self) -> bool {
        self.status == ClientStatus::Active
    }

    /// Check if client is pending
    pub fn is_pending(&self) -> bool {
        self.status == ClientStatus::Pending
    }
}

/// Request to create a new client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateClientRequest {
    /// Company Name
    pub company_name: String,
    /// Contact Person
    pub contact_person: Option<String>,
    /// Status
    pub status: ClientStatus,
    /// Email
    pub email: Option<String>,
    /// Phone
    pub phone: Option<String>,
    /// Address
    pub address: Option<String>,
    /// City
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

impl Default for CreateClientRequest {
    fn default() -> Self {
        Self {
            company_name: String::new(),
            contact_person: None,
            status: ClientStatus::Pending,
            email: None,
            phone: None,
            address: None,
            city: None,
            latitude: None,
            longitude: None,
        }
    }
}

/// Request to update a client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateClientRequest {
    /// Company Name
    pub company_name: Option<String>,
    /// Contact Person
    pub contact_person: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Phone
    pub phone: Option<String>,
    /// Address
    pub address: Option<String>,
    /// City
    pub city: Option<String>,
}
