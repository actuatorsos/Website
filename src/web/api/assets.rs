//! Assets API Endpoints
//!
//! نقاط نهاية API للأصول

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;

use crate::db::AppState;
use crate::i18n::Language;
use crate::models::{Asset, AssetCategory, AssignAssetRequest, CreateAssetRequest};

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/asset_row.html")]
pub struct AssetRowTemplate {
    pub asset: Asset,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/asset_list.html")]
pub struct AssetListTemplate {
    pub assets: Vec<Asset>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize)]
pub struct CreateAssetForm {
    pub name: String,
    pub category: String,
    pub serial_number: Option<String>,
    pub purchase_date: Option<String>,
    pub value: Option<f64>,
    pub location: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AssignAssetForm {
    pub employee_id: String,
    pub location: String,
}

// ============================================================================
// Helpers
// ============================================================================

fn resolve_language(cookies: &Cookies) -> Language {
    if let Some(cookie) = cookies.get("lang") {
        return Language::from_str(cookie.value());
    }
    Language::Arabic
}

fn parse_category(category: &str) -> AssetCategory {
    match category {
        "tools" => AssetCategory::Tools,
        "equipment" => AssetCategory::Equipment,
        "vehicles" => AssetCategory::Vehicles,
        "furniture" => AssetCategory::Furniture,
        "electronics" => AssetCategory::Electronics,
        _ => AssetCategory::Tools,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_asset(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateAssetForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let request = CreateAssetRequest {
        name: form.name,
        category: parse_category(&form.category),
        serial_number: form.serial_number,
        purchase_date: form.purchase_date,
        value: form.value,
        location: form.location,
    };

    match state.create_asset(request).await {
        Ok(asset) => {
            let template = AssetRowTemplate { asset, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="6" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn list_assets(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let assets: Vec<Asset> = state.get_all_assets().await.unwrap_or_default();
    let template = AssetListTemplate { assets, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn assign_asset(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<AssignAssetForm>,
) -> Html<String> {
    let request = AssignAssetRequest {
        employee_id: form.employee_id,
        location: form.location,
    };

    match state.assign_asset(&id, request).await {
        Ok(_) => Html(r#"<span class="text-green-600">✓</span>"#.to_string()),
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn get_assets_json(State(state): State<AppState>) -> Json<Vec<Asset>> {
    let assets: Vec<Asset> = state.get_all_assets().await.unwrap_or_default();
    Json(assets)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_asset))
        .route("/", get(list_assets))
        .route("/json", get(get_assets_json))
        .route("/{id}/assign", post(assign_asset))
}
