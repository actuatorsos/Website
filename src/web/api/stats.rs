//! Stats API Endpoints
//!
//! Simple count endpoints for dashboard stat cards.

use crate::db::AppState;
use axum::{extract::State, response::Html};

/// GET /api/stats/employees - Returns stat card HTML for employee count.
pub async fn employees_count(State(state): State<AppState>) -> Html<String> {
    let count = get_table_count(&state, "employee").await;
    Html(format_stat_html(count, "موظف نشط"))
}

/// GET /api/stats/assets - Returns stat card HTML for asset count.
pub async fn assets_count(State(state): State<AppState>) -> Html<String> {
    let count = get_table_count(&state, "asset").await;
    Html(format_stat_html(count, "إجمالي الأصول"))
}

/// GET /api/stats/machines - Returns stat card HTML for machine count.
pub async fn machines_count(State(state): State<AppState>) -> Html<String> {
    let count = get_table_count(&state, "machine").await;
    Html(format_stat_html(count, "آلة مسجلة"))
}

/// GET /api/stats/repairs - Returns stat card HTML for active repairs count.
pub async fn repairs_count(State(state): State<AppState>) -> Html<String> {
    #[derive(serde::Deserialize)]
    struct CountResult {
        count: i64,
    }

    let result: Vec<CountResult> = state
        .db
        .query("SELECT count() as count FROM repair_operation WHERE status != 'Completed'")
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    let count = result.first().map(|r| r.count).unwrap_or(0);
    Html(format_stat_html(count, "قيد التنفيذ"))
}

/// Allowed table names for count queries (prevents SQL injection).
const ALLOWED_COUNT_TABLES: &[&str] = &[
    "employee", "asset", "machine", "repair_operation",
    "client", "project", "invoice", "organization", "event",
    "trainee", "certificate", "leave_request", "inventory_item",
];

/// Get total count from a table (validates table name against allowlist).
async fn get_table_count(state: &AppState, table: &str) -> i64 {
    if !ALLOWED_COUNT_TABLES.contains(&table) {
        tracing::warn!("Rejected count query for disallowed table: {}", table);
        return 0;
    }

    #[derive(serde::Deserialize)]
    struct CountResult {
        count: i64,
    }

    let query = format!("SELECT count() as count FROM {}", table);

    let result: Vec<CountResult> = state
        .db
        .query(query)
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    result.first().map(|r| r.count).unwrap_or(0)
}

/// Format stat value as HTML (keeps the parent stat-figure from template).
fn format_stat_html(value: i64, description: &str) -> String {
    format!(
        r#"<div class="stat-value">{}</div>
           <div class="stat-desc">{}</div>"#,
        value, description
    )
}
