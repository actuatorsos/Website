//! Store Service — منطق عمليات المتجر
//!
//! حساب المجموع، إنشاء الطلبات، والتكامل مع المخزون

use super::models::*;
use super::repository;
use crate::db::AppState;

/// Generate a unique order number
pub fn generate_order_number() -> String {
    let now = chrono::Utc::now();
    let random: u32 = rand::random::<u16>() as u32;
    format!("DRM-{}-{:04}", now.format("%Y%m%d%H%M"), random)
}

/// Process checkout: cart → order + order_items + clear cart
pub async fn checkout(
    state: &AppState,
    session_id: &str,
    req: &CheckoutRequest,
) -> Result<StoreOrder, String> {
    // 1. Get cart items
    let cart_items = repository::get_cart(state, session_id)
        .await
        .map_err(|e| format!("Failed to get cart: {}", e))?;

    if cart_items.is_empty() {
        return Err("Shopping cart is empty".to_string());
    }

    // 2. Calculate totals
    let subtotal: f64 = cart_items
        .iter()
        .map(|item| item.unit_price * item.quantity as f64)
        .sum();

    // Get store settings for tax
    let settings = repository::get_settings(state)
        .await
        .map_err(|e| format!("Settings error: {}", e))?;

    let tax_rate = settings.as_ref().and_then(|s| s.tax_rate).unwrap_or(0.15); // default 15% VAT

    let tax_amount = subtotal * tax_rate;

    let shipping = settings
        .as_ref()
        .and_then(|s| {
            let free_above = s.free_shipping_above.unwrap_or(f64::MAX);
            if subtotal >= free_above {
                Some(0.0)
            } else {
                s.shipping_flat
            }
        })
        .unwrap_or(0.0);

    let total = subtotal + tax_amount + shipping;

    // 3. Create order
    let order_number = generate_order_number();
    let order = repository::create_order(
        state,
        &order_number,
        req,
        subtotal,
        tax_amount,
        shipping,
        total,
    )
    .await
    .map_err(|e| format!("Failed to create order: {}", e))?;

    // 4. Create order items
    let order_id = order
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .ok_or_else(|| "Order has no ID".to_string())?;

    for cart_item in &cart_items {
        let _ = repository::create_order_item(state, &order_id, cart_item).await;
    }

    // 5. Clear cart
    let _ = repository::clear_cart(state, session_id).await;

    Ok(order)
}
