//! Inventory Advanced Repository — عمليات المخزون في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};
use f64;

pub async fn create_warehouse(
    state: &AppState,
    req: CreateWarehouseRequest,
) -> Result<Warehouse, DbError> {
    let code = req.code;
    let name = req.name;
    let address = req.address;

    let w: Option<Warehouse> = state
        .db
        .query("CREATE warehouse SET code = $code, name = $name, address = $addr, is_active = true")
        .bind(("code", code))
        .bind(("name", name))
        .bind(("addr", address))
        .await?
        .take(0)?;
    w.ok_or(DbError::NotFound)
}

pub async fn get_all_warehouses(state: &AppState) -> Result<Vec<Warehouse>, DbError> {
    let ws: Vec<Warehouse> = state
        .db
        .query("SELECT * FROM warehouse WHERE is_archived = false OR is_archived = NONE")
        .await?
        .take(0)?;
    Ok(ws)
}

pub async fn create_inventory_item(
    state: &AppState,
    req: CreateInventoryItemRequest,
) -> Result<InventoryItem, DbError> {
    let name = req.name;
    let sku = req.sku;
    let desc = req.description;
    let unit = req.unit.unwrap_or_else(|| "piece".to_string());
    let qty = req.quantity.unwrap_or(0);
    let min_qty = req.min_quantity.unwrap_or(0);
    let cost = req.unit_cost.unwrap_or(0.0);
    let total_value = (qty as f64) * cost;
    let loc = req.location_code;

    let item: Option<InventoryItem> = state
        .db
        .query(
            "CREATE inventory_item SET \
             name = $name, sku = $sku, description = $desc, \
             unit = $unit, quantity = $qty, min_quantity = $min_qty, \
             unit_cost = $cost, total_value = $total_value, \
             location_code = $loc",
        )
        .bind(("name", name))
        .bind(("sku", sku))
        .bind(("desc", desc))
        .bind(("unit", unit))
        .bind(("qty", qty))
        .bind(("min_qty", min_qty))
        .bind(("cost", cost))
        .bind(("total_value", total_value))
        .bind(("loc", loc))
        .await?
        .take(0)?;
    item.ok_or(DbError::NotFound)
}

pub async fn get_all_inventory(state: &AppState) -> Result<Vec<InventoryItem>, DbError> {
    let items: Vec<InventoryItem> = state.db
        .query("SELECT * FROM inventory_item WHERE is_archived = false OR is_archived = NONE ORDER BY name ASC")
        .await?.take(0)?;
    Ok(items)
}

pub async fn get_low_stock(state: &AppState) -> Result<Vec<InventoryItem>, DbError> {
    let items: Vec<InventoryItem> = state.db
        .query("SELECT * FROM inventory_item WHERE quantity <= min_quantity AND (is_archived = false OR is_archived = NONE)")
        .await?.take(0)?;
    Ok(items)
}

pub async fn inventory_in(
    state: &AppState,
    req: InventoryInRequest,
) -> Result<(InventoryItem, InventoryTransaction), DbError> {
    let item_id = req.item_id;
    let quantity = req.quantity;
    let unit_cost = req.unit_cost;
    let reference = req.reference;
    let notes = req.notes;

    let item: Option<InventoryItem> = state.db
        .query("UPDATE inventory_item SET quantity += $qty WHERE id = type::thing('inventory_item', $item_id)")
        .bind(("item_id", item_id.clone()))
        .bind(("qty", quantity))
        .await?.take(0)?;

    let item = item.ok_or(DbError::NotFound)?;

    let txn: Option<InventoryTransaction> = state
        .db
        .query(
            "CREATE inventory_transaction SET \
             item = type::thing('inventory_item', $item_id), \
             txn_type = 'in', quantity = $qty, \
             unit_cost = $unit_cost, \
             reference = $ref, notes = $notes",
        )
        .bind(("item_id", item_id))
        .bind(("qty", quantity))
        .bind(("unit_cost", unit_cost))
        .bind(("ref", reference))
        .bind(("notes", notes))
        .await?
        .take(0)?;

    Ok((item, txn.ok_or(DbError::NotFound)?))
}

pub async fn inventory_out(
    state: &AppState,
    req: InventoryOutRequest,
) -> Result<(InventoryItem, InventoryTransaction), DbError> {
    let item_id = req.item_id;
    let quantity = req.quantity;
    let reference = req.reference;
    let reference_type = req.reference_type;
    let notes = req.notes;

    let current: Option<InventoryItem> =
        state.db.select(("inventory_item", item_id.clone())).await?;
    let current = current.ok_or(DbError::NotFound)?;
    let available = current.quantity.unwrap_or(0);

    if available < quantity {
        return Err(DbError::Conflict(format!(
            "نقص في المخزون: المتوفر {} وطلبت {}",
            available, quantity
        )));
    }

    let item: Option<InventoryItem> = state.db
        .query("UPDATE inventory_item SET quantity -= $qty WHERE id = type::thing('inventory_item', $item_id)")
        .bind(("item_id", item_id.clone()))
        .bind(("qty", quantity))
        .await?.take(0)?;

    let item = item.ok_or(DbError::NotFound)?;

    let txn: Option<InventoryTransaction> = state
        .db
        .query(
            "CREATE inventory_transaction SET \
             item = type::thing('inventory_item', $item_id), \
             txn_type = 'out', quantity = $qty, \
             reference = $ref, reference_type = $ref_type, notes = $notes",
        )
        .bind(("item_id", item_id))
        .bind(("qty", quantity))
        .bind(("ref", reference))
        .bind(("ref_type", reference_type))
        .bind(("notes", notes))
        .await?
        .take(0)?;

    Ok((item, txn.ok_or(DbError::NotFound)?))
}

pub async fn inventory_adjust(
    state: &AppState,
    req: InventoryAdjustRequest,
) -> Result<InventoryItem, DbError> {
    let item_id = req.item_id;
    let new_quantity = req.new_quantity;
    let notes = req.notes;

    let item: Option<InventoryItem> = state
        .db
        .update(("inventory_item", item_id.clone()))
        .merge(serde_json::json!({ "quantity": new_quantity }))
        .await?;

    let _ = state
        .db
        .query(
            "CREATE inventory_transaction SET \
             item = type::thing('inventory_item', $item_id), \
             txn_type = 'adjust', quantity = $qty, notes = $notes",
        )
        .bind(("item_id", item_id))
        .bind(("qty", new_quantity))
        .bind(("notes", notes))
        .await;

    item.ok_or(DbError::NotFound)
}

pub async fn get_item_transactions(
    state: &AppState,
    item_id: &str,
) -> Result<Vec<InventoryTransaction>, DbError> {
    let id = item_id.to_string();
    let txns: Vec<InventoryTransaction> = state.db
        .query("SELECT * FROM inventory_transaction WHERE item = type::thing('inventory_item', $id) ORDER BY created_at DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(txns)
}
