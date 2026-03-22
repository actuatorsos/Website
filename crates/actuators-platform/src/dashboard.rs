use std::sync::Arc;

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use actuators_auth::middleware::{AppState, PlatformAuth};
use actuators_core::error::AppError;
use actuators_middleware::csrf::generate_csrf_token;

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "platform/dashboard.html")]
pub struct DashboardTemplate {
    pub lang: String,
    pub full_name: String,
    pub email: String,
    pub tenants: Vec<Value>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "platform/profile.html")]
pub struct ProfileTemplate {
    pub lang: String,
    pub full_name: String,
    pub email: String,
    pub preferred_lang: String,
    pub error: Option<String>,
    pub success: Option<String>,
    pub csrf_token: String,
}

// ── Form types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProfileForm {
    pub full_name: String,
    pub preferred_lang: String,
    #[serde(default)]
    pub _csrf_token: String,
}

// ── Handlers ─────────────────────────────────────────────────────────

/// Render the personal dashboard showing the user's tenant memberships
/// and available actions (create tenant, accept invitations, etc.).
pub async fn dashboard_page(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
) -> Result<Response, AppError> {
    let tenants = state
        .platform_db
        .list_tenants_for_account(&auth.account_id)
        .await?;

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let template = DashboardTemplate {
        lang: "en".to_owned(),
        full_name: auth.full_name,
        email: auth.email,
        tenants,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Render the profile editing form.
pub async fn profile_page(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
) -> Result<Response, AppError> {
    let account = state
        .platform_db
        .get_account_by_id(&auth.account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("account not found".into()))?;

    let preferred_lang = account["preferred_lang"]
        .as_str()
        .unwrap_or("en")
        .to_owned();

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let template = ProfileTemplate {
        lang: preferred_lang.clone(),
        full_name: auth.full_name,
        email: auth.email,
        preferred_lang,
        error: None,
        success: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Handle the profile update form submission.
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
    Form(form): Form<ProfileForm>,
) -> Result<Response, AppError> {
    let full_name = form.full_name.trim().to_owned();
    let preferred_lang = form.preferred_lang.trim().to_owned();

    if full_name.is_empty() {
        let csrf_token = generate_csrf_token();
        let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build();

        let template = ProfileTemplate {
            lang: preferred_lang.clone(),
            full_name: auth.full_name,
            email: auth.email,
            preferred_lang,
            error: Some("Full name cannot be empty.".to_owned()),
            success: None,
            csrf_token,
        };
        let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
        return Ok((CookieJar::new().add(cookie), Html(html)).into_response());
    }

    let fields = serde_json::json!({
        "full_name": full_name,
        "preferred_lang": preferred_lang,
    });

    state
        .platform_db
        .update_account(&auth.account_id, fields)
        .await?;

    info!(
        account_id = %auth.account_id,
        "profile updated"
    );

    let csrf_token = generate_csrf_token();
    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let template = ProfileTemplate {
        lang: preferred_lang.clone(),
        full_name,
        email: auth.email,
        preferred_lang,
        error: None,
        success: Some("Profile updated successfully.".to_owned()),
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}
