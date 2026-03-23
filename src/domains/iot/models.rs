//! IoT Models — نماذج إنترنت الأشياء

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTDevice {
    pub id: Option<Thing>,
    pub name: String,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
    pub api_key: Option<String>,
    pub last_seen: Option<String>,
    pub machine: Option<Thing>,
    pub metadata: Option<String>,
    pub created_at: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub machine_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDeviceStatusRequest {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub id: Option<Thing>,
    pub device: Option<Thing>,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestReadingRequest {
    pub device_id: Option<String>,
    pub api_key: Option<String>,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
}

/// Response returned when a device is created (includes the api_key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeviceResponse {
    pub device: IoTDevice,
    pub api_key: String,
}
