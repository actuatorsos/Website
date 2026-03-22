use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::{json, Value};

use actuators_auth::middleware::AppState;
use actuators_core::error::AppError;

use crate::auth::{TenantAuth, tenant_db};

// ── Form structs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAccountForm {
    pub email: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountForm {
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub preferred_lang: Option<String>,
}

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Accounts</title></head>
<body>
  <h1>Accounts</h1>
  <a href="/app/accounts/new">+ New Account</a>
  <table>
    <thead>
      <tr><th>ID</th><th>Email</th><th>Display Name</th><th>Role</th><th>Status</th><th></th></tr>
    </thead>
    <tbody>
    {% for acct in accounts %}
      <tr>
        <td>{{ acct.id }}</td>
        <td>{{ acct.email }}</td>
        <td>{{ acct.display_name }}</td>
        <td>{{ acct.role }}</td>
        <td>{{ acct.status }}</td>
        <td><a href="/app/accounts/{{ acct.id }}">View</a></td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <a href="/app">&larr; Dashboard</a>
</body>
</html>"#,
    ext = "html"
)]
struct AccountListTemplate {
    accounts: Vec<AccountRow>,
}

#[derive(Debug)]
struct AccountRow {
    id: String,
    email: String,
    display_name: String,
    role: String,
    status: String,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>New Account</title></head>
<body>
  <h1>Create Account</h1>
  <form method="post" action="/app/accounts">
    <label>Email <input type="email" name="email" required></label><br>
    <label>Display Name <input type="text" name="display_name" required></label><br>
    <label>Role
      <select name="role">
        <option value="employee">Employee</option>
        <option value="manager">Manager</option>
        <option value="admin">Admin</option>
      </select>
    </label><br>
    <button type="submit">Create</button>
  </form>
  <a href="/app/accounts">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct CreateAccountTemplate;

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Account — {{ account.display_name }}</title></head>
<body>
  <h1>{{ account.display_name }}</h1>
  <form method="post" action="/app/accounts/{{ account.id }}">
    <label>Display Name <input type="text" name="display_name" value="{{ account.display_name }}"></label><br>
    <label>Role
      <select name="role">
        <option value="employee" {% if account.role == "employee" %}selected{% endif %}>Employee</option>
        <option value="manager" {% if account.role == "manager" %}selected{% endif %}>Manager</option>
        <option value="admin" {% if account.role == "admin" %}selected{% endif %}>Admin</option>
      </select>
    </label><br>
    <label>Status
      <select name="status">
        <option value="active" {% if account.status == "active" %}selected{% endif %}>Active</option>
        <option value="suspended" {% if account.status == "suspended" %}selected{% endif %}>Suspended</option>
      </select>
    </label><br>
    <label>Language
      <select name="preferred_lang">
        <option value="en" {% if account.preferred_lang == "en" %}selected{% endif %}>English</option>
        <option value="ar" {% if account.preferred_lang == "ar" %}selected{% endif %}>Arabic</option>
      </select>
    </label><br>
    <button type="submit">Update</button>
  </form>
  <a href="/app/accounts">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct AccountDetailTemplate {
    account: AccountDetail,
}

#[derive(Debug)]
#[allow(dead_code)]
struct AccountDetail {
    id: String,
    email: String,
    display_name: String,
    role: String,
    status: String,
    preferred_lang: String,
}

// ── RBAC helper ──────────────────────────────────────────────────────

/// Only `admin` and `manager` roles may manage accounts.
fn require_account_management(auth: &TenantAuth) -> Result<(), AppError> {
    match auth.0.role.as_str() {
        "admin" | "super_admin" | "manager" => Ok(()),
        other => Err(AppError::Forbidden(format!(
            "role '{other}' cannot manage accounts"
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

fn value_to_row(v: &Value) -> AccountRow {
    AccountRow {
        id: val_str(v, "id"),
        email: val_str(v, "email"),
        display_name: val_str(v, "display_name"),
        role: val_str(v, "role"),
        status: val_str(v, "status"),
    }
}

fn value_to_detail(v: &Value) -> AccountDetail {
    AccountDetail {
        id: val_str(v, "id"),
        email: val_str(v, "email"),
        display_name: val_str(v, "display_name"),
        role: val_str(v, "role"),
        status: val_str(v, "status"),
        preferred_lang: val_str(v, "preferred_lang"),
    }
}

// ── Handlers ─────────────────────────────────────────────────────────

/// List all tenant accounts (admin/manager only).
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    require_account_management(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db.list_accounts().await?;
    let accounts: Vec<AccountRow> = raw.iter().map(value_to_row).collect();

    let tmpl = AccountListTemplate { accounts };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Render the create-account form.
pub async fn create_page(
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    require_account_management(&auth)?;

    let tmpl = CreateAccountTemplate;
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Create a new tenant account.
pub async fn create(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Form(form): Form<CreateAccountForm>,
) -> Result<impl IntoResponse, AppError> {
    require_account_management(&auth)?;

    if form.email.is_empty() || form.display_name.is_empty() {
        return Err(AppError::Validation("email and display_name are required".into()));
    }

    let db = tenant_db(&state, &auth).await?;

    // platform_id is empty for accounts created through the tenant UI;
    // they will be linked when the user accepts an invitation.
    db.create_account("", &form.email, &form.display_name, &form.role, "en")
        .await?;

    // Log the action.
    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "create",
        "accounts",
        "account",
        None,
        Some(json!({ "email": form.email, "role": form.role })),
        None,
        None,
    )
    .await
    .ok(); // best-effort

    Ok(Redirect::to("/app/accounts"))
}

/// View account detail / edit form.
pub async fn detail(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_account_management(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db
        .get_account_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("account '{id}' not found")))?;

    let account = value_to_detail(&raw);
    let tmpl = AccountDetailTemplate { account };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Update a tenant account.
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
    Form(form): Form<UpdateAccountForm>,
) -> Result<impl IntoResponse, AppError> {
    require_account_management(&auth)?;

    let db = tenant_db(&state, &auth).await?;

    let mut fields = serde_json::Map::new();
    if let Some(ref name) = form.display_name {
        fields.insert("display_name".into(), json!(name));
    }
    if let Some(ref role) = form.role {
        fields.insert("role".into(), json!(role));
    }
    if let Some(ref status) = form.status {
        fields.insert("status".into(), json!(status));
    }
    if let Some(ref lang) = form.preferred_lang {
        fields.insert("preferred_lang".into(), json!(lang));
    }

    db.update_account(&id, Value::Object(fields.clone())).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "update",
        "accounts",
        "account",
        Some(&id),
        Some(Value::Object(fields)),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to(&format!("/app/accounts/{id}")))
}
