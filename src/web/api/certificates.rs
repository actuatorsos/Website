//! Certificate API Endpoints
//!
//! نقاط نهاية API للشهادات

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
use crate::domains::finance::models::{Certificate, CreateCertificateRequest};
use crate::domains::finance::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/certificate_row.html")]
pub struct CertificateRowTemplate {
    pub certificate: Certificate,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new certificate
async fn create_certificate(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(req): Form<CreateCertificateRequest>,
) -> Html<String> {
    let lang = Language::resolve(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = req.validate() {
        return Html(format!(
            "<tr><td colspan='7' class='px-6 py-4 text-error'>Validation Error: {}</td></tr>",
            e
        ));
    }

    match repository::create_certificate(&state, req).await {
        Ok(certificate) => {
            let tmpl = CertificateRowTemplate { certificate, t };
            Html(
                tmpl.render()
                    .unwrap_or_else(|e| format!("<tr><td colspan='7'>{}</td></tr>", e)),
            )
        }
        Err(e) => Html(format!(
            "<tr><td colspan='7' class='px-6 py-4 text-error'>{}</td></tr>",
            e
        )),
    }
}

/// List all certificates
async fn list_certificates(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = Language::resolve(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let certificates: Vec<Certificate> = repository::get_all_certificates(&state)
        .await
        .unwrap_or_default();

    let mut html = String::new();
    for certificate in certificates {
        let tmpl = CertificateRowTemplate {
            certificate,
            t: t.clone(),
        };
        html.push_str(&tmpl.render().unwrap_or_default());
    }

    if html.is_empty() {
        let empty_msg = t
            .get("table_empty")
            .cloned()
            .unwrap_or_else(|| "لا توجد بيانات".to_string());
        html = format!(
            "<tr><td colspan='7' class='px-6 py-8 text-center text-text-secondary'>{}</td></tr>",
            empty_msg
        );
    }

    Html(html)
}

/// Delete a certificate
async fn delete_certificate_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Html<String> {
    match repository::delete_certificate(&state, &id).await {
        Ok(()) => Html(String::new()),
        Err(e) => Html(format!("<span class='text-error'>{}</span>", e)),
    }
}

/// Revoke a certificate
async fn revoke_certificate(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<String>,
) -> Html<String> {
    let lang = Language::resolve(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    match repository::revoke_certificate(&state, &id).await {
        Ok(certificate) => {
            let tmpl = CertificateRowTemplate { certificate, t };
            Html(
                tmpl.render()
                    .unwrap_or_else(|e| format!("<tr><td colspan='7'>{}</td></tr>", e)),
            )
        }
        Err(e) => Html(format!("<span class='text-error'>{}</span>", e)),
    }
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_certificate))
        .route("/", get(list_certificates))
        .route("/{id}", delete(delete_certificate_handler))
        .route("/{id}/revoke", post(revoke_certificate))
}
