//! Health Check Endpoints
//!
//! Liveness and readiness probes for monitoring.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::sync::OnceLock;
use std::time::Instant;

use crate::db::AppState;

/// Application start time, initialised on first health check.
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Public routes (no auth required).
pub fn routes() -> Router<AppState> {
    // Ensure the start-time is recorded as early as possible.
    START_TIME.get_or_init(Instant::now);

    Router::new()
        .route("/health", get(liveness))
        .route("/health/ready", get(readiness))
}

/// `GET /health` — liveness check.
///
/// Returns 200 with uptime in seconds.
async fn liveness() -> impl IntoResponse {
    let uptime = START_TIME
        .get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    Json(json!({
        "status": "ok",
        "uptime_secs": uptime,
    }))
}

/// `GET /health/ready` — readiness check.
///
/// Performs a lightweight DB query; returns 200 if reachable, 503 otherwise.
async fn readiness(State(state): State<AppState>) -> impl IntoResponse {
    // A simple INFO query to verify the connection is alive.
    let result: Result<surrealdb::Response, _> = state.db.query("INFO FOR DB").await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ready" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Health readiness check failed: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unavailable", "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
