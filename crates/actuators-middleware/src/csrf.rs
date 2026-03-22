use axum::{
    body::Body,
    extract::Request,
    http::{header, Method},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use rand::Rng;

use actuators_core::error::AppError;

/// Generate a cryptographically random 64-character hex string for use as a
/// CSRF token.
pub fn generate_csrf_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex_encode(&bytes)
}

/// Double-submit cookie CSRF protection middleware.
///
/// For state-changing methods (POST, PUT, PATCH, DELETE) the middleware
/// compares a submitted token against the value stored in the `act_csrf`
/// cookie.
///
/// The token can be submitted as:
/// - An `X-CSRF-Token` request header, **or**
/// - A `_csrf_token` form field (URL-encoded body).
///
/// If neither matches the cookie value the request is rejected with 403.
///
/// GET / HEAD / OPTIONS requests pass through without checks.
pub async fn csrf_protect(req: Request, next: Next) -> Result<Response, AppError> {
    // Only enforce on state-changing methods.
    let needs_check = matches!(
        *req.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );

    if !needs_check {
        return Ok(next.run(req).await);
    }

    let jar = CookieJar::from_headers(req.headers());

    let cookie_token = jar
        .get("act_csrf")
        .map(|c| c.value().to_owned())
        .unwrap_or_default();

    if cookie_token.is_empty() {
        tracing::warn!("CSRF check failed: act_csrf cookie missing");
        return Err(AppError::Forbidden("CSRF token missing".into()));
    }

    // Check X-CSRF-Token header first.
    let header_token = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned());

    if let Some(ref ht) = header_token {
        if constant_time_eq(ht, &cookie_token) {
            tracing::debug!("CSRF check passed via X-CSRF-Token header");
            return Ok(next.run(req).await);
        }
    }

    // For form submissions, we need to peek at the body for `_csrf_token`.
    // Check the Content-Type to decide whether to look in the body.
    let is_form = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.starts_with("application/x-www-form-urlencoded"))
        .unwrap_or(false);

    if is_form {
        // Buffer the body to inspect the form data.
        let (parts, body) = req.into_parts();
        let bytes = axum::body::to_bytes(body, 1024 * 64) // 64 KiB limit
            .await
            .map_err(|e| AppError::BadRequest(format!("failed to read request body: {e}")))?;

        // Parse form fields.
        let form_token = form_urlencoded::parse(&bytes)
            .find(|(key, _)| key == "_csrf_token")
            .map(|(_, value)| value.into_owned());

        if let Some(ref ft) = form_token {
            if constant_time_eq(ft, &cookie_token) {
                tracing::debug!("CSRF check passed via _csrf_token form field");
                // Reconstruct the request with the buffered body.
                let req = Request::from_parts(parts, Body::from(bytes));
                return Ok(next.run(req).await);
            }
        }

        tracing::warn!("CSRF check failed: token mismatch");
        return Err(AppError::Forbidden("CSRF token mismatch".into()));
    }

    // Neither header nor form token matched.
    tracing::warn!("CSRF check failed: no valid token submitted");
    Err(AppError::Forbidden("CSRF token mismatch".into()))
}

// ── Private helpers ───────────────────────────────────────────────────

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes()
        .iter()
        .zip(b.as_bytes().iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Encode bytes as a lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csrf_token_length() {
        let token = generate_csrf_token();
        assert_eq!(token.len(), 64);
        // Should be valid hex.
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq("abc123", "abc123"));
        assert!(!constant_time_eq("abc123", "abc124"));
        assert!(!constant_time_eq("abc", "abcd"));
    }
}
