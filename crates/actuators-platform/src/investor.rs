use std::sync::Arc;

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};
use serde_json::Value;

use actuators_auth::middleware::{AppState, PlatformAuth};
use actuators_core::error::AppError;

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "platform/super_tenants.html")]
pub struct SuperTenantsTemplate {
    pub lang: String,
    pub tenants: Vec<Value>,
}

#[derive(Template)]
#[template(path = "platform/investor_dashboard.html")]
pub struct InvestorDashboardTemplate {
    pub lang: String,
    pub metrics: Vec<Value>,
    pub total_tenants: usize,
}

// ── Handlers ─────────────────────────────────────────────────────────

/// List all tenants on the platform.
///
/// Requires the authenticated user to be a super admin.
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
) -> Result<Response, AppError> {
    if !auth.is_super_admin {
        return Err(AppError::Forbidden(
            "only super admins can access this page".into(),
        ));
    }

    let mut resp = state
        .platform_db
        .client()
        .query("SELECT * FROM tenant ORDER BY created_at DESC;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let tenants: Vec<Value> = resp
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let template = SuperTenantsTemplate {
        lang: "en".to_owned(),
        tenants,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}

/// Show platform-wide metrics for investor / super-admin oversight.
///
/// Requires the authenticated user to be a super admin.
pub async fn investor_dashboard(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
) -> Result<Response, AppError> {
    if !auth.is_super_admin {
        return Err(AppError::Forbidden(
            "only super admins can access this page".into(),
        ));
    }

    // Fetch total tenant count.
    let mut count_resp = state
        .platform_db
        .client()
        .query("SELECT count() AS total FROM tenant GROUP ALL;")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let count_val: Option<Value> = count_resp
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total_tenants = count_val
        .and_then(|v| v["total"].as_u64())
        .unwrap_or(0) as usize;

    // Fetch recent metrics.
    let metrics = state.platform_db.get_metrics("monthly").await?;

    let template = InvestorDashboardTemplate {
        lang: "en".to_owned(),
        metrics,
        total_tenants,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}
