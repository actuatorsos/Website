use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use tokio::sync::Mutex;

use actuators_core::error::AppError;

// ── Configuration ─────────────────────────────────────────────────────

/// Maximum number of requests allowed per window.
const MAX_REQUESTS: u64 = 100;

/// Window duration in seconds (1 minute).
const WINDOW_SECS: u64 = 60;

// ── State ─────────────────────────────────────────────────────────────

/// Per-IP rate-limiting state.
#[derive(Debug, Clone)]
struct RateEntry {
    count: u64,
    window_start: Instant,
}

/// Shared rate limiter state.
///
/// Wraps a `HashMap` keyed by client IP.  Each entry tracks the number of
/// requests seen in the current window.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<Mutex<HashMap<IpAddr, RateEntry>>>,
    max_requests: u64,
    window_secs: u64,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(MAX_REQUESTS, WINDOW_SECS)
    }
}

impl RateLimiter {
    /// Create a new rate limiter with the given limits.
    pub fn new(max_requests: u64, window_secs: u64) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    /// Check whether the given IP is allowed to proceed.
    ///
    /// Returns `true` if the request is within limits, `false` if the
    /// client has exceeded the rate.
    async fn check(&self, ip: IpAddr) -> bool {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();

        let entry = entries.entry(ip).or_insert(RateEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if expired.
        if now.duration_since(entry.window_start).as_secs() >= self.window_secs {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        if entry.count > self.max_requests {
            tracing::warn!(
                ip = %ip,
                count = entry.count,
                max = self.max_requests,
                "rate limit exceeded"
            );
            false
        } else {
            true
        }
    }
}

// ── Middleware ─────────────────────────────────────────────────────────

/// Create a rate-limiting middleware layer.
///
/// Returns a `RateLimiter` and the middleware function. Install the limiter
/// as Axum extension state and add the middleware to your router:
///
/// ```ignore
/// use actuators_middleware::rate_limit::{RateLimiter, rate_limit_middleware};
/// use axum::{Router, middleware, Extension};
///
/// let limiter = RateLimiter::default();
///
/// let app = Router::new()
///     .route("/api/health", get(health))
///     .layer(Extension(limiter))
///     .layer(middleware::from_fn(rate_limit_middleware));
/// ```
pub async fn rate_limit_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    // Try to get the rate limiter from extensions.
    let limiter = req
        .extensions()
        .get::<RateLimiter>()
        .cloned()
        .unwrap_or_default();

    // Extract the client IP.  Prefer ConnectInfo if available; fall back to
    // a well-known proxy header, then to a loopback address.
    let ip = extract_client_ip(&req);

    if !limiter.check(ip).await {
        return Err(AppError::Forbidden(
            "rate limit exceeded — try again later".into(),
        ));
    }

    Ok(next.run(req).await)
}

/// Best-effort client IP extraction.
fn extract_client_ip(req: &Request) -> IpAddr {
    // 1. Try ConnectInfo (set by axum when using `into_make_service_with_connect_info`).
    if let Some(connect_info) = req
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
    {
        return connect_info.0.ip();
    }

    // 2. Try X-Forwarded-For header (first IP is the original client).
    if let Some(xff) = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = xff.split(',').next() {
            if let Ok(ip) = first.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // 3. Try X-Real-IP header.
    if let Some(xri) = req
        .headers()
        .get("X-Real-IP")
        .and_then(|v| v.to_str().ok())
    {
        if let Ok(ip) = xri.trim().parse::<IpAddr>() {
            return ip;
        }
    }

    // 4. Fall back to localhost.
    tracing::debug!("could not determine client IP; defaulting to 127.0.0.1");
    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
}
