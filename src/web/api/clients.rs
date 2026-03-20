//! Clients API Endpoints
//!
//! نقاط نهاية API لإدارة العملاء

use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Html,
    routing::{delete, get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::customers::models::{Client, ClientStatus, CreateClientRequest};
use crate::domains::customers::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/client_row.html")]
pub struct ClientRowTemplate {
    pub client: Client,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/client_list.html")]
pub struct ClientListTemplate {
    pub clients: Vec<Client>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateClientForm {
    #[validate(length(min = 2, max = 100))]
    pub company_name: String,
    #[validate(length(max = 100))]
    pub contact_person: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 50))]
    pub phone: Option<String>,
    #[validate(length(max = 200))]
    pub address: Option<String>,
    #[validate(length(max = 100))]
    pub city: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateStatusForm {
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

fn parse_status(s: &str) -> ClientStatus {
    match s {
        "active" => ClientStatus::Active,
        "inactive" => ClientStatus::Inactive,
        _ => ClientStatus::Pending,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_client(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateClientForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let request = CreateClientRequest {
        company_name: form.company_name,
        contact_person: form.contact_person,
        status: ClientStatus::Pending,
        email: form.email,
        phone: form.phone,
        address: form.address,
        city: form.city,
        latitude: None,
        longitude: None,
    };

    match repository::create_client(&state, request).await {
        Ok(client) => {
            let template = ClientRowTemplate { client, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn list_clients(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let clients: Vec<Client> = repository::get_all_clients(&state)
        .await
        .unwrap_or_default();
    let template = ClientListTemplate { clients, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn update_client_status(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<String>,
    Form(form): Form<UpdateStatusForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());
    let status = parse_status(&form.status);

    match repository::update_client_status(&state, &id, status).await {
        Ok(client) => {
            let template = ClientRowTemplate { client, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn delete_client(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    match repository::delete_client(&state, &id).await {
        Ok(_) => Html(String::new()),
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

/// Returns client options as HTML for select dropdowns (used by invoices, etc)
pub async fn clients_options(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let clients: Vec<Client> = repository::get_all_clients(&state)
        .await
        .unwrap_or_default();

    if clients.is_empty() {
        let no_clients_msg = t
            .get("no_clients_available")
            .cloned()
            .unwrap_or_else(|| "-- لا يوجد عملاء --".to_string());
        return Html(format!(
            r#"<option value="" disabled>{}</option>"#,
            no_clients_msg
        ));
    }

    let options: String = clients
        .iter()
        .map(|c| {
            format!(
                r#"<option value="{}">{}</option>"#,
                c.id_string(),
                c.company_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Html(options)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_client))
        .route("/", get(list_clients))
        .route("/{id}/status", post(update_client_status))
        .route("/{id}", delete(delete_client))
        .route("/options", get(clients_options))
}
