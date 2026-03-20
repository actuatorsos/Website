//! Web Module
//!
//! طبقة الويب - Routes و Handlers

pub mod admin;
pub mod api;
pub mod public;

use askama::Template;
use axum::Router;
use axum::response::{Html, IntoResponse};

use crate::db::AppState;

// ============================================================================
// Error page templates
// ============================================================================

#[derive(Template)]
#[template(path = "errors/404.html")]
struct NotFoundTemplate {}

/// Fallback handler for unmatched routes — serves the 404 error page
async fn fallback_404() -> impl IntoResponse {
    let template = NotFoundTemplate {};
    (
        axum::http::StatusCode::NOT_FOUND,
        Html(
            template
                .render()
                .unwrap_or_else(|_| "404 - Page Not Found".to_string()),
        ),
    )
}

/// Build the complete application router
pub fn build_router(state: AppState) -> Router {
    // Initialize WebSocket state with broadcast channels
    let (asset_tx, _) = tokio::sync::broadcast::channel(100);
    let (repair_tx, _) = tokio::sync::broadcast::channel(100);
    let (device_tx, _) = tokio::sync::broadcast::channel(100);
    let (notification_tx, _) = tokio::sync::broadcast::channel(100);
    let live_update_state = crate::ws::live_updates::LiveUpdateState {
        asset_tx,
        repair_tx,
        device_tx,
        notification_tx,
    };

    let ws_router = Router::new()
        .route(
            "/ws/live",
            axum::routing::get(crate::ws::live_updates_handler),
        )
        .with_state(live_update_state);

    Router::new()
        .merge(public::routes())
        .merge(admin::public_routes())
        .nest("/admin", admin::routes())
        .nest("/admin", crate::domains::customers::handlers::routes())
        .nest("/admin", crate::domains::hr::handlers::routes())
        .nest("/admin", crate::domains::machinery::handlers::routes())
        .nest("/admin", crate::domains::finance::handlers::routes())
        .nest("/admin", crate::domains::videos::handlers::routes()
            .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024)))
        .nest("/api", api::routes(state.clone()))
        .nest("/api/devices", api::device_routes(state.clone()))
        .merge(api::health::routes())
        .with_state(state)
        .merge(ws_router)
        .fallback(fallback_404)
        // حد أقصى افتراضي لحجم الطلب: 2MB (مسارات الفيديو لها حد خاص بها)
        .layer(axum::extract::DefaultBodyLimit::max(
            2 * 1024 * 1024,
        ))
}
