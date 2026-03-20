//! Manufacturing / BOM Repository — عمليات قاعدة البيانات للإنتاج

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_bom(state: &AppState, req: CreateBomRequest) -> Result<Bom, DbError> {
    let product_id = req.product_id;
    let title = req.title;
    let desc = req.description;
    let labor = req.labor_cost;
    let overhead = req.overhead_cost;
    let yield_qty = req.yield_qty.unwrap_or(1);
    let notes = req.notes;

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM bom GROUP ALL")
        .await?
        .take(0)
        .unwrap_or_default();
    let n = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    let bom_number = format!("BOM-{:04}", n);

    let bom: Option<Bom> = state
        .db
        .query(
            "CREATE bom SET \
             product = type::thing('product', $product_id), \
             bom_number = $bom_number, title = $title, \
             description = $desc, labor_cost = $labor, \
             overhead_cost = $overhead, yield_qty = $yield, version = 1, status = 'draft', \
             notes = $notes",
        )
        .bind(("product_id", product_id))
        .bind(("bom_number", bom_number))
        .bind(("title", title))
        .bind(("desc", desc))
        .bind(("labor", labor))
        .bind(("overhead", overhead))
        .bind(("yield", yield_qty))
        .bind(("notes", notes))
        .await?
        .take(0)?;
    bom.ok_or(DbError::NotFound)
}

pub async fn get_all_boms(state: &AppState) -> Result<Vec<Bom>, DbError> {
    let boms: Vec<Bom> = state.db
        .query("SELECT * FROM bom WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?.take(0)?;
    Ok(boms)
}

pub async fn get_bom(state: &AppState, id: &str) -> Result<Bom, DbError> {
    let id = id.to_string();
    let b: Option<Bom> = state.db.select(("bom", id)).await?;
    b.ok_or(DbError::NotFound)
}

pub async fn add_bom_line(state: &AppState, req: CreateBomLineRequest) -> Result<BomLine, DbError> {
    let bom_id = req.bom_id;
    let desc = req.description;
    let quantity = req.quantity;
    let unit = req.unit.unwrap_or_else(|| "piece".to_string());
    let unit_cost = req.unit_cost;
    let total_cost = unit_cost.unwrap_or(0.0) * quantity;
    let waste = req.waste_percentage;
    let optional = req.is_optional.unwrap_or(false);

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM bom_line WHERE bom = type::thing('bom', $id) GROUP ALL")
        .bind(("id", bom_id.clone()))
        .await?
        .take(0)
        .unwrap_or_default();
    let line_num = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;

    let line: Option<BomLine> = state
        .db
        .query(
            "CREATE bom_line SET \
             bom = type::thing('bom', $bom_id), line_number = $line_num, \
             description = $desc, quantity = $qty, unit = $unit, \
             unit_cost = $cost, total_cost = $total, \
             waste_percentage = $waste, is_optional = $optional",
        )
        .bind(("bom_id", bom_id))
        .bind(("line_num", line_num))
        .bind(("desc", desc))
        .bind(("qty", quantity))
        .bind(("unit", unit))
        .bind(("cost", unit_cost))
        .bind(("total", total_cost))
        .bind(("waste", waste))
        .bind(("optional", optional))
        .await?
        .take(0)?;
    line.ok_or(DbError::NotFound)
}

pub async fn get_bom_lines(state: &AppState, bom_id: &str) -> Result<Vec<BomLine>, DbError> {
    let id = bom_id.to_string();
    let lines: Vec<BomLine> = state
        .db
        .query(
            "SELECT * FROM bom_line WHERE bom = type::thing('bom', $id) ORDER BY line_number ASC",
        )
        .bind(("id", id))
        .await?
        .take(0)?;
    Ok(lines)
}

pub async fn create_production_order(
    state: &AppState,
    req: CreateProductionOrderRequest,
) -> Result<ProductionOrder, DbError> {
    let product_id = req.product_id;
    let bom_id = req.bom_id;
    let quantity = req.quantity;
    let planned_start = req.planned_start;
    let planned_end = req.planned_end;
    let priority = req.priority.unwrap_or_else(|| "medium".to_string());
    let notes = req.notes;

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM production_order GROUP ALL")
        .await?
        .take(0)
        .unwrap_or_default();
    let n = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    let order_number = format!("PO-{:04}", n);

    let order: Option<ProductionOrder> = state
        .db
        .query(
            "CREATE production_order SET \
             order_number = $order_num, \
             product = type::thing('product', $product_id), \
             bom = type::thing('bom', $bom_id), \
             quantity = $qty, \
             planned_start = $planned_start, planned_end = $planned_end, \
             priority = $priority, notes = $notes, status = 'draft'",
        )
        .bind(("order_num", order_number))
        .bind(("product_id", product_id))
        .bind(("bom_id", bom_id))
        .bind(("qty", quantity))
        .bind(("planned_start", planned_start))
        .bind(("planned_end", planned_end))
        .bind(("priority", priority))
        .bind(("notes", notes))
        .await?
        .take(0)?;
    order.ok_or(DbError::NotFound)
}

pub async fn get_all_production_orders(state: &AppState) -> Result<Vec<ProductionOrder>, DbError> {
    let orders: Vec<ProductionOrder> = state.db
        .query("SELECT * FROM production_order WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?.take(0)?;
    Ok(orders)
}

pub async fn get_production_order(state: &AppState, id: &str) -> Result<ProductionOrder, DbError> {
    let id = id.to_string();
    let o: Option<ProductionOrder> = state.db.select(("production_order", id)).await?;
    o.ok_or(DbError::NotFound)
}

pub async fn update_production_status(
    state: &AppState,
    id: &str,
    req: UpdateProductionStatusRequest,
) -> Result<ProductionOrder, DbError> {
    let id = id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let new_status = req.status;
    let notes = req.notes;

    let mut merge_data = serde_json::json!({
        "status": new_status,
        "quality_notes": notes,
    });

    if new_status == "in_production" {
        merge_data["actual_start"] = serde_json::Value::String(now.clone());
    }
    if new_status == "completed" {
        merge_data["actual_end"] = serde_json::Value::String(now);
    }

    let order: Option<ProductionOrder> = state
        .db
        .update(("production_order", id))
        .merge(merge_data)
        .await?;
    order.ok_or(DbError::NotFound)
}

/// Consume BOM materials from inventory — الخصم التلقائي من المخزون
pub async fn consume_materials(
    state: &AppState,
    order_id: &str,
) -> Result<Vec<serde_json::Value>, DbError> {
    let order_id = order_id.to_string();
    let order: Option<ProductionOrder> = state
        .db
        .select(("production_order", order_id.clone()))
        .await?;
    let order = order.ok_or(DbError::NotFound)?;

    let bom_id = order.bom.id.to_raw();
    let production_qty = order.quantity;

    let lines: Vec<BomLine> = state
        .db
        .query("SELECT * FROM bom_line WHERE bom = type::thing('bom', $id)")
        .bind(("id", bom_id))
        .await?
        .take(0)?;

    let mut results = Vec::new();

    for line in &lines {
        if let Some(inv_item) = &line.inventory_item {
            let item_id = inv_item.id.to_raw();
            let waste_factor = 1.0 + line.waste_percentage.unwrap_or(0.0) / 100.0;
            let needed_qty = (line.quantity * (production_qty as f64) * waste_factor)
                .ceil()
                .to_string()
                .parse::<i64>()
                .unwrap_or(0);

            let _ = state.db
                .query(
                    "UPDATE inventory_item SET quantity -= $qty WHERE id = type::thing('inventory_item', $item_id)"
                )
                .bind(("item_id", item_id.clone()))
                .bind(("qty", needed_qty))
                .await;

            let _ = state
                .db
                .query(
                    "CREATE inventory_transaction SET \
                     item = type::thing('inventory_item', $item_id), \
                     txn_type = 'production_consume', quantity = $qty, \
                     reference = $order_id, reference_type = 'production_order'",
                )
                .bind(("item_id", item_id.clone()))
                .bind(("qty", needed_qty))
                .bind(("order_id", order_id.clone()))
                .await;

            results.push(serde_json::json!({
                "item_id": item_id,
                "consumed_qty": needed_qty,
                "status": "consumed"
            }));
        }
    }

    let _ = state
        .db
        .update(("production_order", order_id))
        .merge(serde_json::json!({
            "status": "in_production",
            "actual_start": chrono::Utc::now().to_rfc3339()
        }))
        .await
        .map(|_: Option<serde_json::Value>| ());

    Ok(results)
}
