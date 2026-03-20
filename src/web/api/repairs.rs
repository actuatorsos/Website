//! Repairs API Endpoints
//!
//! نقاط نهاية API لعمليات الإصلاح

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{get, post, put},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::machinery::models::{
    CreateRepairRequest, RepairOperation, RepairStatus, UpdateRepairRequest,
};
use crate::domains::machinery::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/repair_row.html")]
pub struct RepairRowTemplate {
    pub repair: RepairOperation,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/repair_list.html")]
pub struct RepairListTemplate {
    pub repairs: Vec<RepairOperation>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateRepairForm {
    #[validate(length(min = 1, max = 100))]
    pub machine_id: String,
    #[validate(length(max = 100))]
    pub project_id: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub employee_id: String,
    #[validate(length(min = 2, max = 1000))]
    pub description: String,
    #[validate(length(max = 1000))]
    pub diagnosis: Option<String>,
}

#[derive(serde::Deserialize, Validate)]
pub struct UpdateRepairForm {
    #[validate(length(max = 1000))]
    pub diagnosis: Option<String>,
    #[validate(length(max = 1000))]
    pub parts_used: Option<String>,
    #[validate(range(min = 0.0))]
    pub cost: Option<f64>,
    #[validate(length(min = 1, max = 50))]
    pub status: String,
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

fn parse_status(status: &str) -> RepairStatus {
    match status {
        "new" => RepairStatus::New,
        "diagnosing" => RepairStatus::Diagnosing,
        "repairing" => RepairStatus::Repairing,
        "waiting" => RepairStatus::Waiting,
        "completed" => RepairStatus::Completed,
        "cancelled" => RepairStatus::Cancelled,
        _ => RepairStatus::New,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_repair(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateRepairForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="6" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let request = CreateRepairRequest {
        machine_id: form.machine_id,
        project_id: form.project_id,
        employee_id: form.employee_id,
        description: form.description,
        diagnosis: form.diagnosis,
    };

    match repository::create_repair(&state, request).await {
        Ok(repair) => {
            let template = RepairRowTemplate { repair, t };
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

async fn list_repairs(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let repairs: Vec<RepairOperation> = repository::get_all_repairs(&state)
        .await
        .unwrap_or_default();
    let template = RepairListTemplate { repairs, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn get_machine_repairs(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(machine_id): Path<String>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let repairs: Vec<RepairOperation> = repository::get_machine_repairs(&state, &machine_id)
        .await
        .unwrap_or_default();
    let template = RepairListTemplate { repairs, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn update_repair(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateRepairForm>,
) -> Html<String> {
    if let Err(e) = form.validate() {
        return Html(format!(
            "<span class='text-red-600'>Validation Error: {}</span>",
            e
        ));
    }

    let request = UpdateRepairRequest {
        diagnosis: form.diagnosis,
        parts_used: form.parts_used,
        cost: form.cost,
        status: parse_status(&form.status),
    };

    match repository::update_repair(&state, &id, request).await {
        Ok(_) => Html(r#"<span class="text-green-600">✓</span>"#.to_string()),
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn get_repairs_json(State(state): State<AppState>) -> Json<Vec<RepairOperation>> {
    let repairs: Vec<RepairOperation> = repository::get_all_repairs(&state)
        .await
        .unwrap_or_default();
    Json(repairs)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_repair))
        .route("/", get(list_repairs))
        .route("/json", get(get_repairs_json))
        .route("/machine/{machine_id}", get(get_machine_repairs))
        .route("/{id}", put(update_repair))
}
