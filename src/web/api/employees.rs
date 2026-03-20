//! Employees API Endpoints
//!
//! نقاط نهاية API للموظفين

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{delete, get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::hr::models::{CreateEmployeeRequest, Employee, EmployeeRole};
use crate::domains::hr::repository;
use crate::i18n::Language;

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateEmployeeJson {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    #[validate(length(min = 5, max = 50))]
    pub phone: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
    #[validate(length(max = 50))]
    pub national_id: Option<String>,
    #[validate(length(min = 8, max = 50))]
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
// Helpers
// ============================================================================

fn resolve_language(cookies: &Cookies) -> Language {
    if let Some(cookie) = cookies.get("lang") {
        return Language::from_str(cookie.value());
    }
    Language::Arabic
}

fn parse_role(role: &str) -> EmployeeRole {
    match role {
        "technician" => EmployeeRole::Technician,
        "manager" => EmployeeRole::Manager,
        "accountant" => EmployeeRole::Accountant,
        "driver" => EmployeeRole::Driver,
        "admin" => EmployeeRole::Admin,
        _ => EmployeeRole::Technician,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_employee(
    State(state): State<AppState>,
    Json(payload): Json<CreateEmployeeJson>,
) -> axum::response::Result<Json<Employee>, crate::db::DbError> {
    if let Err(e) = payload.validate() {
        return Err(crate::db::DbError::Validation(e.to_string()));
    }

    let request = CreateEmployeeRequest {
        name: payload.name,
        phone: payload.phone,
        email: payload.email,
        role: parse_role(&payload.role),
        national_id: payload.national_id,
        hire_date: payload.hire_date,
        nationality: payload.nationality,
        religion: payload.religion,
        marital_status: payload.marital_status,
        dependents: payload.dependents,
        bank_name: payload.bank_name,
        bank_iban: payload.bank_iban,
        emergency_name: payload.emergency_name,
        emergency_phone: payload.emergency_phone,
        emergency_relation: payload.emergency_relation,
        base_salary: payload.base_salary,
        housing_allowance: payload.housing_allowance,
        transport_allowance: payload.transport_allowance,
        employment_type: payload.employment_type,
    };

    let employee = repository::create_employee(&state, request).await?;
    Ok(Json(employee))
}

async fn list_employees(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<Employee>>, crate::db::DbError> {
    let employees: Vec<Employee> = repository::get_all_employees(&state).await?;
    Ok(Json(employees))
}

async fn delete_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::delete_employee(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_employee).get(list_employees))
        .route("/{id}", delete(delete_employee))
}
