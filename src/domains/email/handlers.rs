//! Email API Handlers — نقاط نهاية API للبريد الإلكتروني

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};

use super::models::*;
use super::repository;
use super::service;
use crate::db::AppState;

// ============================================================================
// Query Params
// ============================================================================

#[derive(serde::Deserialize)]
pub struct LogFilter {
    pub status: Option<String>,
}

// ============================================================================
// Config Handlers
// ============================================================================

/// GET /api/email/config — Get active SMTP config
async fn get_config(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Option<EmailConfig>>, crate::db::DbError> {
    let config = repository::get_active_config(&state).await?;
    Ok(Json(config))
}

/// POST /api/email/config — Create/update SMTP config
async fn save_config(
    State(state): State<AppState>,
    Json(req): Json<UpsertEmailConfigRequest>,
) -> axum::response::Result<Json<EmailConfig>, crate::db::DbError> {
    let config = repository::upsert_config(&state, req).await?;
    Ok(Json(config))
}

// ============================================================================
// Template Handlers
// ============================================================================

/// GET /api/email/templates — List all templates
async fn list_templates(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<EmailTemplate>>, crate::db::DbError> {
    let templates = repository::list_templates(&state).await?;
    Ok(Json(templates))
}

/// GET /api/email/templates/:id — Get template by ID
async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<EmailTemplate>, crate::db::DbError> {
    let template = repository::get_template(&state, &id).await?;
    Ok(Json(template))
}

/// POST /api/email/templates — Create a new template
async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> axum::response::Result<Json<EmailTemplate>, crate::db::DbError> {
    let template = repository::create_template(&state, req).await?;
    Ok(Json(template))
}

/// PUT /api/email/templates/:id — Update a template
async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTemplateRequest>,
) -> axum::response::Result<Json<EmailTemplate>, crate::db::DbError> {
    let template = repository::update_template(&state, &id, req).await?;
    Ok(Json(template))
}

/// DELETE /api/email/templates/:id — Delete a template
async fn delete_template_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::delete_template(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Send Email Handler
// ============================================================================

/// POST /api/email/send — Send emails
async fn send_email(
    State(state): State<AppState>,
    Json(req): Json<SendEmailRequest>,
) -> axum::response::Result<Json<SendEmailResponse>, crate::db::DbError> {
    let response = service::send_email(&state, &req)
        .await
        .map_err(|e| crate::db::DbError::Validation(e))?;
    Ok(Json(response))
}

// ============================================================================
// Logs Handler
// ============================================================================

/// GET /api/email/logs — List email logs
async fn list_logs(
    State(state): State<AppState>,
    Query(filter): Query<LogFilter>,
) -> axum::response::Result<Json<Vec<EmailLog>>, crate::db::DbError> {
    let logs = repository::list_logs(&state, filter.status.as_deref()).await?;
    Ok(Json(logs))
}

/// GET /api/email/recipients — Get all valid email recipients (employees + trainees)
async fn get_recipients(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<serde_json::Value>>, crate::db::DbError> {
    // Get employees with emails
    let employees: Vec<serde_json::Value> = state.db
        .query("SELECT id, name, email, 'employee' AS type FROM employee WHERE email != NONE AND (is_archived = false OR is_archived = NONE)")
        .await?.take(0)?;

    // Get trainees with emails
    let trainees: Vec<serde_json::Value> = state.db
        .query("SELECT id, name, email, 'trainee' AS type FROM trainee WHERE email != NONE AND (is_archived = false OR is_archived = NONE)")
        .await?.take(0)?;

    let mut all = employees;
    all.extend(trainees);
    Ok(Json(all))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        // Config
        .route("/config", get(get_config).post(save_config))
        // Templates
        .route("/templates", get(list_templates).post(create_template))
        .route(
            "/templates/{id}",
            get(get_template)
                .put(update_template)
                .delete(delete_template_handler),
        )
        // Send
        .route("/send", post(send_email))
        // Logs
        .route("/logs", get(list_logs))
        // Recipients
        .route("/recipients", get(get_recipients))
}
