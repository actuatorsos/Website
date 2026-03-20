//! Inventory Advanced Models — نماذج المخزون

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: Option<Thing>,
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub manager: Option<Thing>,
    pub is_active: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub address: Option<String>,
    pub manager_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: Option<Thing>,
    pub sku: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub product: Option<Thing>,
    pub warehouse: Option<Thing>,
    pub vendor: Option<Thing>,
    pub unit: Option<String>,
    pub quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub max_quantity: Option<i64>,
    pub unit_cost: Option<f64>,
    pub total_value: Option<f64>,
    pub location_code: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInventoryItemRequest {
    pub name: String,
    pub sku: Option<String>,
    pub description: Option<String>,
    pub product_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub vendor_id: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub unit_cost: Option<f64>,
    pub location_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: Option<Thing>,
    pub item: Thing,
    pub txn_type: String, // in, out, adjust, transfer, production_consume
    pub quantity: i64,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub reference: Option<String>,
    pub reference_type: Option<String>, // purchase_order, production_order, repair, manual
    pub production_order: Option<Thing>,
    pub notes: Option<String>,
    pub created_by: Option<Thing>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryInRequest {
    pub item_id: String,
    pub quantity: i64,
    pub unit_cost: Option<f64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryOutRequest {
    pub item_id: String,
    pub quantity: i64,
    pub reference: Option<String>,
    pub reference_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryAdjustRequest {
    pub item_id: String,
    pub new_quantity: i64,
    pub notes: Option<String>,
}
