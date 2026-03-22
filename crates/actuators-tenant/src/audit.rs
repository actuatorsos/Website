use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use serde_json::Value;

use actuators_auth::middleware::AppState;
use actuators_core::error::AppError;

use crate::auth::{TenantAuth, tenant_db};

// ── Query struct ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuditFilters {
    pub service: Option<String>,
    pub action: Option<String>,
    pub actor_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ── Template types ───────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
struct AuditRow {
    id: String,
    actor_id: String,
    actor_name: String,
    action: String,
    service: String,
    entity_type: String,
    entity_id: String,
    created_at: String,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Audit Log</title></head>
<body>
  <h1>Audit Log</h1>
  <form method="get" action="/app/audit">
    <label>Service <input type="text" name="service" value="{{ filter_service }}"></label>
    <label>Action <input type="text" name="action" value="{{ filter_action }}"></label>
    <label>Actor <input type="text" name="actor_id" value="{{ filter_actor }}"></label>
    <label>From <input type="date" name="date_from" value="{{ filter_date_from }}"></label>
    <label>To <input type="date" name="date_to" value="{{ filter_date_to }}"></label>
    <button type="submit">Filter</button>
  </form>
  <table>
    <thead>
      <tr>
        <th>Time</th><th>Actor</th><th>Service</th><th>Action</th>
        <th>Entity</th><th>Entity ID</th>
      </tr>
    </thead>
    <tbody id="audit-rows">
    {% for entry in entries %}
      <tr>
        <td>{{ entry.created_at }}</td>
        <td>{{ entry.actor_name }}</td>
        <td>{{ entry.service }}</td>
        <td>{{ entry.action }}</td>
        <td>{{ entry.entity_type }}</td>
        <td>{{ entry.entity_id }}</td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <p>Page {{ page }} | {{ total }} entries shown</p>
  <a href="/app">&larr; Dashboard</a>
</body>
</html>"#,
    ext = "html"
)]
struct AuditListTemplate {
    entries: Vec<AuditRow>,
    filter_service: String,
    filter_action: String,
    filter_actor: String,
    filter_date_from: String,
    filter_date_to: String,
    page: i64,
    total: usize,
}

#[derive(Template)]
#[template(
    source = r#"{% for entry in entries %}
<tr>
  <td>{{ entry.created_at }}</td>
  <td>{{ entry.actor_name }}</td>
  <td>{{ entry.service }}</td>
  <td>{{ entry.action }}</td>
  <td>{{ entry.entity_type }}</td>
  <td>{{ entry.entity_id }}</td>
</tr>
{% endfor %}"#,
    ext = "html"
)]
struct AuditRowsPartialTemplate {
    entries: Vec<AuditRow>,
}

// ── Value helpers ────────────────────────────────────────────────────

fn val_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn value_to_row(v: &Value) -> AuditRow {
    AuditRow {
        id: val_str(v, "id"),
        actor_id: val_str(v, "actor_id"),
        actor_name: val_str(v, "actor_name"),
        action: val_str(v, "action"),
        service: val_str(v, "service"),
        entity_type: val_str(v, "entity_type"),
        entity_id: val_str(v, "entity_id"),
        created_at: val_str(v, "created_at"),
    }
}

// ── Handlers ─────────────────────────────────────────────────────────

/// List audit log entries with optional filters. Supports HTMX partial responses.
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    headers: HeaderMap,
    Query(filters): Query<AuditFilters>,
) -> Result<impl IntoResponse, AppError> {
    // All authenticated users can view the audit log; sensitive data is
    // filtered server-side in more restrictive deployments.
    let db = tenant_db(&state, &auth).await?;

    let page = filters.page.unwrap_or(1).max(1);
    let per_page = filters.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;

    let raw = db
        .list_audit_logs(
            filters.service.as_deref(),
            filters.action.as_deref(),
            filters.actor_id.as_deref(),
            filters.date_from.as_deref(),
            filters.date_to.as_deref(),
            per_page,
            offset,
        )
        .await?;

    let entries: Vec<AuditRow> = raw.iter().map(value_to_row).collect();
    let total = entries.len();

    // HTMX partial: return only the table rows.
    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some();

    if is_htmx {
        let tmpl = AuditRowsPartialTemplate { entries };
        return Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?));
    }

    let tmpl = AuditListTemplate {
        entries,
        filter_service: filters.service.unwrap_or_default(),
        filter_action: filters.action.unwrap_or_default(),
        filter_actor: filters.actor_id.unwrap_or_default(),
        filter_date_from: filters.date_from.unwrap_or_default(),
        filter_date_to: filters.date_to.unwrap_or_default(),
        page,
        total,
    };

    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}
