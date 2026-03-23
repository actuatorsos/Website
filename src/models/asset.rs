//! Asset Model
//!
//! نموذج بيانات أصول الشركة

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

/// Asset category enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetCategory {
    /// Tools and hand equipment
    Tools, // عدة
    /// Heavy equipment
    Equipment, // معدات
    /// Cars and trucks
    Vehicles, // مركبات
    /// Office furniture
    Furniture, // أثاث
    /// Electronic devices
    Electronics, // إلكترونيات
}

impl std::fmt::Display for AssetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetCategory::Tools => write!(f, "عدة"),
            AssetCategory::Equipment => write!(f, "معدات"),
            AssetCategory::Vehicles => write!(f, "مركبات"),
            AssetCategory::Furniture => write!(f, "أثاث"),
            AssetCategory::Electronics => write!(f, "إلكترونيات"),
        }
    }
}

/// Asset status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetStatus {
    /// Available for use
    Available, // متاح
    /// Currently assigned
    InUse, // مستخدم
    /// Under maintenance
    Maintenance, // صيانة
    /// Disposed or sold
    Disposed, // مستهلك
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetStatus::Available => write!(f, "متاح"),
            AssetStatus::InUse => write!(f, "مستخدم"),
            AssetStatus::Maintenance => write!(f, "صيانة"),
            AssetStatus::Disposed => write!(f, "مستهلك"),
        }
    }
}

/// Asset model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Asset name
    pub name: String,
    /// Category
    pub category: AssetCategory,
    /// Serial number
    pub serial_number: Option<String>,
    /// Purchase date
    pub purchase_date: Option<String>,
    /// Monetary value
    pub value: Option<f64>,
    /// Current status
    pub status: AssetStatus,
    /// Physical location
    pub location: Option<String>,
    /// Assigned Employee ID
    pub assigned_to: Option<String>,
    /// Assigned Employee Name (cached)
    pub assigned_employee_name: Option<String>,
}

impl Asset {
    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            AssetStatus::Available => "bg-green-100 text-green-800",
            AssetStatus::InUse => "bg-blue-100 text-blue-800",
            AssetStatus::Maintenance => "bg-yellow-100 text-yellow-800",
            AssetStatus::Disposed => "bg-gray-100 text-gray-800",
        }
    }
}

/// Request to create a new asset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateAssetRequest {
    /// Asset name
    pub name: String,
    /// Category
    pub category: AssetCategory,
    /// Serial number
    pub serial_number: Option<String>,
    /// Purchase date
    pub purchase_date: Option<String>,
    /// Value
    pub value: Option<f64>,
    /// Initial location
    pub location: Option<String>,
}

/// Request to assign asset to employee
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignAssetRequest {
    /// Employee ID to assign to
    pub employee_id: String,
    /// New location
    pub location: String,
}
