// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Authentication & Authorization Middleware
//!
//! JWT token authentication + RBAC role-based access control.
//! Replaces legacy Basic Auth for user-facing routes.

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::Engine;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::db::AppState;
use crate::models::{AccountRole, Claims, CurrentUser};

// ============================================================================
// JWT Token Helpers
// ============================================================================

/// Create a JWT token for an authenticated account
pub fn create_token(
    account_id: &str,
    email: &str,
    role: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: account_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(expiry_hours)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Decode and validate a JWT token  
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

// ============================================================================
// JWT Auth Middleware — extracts CurrentUser from token
// ============================================================================

/// Middleware: requires a valid JWT token.
///
/// Checks (in order):
/// 1. `Authorization: Bearer <token>` header
/// 2. `auth_token` cookie
///
/// On success, injects `CurrentUser` into request extensions.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let token = extract_token(&request);

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing authentication token"})),
            )
                .into_response();
        }
    };

    match decode_token(&token, &state.jwt_secret) {
        Ok(claims) => {
            let current_user = CurrentUser {
                id: claims.sub,
                email: claims.email,
                role: AccountRole::from_str_or_default(&claims.role),
            };
            request.extensions_mut().insert(current_user);
            next.run(request).await
        }
        Err(e) => {
            tracing::warn!("JWT validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})),
            )
                .into_response()
        }
    }
}

/// Extract JWT token from Authorization header or cookie
fn extract_token(request: &Request<Body>) -> Option<String> {
    // Try Authorization: Bearer <token>
    if let Some(auth) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    // Try auth_token cookie
    if let Some(cookie_header) = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
    {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("auth_token=") {
                return Some(token.to_string());
            }
        }
    }

    // Try query string ?token=...
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some(token) = pair.strip_prefix("token=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

// ============================================================================
// RBAC Middleware — role-based access control
// ============================================================================

/// Create a middleware that requires the user to have one of the specified roles.
///
/// Usage in router:
/// ```ignore
/// .route_layer(axum::middleware::from_fn(require_role(&["admin", "manager"])))
/// ```
pub async fn require_admin(request: Request<Body>, next: Next) -> Response {
    check_role(request, next, &[AccountRole::Admin]).await
}

/// Require admin or manager role
pub async fn require_manager(request: Request<Body>, next: Next) -> Response {
    check_role(request, next, &[AccountRole::Admin, AccountRole::Manager]).await
}

/// Require admin, manager, or employee role
pub async fn require_employee(request: Request<Body>, next: Next) -> Response {
    check_role(
        request,
        next,
        &[
            AccountRole::Admin,
            AccountRole::Manager,
            AccountRole::Employee,
        ],
    )
    .await
}

/// Internal: check if CurrentUser has one of the allowed roles
async fn check_role(request: Request<Body>, next: Next, allowed_roles: &[AccountRole]) -> Response {
    let user = request.extensions().get::<CurrentUser>();

    match user {
        Some(user) if allowed_roles.contains(&user.role) => next.run(request).await,
        Some(user) => {
            tracing::warn!(
                "Access denied for user {} (role: {}) — required: {:?}",
                user.email,
                user.role,
                allowed_roles
            );
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Insufficient permissions"})),
            )
                .into_response()
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response(),
    }
}

// ============================================================================
// Legacy Basic Auth — kept for device/API backward compatibility
// ============================================================================

/// Admin credentials from environment (legacy)
pub struct AdminCredentials {
    /// Admin username
    pub username: String,
    /// Admin password
    pub password: String,
}

impl AdminCredentials {
    /// Load admin credentials from environment variables.
    pub fn from_env() -> Self {
        Self {
            username: std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string()),
            password: std::env::var("ADMIN_PASS").unwrap_or_else(|_| "admin".to_string()),
        }
    }

    /// Verify provided credentials (constant-time comparison).
    pub fn verify(&self, username: &str, password: &str) -> bool {
        use subtle::ConstantTimeEq;
        let user_match = self.username.as_bytes().ct_eq(username.as_bytes());
        let pass_match = self.password.as_bytes().ct_eq(password.as_bytes());
        (user_match & pass_match).into()
    }
}

/// Middleware to require Basic Auth for legacy/device routes.
pub async fn require_basic_auth(request: Request, next: Next) -> Response {
    let credentials = AdminCredentials::from_env();

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    if let Some(auth) = auth_header {
        if let Some(encoded) = auth.strip_prefix("Basic ") {
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
                if let Ok(decoded_str) = String::from_utf8(decoded) {
                    if let Some((user, pass)) = decoded_str.split_once(':') {
                        if credentials.verify(user, pass) {
                            return next.run(request).await;
                        }
                    }
                }
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"Admin Area\"")],
        "Unauthorized",
    )
        .into_response()
}
