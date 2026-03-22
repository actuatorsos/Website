//! HR Domain Models
//!
//! نماذج بيانات الموارد البشرية (موظفين، متدربين، دوام)

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// Employee Models
// ============================================================================

// Employee role and status are now plain strings to support
// any custom roles/statuses across different organizations.
// Common roles: technician, manager, accountant, driver, admin, engineer,
// volunteer, intern, designer, developer, sales, support, hr, coordinator, other
// Common statuses: active, on_leave, resigned, suspended, terminated

/// Employee model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Full name
    pub name: String,
    /// Phone number
    pub phone: String,
    /// Email address
    pub email: Option<String>,
    /// Job role (string — supports custom roles per org)
    pub role: String,
    /// National ID number
    pub national_id: Option<String>,
    /// Hire date
    pub hire_date: String,
    /// Employment status (string — active, on_leave, resigned, suspended, terminated)
    #[serde(default = "default_status")]
    pub status: String,

    // --- Extended HR fields ---
    pub nationality: Option<String>,
    pub religion: Option<String>,
    pub marital_status: Option<String>,
    pub dependents: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_iban: Option<String>,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub emergency_relation: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::domains::hr::utils::deserialize_optional_f64"
    )]
    pub base_salary: Option<f64>,
    #[serde(
        default,
        deserialize_with = "crate::domains::hr::utils::deserialize_optional_f64"
    )]
    pub housing_allowance: Option<f64>,
    #[serde(
        default,
        deserialize_with = "crate::domains::hr::utils::deserialize_optional_f64"
    )]
    pub transport_allowance: Option<f64>,
    pub employment_type: Option<String>,
}

fn default_status() -> String { "active".to_string() }

impl Employee {
    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status.as_str() {
            "active" => "bg-green-100 text-green-800",
            "on_leave" => "bg-yellow-100 text-yellow-800",
            "resigned" | "terminated" => "bg-red-100 text-red-800",
            "suspended" => "bg-orange-100 text-orange-800",
            _ => "bg-gray-100 text-gray-800",
        }
    }
}

/// Request to create a new employee
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateEmployeeRequest {
    /// Full name
    pub name: String,
    /// Phone number
    pub phone: String,
    /// Email
    pub email: Option<String>,
    /// Role
    pub role: String,
    /// National ID
    pub national_id: Option<String>,
    /// Hire date
    pub hire_date: String,

    // --- Extended HR fields ---
    pub nationality: Option<String>,
    pub religion: Option<String>,
    pub marital_status: Option<String>,
    pub dependents: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_iban: Option<String>,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub emergency_relation: Option<String>,
    pub base_salary: Option<f64>,
    pub housing_allowance: Option<f64>,
    pub transport_allowance: Option<f64>,
    pub employment_type: Option<String>,
}

// ============================================================================
// Trainee Models
// ============================================================================

/// Trainee status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TraineeStatus {
    /// Currently training
    Active, // نشط
    /// Finished training
    Completed, // منتهي
    /// Cancelled training
    Cancelled, // ملغى
}

impl std::fmt::Display for TraineeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraineeStatus::Active => write!(f, "نشط"),
            TraineeStatus::Completed => write!(f, "منتهي"),
            TraineeStatus::Cancelled => write!(f, "ملغى"),
        }
    }
}

/// Trainee model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trainee {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Creation date
    pub created_at: Option<String>,
    /// Full name
    pub name: String,
    /// Phone number
    pub phone: String,
    /// Email address
    pub email: Option<String>,
    /// Contributing institution
    pub institution: String,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: String,
    /// Training status
    pub status: TraineeStatus,
}

impl Trainee {
    /// Get CSS class for status badge
    pub fn status_class(&self) -> &str {
        match self.status {
            TraineeStatus::Active => "bg-green-100 text-green-800",
            TraineeStatus::Completed => "bg-blue-100 text-blue-800",
            TraineeStatus::Cancelled => "bg-red-100 text-red-800",
        }
    }
}

/// Request to create a new trainee
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTraineeRequest {
    /// Full name
    pub name: String,
    /// Phone number
    pub phone: String,
    /// Email address
    pub email: Option<String>,
    /// Institution
    pub institution: String,
    /// Start date
    pub start_date: String,
    /// End date
    pub end_date: String,
}

// ============================================================================
// Attendance Models
// ============================================================================

/// Person type for attendance (employee or trainee)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PersonType {
    /// Regular employee
    Employee,
    /// Trainee
    Trainee,
}

impl std::fmt::Display for PersonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonType::Employee => write!(f, "موظف"),
            PersonType::Trainee => write!(f, "متدرب"),
        }
    }
}

/// Attendance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendance {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Person ID
    pub person_id: String,
    /// Person Type
    pub person_type: PersonType,
    /// Person Name (cached)
    pub person_name: Option<String>,
    /// Check-in timestamp
    pub check_in: String,
    /// Check-out timestamp
    pub check_out: Option<String>,
    /// Date (YYYY-MM-DD format for filtering)
    pub date: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
}

impl Attendance {
    /// Get ID as string for template use
    pub fn id_string(&self) -> String {
        self.id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default()
    }

    /// Check if person has checked out
    pub fn is_checked_out(&self) -> bool {
        self.check_out.is_some()
    }

    /// Get display name
    pub fn name_display(&self) -> &str {
        self.person_name.as_deref().unwrap_or("-")
    }

    /// Get check-out display
    pub fn checkout_display(&self) -> &str {
        self.check_out.as_deref().unwrap_or("-")
    }

    /// Get notes display
    pub fn notes_display(&self) -> &str {
        self.notes.as_deref().unwrap_or("-")
    }

    /// Calculate work duration as formatted string
    pub fn duration_display(&self) -> String {
        match (&self.check_in, &self.check_out) {
            (cin, Some(cout)) => {
                if let (Ok(start), Ok(end)) = (
                    chrono::DateTime::parse_from_rfc3339(cin),
                    chrono::DateTime::parse_from_rfc3339(cout),
                ) {
                    let duration = end - start;
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;
                    format!("{}:{:02}", hours, minutes)
                } else {
                    "-".to_string()
                }
            }
            _ => "-".to_string(),
        }
    }

    /// Format check-in time to display only time (HH:MM)
    pub fn checkin_time(&self) -> String {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&self.check_in) {
            dt.format("%H:%M").to_string()
        } else {
            self.check_in.clone()
        }
    }

    /// Format check-out time to display only time (HH:MM)
    pub fn checkout_time(&self) -> String {
        match &self.check_out {
            Some(co) => {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(co) {
                    dt.format("%H:%M").to_string()
                } else {
                    co.clone()
                }
            }
            None => "-".to_string(),
        }
    }

    /// Get person type CSS badge class
    pub fn type_class(&self) -> &str {
        match self.person_type {
            PersonType::Employee => "bg-blue-100 text-blue-800",
            PersonType::Trainee => "bg-purple-100 text-purple-800",
        }
    }
}

/// Request to check in
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckInRequest {
    /// Person ID
    pub person_id: String,
    /// Person Type
    pub person_type: PersonType,
    /// Person Name
    pub person_name: Option<String>,
    /// Notes
    pub notes: Option<String>,
}

/// Request to check out
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckOutRequest {
    /// Attendance Record ID
    pub attendance_id: String,
    /// Notes
    pub notes: Option<String>,
}
