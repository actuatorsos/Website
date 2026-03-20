//! Catalog Repository — Database operations for products and services

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_product_category(
    state: &AppState,
    req: CreateProductCategoryRequest,
) -> Result<ProductCategory, DbError> {
    let code = req.code;
    let name = req.name;

    let cat: Option<ProductCategory> = state.db
        .query("CREATE product_category SET code = $code, name = $name, is_active = true, is_archived = false")
        .bind(("code", code))
        .bind(("name", name))
        .await?
        .take(0)?;
    cat.ok_or(DbError::NotFound)
}

pub async fn get_all_product_categories(state: &AppState) -> Result<Vec<ProductCategory>, DbError> {
    let cats: Vec<ProductCategory> = state.db
        .query("SELECT * FROM product_category WHERE is_archived = false OR is_archived = NONE ORDER BY code ASC")
        .await?.take(0)?;
    Ok(cats)
}

pub async fn delete_product_category(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "product_category", id).await?;
    Ok(())
}

pub async fn create_product(
    state: &AppState,
    req: CreateProductRequest,
) -> Result<Product, DbError> {
    let sku = req.sku;
    let name = req.name;
    let description = req.description;
    let product_type = req.product_type;
    let brand = req.brand;
    let model = req.model;
    let unit = req.unit;
    let sell_price = req.sell_price;
    let cost_price = req.cost_price;
    let tax_rate = req.tax_rate;
    let warranty_months = req.warranty_months;
    let min_stock = req.min_stock;

    let product: Option<Product> = state
        .db
        .query(
            "CREATE product SET \
             sku = $sku, name = $name, \
             description = $desc, product_type = $ptype, brand = $brand, model = $model, \
             unit = $unit, sell_price = $sell_price, cost_price = $cost_price, \
             tax_rate = $tax_rate, warranty_months = $warranty, min_stock = $min_stock, \
             is_sellable = true, is_purchasable = true, \
             is_active = true, is_archived = false",
        )
        .bind(("sku", sku))
        .bind(("name", name))
        .bind(("desc", description))
        .bind(("ptype", product_type))
        .bind(("brand", brand))
        .bind(("model", model))
        .bind(("unit", unit))
        .bind(("sell_price", sell_price))
        .bind(("cost_price", cost_price))
        .bind(("tax_rate", tax_rate))
        .bind(("warranty", warranty_months))
        .bind(("min_stock", min_stock))
        .await?
        .take(0)?;
    product.ok_or(DbError::NotFound)
}

pub async fn get_all_products(state: &AppState) -> Result<Vec<Product>, DbError> {
    let products: Vec<Product> = state.db
        .query("SELECT * FROM product WHERE is_archived = false OR is_archived = NONE ORDER BY sku ASC")
        .await?.take(0)?;
    Ok(products)
}

pub async fn get_product(state: &AppState, id: &str) -> Result<Product, DbError> {
    let id = id.to_string();
    let product: Option<Product> = state.db.select(("product", id)).await?;
    product.ok_or(DbError::NotFound)
}

pub async fn update_product(
    state: &AppState,
    id: &str,
    req: UpdateProductRequest,
) -> Result<Product, DbError> {
    let id = id.to_string();
    let product: Option<Product> = state
        .db
        .update(("product", id))
        .merge(serde_json::json!({
            "name": req.name,
            "sell_price": req.sell_price,
            "cost_price": req.cost_price,
            "min_stock": req.min_stock,
            "is_active": req.is_active,
        }))
        .await?;
    product.ok_or(DbError::NotFound)
}

pub async fn delete_product(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "product", id).await?;
    Ok(())
}

pub async fn get_product_stock(
    state: &AppState,
    product_id: &str,
) -> Result<Vec<serde_json::Value>, DbError> {
    let id = product_id.to_string();
    let stock: Vec<serde_json::Value> = state.db
        .query("SELECT quantity, warehouse, unit_cost, total_value FROM inventory_item WHERE product = type::thing('product', $id) AND (is_archived = false OR is_archived = NONE)")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(stock)
}

pub async fn create_service_catalog(
    state: &AppState,
    req: CreateServiceCatalogRequest,
) -> Result<ServiceCatalog, DbError> {
    let code = req.code;
    let name = req.name;
    let category = req.category;
    let description = req.description;
    let base_price = req.base_price;
    let price_type = req.price_type;
    let estimated_hours = req.estimated_hours;
    let tax_rate = req.tax_rate;

    let service: Option<ServiceCatalog> = state
        .db
        .query(
            "CREATE service_catalog SET \
             code = $code, name = $name, \
             category = $cat, description = $desc, base_price = $base_price, \
             price_type = $price_type, estimated_hours = $est_hours, \
             tax_rate = $tax_rate, is_active = true, is_archived = false",
        )
        .bind(("code", code))
        .bind(("name", name))
        .bind(("cat", category))
        .bind(("desc", description))
        .bind(("base_price", base_price))
        .bind(("price_type", price_type))
        .bind(("est_hours", estimated_hours))
        .bind(("tax_rate", tax_rate))
        .await?
        .take(0)?;
    service.ok_or(DbError::NotFound)
}

pub async fn get_all_service_catalog(state: &AppState) -> Result<Vec<ServiceCatalog>, DbError> {
    let services: Vec<ServiceCatalog> = state.db
        .query("SELECT * FROM service_catalog WHERE is_archived = false OR is_archived = NONE ORDER BY code ASC")
        .await?.take(0)?;
    Ok(services)
}

pub async fn delete_service_catalog(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "service_catalog", id).await?;
    Ok(())
}
