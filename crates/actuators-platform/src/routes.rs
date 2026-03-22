use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use actuators_auth::middleware::AppState;

use crate::{auth, dashboard, investor, registration, tenants};

/// Assemble all platform-level HTTP routes into a single [`Router`].
///
/// These routes cover user registration, authentication, the personal
/// dashboard, tenant management, invitation acceptance, and super-admin
/// views.
pub fn platform_routes() -> Router<Arc<AppState>> {
    Router::new()
        // ── Public routes (no auth required) ────────────────────────
        .route("/register", get(registration::register_page).post(registration::register))
        .route("/verify-email", get(registration::verify_email))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/refresh-token", post(auth::refresh_token))
        // ── Authenticated routes ────────────────────────────────────
        .route("/dashboard", get(dashboard::dashboard_page))
        .route("/dashboard/profile", get(dashboard::profile_page).post(dashboard::update_profile))
        .route("/tenants/new", get(tenants::create_page).post(tenants::create_tenant))
        .route("/tenants/new/plan", get(tenants::select_plan_page))
        .route("/tenants/check-slug", post(tenants::check_slug))
        .route("/tenants/{slug}/enter", post(tenants::enter_tenant))
        // ── Super admin routes ──────────────────────────────────────
        .route("/super/tenants", get(investor::list_tenants))
        .route("/super/investor-dashboard", get(investor::investor_dashboard))
        // ── Invitation routes ───────────────────────────────────────
        .route("/invitations/{token}", get(tenants::invitation_page).post(tenants::accept_invitation))
}
