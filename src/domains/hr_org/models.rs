//! HR Organization Models — Organization structure models
//! Departments and positions

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

// ══════════════════════════════════════════════════════════════════
// Department
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Department {
    pub id: Option<RecordId>,
    pub code: String,
    pub name: String,
    pub parent: Option<RecordId>,
    pub manager: Option<RecordId>,
    pub cost_center: Option<RecordId>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
    // Enriched
    pub manager_name: Option<String>,
    pub employee_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub manager_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub manager_id: Option<String>,
    pub is_active: Option<bool>,
}

// ══════════════════════════════════════════════════════════════════
// Position
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Position {
    pub id: Option<RecordId>,
    pub code: String,
    pub title: String,
    pub department: Option<RecordId>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
    // Enriched
    pub department_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreatePositionRequest {
    pub code: String,
    pub title: String,
    pub department_id: Option<String>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdatePositionRequest {
    pub title: Option<String>,
    pub department_id: Option<String>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
    pub is_active: Option<bool>,
}
