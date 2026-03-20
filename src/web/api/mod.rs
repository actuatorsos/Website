//! API Routes
//!
//! نقاط النهاية للـ HTMX و REST API

mod accounts;
mod assets;
mod attendance;
pub mod auth;
pub mod backup;
mod certificates;
pub mod charts;
mod clients;
pub mod devices;
mod employees;
mod events;
pub mod export;
pub mod health;
mod organizations;
mod screenshot;
mod invoices;
mod machines;
mod pdf;
mod projects;
mod repairs;
pub mod search;
pub mod stats;
mod trainees;

use axum::Router;

use crate::db::AppState;
use crate::middleware::auth::require_auth;
use crate::middleware::device_auth::device_auth_middleware;

// ── New domain handlers ──────────────────────────────────────────
use crate::domains::agent::handlers as agent_handlers;
use crate::domains::catalog::handlers as catalog_handlers;
use crate::domains::crm::handlers as crm_handlers;
use crate::domains::documents::handlers as documents_handlers;
use crate::domains::email::handlers as email_handlers;
use crate::domains::hr_compliance::handlers as compliance_handlers;
use crate::domains::hr_org::handlers as hr_org_handlers;
use crate::domains::inventory_adv::handlers as inventory_handlers;
use crate::domains::leave::handlers as leave_handlers;
use crate::domains::manufacturing::handlers as manufacturing_handlers;
use crate::domains::payroll_adv::handlers as payroll_handlers;
use crate::domains::projects_adv::handlers as projects_adv_handlers;
use crate::domains::store::handlers as store_handlers;
use crate::domains::training::handlers as training_handlers;

// ============================================================================
// Routes
// ============================================================================

/// Configures routes for the API module
pub fn routes(state: AppState) -> Router<AppState> {
    // Public routes — no JWT required
    let public = Router::new()
        .merge(health::routes())
        .nest("/auth", auth::public_routes())
        .nest("/store", store_handlers::public_routes())
        .nest("/screenshot", screenshot::routes());

    // Protected routes — JWT required
    let protected = Router::new()
        .nest("/auth", auth::protected_routes())
        // Chart API routes
        .route(
            "/charts/assets-by-status",
            axum::routing::get(charts::assets_by_status),
        )
        .route(
            "/charts/repairs-by-month",
            axum::routing::get(charts::repairs_by_month),
        )
        .route(
            "/charts/employee-workload",
            axum::routing::get(charts::employee_workload),
        )
        .route(
            "/charts/asset-value-by-category",
            axum::routing::get(charts::asset_value_by_category),
        )
        // Search API
        .route("/search", axum::routing::get(search::global_search))
        // Export API — CSV download
        .route("/export", axum::routing::get(export::export_csv))
        // Stats API for dashboard cards
        .route(
            "/stats/employees",
            axum::routing::get(stats::employees_count),
        )
        .route("/stats/assets", axum::routing::get(stats::assets_count))
        .route("/stats/machines", axum::routing::get(stats::machines_count))
        .route("/stats/repairs", axum::routing::get(stats::repairs_count))
        // Devices Management
        .route(
            "/devices/register",
            axum::routing::post(devices::register_device),
        )
        .route("/devices", axum::routing::get(devices::list_devices))
        // ── Existing HTMX CRUD (Basic Auth usually) ───────────────────
        .nest("/clients", clients::routes())
        .nest("/employees", employees::routes())
        .nest("/trainees", trainees::routes())
        .nest("/attendance", attendance::routes())
        .nest("/machines", machines::routes())
        .nest("/assets", assets::routes())
        .nest("/projects", projects::routes())
        .nest("/repairs", repairs::routes())
        .nest("/invoices", invoices::routes())
        .nest("/certificates", certificates::routes())
        .nest("/organizations", organizations::routes())
        .nest("/events", events::routes())
        .nest("/pdf", pdf::routes())
        .nest("/accounts", accounts::routes())
        .nest("/projects-adv", projects_adv_handlers::routes())
        .nest("/documents", documents_handlers::routes())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    // Advanced modules — require Manager role
    let manager_routes = Router::new()
        .nest("/crm", crm_handlers::routes())
        .nest("/catalog", catalog_handlers::routes())
        .nest("/inventory-adv", inventory_handlers::routes())
        .nest("/leave", leave_handlers::routes())
        .nest("/training", training_handlers::routes())
        .nest("/manufacturing", manufacturing_handlers::routes())
        .route_layer(axum::middleware::from_fn(
            crate::middleware::auth::require_manager,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    // Highly sensitive modules — require Admin role
    let admin_routes = Router::new()
        .nest("/hr-org", hr_org_handlers::routes())
        .nest("/payroll", payroll_handlers::routes())
        .nest("/compliance", compliance_handlers::routes())
        .nest("/email", email_handlers::routes())
        .nest("/agents", agent_handlers::admin_routes())
        .nest("/store-admin", store_handlers::admin_routes())
        .route("/backup", axum::routing::get(backup::export_backup))
        .route_layer(axum::middleware::from_fn(
            crate::middleware::auth::require_admin,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    // Notification routes — for authenticated users
    let notification_routes = Router::new()
        .nest("/notifications", agent_handlers::notification_routes())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_auth,
        ));

    // AI Agent action routes — authenticated via API Key (not JWT)
    let agent_action_routes = Router::new()
        .nest("/agent-actions", agent_handlers::agent_action_routes())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::agent_auth::require_agent_key,
        ));

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(manager_routes)
        .merge(admin_routes)
        .merge(notification_routes)
        .merge(agent_action_routes)
}

/// Device API routes (separate for different auth middleware)
pub fn device_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/status", axum::routing::post(devices::report_status))
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            device_auth_middleware,
        ))
}
