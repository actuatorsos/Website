//! Device API
//!
//! REST API for embedded devices (ESP32/STM32) to report status.

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::db::AppState;

/// Device status report payload.
#[derive(Deserialize)]
pub struct DeviceStatusReport {
    /// Unique device identifier.
    pub device_id: String,
    /// Current device status (online, offline, error, etc.).
    pub status: String,
    /// Optional temperature reading in Celsius.
    pub temperature: Option<f32>,
    /// Optional humidity reading as percentage.
    pub humidity: Option<f32>,
    /// Device uptime in seconds.
    pub uptime_seconds: u64,
    /// Unix timestamp of the report.
    pub timestamp: i64,
    /// Optional error message if status is "error".
    pub error_message: Option<String>,
    /// Optional firmware version.
    pub firmware_version: Option<String>,
}

/// Device registration request.
#[derive(Deserialize)]
pub struct DeviceRegistration {
    /// Desired device ID.
    pub device_id: String,
    /// Device type (ESP32, STM32, etc.).
    pub device_type: String,
    /// Device name/description.
    pub name: String,
    /// Location where device is installed.
    pub location: Option<String>,
}

/// Device registration response.
#[derive(Serialize)]
pub struct DeviceRegistrationResponse {
    /// Assigned device ID.
    pub device_id: String,
    /// Secret key for HMAC authentication.
    pub secret: String,
    /// API endpoint for status reports.
    pub status_endpoint: String,
}

/// Device status response.
#[derive(Serialize)]
pub struct DeviceStatusResponse {
    /// Whether report was accepted.
    pub accepted: bool,
    /// Server timestamp.
    pub server_time: i64,
    /// Optional command for device to execute.
    pub command: Option<DeviceCommand>,
}

/// Command to send to device.
#[derive(Serialize)]
pub struct DeviceCommand {
    /// Command type (reboot, update, configure, etc.).
    pub command_type: String,
    /// Command parameters as JSON.
    pub params: Option<serde_json::Value>,
}

/// Device info for listing.
#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device ID.
    pub device_id: String,
    /// Device type.
    pub device_type: String,
    /// Device name.
    pub name: String,
    /// Location.
    pub location: Option<String>,
    /// Current status.
    pub status: Option<String>,
    /// Last seen timestamp.
    pub last_seen: Option<String>,
}

/// POST /api/v1/devices/status
///
/// Receive status report from a device.
/// Requires device authentication via HMAC headers.
pub async fn report_status(
    State(state): State<AppState>,
    Json(payload): Json<DeviceStatusReport>,
) -> Result<Json<DeviceStatusResponse>, StatusCode> {
    // Validate payload
    if payload.device_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Clone values for 'static requirement
    let device_id = payload.device_id.clone();
    let status = payload.status.clone();
    let error_msg = payload.error_message.clone();
    let firmware = payload.firmware_version.clone();

    // Update device status in database
    let result = state
        .db
        .query(
            r#"
            UPDATE device_status 
            SET 
                status = $status,
                temperature = $temp,
                humidity = $humidity,
                uptime_seconds = $uptime,
                error_message = $error,
                firmware_version = $firmware,
                last_seen = time::now()
            WHERE device_id = $device_id
        "#,
        )
        .bind(("device_id", device_id.clone()))
        .bind(("status", status))
        .bind(("temp", payload.temperature))
        .bind(("humidity", payload.humidity))
        .bind(("uptime", payload.uptime_seconds))
        .bind(("error", error_msg))
        .bind(("firmware", firmware))
        .await;

    if let Err(e) = result {
        tracing::error!("Failed to update device status: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Check for pending commands
    let command = get_pending_command(&state, &device_id).await;

    Ok(Json(DeviceStatusResponse {
        accepted: true,
        server_time: chrono::Utc::now().timestamp(),
        command,
    }))
}

/// POST /api/v1/devices/register
///
/// Register a new device and generate API credentials.
/// This endpoint should be protected with admin authentication.
pub async fn register_device(
    State(state): State<AppState>,
    Json(payload): Json<DeviceRegistration>,
) -> Result<Json<DeviceRegistrationResponse>, StatusCode> {
    // Validate payload
    if payload.device_id.is_empty() || payload.device_type.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Clone values for 'static requirement and response
    let device_id = payload.device_id.clone();
    let device_type = payload.device_type.clone();
    let name = payload.name.clone();
    let location = payload.location.clone();

    // Generate secure secret
    let secret = generate_device_secret();
    let secret_clone = secret.clone();

    // Create device record
    let result = state
        .db
        .query(
            r#"
            CREATE device SET
                device_id = $device_id,
                device_type = $device_type,
                name = $name,
                location = $location,
                secret = $secret,
                created_at = time::now(),
                last_seen = time::now()
        "#,
        )
        .bind(("device_id", device_id.clone()))
        .bind(("device_type", device_type))
        .bind(("name", name))
        .bind(("location", location))
        .bind(("secret", secret_clone))
        .await;

    if let Err(e) = result {
        tracing::error!("Failed to register device: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Create initial status record
    let device_id_for_status = device_id.clone();
    let _ = state
        .db
        .query(
            r#"
            CREATE device_status SET
                device_id = $device_id,
                status = 'registered',
                last_seen = time::now()
        "#,
        )
        .bind(("device_id", device_id_for_status))
        .await;

    tracing::info!("Device {} registered successfully", device_id);

    Ok(Json(DeviceRegistrationResponse {
        device_id,
        secret,
        status_endpoint: "/api/v1/devices/status".to_string(),
    }))
}

/// GET /api/v1/devices
///
/// List all registered devices and their status.
pub async fn list_devices(
    State(state): State<AppState>,
) -> Result<Json<Vec<DeviceInfo>>, StatusCode> {
    let devices: Vec<DeviceInfo> = state.db
        .query(r#"
            SELECT 
                device_id,
                device_type,
                name,
                location,
                (SELECT status, last_seen FROM device_status WHERE device_id = $parent.device_id)[0] as status_info
            FROM device
            ORDER BY last_seen DESC
        "#)
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    Ok(Json(devices))
}

/// Generate a secure random secret for device authentication.
fn generate_device_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex::encode(bytes)
}

/// Get pending command for a device.
async fn get_pending_command(state: &AppState, device_id: &str) -> Option<DeviceCommand> {
    #[derive(serde::Deserialize)]
    struct CommandRecord {
        command_type: String,
        params: Option<serde_json::Value>,
    }

    // Clone for 'static requirement
    let id = device_id.to_string();

    let command: Option<CommandRecord> = state
        .db
        .query(
            r#"
            SELECT command_type, params 
            FROM device_command 
            WHERE device_id = $id AND executed = false
            ORDER BY created_at ASC
            LIMIT 1
        "#,
        )
        .bind(("id", id))
        .await
        .ok()?
        .take(0)
        .ok()?;

    command.map(|c| DeviceCommand {
        command_type: c.command_type,
        params: c.params,
    })
}
