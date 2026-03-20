use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put},
};
use std::collections::HashMap;

async fn list_leaves(
    State(s): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<LeaveRequest>>, DbError> {
    let status = params.get("status").map(|s| s.as_str());
    Ok(Json(repo::get_all_leave_requests(&s, status).await?))
}
async fn create_leave(
    State(s): State<AppState>,
    Json(req): Json<CreateLeaveRequest>,
) -> Result<Json<LeaveRequest>, DbError> {
    Ok(Json(repo::create_leave_request(&s, req).await?))
}
async fn employee_leaves(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<LeaveRequest>>, DbError> {
    Ok(Json(repo::get_leave_requests_by_employee(&s, &id).await?))
}
async fn approve_leave(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ApproveLeaveRequest>,
) -> Result<Json<LeaveRequest>, DbError> {
    Ok(Json(repo::approve_leave(&s, &id, req).await?))
}
async fn reject_leave(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RejectLeaveRequest>,
) -> Result<Json<LeaveRequest>, DbError> {
    Ok(Json(repo::reject_leave(&s, &id, req).await?))
}
async fn leave_balance(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Option<LeaveBalance>>, DbError> {
    Ok(Json(repo::get_leave_balance(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/requests", get(list_leaves).post(create_leave))
        .route("/requests/{employee_id}", get(employee_leaves))
        .route("/{id}/approve", put(approve_leave))
        .route("/{id}/reject", put(reject_leave))
        .route("/balance/{employee_id}", get(leave_balance))
}
