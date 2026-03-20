use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post, put},
};

async fn list_boms(State(s): State<AppState>) -> Result<Json<Vec<Bom>>, DbError> {
    Ok(Json(repo::get_all_boms(&s).await?))
}
async fn create_bom(
    State(s): State<AppState>,
    Json(req): Json<CreateBomRequest>,
) -> Result<Json<Bom>, DbError> {
    Ok(Json(repo::create_bom(&s, req).await?))
}
async fn get_bom(State(s): State<AppState>, Path(id): Path<String>) -> Result<Json<Bom>, DbError> {
    Ok(Json(repo::get_bom(&s, &id).await?))
}
async fn add_bom_line(
    State(s): State<AppState>,
    Json(req): Json<CreateBomLineRequest>,
) -> Result<Json<BomLine>, DbError> {
    Ok(Json(repo::add_bom_line(&s, req).await?))
}
async fn get_bom_lines(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<BomLine>>, DbError> {
    Ok(Json(repo::get_bom_lines(&s, &id).await?))
}
async fn list_orders(State(s): State<AppState>) -> Result<Json<Vec<ProductionOrder>>, DbError> {
    Ok(Json(repo::get_all_production_orders(&s).await?))
}
async fn create_order(
    State(s): State<AppState>,
    Json(req): Json<CreateProductionOrderRequest>,
) -> Result<Json<ProductionOrder>, DbError> {
    Ok(Json(repo::create_production_order(&s, req).await?))
}
async fn get_order(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProductionOrder>, DbError> {
    Ok(Json(repo::get_production_order(&s, &id).await?))
}
async fn update_order_status(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductionStatusRequest>,
) -> Result<Json<ProductionOrder>, DbError> {
    Ok(Json(repo::update_production_status(&s, &id, req).await?))
}
async fn consume_materials(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, DbError> {
    Ok(Json(repo::consume_materials(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bom", get(list_boms).post(create_bom))
        .route("/bom/{id}", get(get_bom))
        .route("/bom/{id}/lines", get(get_bom_lines))
        .route("/bom/lines", post(add_bom_line))
        .route("/production-orders", get(list_orders).post(create_order))
        .route("/production-orders/{id}", get(get_order))
        .route("/production-orders/{id}/status", put(update_order_status))
        .route("/production-orders/{id}/consume", post(consume_materials))
}
