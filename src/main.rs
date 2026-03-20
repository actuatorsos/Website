//! Dr. Machine Web Application - Entry Point
#![warn(missing_docs)]
//!
//! نقطة الدخول الرئيسية للتطبيق

use std::net::SocketAddr;

use Actuators::config::AppConfig;
use Actuators::db::AppState;
use Actuators::web::build_router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dr_machine_web=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!("Could not load .env file: {}", e);
    }

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Initialize database connection and app state
    let state = AppState::new(&config).await?;
    tracing::info!("Database connection established");

    // Seed default data (runs only if data doesn't exist)
    Actuators::db::seed::seed_all(&state.db).await;
    tracing::info!("Seed data ready");

    // Build router
    let app = build_router(state)
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .layer(tower_cookies::CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid server address");

    tracing::info!("🚀 Server starting at http://{}", addr);

    // Spawn background task to periodically clean up rate limiter state (every 5 minutes)
    tokio::spawn(async {
        let rate_limit_state = Actuators::middleware::rate_limit::new_rate_limit_state();
        let cleanup_interval = std::time::Duration::from_secs(300);
        let max_age = std::time::Duration::from_secs(120);
        loop {
            tokio::time::sleep(cleanup_interval).await;
            Actuators::middleware::rate_limit::cleanup_rate_limit_state(
                rate_limit_state.clone(),
                max_age,
            )
            .await;
            tracing::debug!("Rate limit state cleaned up");
        }
    });

    // Graceful shutdown on SIGINT/SIGTERM
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Received Ctrl+C, shutting down..."); },
        _ = terminate => { tracing::info!("Received SIGTERM, shutting down..."); },
    }
}
