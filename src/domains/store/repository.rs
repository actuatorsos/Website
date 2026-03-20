//! Store Repository — عمليات قاعدة البيانات للمتجر

use super::models::*;
use crate::db::{AppState, DbError};

// ============================================================================
// Store Settings
// ============================================================================

pub async fn get_settings(state: &AppState) -> Result<Option<StoreSettings>, DbError> {
    let settings: Option<StoreSettings> = state
        .db
        .query("SELECT * FROM store_settings LIMIT 1")
        .await?
        .take(0)?;
    Ok(settings)
}

pub async fn upsert_settings(
    state: &AppState,
    req: UpsertStoreSettingsRequest,
) -> Result<StoreSettings, DbError> {
    let currency = req.currency.unwrap_or_else(|| "SAR".to_string());
    let is_active = req.is_active.unwrap_or(true);
    let settings: Option<StoreSettings> = state.db
        .query("UPSERT store_settings SET store_name = $sn, store_name_en = $sne, description = $desc, logo_url = $logo, currency = $cur, tax_rate = $tax, shipping_flat = $ship, free_shipping_above = $free, is_active = $active, contact_email = $cemail, contact_phone = $cphone, updated_at = time::now()")
        .bind(("sn", req.store_name))
        .bind(("sne", req.store_name_en))
        .bind(("desc", req.description))
        .bind(("logo", req.logo_url))
        .bind(("cur", currency))
        .bind(("tax", req.tax_rate))
        .bind(("ship", req.shipping_flat))
        .bind(("free", req.free_shipping_above))
        .bind(("active", is_active))
        .bind(("cemail", req.contact_email))
        .bind(("cphone", req.contact_phone))
        .await?.take(0)?;
    settings.ok_or(DbError::NotFound)
}

// ============================================================================
// Storefront (Public Product/Service Listing)
// ============================================================================

pub async fn list_storefront_products(state: &AppState) -> Result<Vec<StorefrontItem>, DbError> {
    let items: Vec<StorefrontItem> = state.db
        .query("SELECT id, name, description, sell_price AS price, currency, image_url, 'product' AS item_type, true AS in_stock FROM product WHERE is_sellable = true AND is_active = true AND (is_archived = false OR is_archived = NONE) ORDER BY name ASC")
        .await?.take(0)?;
    Ok(items)
}

pub async fn list_storefront_services(state: &AppState) -> Result<Vec<StorefrontItem>, DbError> {
    let items: Vec<StorefrontItem> = state.db
        .query("SELECT id, name, description, base_price AS price, currency, NONE AS image_url, 'service' AS item_type, true AS in_stock FROM service_catalog WHERE is_active = true AND (is_archived = false OR is_archived = NONE) ORDER BY name ASC")
        .await?.take(0)?;
    Ok(items)
}

pub async fn list_storefront_categories(
    state: &AppState,
) -> Result<Vec<serde_json::Value>, DbError> {
    let cats: Vec<serde_json::Value> = state.db
        .query("SELECT id, code, name, image_url FROM product_category WHERE is_active = true AND (is_archived = false OR is_archived = NONE) ORDER BY name ASC")
        .await?.take(0)?;
    Ok(cats)
}

// ============================================================================
// Shopping Cart
// ============================================================================

pub async fn get_cart(state: &AppState, session_id: &str) -> Result<Vec<CartItem>, DbError> {
    let items: Vec<CartItem> = state
        .db
        .query("SELECT * FROM shopping_cart WHERE session_id = $sid ORDER BY added_at ASC")
        .bind(("sid", session_id.to_string()))
        .await?
        .take(0)?;
    Ok(items)
}

pub async fn add_to_cart(
    state: &AppState,
    session_id: &str,
    item: &AddToCartRequest,
    name: &str,
    price: f64,
) -> Result<CartItem, DbError> {
    let qty = item.quantity.unwrap_or(1);
    let cart_item: Option<CartItem> = state.db
        .query("CREATE shopping_cart SET session_id = $sid, item_type = $itype, product = $pid, service = $svc, name = $name, quantity = $qty, unit_price = $price")
        .bind(("sid", session_id.to_string()))
        .bind(("itype", item.item_type.clone()))
        .bind(("pid", item.product_id.as_ref().map(|id| format!("product:{}", id))))
        .bind(("svc", item.service_id.as_ref().map(|id| format!("service_catalog:{}", id))))
        .bind(("name", name.to_string()))
        .bind(("qty", qty))
        .bind(("price", price))
        .await?.take(0)?;
    cart_item.ok_or(DbError::NotFound)
}

pub async fn remove_from_cart(state: &AppState, id: &str) -> Result<(), DbError> {
    let _: Option<CartItem> = state.db.delete(("shopping_cart", id)).await?;
    Ok(())
}

pub async fn clear_cart(state: &AppState, session_id: &str) -> Result<(), DbError> {
    state
        .db
        .query("DELETE FROM shopping_cart WHERE session_id = $sid")
        .bind(("sid", session_id.to_string()))
        .await?;
    Ok(())
}

// ============================================================================
// Orders
// ============================================================================

pub async fn create_order(
    state: &AppState,
    order_number: &str,
    req: &CheckoutRequest,
    subtotal: f64,
    tax: f64,
    shipping: f64,
    total: f64,
) -> Result<StoreOrder, DbError> {
    let order: Option<StoreOrder> = state.db
        .query("CREATE store_order SET order_number = $num, customer_name = $cname, customer_email = $cemail, customer_phone = $cphone, subtotal = $sub, tax_amount = $tax, shipping_cost = $ship, total = $total, shipping_address = $addr, notes = $notes")
        .bind(("num", order_number.to_string()))
        .bind(("cname", req.customer_name.clone()))
        .bind(("cemail", req.customer_email.clone()))
        .bind(("cphone", req.customer_phone.clone()))
        .bind(("sub", subtotal))
        .bind(("tax", tax))
        .bind(("ship", shipping))
        .bind(("total", total))
        .bind(("addr", req.shipping_address.clone()))
        .bind(("notes", req.notes.clone()))
        .await?.take(0)?;
    order.ok_or(DbError::NotFound)
}

pub async fn create_order_item(
    state: &AppState,
    order_id: &str,
    item: &CartItem,
) -> Result<OrderItem, DbError> {
    let total = item.unit_price * item.quantity as f64;
    let oi: Option<OrderItem> = state.db
        .query("CREATE order_item SET order = type::thing('store_order', $oid), item_type = $itype, name = $name, quantity = $qty, unit_price = $up, total_price = $tp")
        .bind(("oid", order_id.to_string()))
        .bind(("itype", item.item_type.clone()))
        .bind(("name", item.name.clone()))
        .bind(("qty", item.quantity))
        .bind(("up", item.unit_price))
        .bind(("tp", total))
        .await?.take(0)?;
    oi.ok_or(DbError::NotFound)
}

pub async fn list_orders(
    state: &AppState,
    status: Option<&str>,
) -> Result<Vec<StoreOrder>, DbError> {
    let orders: Vec<StoreOrder> = match status {
        Some(s) => state
            .db
            .query("SELECT * FROM store_order WHERE status = $s ORDER BY created_at DESC LIMIT 100")
            .bind(("s", s.to_string()))
            .await?
            .take(0)?,
        None => state
            .db
            .query("SELECT * FROM store_order ORDER BY created_at DESC LIMIT 100")
            .await?
            .take(0)?,
    };
    Ok(orders)
}

pub async fn get_order(state: &AppState, id: &str) -> Result<StoreOrder, DbError> {
    let order: Option<StoreOrder> = state.db.select(("store_order", id)).await?;
    order.ok_or(DbError::NotFound)
}

pub async fn get_order_items(state: &AppState, order_id: &str) -> Result<Vec<OrderItem>, DbError> {
    let items: Vec<OrderItem> = state
        .db
        .query("SELECT * FROM order_item WHERE order = type::thing('store_order', $oid)")
        .bind(("oid", order_id.to_string()))
        .await?
        .take(0)?;
    Ok(items)
}

pub async fn update_order_status(
    state: &AppState,
    id: &str,
    status: &str,
) -> Result<StoreOrder, DbError> {
    let order: Option<StoreOrder> = state.db
        .query("UPDATE type::thing('store_order', $id) SET status = $status, updated_at = time::now() RETURN AFTER")
        .bind(("id", id.to_string()))
        .bind(("status", status.to_string()))
        .await?.take(0)?;
    order.ok_or(DbError::NotFound)
}

// ============================================================================
// Payments
// ============================================================================

pub async fn create_payment(
    state: &AppState,
    req: &CreatePaymentRequest,
) -> Result<Payment, DbError> {
    let payment: Option<Payment> = state.db
        .query("CREATE payment SET order = type::thing('store_order', $oid), amount = $amt, method = $method, external_ref = $ref, notes = $notes")
        .bind(("oid", req.order_id.clone()))
        .bind(("amt", req.amount))
        .bind(("method", req.method.clone()))
        .bind(("ref", req.external_ref.clone()))
        .bind(("notes", req.notes.clone()))
        .await?.take(0)?;
    payment.ok_or(DbError::NotFound)
}

pub async fn list_order_payments(
    state: &AppState,
    order_id: &str,
) -> Result<Vec<Payment>, DbError> {
    let payments: Vec<Payment> = state.db
        .query("SELECT * FROM payment WHERE order = type::thing('store_order', $oid) ORDER BY created_at DESC")
        .bind(("oid", order_id.to_string()))
        .await?.take(0)?;
    Ok(payments)
}
