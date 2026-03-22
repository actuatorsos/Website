//! Inventory Advanced Repository — عمليات المخزون في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_warehouse(
    state: &AppState,
    req: CreateWarehouseRequest,
) -> Result<Warehouse, DbError> {
    let w: Option<Warehouse> = state
        .db
        .query(
            "CREATE warehouse SET \
             name = $name, code = $code, address = $addr, \
             capacity = $capacity, notes = $notes, is_active = true, is_archived = false",
        )
        .bind(("name", req.name))
        .bind(("code", req.code))
        .bind(("addr", req.address))
        .bind(("capacity", req.capacity))
        .bind(("notes", req.notes))
        .await?
        .take(0)?;
    w.ok_or(DbError::NotFound)
}

pub async fn get_all_warehouses(state: &AppState, org_id: Option<&str>) -> Result<Vec<Warehouse>, DbError> {
    let ws: Vec<Warehouse> = if let Some(org) = org_id {
        state
            .db
            .query("SELECT * FROM warehouse WHERE (is_archived = false OR is_archived = NONE) AND organization = type::record($org)")
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query("SELECT * FROM warehouse WHERE is_archived = false OR is_archived = NONE")
            .await?
            .take(0)?
    };
    Ok(ws)
}

pub async fn create_inventory_item(
    state: &AppState,
    req: CreateInventoryItemRequest,
) -> Result<InventoryItem, DbError> {
    let unit = req.unit.unwrap_or_else(|| "piece".to_string());
    let qty = req.quantity.unwrap_or(0);
    let min_qty = req.min_quantity.unwrap_or(5);
    let category = req.category.unwrap_or_else(|| "other".to_string());
    let currency = req.currency.unwrap_or_else(|| "SAR".to_string());

    let item: Option<InventoryItem> = state
        .db
        .query(
            "CREATE inventory_item SET \
             name = $name, sku = $sku, category = $category, \
             unit = $unit, quantity = $qty, min_quantity = $min_qty, \
             unit_cost = $cost, currency = $currency, \
             location = $location, supplier = $supplier, \
             notes = $notes, barcode = $barcode, \
             warehouse = IF $wh_id != NONE THEN type::thing('warehouse', $wh_id) ELSE NONE END, \
             vendor = IF $vn_id != NONE THEN type::thing('vendor', $vn_id) ELSE NONE END, \
             is_archived = false",
        )
        .bind(("name", req.name))
        .bind(("sku", req.sku))
        .bind(("category", category))
        .bind(("unit", unit))
        .bind(("qty", qty))
        .bind(("min_qty", min_qty))
        .bind(("cost", req.unit_cost))
        .bind(("currency", currency))
        .bind(("location", req.location))
        .bind(("supplier", req.supplier))
        .bind(("notes", req.notes))
        .bind(("barcode", req.barcode))
        .bind(("wh_id", req.warehouse_id))
        .bind(("vn_id", req.vendor_id))
        .await?
        .take(0)?;
    item.ok_or(DbError::NotFound)
}

pub async fn get_all_inventory(
    state: &AppState,
    org_id: Option<&str>,
) -> Result<Vec<InventoryItem>, DbError> {
    let items: Vec<InventoryItem> = if let Some(org) = org_id {
        state
            .db
            .query(
                "SELECT * FROM inventory_item \
                 WHERE (is_archived = false OR is_archived = NONE) \
                 AND organization = type::record($org) ORDER BY name ASC",
            )
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query(
                "SELECT * FROM inventory_item \
                 WHERE is_archived = false OR is_archived = NONE \
                 ORDER BY name ASC",
            )
            .await?
            .take(0)?
    };
    Ok(items)
}

pub async fn get_low_stock(state: &AppState, org_id: Option<&str>) -> Result<Vec<InventoryItem>, DbError> {
    let items: Vec<InventoryItem> = if let Some(org) = org_id {
        state
            .db
            .query(
                "SELECT * FROM inventory_item \
                 WHERE quantity <= min_quantity \
                 AND (is_archived = false OR is_archived = NONE) \
                 AND organization = $org",
            )
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query(
                "SELECT * FROM inventory_item \
                 WHERE quantity <= min_quantity \
                 AND (is_archived = false OR is_archived = NONE)",
            )
            .await?
            .take(0)?
    };
    Ok(items)
}

pub async fn inventory_in(
    state: &AppState,
    req: InventoryInRequest,
) -> Result<(InventoryItem, InventoryTransaction), DbError> {
    let item_id = req.item_id;
    let quantity = req.quantity;
    let unit_cost = req.unit_cost;
    let total_cost = unit_cost.map(|c| c * quantity as f64);
    let reference = req.reference;
    let notes = req.notes;

    let item: Option<InventoryItem> = state
        .db
        .query(
            "UPDATE inventory_item SET quantity += $qty \
             WHERE id = type::thing('inventory_item', $item_id)",
        )
        .bind(("item_id", item_id.clone()))
        .bind(("qty", quantity))
        .await?
        .take(0)?;

    let item = item.ok_or(DbError::NotFound)?;

    let txn: Option<InventoryTransaction> = state
        .db
        .query(
            "CREATE inventory_transaction SET \
             item = type::thing('inventory_item', $item_id), \
             txn_type = 'in', quantity = $qty, \
             unit_cost = $unit_cost, total_cost = $total_cost, \
             reference = $ref, notes = $notes",
        )
        .bind(("item_id", item_id))
        .bind(("qty", quantity))
        .bind(("unit_cost", unit_cost))
        .bind(("total_cost", total_cost))
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

    let item: Option<InventoryItem> = state
        .db
        .query(
            "UPDATE inventory_item SET quantity -= $qty \
             WHERE id = type::thing('inventory_item', $item_id)",
        )
        .bind(("item_id", item_id.clone()))
        .bind(("qty", quantity))
        .await?
        .take(0)?;

    let item = item.ok_or(DbError::NotFound)?;

    let txn: Option<InventoryTransaction> = state
        .db
        .query(
            "CREATE inventory_transaction SET \
             item = type::thing('inventory_item', $item_id), \
             txn_type = 'out', quantity = $qty, \
             reference = $ref, notes = $notes",
        )
        .bind(("item_id", item_id))
        .bind(("qty", quantity))
        .bind(("ref", reference))
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
    let txns: Vec<InventoryTransaction> = state
        .db
        .query(
            "SELECT * FROM inventory_transaction \
             WHERE item = type::thing('inventory_item', $id) \
             ORDER BY created_at DESC",
        )
        .bind(("id", id))
        .await?
        .take(0)?;
    Ok(txns)
}

pub async fn get_all_transactions(
    state: &AppState,
) -> Result<Vec<InventoryTransaction>, DbError> {
    let txns: Vec<InventoryTransaction> = state
        .db
        .query(
            "SELECT * FROM inventory_transaction \
             ORDER BY created_at DESC LIMIT 100",
        )
        .await?
        .take(0)?;
    Ok(txns)
}
