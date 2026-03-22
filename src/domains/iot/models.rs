//! IoT Models — نماذج إنترنت الأشياء

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct IoTDevice {
    pub id: Option<RecordId>,
    pub name: String,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub status: Option<String>,
    pub api_key: Option<String>,
    pub last_seen: Option<String>,
    pub machine: Option<RecordId>,
    pub metadata: Option<String>,
    pub created_at: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub device_type: Option<String>,
    pub serial_number: Option<String>,
    pub location: Option<String>,
    pub machine_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateDeviceStatusRequest {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct SensorReading {
    pub id: Option<RecordId>,
    pub device: Option<RecordId>,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct IngestReadingRequest {
    pub device_id: Option<String>,
    pub api_key: Option<String>,
    pub metric: String,
    pub value: f64,
    pub unit: Option<String>,
}

/// Response returned when a device is created (includes the api_key)
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateDeviceResponse {
    pub device: IoTDevice,
    pub api_key: String,
}
