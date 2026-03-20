//! Database Backup & Restore API
//! Admin-only endpoints for backing up and restoring the database.

use axum::{
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use crate::db::AppState;

/// GET /api/backup — Export entire database as JSON
pub async fn export_backup(State(state): State<AppState>) -> Response {
    // Query all tables data
    let tables = vec![
        "account", "employee", "client", "asset", "machine",
        "project", "repair_operation", "invoice", "certificate",
        "organization", "event", "trainee", "leave_request",
        "attendance", "notification", "audit_log",
    ];

    let mut backup = serde_json::Map::new();
    backup.insert("version".to_string(), serde_json::json!("1.0"));
    backup.insert("timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));

    let mut data = serde_json::Map::new();
    for table in &tables {
        let query = format!("SELECT * FROM {}", table);
        let result: Vec<serde_json::Value> = state.db
            .query(&query)
            .await
            .ok()
            .and_then(|mut r| r.take(0).ok())
            .unwrap_or_default();
        data.insert(table.to_string(), serde_json::json!(result));
    }
    backup.insert("data".to_string(), serde_json::Value::Object(data));

    let json = serde_json::to_string_pretty(&serde_json::Value::Object(backup))
        .unwrap_or_else(|_| "{}".to_string());

    let filename = format!("actuators_backup_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
                .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"backup.json\"")),
        )
        .body(axum::body::Body::from(json))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
