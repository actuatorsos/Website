use std::sync::Arc;

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::{json, Value};

use actuators_auth::middleware::AppState;
use actuators_core::error::AppError;

use crate::auth::{TenantAuth, tenant_db};

// ── Form structs ─────────────────────────────────────────────────────

/// Form for updating multiple settings at once.
///
/// Each entry is a key-value pair submitted as parallel arrays:
/// `keys[]=foo&values[]=bar&value_types[]=string`
#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub value_types: Vec<String>,
}

// ── Template types ───────────────────────────────────────────────────

#[derive(Debug)]
struct SettingRow {
    key: String,
    value: String,
    value_type: String,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Settings</title></head>
<body>
  <h1>Tenant Settings</h1>
  <form method="post" action="/app/settings">
    <table>
      <thead>
        <tr><th>Key</th><th>Value</th><th>Type</th></tr>
      </thead>
      <tbody>
      {% for s in settings %}
        <tr>
          <td>
            <input type="text" name="keys" value="{{ s.key }}" readonly>
          </td>
          <td>
            <input type="text" name="values" value="{{ s.value }}">
          </td>
          <td>
            <input type="text" name="value_types" value="{{ s.value_type }}" readonly>
          </td>
        </tr>
      {% endfor %}
      </tbody>
    </table>
    <button type="submit">Save All</button>
  </form>
  <a href="/app">&larr; Dashboard</a>
</body>
</html>"#,
    ext = "html"
)]
struct SettingsTemplate {
    settings: Vec<SettingRow>,
}

// ── RBAC ─────────────────────────────────────────────────────────────

fn require_admin(auth: &TenantAuth) -> Result<(), AppError> {
    match auth.0.role.as_str() {
        "admin" | "super_admin" => Ok(()),
        other => Err(AppError::Forbidden(format!(
            "role '{other}' cannot access settings"
        ))),
    }
}

// ── Value helpers ────────────────────────────────────────────────────

fn val_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

// ── Handlers ─────────────────────────────────────────────────────────

/// Display all tenant settings (admin only).
pub async fn index(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db.list_settings().await?;

    let settings: Vec<SettingRow> = raw
        .iter()
        .map(|v| SettingRow {
            key: val_str(v, "key"),
            value: val_str(v, "value"),
            value_type: val_str(v, "value_type"),
        })
        .collect();

    let tmpl = SettingsTemplate { settings };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Update tenant settings (admin only).
///
/// Expects parallel arrays: `keys`, `values`, and `value_types`.
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Form(form): Form<SettingsForm>,
) -> Result<impl IntoResponse, AppError> {
    require_admin(&auth)?;

    let db = tenant_db(&state, &auth).await?;

    let count = form.keys.len().min(form.values.len());

    for i in 0..count {
        let key = &form.keys[i];
        let value = &form.values[i];
        let value_type = form
            .value_types
            .get(i)
            .map(String::as_str)
            .unwrap_or("string");

        db.set_setting(key, value, value_type, Some(&auth.0.account_id))
            .await?;
    }

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "update",
        "settings",
        "setting",
        None,
        Some(json!({ "updated_keys": &form.keys[..count] })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to("/app/settings"))
}
