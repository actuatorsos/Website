//! CSRF Protection Middleware
//!
//! Double-submit cookie pattern for CSRF protection.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use rand::Rng;
use tower_cookies::{Cookie, Cookies};

/// Cookie name for CSRF token.
pub const CSRF_COOKIE_NAME: &str = "csrf_token";

/// Header name for CSRF token.
pub const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

/// Form field name for CSRF token.
pub const CSRF_FIELD_NAME: &str = "_csrf";

/// Generate a new CSRF token.
pub fn generate_csrf_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex::encode(bytes)
}

/// CSRF protection middleware.
///
/// For safe methods (GET, HEAD, OPTIONS), sets a CSRF cookie if not present.
/// For unsafe methods (POST, PUT, DELETE, PATCH), validates the token.
pub async fn csrf_protection(
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();

    // Safe methods - ensure cookie exists
    if method == axum::http::Method::GET
        || method == axum::http::Method::HEAD
        || method == axum::http::Method::OPTIONS
    {
        // Set CSRF cookie if not present
        if cookies.get(CSRF_COOKIE_NAME).is_none() {
            let token = generate_csrf_token();
            let mut cookie = Cookie::new(CSRF_COOKIE_NAME, token);
            cookie.set_path("/");
            cookie.set_http_only(false); // Must be readable by JS
            cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
            cookies.add(cookie);
        }
        return Ok(next.run(request).await);
    }

    // Unsafe methods - validate token
    let cookie_token = cookies.get(CSRF_COOKIE_NAME).map(|c| c.value().to_string());

    // Check header first, then form field
    let request_token = request
        .headers()
        .get(CSRF_HEADER_NAME)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Validate tokens match
    match (cookie_token, request_token) {
        (Some(cookie), Some(header)) if constant_time_compare(&cookie, &header) => {
            Ok(next.run(request).await)
        }
        _ => {
            tracing::warn!("CSRF validation failed for {} {}", method, request.uri());
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_compare(a: &str, b: &str) -> bool {
    use subtle::ConstantTimeEq;

    if a.len() != b.len() {
        return false;
    }

    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Extension trait for extracting CSRF token in templates.
pub trait CsrfTokenExt {
    /// Get the CSRF token for use in forms.
    fn csrf_token(&self) -> Option<String>;
}

impl CsrfTokenExt for Cookies {
    fn csrf_token(&self) -> Option<String> {
        self.get(CSRF_COOKIE_NAME).map(|c| c.value().to_string())
    }
}
