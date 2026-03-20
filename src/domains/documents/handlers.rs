use super::models::{DocumentResponse, UpdateDocumentRequest};
use super::repository;
use crate::db::{AppState, DbError};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn get_document_handler(
    State(state): State<AppState>,
    Path((entity_type, id)): Path<(String, String)>,
) -> Result<Json<DocumentResponse>, DbError> {
    // Strip entity_type from ID if it's there (e.g., 'machine:' being passed)
    let pure_id = id.replace(&format!("{}:", entity_type), "");

    let doc = repository::get_document_profile(&state, &entity_type, &pure_id).await?;
    Ok(Json(doc))
}

pub async fn update_document_handler(
    State(state): State<AppState>,
    Path((entity_type, id)): Path<(String, String)>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> Result<Json<serde_json::Value>, DbError> {
    let pure_id = id.replace(&format!("{}:", entity_type), "");
    repository::update_document(&state, &entity_type, &pure_id, payload).await?;
    Ok(Json(serde_json::json!({"status": "success"})))
}

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route(
        "/{type}/{id}",
        axum::routing::get(get_document_handler).post(update_document_handler),
    )
}
