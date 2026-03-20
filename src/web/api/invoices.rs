//! Invoices API Endpoints
//!
//! نقاط نهاية API للفواتير

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{delete, get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::customers::models::Client;
use crate::domains::finance::models::{CreateInvoiceRequest, Invoice, InvoiceItem, InvoiceStatus};
use crate::domains::finance::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/invoice_row.html")]
pub struct InvoiceRowTemplate {
    pub invoice: Invoice,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/invoice_list.html")]
pub struct InvoiceListTemplate {
    pub invoices: Vec<Invoice>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateInvoiceForm {
    #[validate(length(min = 1, max = 100))]
    pub client_id: String,
    #[validate(length(min = 8, max = 50))]
    pub invoice_date: String,
    #[validate(length(max = 50))]
    pub due_date: Option<String>,
    pub tax_rate: Option<String>,
    #[validate(length(max = 1000))]
    pub notes: Option<String>,
    // Items are submitted as parallel arrays
    pub item_description: Vec<String>,
    pub item_quantity: Vec<String>,
    pub item_unit_price: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateInvoiceStatusForm {
    pub status: String,
}

#[derive(serde::Deserialize)]
pub struct RecordPaymentForm {
    pub amount: f64,
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

fn parse_status(status: &str) -> InvoiceStatus {
    match status {
        "draft" => InvoiceStatus::Draft,
        "sent" => InvoiceStatus::Sent,
        "partially_paid" => InvoiceStatus::PartiallyPaid,
        "paid" => InvoiceStatus::Paid,
        "cancelled" => InvoiceStatus::Cancelled,
        "overdue" => InvoiceStatus::Overdue,
        _ => InvoiceStatus::Draft,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_invoice(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateInvoiceForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    // Build items from parallel arrays
    let items: Vec<InvoiceItem> = form
        .item_description
        .iter()
        .zip(form.item_quantity.iter())
        .zip(form.item_unit_price.iter())
        .filter_map(|((desc, qty), price)| {
            let q = qty.parse::<f64>().ok()?;
            let p = price.parse::<f64>().ok()?;
            Some(InvoiceItem {
                description: desc.clone(),
                quantity: q,
                unit_price: p,
                total: q * p,
            })
        })
        .collect();

    if items.is_empty() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">يجب إضافة بند واحد على الأقل</td></tr>"#,
        ));
    }

    let request = CreateInvoiceRequest {
        client_id: form.client_id,
        invoice_date: form.invoice_date,
        due_date: form.due_date,
        items,
        tax_rate: form
            .tax_rate
            .as_deref()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or_default(),
        notes: form.notes,
    };

    match repository::create_invoice(&state, request).await {
        Ok(invoice) => {
            let template = InvoiceRowTemplate { invoice, t };
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

async fn list_invoices(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let invoices: Vec<Invoice> = repository::get_all_invoices(&state)
        .await
        .unwrap_or_default();
    let template = InvoiceListTemplate { invoices, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateInvoiceStatusForm>,
) -> Html<String> {
    let status = parse_status(&form.status);
    match repository::update_invoice_status(&state, &id, status).await {
        Ok(_) => Html(r#"<span class="text-green-600">تم التحديث</span>"#.to_string()),
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn record_payment(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<String>,
    Form(form): Form<RecordPaymentForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    match repository::record_invoice_payment(&state, &id, form.amount).await {
        Ok(invoice) => {
            let template = InvoiceRowTemplate { invoice, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn delete_invoice(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    match repository::delete_invoice(&state, &id).await {
        Ok(_) => Html(String::new()), // Return empty to remove from DOM
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn get_invoices_json(State(state): State<AppState>) -> Json<Vec<Invoice>> {
    let invoices: Vec<Invoice> = repository::get_all_invoices(&state)
        .await
        .unwrap_or_default();
    Json(invoices)
}

async fn get_clients_json(State(state): State<AppState>) -> Json<Vec<Client>> {
    let clients: Vec<Client> = crate::domains::customers::repository::get_all_clients(&state)
        .await
        .unwrap_or_default();
    Json(clients)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_invoice))
        .route("/", get(list_invoices))
        .route("/json", get(get_invoices_json))
        .route("/clients-json", get(get_clients_json))
        .route("/{id}/status", post(update_status))
        .route("/{id}/payment", post(record_payment))
        .route("/{id}", delete(delete_invoice))
}
