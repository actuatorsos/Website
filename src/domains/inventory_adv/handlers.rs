use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post},
};

async fn list_warehouses(State(s): State<AppState>) -> Result<Json<Vec<Warehouse>>, DbError> {
    Ok(Json(repo::get_all_warehouses(&s).await?))
}
async fn create_warehouse(
    State(s): State<AppState>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Warehouse>, DbError> {
    Ok(Json(repo::create_warehouse(&s, req).await?))
}
async fn list_inventory(State(s): State<AppState>) -> Result<Json<Vec<InventoryItem>>, DbError> {
    Ok(Json(repo::get_all_inventory(&s).await?))
}
async fn create_inventory_item(
    State(s): State<AppState>,
    Json(req): Json<CreateInventoryItemRequest>,
) -> Result<Json<InventoryItem>, DbError> {
    Ok(Json(repo::create_inventory_item(&s, req).await?))
}
async fn low_stock(State(s): State<AppState>) -> Result<Json<Vec<InventoryItem>>, DbError> {
    Ok(Json(repo::get_low_stock(&s).await?))
}
async fn inventory_in(
    State(s): State<AppState>,
    Json(req): Json<InventoryInRequest>,
) -> Result<Json<serde_json::Value>, DbError> {
    let (item, txn) = repo::inventory_in(&s, req).await?;
    Ok(Json(
        serde_json::json!({ "item": item, "transaction": txn }),
    ))
}
async fn inventory_out(
    State(s): State<AppState>,
    Json(req): Json<InventoryOutRequest>,
) -> Result<Json<serde_json::Value>, DbError> {
    let (item, txn) = repo::inventory_out(&s, req).await?;
    Ok(Json(
        serde_json::json!({ "item": item, "transaction": txn }),
    ))
}
async fn inventory_adjust(
    State(s): State<AppState>,
    Json(req): Json<InventoryAdjustRequest>,
) -> Result<Json<InventoryItem>, DbError> {
    Ok(Json(repo::inventory_adjust(&s, req).await?))
}
async fn item_transactions(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<InventoryTransaction>>, DbError> {
    Ok(Json(repo::get_item_transactions(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/warehouses", get(list_warehouses).post(create_warehouse))
        .route(
            "/inventory",
            get(list_inventory).post(create_inventory_item),
        )
        .route("/inventory/low-stock", get(low_stock))
        .route("/inventory/in", post(inventory_in))
        .route("/inventory/out", post(inventory_out))
        .route("/inventory/adjust", post(inventory_adjust))
        .route("/inventory/transactions/{item_id}", get(item_transactions))
}
