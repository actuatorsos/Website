use super::models::*;
use super::repository as repo;
use crate::db::{self, AppState, DbError};
use crate::models::CurrentUser;
use axum::{
    Router,
    extract::{Extension, Path, State},
    response::Json,
    routing::{get, post},
};

async fn list_warehouses(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<Warehouse>>, DbError> {
    Ok(Json(repo::get_all_warehouses(&s, user.organization_id.as_deref()).await?))
}
async fn create_warehouse(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<Warehouse>, DbError> {
    let wh = repo::create_warehouse(&s, req).await?;
    let _ = db::audit_log(
        &s.db,
        Some(&user.email),
        "create",
        "warehouse",
        wh.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await;
    Ok(Json(wh))
}
async fn list_inventory(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<InventoryItem>>, DbError> {
    Ok(Json(repo::get_all_inventory(&s, user.organization_id.as_deref()).await?))
}
async fn create_inventory_item(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateInventoryItemRequest>,
) -> Result<Json<InventoryItem>, DbError> {
    let item = repo::create_inventory_item(&s, req).await?;
    let _ = db::audit_log(
        &s.db,
        Some(&user.email),
        "create",
        "inventory_item",
        item.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await;
    Ok(Json(item))
}
async fn low_stock(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<InventoryItem>>, DbError> {
    Ok(Json(repo::get_low_stock(&s, user.organization_id.as_deref()).await?))
}
async fn inventory_in(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<InventoryInRequest>,
) -> Result<Json<serde_json::Value>, DbError> {
    let item_id = req.item_id.clone();
    let (item, txn) = repo::inventory_in(&s, req).await?;
    let _ = db::audit_log(
        &s.db,
        Some(&user.email),
        "update",
        "inventory_item",
        Some(&item_id),
        None,
        Some(serde_json::json!({ "action": "inventory_in", "quantity": item.quantity })),
    )
    .await;
    Ok(Json(
        serde_json::json!({ "item": item, "transaction": txn }),
    ))
}
async fn inventory_out(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<InventoryOutRequest>,
) -> Result<Json<serde_json::Value>, DbError> {
    let item_id = req.item_id.clone();
    let (item, txn) = repo::inventory_out(&s, req).await?;
    let _ = db::audit_log(
        &s.db,
        Some(&user.email),
        "update",
        "inventory_item",
        Some(&item_id),
        None,
        Some(serde_json::json!({ "action": "inventory_out", "quantity": item.quantity })),
    )
    .await;
    Ok(Json(
        serde_json::json!({ "item": item, "transaction": txn }),
    ))
}
async fn inventory_adjust(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<InventoryAdjustRequest>,
) -> Result<Json<InventoryItem>, DbError> {
    let item_id = req.item_id.clone();
    let new_qty = req.new_quantity;
    let item = repo::inventory_adjust(&s, req).await?;
    let _ = db::audit_log(
        &s.db,
        Some(&user.email),
        "update",
        "inventory_item",
        Some(&item_id),
        None,
        Some(serde_json::json!({ "action": "inventory_adjust", "new_quantity": new_qty })),
    )
    .await;
    Ok(Json(item))
}
async fn item_transactions(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<InventoryTransaction>>, DbError> {
    Ok(Json(repo::get_item_transactions(&s, &id).await?))
}
async fn all_transactions(
    State(s): State<AppState>,
) -> Result<Json<Vec<InventoryTransaction>>, DbError> {
    Ok(Json(repo::get_all_transactions(&s).await?))
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
        .route("/inventory/transactions", get(all_transactions))
        .route("/inventory/transactions/{item_id}", get(item_transactions))
}
