//! Global Search API
//!
//! Full-text search across all entities for Command Palette (Ctrl+K).

use axum::{
    extract::{Query, State},
    response::Html,
};
use serde::{Deserialize, Serialize};

use crate::db::AppState;

/// Search query parameters.
#[derive(Deserialize)]
pub struct SearchQuery {
    /// Search query string.
    pub q: String,
}

/// Search result item.
#[derive(Serialize, Deserialize, Clone)]
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
    Query(params): Query<SearchQuery>,
) -> Html<String> {
    let query = params.q.trim();

    if query.len() < 2 {
        return Html(r#"<div class="text-base-content/50 text-center py-4">اكتب حرفين على الأقل للبحث...</div>"#.to_string());
    }

    let pattern = format!("%{}%", query);

    // Search across multiple tables
    let results = search_all_entities(&state, pattern).await;

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
async fn search_all_entities(state: &AppState, pattern: String) -> Vec<SearchResult> {
    let mut all_results = Vec::new();

    // Search employees
    if let Ok(employees) = search_employees(state, pattern.clone()).await {
        all_results.extend(employees);
    }

    // Search assets
    if let Ok(assets) = search_assets(state, pattern.clone()).await {
        all_results.extend(assets);
    }

    // Search machines
    if let Ok(machines) = search_machines(state, pattern.clone()).await {
        all_results.extend(machines);
    }

    // Search clients
    if let Ok(clients) = search_clients(state, pattern.clone()).await {
        all_results.extend(clients);
    }

    // Search invoices
    if let Ok(invoices) = search_invoices(state, pattern.clone()).await {
        all_results.extend(invoices);
    }

    // Search projects
    if let Ok(projects) = search_projects(state, pattern.clone()).await {
        all_results.extend(projects);
    }

    // Search certificates
    if let Ok(certificates) = search_certificates(state, pattern).await {
        all_results.extend(certificates);
    }

    // Limit total results
    all_results.truncate(15);
    all_results
}

async fn search_employees(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct EmployeeResult {
        id: surrealdb::sql::Thing,
        name: String,
        role: Option<String>,
    }

    let results: Vec<EmployeeResult> = state
        .db
        .query("SELECT id, name, role FROM employee WHERE name CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|e| SearchResult {
            id: e.id.id.to_string(),
            name: e.name,
            entity_type: "employee".to_string(),
            url: format!("/admin/employees/{}", e.id.id),
            subtitle: e.role,
        })
        .collect())
}

async fn search_assets(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct AssetResult {
        id: surrealdb::sql::Thing,
        name: String,
        category: Option<String>,
    }

    let results: Vec<AssetResult> = state
        .db
        .query("SELECT id, name, category FROM asset WHERE name CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|a| SearchResult {
            id: a.id.id.to_string(),
            name: a.name,
            entity_type: "asset".to_string(),
            url: format!("/admin/assets/{}", a.id.id),
            subtitle: a.category,
        })
        .collect())
}

async fn search_machines(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct MachineResult {
        id: surrealdb::sql::Thing,
        serial_number: String,
        model: Option<String>,
    }

    let results: Vec<MachineResult> = state.db
        .query("SELECT id, serial_number, model FROM machine WHERE serial_number CONTAINS $pattern OR model CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|m| SearchResult {
            id: m.id.id.to_string(),
            name: m.serial_number,
            entity_type: "machine".to_string(),
            url: format!("/admin/machines/{}", m.id.id),
            subtitle: m.model,
        })
        .collect())
}

async fn search_clients(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct ClientResult {
        id: surrealdb::sql::Thing,
        name: String,
        phone: Option<String>,
    }

    let results: Vec<ClientResult> = state
        .db
        .query("SELECT id, name, phone FROM client WHERE name CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|c| SearchResult {
            id: c.id.id.to_string(),
            name: c.name,
            entity_type: "client".to_string(),
            url: format!("/admin/customers/{}", c.id.id),
            subtitle: c.phone,
        })
        .collect())
}

async fn search_invoices(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct InvoiceResult {
        id: surrealdb::sql::Thing,
        invoice_number: String,
        client_name: Option<String>,
    }

    let results: Vec<InvoiceResult> = state
        .db
        .query("SELECT id, invoice_number, client_name FROM invoice WHERE invoice_number CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|i| SearchResult {
            id: i.id.id.to_string(),
            name: i.invoice_number,
            entity_type: "invoice".to_string(),
            url: format!("/admin/invoices/{}", i.id.id),
            subtitle: i.client_name,
        })
        .collect())
}

async fn search_projects(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct ProjectResult {
        id: surrealdb::sql::Thing,
        title: String,
        customer_name: Option<String>,
    }

    let results: Vec<ProjectResult> = state
        .db
        .query("SELECT id, title, customer_name FROM project WHERE title CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|p| SearchResult {
            id: p.id.id.to_string(),
            name: p.title,
            entity_type: "project".to_string(),
            url: format!("/admin/projects/{}", p.id.id),
            subtitle: p.customer_name,
        })
        .collect())
}

async fn search_certificates(state: &AppState, pattern: String) -> Result<Vec<SearchResult>, ()> {
    #[derive(serde::Deserialize)]
    struct CertificateResult {
        id: surrealdb::sql::Thing,
        credential_id: String,
        trainee_name: String,
    }

    let results: Vec<CertificateResult> = state.db
        .query("SELECT id, credential_id, trainee_name FROM certificate WHERE credential_id CONTAINS $pattern OR trainee_name CONTAINS $pattern LIMIT 5")
        .bind(("pattern", pattern))
        .await
        .map_err(|_| ())?
        .take(0)
        .map_err(|_| ())?;

    Ok(results
        .into_iter()
        .map(|c| SearchResult {
            id: c.id.id.to_string(),
            name: c.credential_id,
            entity_type: "certificate".to_string(),
            url: format!("/admin/certificates/{}", c.id.id),
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
