//! AI Legal Handlers — نقاط نهاية API للمستشار القانوني السوري

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};

use crate::db::AppState;
use crate::models::CurrentUser;

use super::models::*;
use super::repository;
use super::service;

// ============================================================================
// Query Params
// ============================================================================

#[derive(serde::Deserialize)]
pub struct SpecialtyFilter {
    pub specialty: Option<String>,
}

// ============================================================================
// Chat — المحادثة مع المستشار القانوني
// ============================================================================

/// POST /api/legal/chat — Chat with the AI legal advisor
async fn chat(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<ChatRequest>,
) -> axum::response::Result<Json<ChatResponse>, crate::db::DbError> {
    let response = service::chat(
        &state,
        &user.id,
        &req.message,
        req.specialty.as_deref(),
        req.session_id.as_deref(),
    )
    .await?;

    Ok(Json(response))
}

// ============================================================================
// Sessions — جلسات المحادثة
// ============================================================================

/// GET /api/legal/sessions — List user's chat sessions
async fn list_sessions(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> axum::response::Result<Json<Vec<SessionResponse>>, crate::db::DbError> {
    let sessions = repository::get_sessions(&state, &user.id).await?;

    let response: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id.map(|t| t.id.to_raw()).unwrap_or_default(),
            title: s.title,
            specialty: s.specialty,
            message_count: s.message_count,
            created_at: s.created_at.unwrap_or_default(),
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/legal/sessions/:id/messages — Get messages for a session
async fn get_session_messages(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<Vec<ChatMessage>>, crate::db::DbError> {
    // Verify session belongs to user
    let session = repository::get_session(&state, &id).await?;
    if session.user.as_ref().map(|t| t.id.to_raw()).as_deref() != Some(&user.id) {
        return Err(crate::db::DbError::Forbidden(
            "Session does not belong to this user".to_string(),
        ));
    }

    let messages = repository::get_messages(&state, &id).await?;
    Ok(Json(messages))
}

/// DELETE /api/legal/sessions/:id — Delete a chat session
async fn delete_session(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    // Verify session belongs to user
    let session = repository::get_session(&state, &id).await?;
    if session.user.as_ref().map(|t| t.id.to_raw()).as_deref() != Some(&user.id) {
        return Err(crate::db::DbError::Forbidden(
            "Session does not belong to this user".to_string(),
        ));
    }

    repository::delete_session(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Search — البحث في المواد القانونية
// ============================================================================

/// POST /api/legal/search — Search legal articles
async fn search_articles(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<SearchRequest>,
) -> axum::response::Result<Json<Vec<SearchResult>>, crate::db::DbError> {
    let limit = req.limit.unwrap_or(10).min(50);
    let results =
        repository::search_articles(&state, &req.query, req.specialty.as_deref(), limit).await?;

    // Log the search
    let _ = repository::log_search(
        &state,
        &user.id,
        &req.query,
        req.specialty.as_deref(),
        results.len() as i32,
    )
    .await;

    Ok(Json(results))
}

// ============================================================================
// Specialties — التخصصات القانونية
// ============================================================================

/// GET /api/legal/specialties — List available legal specialties
async fn list_specialties() -> Json<Vec<SpecialtyInfo>> {
    Json(repository::get_specialties())
}

// ============================================================================
// Procedures — الإجراءات القانونية
// ============================================================================

/// GET /api/legal/procedures — List legal procedures
async fn list_procedures(
    State(state): State<AppState>,
    Query(filter): Query<SpecialtyFilter>,
) -> axum::response::Result<Json<Vec<LegalProcedure>>, crate::db::DbError> {
    let procedures = repository::get_procedures(&state, filter.specialty.as_deref()).await?;
    Ok(Json(procedures))
}

// ============================================================================
// Routes — المسارات
// ============================================================================

/// Legal advisor routes (requires JWT auth)
pub fn legal_routes() -> Router<AppState> {
    Router::new()
        .route("/chat", post(chat))
        .route("/sessions", get(list_sessions))
        .route(
            "/sessions/{id}/messages",
            get(get_session_messages),
        )
        .route("/sessions/{id}", delete(delete_session))
        .route("/search", post(search_articles))
        .route("/specialties", get(list_specialties))
        .route("/procedures", get(list_procedures))
}
