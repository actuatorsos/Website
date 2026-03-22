//! Field Service Handlers — معالجات الخدمة الميدانية

use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use crate::models::CurrentUser;
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Json,
    routing::{get, put, delete},
};

async fn list_tickets(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<ServiceTicket>>, DbError> {
    Ok(Json(repo::get_all_tickets(&s, user.organization_id.as_deref()).await?))
}

async fn create_ticket(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<ServiceTicket>, DbError> {
    Ok(Json(repo::create_ticket(&s, req, user.organization_id.as_deref()).await?))
}

async fn update_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTicketStatus>,
) -> Result<Json<ServiceTicket>, DbError> {
    Ok(Json(
        repo::update_ticket_status(&s, &id, &req.status, req.notes).await?,
    ))
}

async fn delete_ticket(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ServiceTicket>, DbError> {
    Ok(Json(repo::delete_ticket(&s, &id).await?))
}

async fn my_tickets(
    State(s): State<AppState>,
    Path(emp_id): Path<String>,
) -> Result<Json<Vec<ServiceTicket>>, DbError> {
    Ok(Json(repo::get_tickets_by_employee(&s, &emp_id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tickets", get(list_tickets).post(create_ticket))
        .route("/tickets/{id}/status", put(update_status))
        .route("/tickets/{id}", delete(delete_ticket))
        .route("/tickets/employee/{emp_id}", get(my_tickets))
}
