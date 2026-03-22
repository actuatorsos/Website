//! Store Models — نماذج المتجر الإلكتروني

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

// ============================================================================
// Store Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct StoreSettings {
    pub id: Option<RecordId>,
    pub store_name: String,
    pub store_name_en: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub currency: Option<String>,
    pub tax_rate: Option<f64>,
    pub shipping_flat: Option<f64>,
    pub free_shipping_above: Option<f64>,
    #[serde(default)]
    pub is_active: bool,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpsertStoreSettingsRequest {
    pub store_name: String,
    pub store_name_en: Option<String>,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub currency: Option<String>,
    pub tax_rate: Option<f64>,
    pub shipping_flat: Option<f64>,
    pub free_shipping_above: Option<f64>,
    pub is_active: Option<bool>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
}

// ============================================================================
// Store Order
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct StoreOrder {
    pub id: Option<RecordId>,
    pub order_number: String,
    pub customer: Option<RecordId>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub status: String,
    pub subtotal: f64,
    pub tax_amount: f64,
    pub shipping_cost: f64,
    pub discount: f64,
    pub total: f64,
    pub currency: Option<String>,
    pub shipping_address: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
    pub notes: Option<String>,
}

// ============================================================================
// Order Item
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct OrderItem {
    pub id: Option<RecordId>,
    pub order: Option<RecordId>,
    pub item_type: String,
    pub product: Option<RecordId>,
    pub service: Option<RecordId>,
    pub name: String,
    pub sku: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
    pub notes: Option<String>,
}

// ============================================================================
// Shopping Cart
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CartItem {
    pub id: Option<RecordId>,
    pub owner: Option<RecordId>,
    pub session_id: Option<String>,
    pub item_type: String,
    pub product: Option<RecordId>,
    pub service: Option<RecordId>,
    pub name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub added_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct AddToCartRequest {
    pub item_type: String, // "product" or "service"
    pub product_id: Option<String>,
    pub service_id: Option<String>,
    pub quantity: Option<i32>,
}

// ============================================================================
// Payment
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Payment {
    pub id: Option<RecordId>,
    pub order: Option<RecordId>,
    pub amount: f64,
    pub currency: Option<String>,
    pub method: String,
    pub status: String,
    pub external_ref: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreatePaymentRequest {
    pub order_id: String,
    pub amount: f64,
    pub method: String,
    pub external_ref: Option<String>,
    pub notes: Option<String>,
}

// ============================================================================
// Checkout Request (from cart → order)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CheckoutRequest {
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub shipping_address: Option<serde_json::Value>,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
}

/// Public storefront product/service listing item
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct StorefrontItem {
    pub id: Option<RecordId>,
    pub name: String,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
    pub item_type: String, // "product" or "service"
    pub in_stock: Option<bool>,
}
