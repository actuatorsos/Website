//! Machinery Domain Models
//!
//! نماذج بيانات القسم الصناعي والآلات، وتشمل الآلات والمشاريع وعمليات الإصلاح

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// Machine Models
// ============================================================================

/// Machine status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MachineStatus {
    /// Fully operational
    Working, // تعمل
    /// Broken/Non-functional
    Broken, // معطلة
    /// Under repair
    Repairing, // قيد الإصلاح
    /// Sold to third party
    Sold, // مباعة
}

impl std::fmt::Display for MachineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineStatus::Working => write!(f, "تعمل"),
            MachineStatus::Broken => write!(f, "معطلة"),
            MachineStatus::Repairing => write!(f, "قيد الإصلاح"),
            MachineStatus::Sold => write!(f, "مباعة"),
        }
    }
}

/// Machine model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Owner ID
    pub customer_id: String,
    /// Owner Name (cached)
    pub customer_name: Option<String>,
    /// Serial number
    pub serial_number: String,
    /// Machine model
    pub model: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Purchase date
    pub purchase_date: Option<String>,
    /// Operational status
    pub status: MachineStatus,
}

impl Machine {
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

    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            MachineStatus::Working => "bg-green-100 text-green-800",
            MachineStatus::Broken => "bg-red-100 text-red-800",
            MachineStatus::Repairing => "bg-yellow-100 text-yellow-800",
            MachineStatus::Sold => "bg-gray-100 text-gray-800",
        }
    }
}

/// Request to create a new machine
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateMachineRequest {
    /// Owner ID
    pub customer_id: String,
    /// Serial number
    pub serial_number: String,
    /// Model
    pub model: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Purchase date
    pub purchase_date: Option<String>,
}

// ============================================================================
// Project Models
// ============================================================================

/// Project status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    /// New project
    New, // جديد
    /// Project in progress
    InProgress, // جاري
    /// Project on hold
    OnHold, // متوقف
    /// Project completed
    Completed, // منتهي
    /// Project cancelled
    Cancelled, // ملغى
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::New => write!(f, "جديد"),
            ProjectStatus::InProgress => write!(f, "جاري"),
            ProjectStatus::OnHold => write!(f, "متوقف"),
            ProjectStatus::Completed => write!(f, "منتهي"),
            ProjectStatus::Cancelled => write!(f, "ملغى"),
        }
    }
}

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Customer ID
    pub customer_id: String,
    /// Customer Name (cached)
    pub customer_name: Option<String>,
    /// Project title
    pub title: String,
    /// Project description
    pub description: Option<String>,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: Option<String>,
    /// Current status
    pub status: ProjectStatus,
    /// Project budget
    pub budget: Option<f64>,
}

impl Project {
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

    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            ProjectStatus::New => "bg-blue-100 text-blue-800",
            ProjectStatus::InProgress => "bg-yellow-100 text-yellow-800",
            ProjectStatus::OnHold => "bg-orange-100 text-orange-800",
            ProjectStatus::Completed => "bg-green-100 text-green-800",
            ProjectStatus::Cancelled => "bg-red-100 text-red-800",
        }
    }
}

/// Request to create a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProjectRequest {
    /// Customer ID
    pub customer_id: String,
    /// Project title
    pub title: String,
    /// Project description
    pub description: Option<String>,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: Option<String>,
    /// Project budget
    pub budget: Option<f64>,
}

// ============================================================================
// Repair Models
// ============================================================================

/// Repair status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepairStatus {
    /// New repair request
    New, // جديد
    /// Diagnosing the issue
    Diagnosing, // تشخيص
    /// Repair in progress
    Repairing, // جاري الإصلاح
    /// Waiting for parts
    Waiting, // انتظار قطع
    /// Repair completed
    Completed, // منتهي
    /// Repair cancelled
    Cancelled, // ملغى
}

impl std::fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairStatus::New => write!(f, "جديد"),
            RepairStatus::Diagnosing => write!(f, "تشخيص"),
            RepairStatus::Repairing => write!(f, "جاري الإصلاح"),
            RepairStatus::Waiting => write!(f, "انتظار قطع"),
            RepairStatus::Completed => write!(f, "منتهي"),
            RepairStatus::Cancelled => write!(f, "ملغى"),
        }
    }
}

/// Repair operation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairOperation {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Machine ID
    pub machine_id: String,
    /// Machine Serial (cached)
    pub machine_serial: Option<String>,
    /// Project ID (optional)
    pub project_id: Option<String>,
    /// Project Title (cached)
    pub project_title: Option<String>,
    /// Employee ID
    pub employee_id: String,
    /// Employee Name (cached)
    pub employee_name: Option<String>,
    /// Customer Name (cached)
    pub customer_name: Option<String>,
    /// Issue description
    pub description: String,
    /// Technical diagnosis
    pub diagnosis: Option<String>,
    /// Parts used for repair
    pub parts_used: Option<String>,
    /// Repair cost
    pub cost: Option<f64>,
    /// Start time
    pub start_time: String,
    /// End time
    pub end_time: Option<String>,
    /// Current status
    pub status: RepairStatus,
}

impl RepairOperation {
    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            RepairStatus::New => "bg-blue-100 text-blue-800",
            RepairStatus::Diagnosing => "bg-purple-100 text-purple-800",
            RepairStatus::Repairing => "bg-yellow-100 text-yellow-800",
            RepairStatus::Waiting => "bg-orange-100 text-orange-800",
            RepairStatus::Completed => "bg-green-100 text-green-800",
            RepairStatus::Cancelled => "bg-red-100 text-red-800",
        }
    }
}

/// Request to create a new repair operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRepairRequest {
    /// Machine ID
    pub machine_id: String,
    /// Project ID (optional)
    pub project_id: Option<String>,
    /// Employee ID
    pub employee_id: String,
    /// Description of the issue
    pub description: String,
    /// Initial diagnosis
    pub diagnosis: Option<String>,
}

/// Request to update repair operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateRepairRequest {
    /// Updated diagnosis
    pub diagnosis: Option<String>,
    /// Parts used
    pub parts_used: Option<String>,
    /// Total cost
    pub cost: Option<f64>,
    /// New status
    pub status: RepairStatus,
}
