use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use serde_json::Value;

use actuators_auth::middleware::AppState;
use actuators_core::{error::AppError, types::TenantAuthContext};
use actuators_db::tenant::TenantDb;

// ── Local TenantAuth extractor ───────────────────────────────────────

/// Axum extractor that reads a [`TenantAuthContext`] from the request
/// extensions.
///
/// An upstream middleware (e.g. `resolve_tenant` + a tenant-auth layer)
/// is expected to have verified the JWT and inserted the context.
pub struct TenantAuth(pub TenantAuthContext);

impl<S> FromRequestParts<S> for TenantAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<TenantAuthContext>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("missing tenant auth context".into()))?;
        Ok(TenantAuth(ctx))
    }
}

// ── Helper: obtain a TenantDb from the auth context ──────────────────

/// Get a [`TenantDb`] for the authenticated tenant.
///
/// Uses the `tenant_db` field from [`TenantAuthContext`] (the SurrealDB
/// database name) to fetch a connection from the pool.
pub(crate) async fn tenant_db(
    state: &AppState,
    auth: &TenantAuth,
) -> Result<TenantDb, AppError> {
    state.tenant_pool.get(&auth.0.tenant_db).await
}

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Dashboard -- {{ tenant_slug }}</title></head>
<body>
  <h1>Welcome, {{ display_name }}</h1>
  <p>Tenant: <strong>{{ tenant_slug }}</strong> | Role: {{ role }}</p>
  <section>
    <h2>Overview</h2>
    <ul>
      <li>Employees: {{ employee_count }}</li>
      <li>Departments: {{ department_count }}</li>
      <li>Recent audit entries: {{ audit_count }}</li>
    </ul>
  </section>
  <nav>
    <a href="/app/accounts">Accounts</a> |
    <a href="/app/hr/employees">Employees</a> |
    <a href="/app/hr/departments">Departments</a> |
    <a href="/app/settings">Settings</a> |
    <a href="/app/audit">Audit Log</a>
    <form method="post" action="/app/logout" style="display:inline">
      <button type="submit">Logout</button>
    </form>
  </nav>
</body>
</html>"#,
    ext = "html"
)]
struct DashboardTemplate {
    tenant_slug: String,
    display_name: String,
    role: String,
    employee_count: usize,
    department_count: usize,
    audit_count: usize,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Tenant Login</title></head>
<body>
  <h1>Tenant Login</h1>
  <form method="post" action="/app/login">
    <label>Email <input type="email" name="email" required></label><br>
    <label>Password <input type="password" name="password" required></label><br>
    <button type="submit">Login</button>
  </form>
</body>
</html>"#,
    ext = "html"
)]
struct LoginTemplate;

// ── Handlers ─────────────────────────────────────────────────────────

/// Render the tenant dashboard with summary statistics.
pub async fn tenant_dashboard(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = tenant_db(&state, &auth).await?;
    let ctx = &auth.0;

    // Gather summary stats.
    let employees = db.list_employees(None, None, None, 0, 0).await?;
    let departments = db.list_departments().await?;
    let recent_audit = db
        .list_audit_logs(None, None, None, None, None, 5, 0)
        .await?;

    let display_name = match db.get_account_by_platform_id(&ctx.platform_id).await? {
        Some(acct) => acct
            .get("display_name")
            .and_then(Value::as_str)
            .unwrap_or(&ctx.account_id)
            .to_owned(),
        None => ctx.account_id.clone(),
    };

    let tmpl = DashboardTemplate {
        tenant_slug: ctx.tenant_slug.clone(),
        display_name,
        role: ctx.role.clone(),
        employee_count: employees.len(),
        department_count: departments.len(),
        audit_count: recent_audit.len(),
    };

    Ok(Html(
        tmpl.render()
            .map_err(|e| AppError::Internal(e.to_string()))?,
    ))
}

/// Render the tenant login page.
pub async fn tenant_login_page() -> impl IntoResponse {
    let tmpl = LoginTemplate;
    Html(tmpl.render().unwrap_or_default())
}

/// Handle tenant login form submission (stub -- redirects to login page).
pub async fn tenant_login() -> impl IntoResponse {
    // Full implementation would validate credentials, issue a JWT, and
    // set the `act_tenant_token` cookie.  For now, redirect back.
    Redirect::to("/app/login")
}

/// Clear the tenant auth cookie and redirect to the login page.
pub async fn tenant_logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove("act_tenant_token");
    (jar, Redirect::to("/app/login"))
}
