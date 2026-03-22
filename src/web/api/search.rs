//! Global Search API
//!
//! Full-text search across all entities for Command Palette (Ctrl+K).

use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

use crate::db::AppState;
use crate::models::CurrentUser;

/// Search query parameters.
#[derive(Deserialize)]
pub struct SearchQuery {
    /// Search query string.
    pub q: String,
}

/// Search result item.
#[derive(Serialize, Deserialize, SurrealValue, Clone)]
pub struct SearchResult {
    /// Entity ID.
    pub id: String,
    /// Display name/title.
    pub name: String,
    /// Entity type (employee, asset, machine, etc.).
    pub entity_type: String,
    /// URL to navigate to.
    pub url: String,
    /// Optional subtitle for context.
    pub subtitle: Option<String>,
}

/// GET /api/search?q=query
///
/// Returns HTMX fragment with search results.
pub async fn global_search(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(params): Query<SearchQuery>,
) -> Html<String> {
    let query = params.q.trim();

    if query.len() < 2 {
        return Html(r#"<div class="text-base-content/50 text-center py-4">اكتب حرفين على الأقل للبحث...</div>"#.to_string());
    }

    let pattern = format!("%{}%", query);

    // Search across multiple tables
    let results = search_all_entities(&state, pattern, user.organization_id.as_deref()).await;

    if results.is_empty() {
        return Html(format!(
            r#"<div class="text-base-content/50 text-center py-4">لا توجد نتائج لـ "{}"</div>"#,
            query
        ));
    }

    // Render results as HTMX fragment
    Html(render_search_results(&results))
}

/// Search all entity types and combine results.
async fn search_all_entities(state: &AppState, pattern: String, org_id: Option<&str>) -> Vec<SearchResult> {
    let mut all_results = Vec::new();

    // Search employees
    if let Ok(employees) = search_employees(state, pattern.clone(), org_id).await {
        all_results.extend(employees);
    }

    // Search assets
    if let Ok(assets) = search_assets(state, pattern.clone(), org_id).await {
        all_results.extend(assets);
    }

    // Search machines
    if let Ok(machines) = search_machines(state, pattern.clone(), org_id).await {
        all_results.extend(machines);
    }

    // Search clients
    if let Ok(clients) = search_clients(state, pattern.clone(), org_id).await {
        all_results.extend(clients);
    }

    // Search invoices
    if let Ok(invoices) = search_invoices(state, pattern.clone(), org_id).await {
        all_results.extend(invoices);
    }

    // Search projects
    if let Ok(projects) = search_projects(state, pattern.clone(), org_id).await {
        all_results.extend(projects);
    }

    // Search certificates (no org filter — certificates are cross-org)
    if let Ok(certificates) = search_certificates(state, pattern).await {
        all_results.extend(certificates);
    }

    // Limit total results
    all_results.truncate(15);
    all_results
}

async fn search_employees(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct EmployeeResult {
        id: surrealdb::types::RecordId,
        name: String,
        role: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, name, role FROM employee WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, name, role FROM employee WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<EmployeeResult> = state
        .db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|e| SearchResult {
            id: crate::db::record_id_to_raw(&e.id),
            name: e.name,
            entity_type: "employee".to_string(),
            url: format!("/admin/employees/{}", crate::db::record_id_to_raw(&e.id)),
            subtitle: e.role,
        })
        .collect())
}

async fn search_assets(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct AssetResult {
        id: surrealdb::types::RecordId,
        name: String,
        category: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, name, category FROM asset WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, name, category FROM asset WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<AssetResult> = state
        .db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|a| SearchResult {
            id: crate::db::record_id_to_raw(&a.id),
            name: a.name,
            entity_type: "asset".to_string(),
            url: format!("/admin/assets/{}", crate::db::record_id_to_raw(&a.id)),
            subtitle: a.category,
        })
        .collect())
}

async fn search_machines(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct MachineResult {
        id: surrealdb::types::RecordId,
        serial_number: String,
        model: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, serial_number, model FROM machine WHERE (serial_number CONTAINS $pattern OR model CONTAINS $pattern) AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, serial_number, model FROM machine WHERE (serial_number CONTAINS $pattern OR model CONTAINS $pattern) AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<MachineResult> = state.db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|m| SearchResult {
            id: crate::db::record_id_to_raw(&m.id),
            name: m.serial_number,
            entity_type: "machine".to_string(),
            url: format!("/admin/machines/{}", crate::db::record_id_to_raw(&m.id)),
            subtitle: m.model,
        })
        .collect())
}

async fn search_clients(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct ClientResult {
        id: surrealdb::types::RecordId,
        name: String,
        phone: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, name, phone FROM client WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, name, phone FROM client WHERE name CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<ClientResult> = state
        .db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|c| SearchResult {
            id: crate::db::record_id_to_raw(&c.id),
            name: c.name,
            entity_type: "client".to_string(),
            url: format!("/admin/customers/{}", crate::db::record_id_to_raw(&c.id)),
            subtitle: c.phone,
        })
        .collect())
}

async fn search_invoices(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct InvoiceResult {
        id: surrealdb::types::RecordId,
        invoice_number: String,
        client_name: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, invoice_number, client_name FROM invoice WHERE invoice_number CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, invoice_number, client_name FROM invoice WHERE invoice_number CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<InvoiceResult> = state
        .db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|i| SearchResult {
            id: crate::db::record_id_to_raw(&i.id),
            name: i.invoice_number,
            entity_type: "invoice".to_string(),
            url: format!("/admin/invoices/{}", crate::db::record_id_to_raw(&i.id)),
            subtitle: i.client_name,
        })
        .collect())
}

async fn search_projects(state: &AppState, pattern: String, org_id: Option<&str>) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct ProjectResult {
        id: surrealdb::types::RecordId,
        title: String,
        customer_name: Option<String>,
    }

    let query = if org_id.is_some() {
        "SELECT id, title, customer_name FROM project WHERE title CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) AND organization = type::record($org) LIMIT 5"
    } else {
        "SELECT id, title, customer_name FROM project WHERE title CONTAINS $pattern AND (is_archived = false OR is_archived = NONE) LIMIT 5"
    };

    let results: Vec<ProjectResult> = state
        .db
        .query(query)
        .bind(("pattern", pattern))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|p| SearchResult {
            id: crate::db::record_id_to_raw(&p.id),
            name: p.title,
            entity_type: "project".to_string(),
            url: format!("/admin/projects/{}", crate::db::record_id_to_raw(&p.id)),
            subtitle: p.customer_name,
        })
        .collect())
}

async fn search_certificates(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize, SurrealValue)]
    struct CertificateResult {
        id: surrealdb::types::RecordId,
        credential_id: String,
        trainee_name: String,
    }

    let results: Vec<CertificateResult> = state.db
        .query("SELECT id, credential_id, trainee_name FROM certificate WHERE (credential_id CONTAINS $pattern OR trainee_name CONTAINS $pattern) AND (is_archived = false OR is_archived = NONE) LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|c| SearchResult {
            id: crate::db::record_id_to_raw(&c.id),
            name: c.credential_id,
            entity_type: "certificate".to_string(),
            url: format!("/admin/certificates/{}", crate::db::record_id_to_raw(&c.id)),
            subtitle: Some(c.trainee_name),
        })
        .collect())
}

/// Render search results as HTML.
fn render_search_results(results: &[SearchResult]) -> String {
    let mut html = String::from(r#"<ul class="menu bg-base-200 rounded-box">"#);

    for result in results {
        let icon = get_entity_icon(&result.entity_type);
        let type_label = get_entity_label(&result.entity_type);
        let subtitle = result.subtitle.as_deref().unwrap_or("");

        html.push_str(&format!(
            r#"<li>
                <a href="{}" class="flex items-center gap-3">
                    <span class="text-xl">{}</span>
                    <div class="flex-1">
                        <div class="font-medium">{}</div>
                        <div class="text-xs text-base-content/50">{} {}</div>
                    </div>
                    <kbd class="kbd kbd-sm">{}</kbd>
                </a>
            </li>"#,
            result.url,
            icon,
            result.name,
            type_label,
            subtitle,
            result
                .entity_type
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
        ));
    }

    html.push_str("</ul>");
    html
}

fn get_entity_icon(entity_type: &str) -> &'static str {
    match entity_type {
        "employee" => "👤",
        "asset" => "🔧",
        "machine" => "⚙️",
        "client" => "🏢",
        "invoice" => "🧾",
        "project" => "📁",
        "certificate" => "🎓",
        _ => "📄",
    }
}

fn get_entity_label(entity_type: &str) -> &'static str {
    match entity_type {
        "employee" => "موظف",
        "asset" => "أصل",
        "machine" => "آلة",
        "client" => "عميل",
        "invoice" => "فاتورة",
        "project" => "مشروع",
        "certificate" => "شهادة",
        _ => "",
    }
}
