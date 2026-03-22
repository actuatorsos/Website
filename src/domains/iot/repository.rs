//! IoT Repository — عمليات إنترنت الأشياء في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};
use uuid::Uuid;

/// Generate a random API key for device authentication
fn generate_api_key() -> String {
    format!("iot_{}", Uuid::new_v4().to_string().replace('-', ""))
}

/// Get all non-archived IoT devices
pub async fn get_all_devices(state: &AppState, org_id: Option<&str>) -> Result<Vec<IoTDevice>, DbError> {
    let devices: Vec<IoTDevice> = if let Some(org) = org_id {
        state
            .db
            .query("SELECT * FROM iot_device WHERE (is_archived = false OR is_archived = NONE) AND organization = type::record($org) ORDER BY created_at DESC")
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query("SELECT * FROM iot_device WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
            .await?
            .take(0)?
    };
    Ok(devices)
}

/// Create a new IoT device with a generated API key
pub async fn create_device(
    state: &AppState,
    req: CreateDeviceRequest,
) -> Result<CreateDeviceResponse, DbError> {
    let api_key = generate_api_key();
    let name = req.name;
    let device_type = req.device_type.unwrap_or_else(|| "sensor".to_string());
    let serial_number = req.serial_number;
    let location = req.location;
    let machine_id = req.machine_id;

    let query = if machine_id.is_some() {
        "CREATE iot_device SET \
         name = $name, device_type = $device_type, serial_number = $serial_number, \
         location = $location, api_key = $api_key, status = 'active', \
         machine = type::thing('machine', $machine_id)"
    } else {
        "CREATE iot_device SET \
         name = $name, device_type = $device_type, serial_number = $serial_number, \
         location = $location, api_key = $api_key, status = 'active', \
         machine = NONE"
    };

    let device: Option<IoTDevice> = state
        .db
        .query(query)
        .bind(("name", name))
        .bind(("device_type", device_type))
        .bind(("serial_number", serial_number))
        .bind(("location", location))
        .bind(("api_key", api_key.clone()))
        .bind(("machine_id", machine_id))
        .await?
        .take(0)?;

    let device = device.ok_or(DbError::NotFound)?;
    Ok(CreateDeviceResponse { device, api_key })
}

/// Update device status (active, inactive, maintenance, offline)
pub async fn update_device_status(
    state: &AppState,
    id: &str,
    status: &str,
) -> Result<IoTDevice, DbError> {
    let valid = ["active", "inactive", "maintenance", "offline"];
    if !valid.contains(&status) {
        return Err(DbError::Validation(format!(
            "Invalid status '{}'. Must be one of: {:?}",
            status, valid
        )));
    }

    let device: Option<IoTDevice> = state
        .db
        .query("UPDATE type::thing('iot_device', $id) SET status = $status")
        .bind(("id", id.to_string()))
        .bind(("status", status.to_string()))
        .await?
        .take(0)?;
    device.ok_or(DbError::NotFound)
}

/// Soft-delete a device (set is_archived = true)
pub async fn delete_device(state: &AppState, id: &str) -> Result<IoTDevice, DbError> {
    let device: Option<IoTDevice> = state
        .db
        .query("UPDATE type::thing('iot_device', $id) SET is_archived = true")
        .bind(("id", id.to_string()))
        .await?
        .take(0)?;
    device.ok_or(DbError::NotFound)
}

/// Ingest a sensor reading and update last_seen on the device
pub async fn ingest_reading(
    state: &AppState,
    device_id: &str,
    metric: &str,
    value: f64,
    unit: Option<String>,
) -> Result<SensorReading, DbError> {
    // Update device last_seen
    let _: Option<IoTDevice> = state
        .db
        .query("UPDATE type::thing('iot_device', $id) SET last_seen = time::now()")
        .bind(("id", device_id.to_string()))
        .await?
        .take(0)?;

    // Create reading
    let reading: Option<SensorReading> = state
        .db
        .query(
            "CREATE sensor_reading SET \
             device = type::thing('iot_device', $device_id), \
             metric = $metric, value = $value, unit = $unit",
        )
        .bind(("device_id", device_id.to_string()))
        .bind(("metric", metric.to_string()))
        .bind(("value", value))
        .bind(("unit", unit))
        .await?
        .take(0)?;
    reading.ok_or(DbError::NotFound)
}

/// Get recent readings for a device (default limit 100)
pub async fn get_readings(
    state: &AppState,
    device_id: &str,
    limit: Option<u32>,
) -> Result<Vec<SensorReading>, DbError> {
    let lim = limit.unwrap_or(100) as i64;
    let readings: Vec<SensorReading> = state
        .db
        .query(
            "SELECT * FROM sensor_reading \
             WHERE device = type::thing('iot_device', $device_id) \
             ORDER BY timestamp DESC LIMIT $lim",
        )
        .bind(("device_id", device_id.to_string()))
        .bind(("lim", lim))
        .await?
        .take(0)?;
    Ok(readings)
}

/// Find a device by its API key (for device authentication)
pub async fn get_device_by_api_key(
    state: &AppState,
    api_key: &str,
) -> Result<IoTDevice, DbError> {
    let devices: Vec<IoTDevice> = state
        .db
        .query("SELECT * FROM iot_device WHERE api_key = $api_key AND is_archived = false LIMIT 1")
        .bind(("api_key", api_key.to_string()))
        .await?
        .take(0)?;
    devices.into_iter().next().ok_or(DbError::NotFound)
}
