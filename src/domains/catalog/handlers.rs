use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post, put},
};

async fn list_categories(State(s): State<AppState>) -> Result<Json<Vec<ProductCategory>>, DbError> {
    Ok(Json(repo::get_all_product_categories(&s).await?))
}
async fn create_category(
    State(s): State<AppState>,
    Json(req): Json<CreateProductCategoryRequest>,
) -> Result<Json<ProductCategory>, DbError> {
    Ok(Json(repo::create_product_category(&s, req).await?))
}
async fn delete_category(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_product_category(&s, &id).await?;
    Ok(Json(()))
}
async fn list_products(State(s): State<AppState>) -> Result<Json<Vec<Product>>, DbError> {
    Ok(Json(repo::get_all_products(&s).await?))
}
async fn create_product(
    State(s): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<Product>, DbError> {
    Ok(Json(repo::create_product(&s, req).await?))
}
async fn get_product(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Product>, DbError> {
    Ok(Json(repo::get_product(&s, &id).await?))
}
async fn update_product(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<Product>, DbError> {
    Ok(Json(repo::update_product(&s, &id, req).await?))
}
async fn delete_product(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_product(&s, &id).await?;
    Ok(Json(()))
}
async fn get_product_stock(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, DbError> {
    Ok(Json(repo::get_product_stock(&s, &id).await?))
}
async fn list_services(State(s): State<AppState>) -> Result<Json<Vec<ServiceCatalog>>, DbError> {
    Ok(Json(repo::get_all_service_catalog(&s).await?))
}
async fn create_service(
    State(s): State<AppState>,
    Json(req): Json<CreateServiceCatalogRequest>,
) -> Result<Json<ServiceCatalog>, DbError> {
    Ok(Json(repo::create_service_catalog(&s, req).await?))
}
async fn delete_service(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_service_catalog(&s, &id).await?;
    Ok(Json(()))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/product-categories",
            get(list_categories).post(create_category),
        )
        .route("/product-categories/{id}", delete(delete_category))
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/{id}",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/products/{id}/stock", get(get_product_stock))
        .route("/service-catalog", get(list_services).post(create_service))
        .route("/service-catalog/{id}", delete(delete_service))
}
