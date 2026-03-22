//! Client Portal Handlers — معالجات بوابة العميل

use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use crate::models::CurrentUser;
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post, put, delete},
};

async fn list_tickets(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<SupportTicket>>, DbError> {
    Ok(Json(repo::get_all_tickets(&s, user.organization_id.as_deref()).await?))
}

async fn get_ticket(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SupportTicket>, DbError> {
    Ok(Json(repo::get_ticket_by_id(&s, &id).await?))
}

async fn create_ticket(
    State(s): State<AppState>,
    Json(req): Json<CreateTicketRequest>,
) -> Result<Json<SupportTicket>, DbError> {
    Ok(Json(repo::create_ticket(&s, req, "system").await?))
}

async fn add_reply(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateReplyRequest>,
) -> Result<Json<TicketReply>, DbError> {
    let role = req.author_role.unwrap_or_else(|| "client".to_string());
    Ok(Json(
        repo::add_reply(&s, &id, &req.message, "system", &role).await?,
    ))
}

async fn update_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTicketStatus>,
) -> Result<Json<SupportTicket>, DbError> {
    Ok(Json(
        repo::update_ticket_status(&s, &id, &req.status, req.resolution).await?,
    ))
}

async fn close_ticket(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CloseTicketRequest>,
) -> Result<Json<SupportTicket>, DbError> {
    Ok(Json(
        repo::close_ticket(&s, &id, req.resolution, req.rating).await?,
    ))
}

async fn soft_delete_ticket(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SupportTicket>, DbError> {
    Ok(Json(repo::delete_ticket(&s, &id).await?))
}

async fn list_replies(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<TicketReply>>, DbError> {
    Ok(Json(repo::get_ticket_replies(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tickets", get(list_tickets).post(create_ticket))
        .route("/tickets/{id}", get(get_ticket).delete(soft_delete_ticket))
        .route("/tickets/{id}/reply", post(add_reply))
        .route("/tickets/{id}/replies", get(list_replies))
        .route("/tickets/{id}/status", put(update_status))
        .route("/tickets/{id}/close", put(close_ticket))
}
