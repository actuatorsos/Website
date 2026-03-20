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

async fn list_all_warnings(
    State(s): State<AppState>,
) -> Result<Json<Vec<Warning>>, DbError> {
    Ok(Json(repo::get_all_warnings(&s).await?))
}
async fn create_warning(
    State(s): State<AppState>,
    Json(req): Json<CreateWarningRequest>,
) -> Result<Json<Warning>, DbError> {
    Ok(Json(repo::create_warning(&s, req).await?))
}
async fn list_employee_warnings(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Warning>>, DbError> {
    Ok(Json(repo::get_employee_warnings(&s, &id).await?))
}
async fn calculate_eos(
    State(s): State<AppState>,
    Json(req): Json<CalculateEosRequest>,
) -> Result<Json<EndOfService>, DbError> {
    Ok(Json(repo::calculate_eos(&s, req).await?))
}
async fn list_all_overtime(
    State(s): State<AppState>,
) -> Result<Json<Vec<OvertimeRequest>>, DbError> {
    Ok(Json(repo::get_all_overtime(&s).await?))
}
async fn create_overtime(
    State(s): State<AppState>,
    Json(req): Json<CreateOvertimeRequest>,
) -> Result<Json<OvertimeRequest>, DbError> {
    Ok(Json(repo::create_overtime_request(&s, req).await?))
}
async fn list_employee_overtime(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<OvertimeRequest>>, DbError> {
    Ok(Json(repo::get_employee_overtime(&s, &id).await?))
}
async fn approve_overtime(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<OvertimeRequest>, DbError> {
    Ok(Json(repo::approve_overtime(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/warnings", get(list_all_warnings).post(create_warning))
        .route("/warnings/{employee_id}", get(list_employee_warnings))
        .route("/eos/calculate", post(calculate_eos))
        .route("/overtime", get(list_all_overtime).post(create_overtime))
        .route("/overtime/{employee_id}", get(list_employee_overtime))
        .route("/overtime/{id}/approve", put(approve_overtime))
}
