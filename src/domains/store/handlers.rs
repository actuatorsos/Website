//! Store API Handlers — نقاط نهاية API للمتجر الإلكتروني

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};

use super::models::*;
use super::repository;
use super::service;
use crate::db::AppState;

// ============================================================================
// Query Params
// ============================================================================

#[derive(serde::Deserialize)]
pub struct OrderFilter {
    pub status: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CartQuery {
    pub session_id: String,
}

// ============================================================================
// Public Storefront (no auth required)
// ============================================================================

/// GET /api/store/products — Public product listing
async fn list_products(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<StorefrontItem>>, crate::db::DbError> {
    let items = repository::list_storefront_products(&state).await?;
    Ok(Json(items))
}

/// GET /api/store/services — Public service listing
async fn list_services(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<StorefrontItem>>, crate::db::DbError> {
    let items = repository::list_storefront_services(&state).await?;
    Ok(Json(items))
}

/// GET /api/store/categories — Public categories
async fn list_categories(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Vec<serde_json::Value>>, crate::db::DbError> {
    let cats = repository::list_storefront_categories(&state).await?;
    Ok(Json(cats))
}

/// GET /api/store/settings — Public store info
async fn get_settings_public(
    State(state): State<AppState>,
) -> axum::response::Result<Json<Option<StoreSettings>>, crate::db::DbError> {
    let settings = repository::get_settings(&state).await?;
    Ok(Json(settings))
}

// ============================================================================
// Shopping Cart
// ============================================================================

/// GET /api/store/cart?session_id=xxx — Get cart
async fn get_cart(
    State(state): State<AppState>,
    Query(q): Query<CartQuery>,
) -> axum::response::Result<Json<Vec<CartItem>>, crate::db::DbError> {
    let items = repository::get_cart(&state, &q.session_id).await?;
    Ok(Json(items))
}

/// POST /api/store/cart?session_id=xxx — Add item to cart
async fn add_to_cart(
    State(state): State<AppState>,
    Query(q): Query<CartQuery>,
    Json(req): Json<AddToCartRequest>,
) -> axum::response::Result<Json<CartItem>, crate::db::DbError> {
    // Resolve name and price from product or service
    let (name, price) = resolve_item_info(&state, &req)
        .await
        .map_err(|e| crate::db::DbError::Validation(e))?;

    let item = repository::add_to_cart(&state, &q.session_id, &req, &name, price).await?;
    Ok(Json(item))
}

/// DELETE /api/store/cart/:id — Remove item
async fn remove_from_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::remove_from_cart(&state, &id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Checkout
// ============================================================================

/// POST /api/store/checkout?session_id=xxx — Transform cart into order
async fn checkout(
    State(state): State<AppState>,
    Query(q): Query<CartQuery>,
    Json(req): Json<CheckoutRequest>,
) -> axum::response::Result<Json<StoreOrder>, crate::db::DbError> {
    let order = service::checkout(&state, &q.session_id, &req)
        .await
        .map_err(|e| crate::db::DbError::Validation(e))?;
    Ok(Json(order))
}

// ============================================================================
// Admin Order Management
// ============================================================================

/// GET /api/store/orders — List all orders (admin)
async fn list_orders(
    State(state): State<AppState>,
    Query(filter): Query<OrderFilter>,
) -> axum::response::Result<Json<Vec<StoreOrder>>, crate::db::DbError> {
    let orders = repository::list_orders(&state, filter.status.as_deref()).await?;
    Ok(Json(orders))
}

/// GET /api/store/orders/:id — Get order details
async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    let order = repository::get_order(&state, &id).await?;
    let items = repository::get_order_items(&state, &id).await?;
    let payments = repository::list_order_payments(&state, &id).await?;
    Ok(Json(serde_json::json!({
        "order": order,
        "items": items,
        "payments": payments,
    })))
}

/// PUT /api/store/orders/:id/status — Update order status
async fn update_order_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> axum::response::Result<Json<StoreOrder>, crate::db::DbError> {
    let order = repository::update_order_status(&state, &id, &req.status).await?;
    Ok(Json(order))
}

// ============================================================================
// Admin Settings
// ============================================================================

/// POST /api/store/admin/settings — Save store settings
async fn save_settings(
    State(state): State<AppState>,
    Json(req): Json<UpsertStoreSettingsRequest>,
) -> axum::response::Result<Json<StoreSettings>, crate::db::DbError> {
    let settings = repository::upsert_settings(&state, req).await?;
    Ok(Json(settings))
}

// ============================================================================
// Payments
// ============================================================================

/// POST /api/store/payments — Record a payment
async fn create_payment(
    State(state): State<AppState>,
    Json(req): Json<CreatePaymentRequest>,
) -> axum::response::Result<Json<Payment>, crate::db::DbError> {
    let payment = repository::create_payment(&state, &req).await?;
    Ok(Json(payment))
}

// ============================================================================
// Helpers
// ============================================================================

/// Resolve item name and price from product or service
async fn resolve_item_info(
    state: &AppState,
    req: &AddToCartRequest,
) -> Result<(String, f64), String> {
    match req.item_type.as_str() {
        "product" => {
            let pid = req
                .product_id
                .as_ref()
                .ok_or_else(|| "product_id required for product items".to_string())?;
            let product: Option<serde_json::Value> = state.db
                .query("SELECT name, sell_price FROM product WHERE id = type::thing('product', $id) LIMIT 1")
                .bind(("id", pid.to_string()))
                .await
                .map_err(|e| format!("DB error: {}", e))?
                .take(0)
                .map_err(|e| format!("Parse error: {}", e))?;
            let p = product.ok_or_else(|| "Product not found".to_string())?;
            let name = p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let price = p.get("sell_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok((name, price))
        }
        "service" => {
            let sid = req
                .service_id
                .as_ref()
                .ok_or_else(|| "service_id required for service items".to_string())?;
            let svc: Option<serde_json::Value> = state.db
                .query("SELECT name, base_price FROM service_catalog WHERE id = type::thing('service_catalog', $id) LIMIT 1")
                .bind(("id", sid.to_string()))
                .await
                .map_err(|e| format!("DB error: {}", e))?
                .take(0)
                .map_err(|e| format!("Parse error: {}", e))?;
            let s = svc.ok_or_else(|| "Service not found".to_string())?;
            let name = s
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let price = s.get("base_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok((name, price))
        }
        _ => Err("item_type must be 'product' or 'service'".to_string()),
    }
}

// ============================================================================
// Routes
// ============================================================================

/// Public storefront routes (no auth)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products))
        .route("/services", get(list_services))
        .route("/categories", get(list_categories))
        .route("/info", get(get_settings_public))
        .route("/cart", get(get_cart).post(add_to_cart))
        .route("/cart/{id}", delete(remove_from_cart))
        .route("/checkout", post(checkout))
}

/// Admin store management routes
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/orders", get(list_orders))
        .route("/orders/{id}", get(get_order))
        .route("/orders/{id}/status", put(update_order_status))
        .route("/settings", get(get_settings_public).post(save_settings))
        .route("/payments", post(create_payment))
}
