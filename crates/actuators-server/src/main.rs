use std::sync::Arc;

use axum::middleware;
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

use actuators_auth::middleware::AppState;
use actuators_core::config::AppConfig;
use actuators_db::{platform::PlatformDb, pool::TenantConnectionPool};
use actuators_middleware::tenant_resolver::resolve_tenant;
use actuators_platform::routes::platform_routes;
use actuators_tenant::routes::tenant_routes;

#[tokio::main]
async fn main() {
    // ── Logging ───────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("starting Actuators ERP server");

    // ── Configuration ─────────────────────────────────────────────────
    let config = AppConfig::from_env().expect("failed to load configuration");
    let bind_addr = format!("{}:{}", config.host, config.port);

    // ── Database connections ──────────────────────────────────────────
    tracing::info!(url = %config.surrealdb_url, "connecting to SurrealDB");
    let platform_db = PlatformDb::connect(&config)
        .await
        .expect("failed to connect to platform database");

    let tenant_pool = TenantConnectionPool::new(config.clone());

    // ── Shared state ─────────────────────────────────────────────────
    let state = Arc::new(AppState {
        config: config.clone(),
        platform_db: Arc::new(platform_db),
        tenant_pool: Arc::new(tenant_pool),
    });

    // ── Static file serving (existing marketing site) ────────────────
    let static_service = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // ── Router assembly ──────────────────────────────────────────────
    let app = axum::Router::new()
        // Platform routes (registration, login, dashboard, tenants, super admin)
        .merge(platform_routes())
        // Tenant routes (/app/*) — all go through the tenant resolver middleware
        .merge(tenant_routes())
        // Tenant resolver middleware for /app/* routes
        .layer(middleware::from_fn_with_state(state.clone(), resolve_tenant))
        // Global middleware layers
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        // Serve existing static marketing files as fallback
        .fallback_service(static_service)
        // Inject shared state
        .with_state(state);

    // ── Start server ─────────────────────────────────────────────────
    tracing::info!(addr = %bind_addr, "listening");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

/// Wait for Ctrl+C to initiate graceful shutdown.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    tracing::info!("shutdown signal received, draining connections");
}
