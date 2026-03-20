//! HR Organization Models — Organization structure models
//! Departments and positions

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ══════════════════════════════════════════════════════════════════
// Department
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: Option<Thing>,
    pub code: String,
    pub name: String,
    pub parent: Option<Thing>,
    pub manager: Option<Thing>,
    pub cost_center: Option<Thing>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub manager_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub manager_id: Option<String>,
    pub is_active: Option<bool>,
}

// ══════════════════════════════════════════════════════════════════
// Position
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Option<Thing>,
    pub code: String,
    pub title: String,
    pub department: Option<Thing>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePositionRequest {
    pub code: String,
    pub title: String,
    pub department_id: Option<String>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePositionRequest {
    pub title: Option<String>,
    pub department_id: Option<String>,
    pub grade: Option<String>,
    pub min_salary: Option<f64>,
    pub max_salary: Option<f64>,
    pub is_active: Option<bool>,
}
