use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use tracing::info;

use actuators_auth::middleware::{AppState, OptionalPlatformAuth};
use actuators_auth::password;
use actuators_core::error::AppError;
use actuators_middleware::csrf::generate_csrf_token;

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "platform/register.html")]
pub struct RegisterTemplate {
    pub lang: String,
    pub error: Option<String>,
    pub success: Option<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "platform/verify_email.html")]
pub struct VerifyEmailTemplate {
    pub lang: String,
    pub success: bool,
    pub message: String,
}

// ── Form types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub full_name: String,
    pub preferred_lang: Option<String>,
    #[serde(default)]
    pub _csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyParams {
    pub token: String,
}

// ── Handlers ─────────────────────────────────────────────────────────

/// Render the registration page.
///
/// If the user is already authenticated, redirect them to the dashboard.
pub async fn register_page(
    auth: OptionalPlatformAuth,
) -> Result<Response, AppError> {
    if auth.0.is_some() {
        return Ok(Redirect::to("/dashboard").into_response());
    }

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let template = RegisterTemplate {
        lang: "en".to_owned(),
        error: None,
        success: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Handle the registration form submission.
///
/// Validates the form data, checks that the email is not already taken,
/// hashes the password, creates the platform account, and redirects to
/// the login page with a success message.
pub async fn register(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Result<Response, AppError> {
    // ── Validate inputs ────────────────────────────────────────────
    let email = form.email.trim().to_lowercase();
    let full_name = form.full_name.trim().to_owned();
    let preferred_lang = form.preferred_lang.unwrap_or_else(|| "en".to_owned());

    if email.is_empty() || full_name.is_empty() {
        return render_register_error("Email and full name are required.", &jar);
    }

    if !email.contains('@') || !email.contains('.') {
        return render_register_error("Please enter a valid email address.", &jar);
    }

    if form.password.len() < 8 {
        return render_register_error("Password must be at least 8 characters.", &jar);
    }

    if form.password != form.confirm_password {
        return render_register_error("Passwords do not match.", &jar);
    }

    // ── Check if email is already registered ───────────────────────
    let existing = state.platform_db.get_account_by_email(&email).await?;
    if existing.is_some() {
        return render_register_error(
            "An account with this email already exists.",
            &jar,
        );
    }

    // ── Hash password and create account ───────────────────────────
    let password_hash = password::hash_password(&form.password)?;

    let _account = state
        .platform_db
        .create_account(&email, &password_hash, &full_name, &preferred_lang)
        .await?;

    info!(email = %email, "new platform account created");

    Ok(Redirect::to("/login?registered=true").into_response())
}

/// Verify an email address using the token from the query string.
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VerifyParams>,
) -> Result<Response, AppError> {
    let verified = state.platform_db.verify_email(&params.token).await?;

    let template = VerifyEmailTemplate {
        lang: "en".to_owned(),
        success: verified,
        message: if verified {
            "Your email has been verified. You can now log in.".to_owned()
        } else {
            "Invalid or expired verification link.".to_owned()
        },
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}

// ── Private helpers ──────────────────────────────────────────────────

/// Re-render the registration form with an error message.
fn render_register_error(message: &str, jar: &CookieJar) -> Result<Response, AppError> {
    let csrf_token = jar
        .get("act_csrf")
        .map(|c| c.value().to_owned())
        .unwrap_or_else(generate_csrf_token);

    let template = RegisterTemplate {
        lang: "en".to_owned(),
        error: Some(message.to_owned()),
        success: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}
