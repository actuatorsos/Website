//! Agent API Handlers — نقاط نهاية API لإدارة الوكلاء والسياسات والموافقات

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};

use crate::models::CurrentUser;

use super::models::*;
use super::repository;
use super::service;
use crate::db::AppState;

// ============================================================================
// Query Params
// ============================================================================

#[derive(serde::Deserialize)]
pub struct ApprovalFilter {
    pub status: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NotifFilter {
    pub unread_only: Option<bool>,
}

// ============================================================================
// Agent Management (Admin only)
// ============================================================================

/// POST /api/agents — Create a new AI agent
async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> axum::response::Result<Json<CreateAgentResponse>, crate::db::DbError> {
    let api_key = service::generate_api_key();
    let hash = service::hash_api_key(&api_key).map_err(|e| crate::db::DbError::Validation(e))?;

    let rate_limit = req.rate_limit.unwrap_or(100);
    let agent = repository::create_agent(
        &state,
        &req.name,
        req.description.as_deref(),
        &hash,
        &req.scopes,
        rate_limit,
    )
    .await?;

    Ok(Json(CreateAgentResponse { agent, api_key }))
}

/// GET /api/agents — List all agents
async fn list_agents(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<AgentAccount>>, crate::db::DbError> {
    let agents = repository::list_agents(&state).await?;
    Ok(Json(agents))
}

/// GET /api/agents/:id — Get agent details
async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<AgentAccount>, crate::db::DbError> {
    let agent = repository::get_agent(&state, &id).await?;
    Ok(Json(agent))
}

/// PUT /api/agents/:id — Update agent
async fn update_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> axum::response::Result<Json<AgentAccount>, crate::db::DbError> {
    let agent = repository::update_agent(&state, &id, req).await?;
    Ok(Json(agent))
}

/// DELETE /api/agents/:id — Delete agent
async fn delete_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::delete_agent(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Policy Management (Admin only)
// ============================================================================

/// GET /api/agents/policies — List all policies
async fn list_policies(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<ActionPolicy>>, crate::db::DbError> {
    let policies = repository::list_policies(&state).await?;
    Ok(Json(policies))
}

/// POST /api/agents/policies — Create a new policy
async fn create_policy(
    State(state): State<AppState>,
    Json(req): Json<CreatePolicyRequest>,
) -> axum::response::Result<Json<ActionPolicy>, crate::db::DbError> {
    let policy = repository::create_policy(&state, req).await?;
    Ok(Json(policy))
}

// ============================================================================
// Approval Management (Admin/Manager)
// ============================================================================

/// GET /api/agents/approvals — List approvals
async fn list_approvals(
    State(state): State<AppState>,
    Query(filter): Query<ApprovalFilter>,
) -> axum::response::Result<Json<Vec<ApprovalRequest>>, crate::db::DbError> {
    let approvals = repository::list_approvals(&state, filter.status.as_deref()).await?;
    Ok(Json(approvals))
}

/// PUT /api/agents/approvals/:id/review — Approve or reject
async fn review_approval(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ReviewApprovalRequest>,
) -> axum::response::Result<Json<ApprovalRequest>, crate::db::DbError> {
    let approval =
        repository::review_approval(&state, &id, req.approved, None, req.note.as_deref()).await?;
    Ok(Json(approval))
}

// ============================================================================
// Agent Action Endpoint (used by AI agents via API Key)
// ============================================================================

/// POST /api/agent-actions/execute — Agent requests to perform an action
async fn execute_action(
    State(state): State<AppState>,
    axum::Extension(agent): axum::Extension<AgentAccount>,
    Json(req): Json<AgentActionRequest>,
) -> axum::response::Result<Json<PolicyDecision>, crate::db::DbError> {
    let decision = service::evaluate_action(&state, &agent, &req)
        .await
        .map_err(|e| crate::db::DbError::Validation(e))?;

    // Log usage
    let agent_id = agent.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
    let _ = repository::log_usage(
        &state,
        &agent_id,
        "/api/agent-actions/execute",
        "POST",
        Some(&req.action),
        if decision.allowed { 200 } else { 202 },
        None,
        Some(decision.allowed),
        Some(decision.decision == "blocked"),
    )
    .await;

    Ok(Json(decision))
}

// ============================================================================
// Notification Endpoints
// ============================================================================

/// GET /api/notifications — List notifications for current user
async fn list_notifications(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(filter): Query<NotifFilter>,
) -> axum::response::Result<Json<Vec<Notification>>, crate::db::DbError> {
    let query = if filter.unread_only.unwrap_or(false) {
        "SELECT * FROM notification WHERE recipient = type::thing('account', $user_id) AND is_read = false ORDER BY created_at DESC LIMIT 50"
    } else {
        "SELECT * FROM notification WHERE recipient = type::thing('account', $user_id) ORDER BY created_at DESC LIMIT 50"
    };

    let notifs: Vec<Notification> = state
        .db
        .query(query)
        .bind(("user_id", user.id.clone()))
        .await?
        .take(0)?;
    Ok(Json(notifs))
}

/// PUT /api/notifications/:id/read — Mark notification as read
async fn mark_notification_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::mark_read(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Routes
// ============================================================================

/// Admin routes for managing agents (requires admin auth)
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        // Agent CRUD
        .route("/", get(list_agents).post(create_agent))
        .route(
            "/{id}",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
        // Policies
        .route("/policies", get(list_policies).post(create_policy))
        // Approvals
        .route("/approvals", get(list_approvals))
        .route("/approvals/{id}/review", put(review_approval))
}

/// Agent action routes (authenticated via API Key, not JWT)
pub fn agent_action_routes() -> Router<AppState> {
    Router::new().route("/execute", post(execute_action))
}

/// Notification routes (for authenticated users)
pub fn notification_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications))
        .route("/{id}/read", put(mark_notification_read))
}
