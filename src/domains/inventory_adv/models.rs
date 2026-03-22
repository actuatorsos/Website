//! Inventory Advanced Models — نماذج المخزون

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Warehouse {
    pub id: Option<RecordId>,
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub manager: Option<RecordId>,
    pub is_active: Option<bool>,
    pub capacity: Option<i64>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateWarehouseRequest {
    pub name: String,
    pub code: String,
    pub address: Option<String>,
    pub manager_id: Option<String>,
    pub capacity: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct InventoryItem {
    pub id: Option<RecordId>,
    pub sku: Option<String>,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub unit_cost: Option<f64>,
    pub currency: Option<String>,
    pub location: Option<String>,
    pub supplier: Option<String>,
    pub notes: Option<String>,
    pub organization: Option<RecordId>,
    pub warehouse: Option<RecordId>,
    pub vendor: Option<RecordId>,
    pub barcode: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateInventoryItemRequest {
    pub name: String,
    pub sku: Option<String>,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub quantity: Option<i64>,
    pub min_quantity: Option<i64>,
    pub unit_cost: Option<f64>,
    pub currency: Option<String>,
    pub location: Option<String>,
    pub supplier: Option<String>,
    pub notes: Option<String>,
    pub warehouse_id: Option<String>,
    pub vendor_id: Option<String>,
    pub barcode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct InventoryTransaction {
    pub id: Option<RecordId>,
    pub item: Option<RecordId>,
    pub txn_type: String,
    pub quantity: i64,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub employee: Option<RecordId>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct InventoryInRequest {
    pub item_id: String,
    pub quantity: i64,
    pub unit_cost: Option<f64>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct InventoryOutRequest {
    pub item_id: String,
    pub quantity: i64,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct InventoryAdjustRequest {
    pub item_id: String,
    pub new_quantity: i64,
    pub notes: Option<String>,
}
