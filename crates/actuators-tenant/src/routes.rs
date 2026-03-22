use std::sync::Arc;

use axum::{middleware, Router, routing::{get, post}};

use actuators_auth::middleware::AppState;
use actuators_middleware::service_gate::require_service;

use crate::{accounts, audit, auth, departments, employees, settings};

/// Build the tenant-scoped router.
///
/// All routes live under the `/app` prefix and require a resolved
/// [`TenantContext`](actuators_core::types::TenantContext) in the
/// request extensions (set by the tenant-resolver middleware).
pub fn tenant_routes() -> Router<Arc<AppState>> {
    // ── HR routes (service-gated to "hr") ────────────────────────
    let hr_routes = Router::new()
        // Employees
        .route("/app/hr/employees", get(employees::list).post(employees::create))
        .route("/app/hr/employees/new", get(employees::create_page))
        .route("/app/hr/employees/{id}", get(employees::detail).post(employees::update))
        .route("/app/hr/employees/{id}/edit", get(employees::edit_page))
        .route("/app/hr/employees/{id}/terminate", post(employees::terminate))
        .route(
            "/app/hr/employees/{id}/contracts",
            get(employees::list_contracts).post(employees::create_contract),
        )
        // Departments
        .route("/app/hr/departments", get(departments::list).post(departments::create))
        .route("/app/hr/departments/new", get(departments::create_page))
        .route("/app/hr/departments/{id}", get(departments::detail).post(departments::update))
        .route_layer(middleware::from_fn(require_service("hr")));

    // ── Assemble ─────────────────────────────────────────────────
    Router::new()
        // Auth / dashboard
        .route("/app", get(auth::tenant_dashboard))
        .route("/app/login", get(auth::tenant_login_page).post(auth::tenant_login))
        .route("/app/logout", post(auth::tenant_logout))
        // Accounts (admin/manager in-handler check)
        .route("/app/accounts", get(accounts::list).post(accounts::create))
        .route("/app/accounts/new", get(accounts::create_page))
        .route("/app/accounts/{id}", get(accounts::detail).post(accounts::update))
        // Settings (admin-only in-handler check)
        .route("/app/settings", get(settings::index).post(settings::update))
        // Audit log
        .route("/app/audit", get(audit::list))
        // Merge HR sub-router (service-gated)
        .merge(hr_routes)
}
