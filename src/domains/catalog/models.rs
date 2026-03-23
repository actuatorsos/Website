//! Product & Service Catalog Models

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ══════════════════════════════════════════════════════════════════
// ProductCategory
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: Option<Thing>,
    pub code: String,
    pub name: String,
    pub parent: Option<Thing>,
    pub image_url: Option<String>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductCategoryRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<String>,
}

// ══════════════════════════════════════════════════════════════════
// Product
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Option<Thing>,
    pub sku: String,
    pub name: String,
    pub category: Option<Thing>,
    pub product_type: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub sell_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub currency: Option<String>,
    pub tax_rate: Option<f64>,
    pub weight_kg: Option<f64>,
    pub dimensions: Option<String>,
    pub image_url: Option<String>,
    pub vendor: Option<Thing>,
    pub warranty_months: Option<i64>,
    pub min_stock: Option<i64>,
    pub has_bom: Option<bool>,
    pub active_bom: Option<Thing>,
    pub is_sellable: Option<bool>,
    pub is_purchasable: Option<bool>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub name: String,
    pub category_id: Option<String>,
    pub product_type: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub sell_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub tax_rate: Option<f64>,
    pub vendor_id: Option<String>,
    pub warranty_months: Option<i64>,
    pub min_stock: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub sell_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub min_stock: Option<i64>,
    pub is_active: Option<bool>,
}

// ══════════════════════════════════════════════════════════════════
// ServiceCatalog
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCatalog {
    pub id: Option<Thing>,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub base_price: Option<f64>,
    pub price_type: Option<String>,
    pub estimated_hours: Option<f64>,
    pub currency: Option<String>,
    pub tax_rate: Option<f64>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceCatalogRequest {
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub base_price: Option<f64>,
    pub price_type: Option<String>,
    pub estimated_hours: Option<f64>,
    pub tax_rate: Option<f64>,
}
