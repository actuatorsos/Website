use std::sync::Arc;

use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, warn};

use actuators_auth::jwt::{self, PlatformClaims};
use actuators_auth::middleware::{AppState, OptionalPlatformAuth};
use actuators_auth::password;
use actuators_core::error::AppError;
use actuators_middleware::csrf::generate_csrf_token;

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "platform/login.html")]
pub struct LoginTemplate {
    pub lang: String,
    pub error: Option<String>,
    pub success: Option<String>,
    pub csrf_token: String,
}

// ── Form types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub _csrf_token: String,
}

// ── Handlers ─────────────────────────────────────────────────────────

/// Render the login page.
///
/// If the user is already authenticated, redirect them to the dashboard.
pub async fn login_page(
    auth: OptionalPlatformAuth,
) -> Result<Response, AppError> {
    if auth.0.is_some() {
        return Ok(Redirect::to("/dashboard").into_response());
    }

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let template = LoginTemplate {
        lang: "en".to_owned(),
        error: None,
        success: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Handle the login form submission.
///
/// Verifies the user's credentials, creates a platform JWT and a refresh
/// token, sets both as HttpOnly cookies, and redirects to the dashboard.
pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let email = form.email.trim().to_lowercase();

    // ── Look up the account ────────────────────────────────────────
    let account = state
        .platform_db
        .get_account_by_email(&email)
        .await?
        .ok_or_else(|| {
            warn!(email = %email, "login attempt for non-existent account");
            AppError::Unauthorized("Invalid email or password.".into())
        })?;

    // ── Extract fields from the JSON Value ─────────────────────────
    let password_hash = account["password_hash"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing password_hash in account".into()))?;

    let valid = password::verify_password(&form.password, password_hash)?;
    if !valid {
        warn!(email = %email, "login failed: wrong password");
        return render_login_error("Invalid email or password.", &jar);
    }

    let account_id = account["id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing id in account".into()))?;

    let full_name = account["full_name"]
        .as_str()
        .unwrap_or("")
        .to_owned();

    let is_super_admin = account["is_super_admin"]
        .as_bool()
        .unwrap_or(false);

    // ── Create platform JWT ────────────────────────────────────────
    let now = Utc::now().timestamp() as usize;
    let claims = PlatformClaims {
        sub: account_id.to_owned(),
        email: email.clone(),
        full_name: full_name.clone(),
        is_super_admin,
        iat: now,
        exp: now + state.config.jwt_access_lifetime_secs as usize,
    };

    let access_token = jwt::create_platform_token(&claims, &state.config.jwt_secret)?;

    // ── Create and store refresh token ─────────────────────────────
    let refresh_token = jwt::generate_refresh_token();
    let refresh_hash = sha256_hex(&refresh_token);
    let expires_at = Utc::now()
        + chrono::Duration::seconds(state.config.jwt_refresh_lifetime_secs as i64);

    state
        .platform_db
        .store_refresh_token(
            account_id,
            &refresh_hash,
            "web",
            &expires_at.to_rfc3339(),
        )
        .await?;

    info!(email = %email, "login successful");

    // ── Set cookies ────────────────────────────────────────────────
    let access_cookie = Cookie::build(("act_platform_token", access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(
            state.config.jwt_access_lifetime_secs as i64,
        ))
        .build();

    let refresh_cookie = Cookie::build(("act_refresh_token", refresh_token))
        .path("/refresh-token")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(
            state.config.jwt_refresh_lifetime_secs as i64,
        ))
        .build();

    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((jar, Redirect::to("/dashboard")).into_response())
}

/// Clear all authentication cookies and redirect to the login page.
pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    // Revoke the refresh token from the database if present.
    if let Some(cookie) = jar.get("act_refresh_token") {
        let hash = sha256_hex(cookie.value());
        let _ = state.platform_db.delete_refresh_token(&hash).await;
    }

    let jar = jar
        .remove(Cookie::build("act_platform_token").path("/").build())
        .remove(Cookie::build("act_refresh_token").path("/refresh-token").build())
        .remove(Cookie::build("act_tenant_token").path("/").build())
        .remove(Cookie::build("act_csrf").path("/").build());

    Ok((jar, Redirect::to("/login")).into_response())
}

/// Validate the refresh token, issue a new access token, and rotate the
/// refresh token.
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let refresh_value = jar
        .get("act_refresh_token")
        .map(|c| c.value().to_owned())
        .ok_or_else(|| AppError::Unauthorized("missing refresh token".into()))?;

    let refresh_hash = sha256_hex(&refresh_value);

    // ── Look up the stored token ───────────────────────────────────
    let stored = state
        .platform_db
        .get_refresh_token(&refresh_hash)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid refresh token".into()))?;

    let account_id = stored["account_id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("missing account_id in refresh token".into()))?;

    // ── Check expiration ───────────────────────────────────────────
    if let Some(expires_str) = stored["expires_at"].as_str() {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_str) {
            if Utc::now() > expires {
                state.platform_db.delete_refresh_token(&refresh_hash).await?;
                return Err(AppError::Unauthorized("refresh token expired".into()));
            }
        }
    }

    // ── Fetch the account ──────────────────────────────────────────
    let account = state
        .platform_db
        .get_account_by_id(account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("account not found".into()))?;

    let email = account["email"]
        .as_str()
        .unwrap_or("")
        .to_owned();

    let full_name = account["full_name"]
        .as_str()
        .unwrap_or("")
        .to_owned();

    let is_super_admin = account["is_super_admin"]
        .as_bool()
        .unwrap_or(false);

    // ── Issue new access token ─────────────────────────────────────
    let now = Utc::now().timestamp() as usize;
    let claims = PlatformClaims {
        sub: account_id.to_owned(),
        email,
        full_name,
        is_super_admin,
        iat: now,
        exp: now + state.config.jwt_access_lifetime_secs as usize,
    };

    let new_access_token = jwt::create_platform_token(&claims, &state.config.jwt_secret)?;

    // ── Rotate refresh token ───────────────────────────────────────
    state.platform_db.delete_refresh_token(&refresh_hash).await?;

    let new_refresh_token = jwt::generate_refresh_token();
    let new_refresh_hash = sha256_hex(&new_refresh_token);
    let new_expires_at = Utc::now()
        + chrono::Duration::seconds(state.config.jwt_refresh_lifetime_secs as i64);

    state
        .platform_db
        .store_refresh_token(
            account_id,
            &new_refresh_hash,
            "web",
            &new_expires_at.to_rfc3339(),
        )
        .await?;

    // ── Set new cookies ────────────────────────────────────────────
    let access_cookie = Cookie::build(("act_platform_token", new_access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(
            state.config.jwt_access_lifetime_secs as i64,
        ))
        .build();

    let refresh_cookie = Cookie::build(("act_refresh_token", new_refresh_token))
        .path("/refresh-token")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(
            state.config.jwt_refresh_lifetime_secs as i64,
        ))
        .build();

    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((jar, axum::Json(serde_json::json!({"ok": true}))).into_response())
}

// ── Private helpers ──────────────────────────────────────────────────

/// Re-render the login form with an error message.
fn render_login_error(message: &str, jar: &CookieJar) -> Result<Response, AppError> {
    let csrf_token = jar
        .get("act_csrf")
        .map(|c| c.value().to_owned())
        .unwrap_or_else(generate_csrf_token);

    let template = LoginTemplate {
        lang: "en".to_owned(),
        error: Some(message.to_owned()),
        success: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}

/// Compute a SHA-256 hex digest of the given input string.
///
/// Used for hashing refresh tokens before storage so that the raw token
/// is never persisted.
fn sha256_hex(input: &str) -> String {
    use std::fmt::Write;
    let digest = <sha2::Sha256 as sha2::Digest>::digest(input.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
