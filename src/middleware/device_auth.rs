//! Device Authentication Middleware
//!
//! HMAC-based authentication for embedded devices (ESP32/STM32).

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, header::HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::db::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Header names for device authentication.
pub const DEVICE_ID_HEADER: &str = "X-Device-ID";
pub const SIGNATURE_HEADER: &str = "X-Signature";
pub const TIMESTAMP_HEADER: &str = "X-Timestamp";

/// Maximum age of a request (prevents replay attacks).
const MAX_REQUEST_AGE_SECS: i64 = 300; // 5 minutes

/// Verify HMAC signature for device requests.
///
/// Expected signature format: HMAC-SHA256(device_id + timestamp + body)
pub fn verify_hmac(
    device_id: &str,
    timestamp: &str,
    body: &[u8],
    secret: &[u8],
    provided_signature: &str,
) -> Result<(), &'static str> {
    // Create HMAC instance
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| "Invalid secret key length")?;

    // Update with message components
    mac.update(device_id.as_bytes());
    mac.update(timestamp.as_bytes());
    mac.update(body);

    // Decode provided signature from hex
    let provided_bytes = hex::decode(provided_signature).map_err(|_| "Invalid signature format")?;

    // Verify (constant-time comparison)
    mac.verify_slice(&provided_bytes)
        .map_err(|_| "Signature verification failed")
}

/// Check if timestamp is within acceptable range.
pub fn verify_timestamp(timestamp: &str) -> Result<(), &'static str> {
    let ts: i64 = timestamp.parse().map_err(|_| "Invalid timestamp format")?;
    let now = chrono::Utc::now().timestamp();

    if (now - ts).abs() > MAX_REQUEST_AGE_SECS {
        return Err("Request timestamp too old or in future");
    }

    Ok(())
}

/// Middleware to authenticate device requests using HMAC.
pub async fn device_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Extract required headers
    let device_id = match headers.get(DEVICE_ID_HEADER).and_then(|v| v.to_str().ok()) {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing X-Device-ID header").into_response();
        }
    };

    let signature = match headers.get(SIGNATURE_HEADER).and_then(|v| v.to_str().ok()) {
        Some(sig) => sig,
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing X-Signature header").into_response();
        }
    };

    let timestamp = match headers.get(TIMESTAMP_HEADER).and_then(|v| v.to_str().ok()) {
        Some(ts) => ts,
        None => {
            return (StatusCode::UNAUTHORIZED, "Missing X-Timestamp header").into_response();
        }
    };

    // Verify timestamp to prevent replay attacks
    if let Err(e) = verify_timestamp(timestamp) {
        tracing::warn!("Device auth failed for {}: {}", device_id, e);
        return (StatusCode::UNAUTHORIZED, e).into_response();
    }

    // Get device secret from database
    let secret = match get_device_secret(&state, device_id).await {
        Ok(s) => s,
        Err(_) => {
            tracing::warn!("Unknown device attempted auth: {}", device_id);
            return (StatusCode::UNAUTHORIZED, "Unknown device").into_response();
        }
    };

    // For body verification, we would need to buffer the body
    // For now, verify without body (headers only)
    if let Err(e) = verify_hmac(device_id, timestamp, &[], secret.as_bytes(), signature) {
        tracing::warn!("HMAC verification failed for device {}: {}", device_id, e);
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    tracing::info!("Device {} authenticated successfully", device_id);
    next.run(request).await
}

/// Retrieve device secret from database.
async fn get_device_secret(state: &AppState, device_id: &str) -> Result<String, &'static str> {
    // Clone for 'static requirement
    let id = device_id.to_string();

    let result: Vec<DeviceRecord> = state
        .db
        .query("SELECT secret FROM device WHERE device_id = $id")
        .bind(("id", id))
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    result
        .into_iter()
        .next()
        .map(|r| r.secret)
        .ok_or("Device not found")
}

#[derive(serde::Deserialize)]
struct DeviceRecord {
    secret: String,
}
