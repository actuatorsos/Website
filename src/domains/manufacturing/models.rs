//! Manufacturing / BOM Models — نماذج قوائم المواد والإنتاج

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bom {
    pub id: Option<Thing>,
    pub product: Thing,
    pub bom_number: String,
    pub title: String,
    pub version: Option<i64>,
    pub description: Option<String>,
    pub total_cost: Option<f64>,
    pub labor_cost: Option<f64>,
    pub overhead_cost: Option<f64>,
    pub yield_qty: Option<i64>,
    pub unit: Option<String>,
    pub status: Option<String>, // draft, active, obsolete
    pub approved_by: Option<Thing>,
    pub approved_at: Option<String>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBomRequest {
    pub product_id: String,
    pub title: String,
    pub description: Option<String>,
    pub labor_cost: Option<f64>,
    pub overhead_cost: Option<f64>,
    pub yield_qty: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomLine {
    pub id: Option<Thing>,
    pub bom: Thing,
    pub line_number: i64,
    pub component: Option<Thing>,
    pub inventory_item: Option<Thing>,
    pub description: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub unit_cost: Option<f64>,
    pub total_cost: Option<f64>,
    pub waste_percentage: Option<f64>,
    pub is_optional: Option<bool>,
    pub substitute: Option<Thing>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBomLineRequest {
    pub bom_id: String,
    pub inventory_item_id: Option<String>,
    pub component_product_id: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub unit: Option<String>,
    pub unit_cost: Option<f64>,
    pub waste_percentage: Option<f64>,
    pub is_optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub id: Option<Thing>,
    pub order_number: String,
    pub product: Thing,
    pub bom: Thing,
    pub quantity: i64,
    pub client: Option<Thing>,
    pub project: Option<Thing>,
    pub department: Option<Thing>,
    pub assigned_to: Option<Thing>,
    pub planned_start: Option<String>,
    pub planned_end: Option<String>,
    pub actual_start: Option<String>,
    pub actual_end: Option<String>,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub quality_notes: Option<String>,
    pub serial_numbers: Option<Vec<String>>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductionOrderRequest {
    pub product_id: String,
    pub bom_id: String,
    pub quantity: i64,
    pub client_id: Option<String>,
    pub project_id: Option<String>,
    pub department_id: Option<String>,
    pub planned_start: Option<String>,
    pub planned_end: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductionStatusRequest {
    pub status: String,
    pub notes: Option<String>,
}
