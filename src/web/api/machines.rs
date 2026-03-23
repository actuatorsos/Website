//! Machines API Endpoints
//!
//! نقاط نهاية API للآلات

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::machinery::models::{CreateMachineRequest, Machine, MachineStatus};
use crate::domains::machinery::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/machine_row.html")]
pub struct MachineRowTemplate {
    pub machine: Machine,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/machine_list.html")]
pub struct MachineListTemplate {
    pub machines: Vec<Machine>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateMachineForm {
    #[validate(length(min = 1, max = 100))]
    pub customer_id: String,
    #[validate(length(min = 1, max = 100))]
    pub serial_number: String,
    #[validate(length(min = 1, max = 100))]
    pub model: String,
    #[validate(length(min = 1, max = 100))]
    pub manufacturer: String,
    #[validate(length(max = 50))]
    pub purchase_date: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateMachineStatusForm {
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

fn parse_status(status: &str) -> MachineStatus {
    match status {
        "working" => MachineStatus::Working,
        "broken" => MachineStatus::Broken,
        "repairing" => MachineStatus::Repairing,
        "sold" => MachineStatus::Sold,
        _ => MachineStatus::Working,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_machine(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateMachineForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="5" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let request = CreateMachineRequest {
        customer_id: form.customer_id,
        serial_number: form.serial_number,
        model: form.model,
        manufacturer: form.manufacturer,
        purchase_date: form.purchase_date,
    };

    match repository::create_machine(&state, request).await {
        Ok(machine) => {
            let template = MachineRowTemplate { machine, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="5" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn list_machines(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let machines: Vec<Machine> = repository::get_all_machines(&state)
        .await
        .unwrap_or_default();
    let template = MachineListTemplate { machines, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn get_customer_machines(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(customer_id): Path<String>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let machines: Vec<Machine> = repository::get_customer_machines(&state, &customer_id)
        .await
        .unwrap_or_default();
    let template = MachineListTemplate { machines, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateMachineStatusForm>,
) -> Html<String> {
    let status = parse_status(&form.status);
    match repository::update_machine_status(&state, &id, status).await {
        Ok(_) => Html(r#"<span class="text-green-600">✓</span>"#.to_string()),
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn get_machines_json(State(state): State<AppState>) -> Json<Vec<Machine>> {
    let machines: Vec<Machine> = repository::get_all_machines(&state)
        .await
        .unwrap_or_default();
    Json(machines)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_machine))
        .route("/", get(list_machines))
        .route("/json", get(get_machines_json))
        .route("/customer/{customer_id}", get(get_customer_machines))
        .route("/{id}/status", post(update_status))
}
