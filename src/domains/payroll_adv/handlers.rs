use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post, put},
};

async fn get_salary(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Option<Salary>>, DbError> {
    Ok(Json(repo::get_employee_salary(&s, &id).await?))
}
async fn update_salary(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSalaryRequest>,
) -> Result<Json<Salary>, DbError> {
    Ok(Json(repo::update_salary(&s, &id, req).await?))
}
async fn generate_payroll(
    State(s): State<AppState>,
    Json(req): Json<GeneratePayrollRequest>,
) -> Result<Json<Vec<PayrollRecord>>, DbError> {
    Ok(Json(repo::generate_payroll(&s, req).await?))
}
async fn get_payroll_month(
    State(s): State<AppState>,
    Path(month): Path<String>,
) -> Result<Json<Vec<PayrollRecord>>, DbError> {
    Ok(Json(repo::get_payroll_by_month(&s, &month).await?))
}
async fn approve_payroll(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PayrollRecord>, DbError> {
    Ok(Json(repo::approve_payroll(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/salary/{employee_id}", get(get_salary).put(update_salary))
        .route("/generate", post(generate_payroll))
        .route("/month/{month}", get(get_payroll_month))
        .route("/{id}/approve", put(approve_payroll))
}
