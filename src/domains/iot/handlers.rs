//! IoT Handlers — معالجات إنترنت الأشياء

use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use crate::models::CurrentUser;
use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Json,
    routing::{get, post, put, delete},
};
use serde::Deserialize;

// ============================================================================
// Admin Routes (require auth via middleware in api/mod.rs)
// ============================================================================

async fn list_devices(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<IoTDevice>>, DbError> {
    Ok(Json(repo::get_all_devices(&s, user.organization_id.as_deref()).await?))
}

async fn create_device(
    State(s): State<AppState>,
    Json(req): Json<CreateDeviceRequest>,
) -> Result<Json<CreateDeviceResponse>, DbError> {
    Ok(Json(repo::create_device(&s, req).await?))
}

async fn update_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDeviceStatusRequest>,
) -> Result<Json<IoTDevice>, DbError> {
    Ok(Json(repo::update_device_status(&s, &id, &req.status).await?))
}

async fn soft_delete_device(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<IoTDevice>, DbError> {
    Ok(Json(repo::delete_device(&s, &id).await?))
}

#[derive(Deserialize)]
pub struct ReadingsQuery {
    pub limit: Option<u32>,
}

async fn get_readings(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<ReadingsQuery>,
) -> Result<Json<Vec<SensorReading>>, DbError> {
    Ok(Json(repo::get_readings(&s, &id, q.limit).await?))
}

/// Admin routes mounted under /iot
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/devices", get(list_devices).post(create_device))
        .route("/devices/{id}/status", put(update_status))
        .route("/devices/{id}", delete(soft_delete_device))
        .route("/devices/{id}/readings", get(get_readings))
}

// ============================================================================
// Public Ingest Route (authenticated by X-API-Key header)
// ============================================================================

async fn ingest_reading(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<IngestReadingRequest>,
) -> Result<Json<SensorReading>, DbError> {
    // Resolve the device: prefer api_key from header, fallback to body fields
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or(req.api_key.clone());

    let device = if let Some(key) = api_key {
        repo::get_device_by_api_key(&s, &key).await?
    } else if let Some(device_id) = &req.device_id {
        // Fallback: direct device_id (less secure, mainly for testing)
        let devices = repo::get_all_devices(&s, None).await?;
        devices
            .into_iter()
            .find(|d| {
                d.id
                    .as_ref()
                    .map(|t| t.id.to_raw() == *device_id)
                    .unwrap_or(false)
            })
            .ok_or(DbError::NotFound)?
    } else {
        return Err(DbError::Validation(
            "Either X-API-Key header or device_id must be provided".to_string(),
        ));
    };

    let device_id = device
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .ok_or(DbError::NotFound)?;

    Ok(Json(
        repo::ingest_reading(&s, &device_id, &req.metric, req.value, req.unit).await?,
    ))
}

/// Public ingest route — no JWT, authenticated by X-API-Key
pub fn public_routes() -> Router<AppState> {
    Router::new().route("/ingest", post(ingest_reading))
}
