//! Rate Limiting Middleware
//!
//! Protects against brute-force and DoS attacks.

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Tracks request count and window start time per IP.
#[derive(Clone, Debug)]
pub struct RateLimitEntry {
    /// Number of requests in current window.
    pub count: u32,
    /// When the current window started.
    pub window_start: Instant,
}

/// Shared state for rate limiting across all requests.
pub type RateLimitState = Arc<RwLock<HashMap<String, RateLimitEntry>>>;

/// Configuration for rate limiting.
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window.
    pub max_requests: u32,
    /// Window duration.
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Create a new rate limit state.
pub fn new_rate_limit_state() -> RateLimitState {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Rate limiting middleware.
///
/// Limits requests per IP address within a sliding time window.
/// Returns 429 Too Many Requests when limit is exceeded.
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<RateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = RateLimitConfig::default();

    // Proxy-aware IP extraction:
    // 1. X-Real-IP header (set by reverse proxies like nginx)
    // 2. X-Forwarded-For header (first IP = original client)
    // 3. Fall back to direct connection address
    let ip = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| addr.ip().to_string());

    // Check and update rate limit
    let allowed = {
        let mut limits = state.write().await;
        let entry = limits.entry(ip.clone()).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: Instant::now(),
        });

        // Reset window if expired
        if entry.window_start.elapsed() > config.window_duration {
            entry.count = 0;
            entry.window_start = Instant::now();
        }

        entry.count += 1;
        entry.count <= config.max_requests
    };

    if !allowed {
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too Many Requests - Please slow down",
        )
            .into_response();
    }

    next.run(request).await
}

/// Cleanup old entries from rate limit state.
/// Should be called periodically to prevent memory leaks.
pub async fn cleanup_rate_limit_state(state: RateLimitState, max_age: Duration) {
    let mut limits = state.write().await;
    limits.retain(|_, entry| entry.window_start.elapsed() < max_age);
}
