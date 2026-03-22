use std::env;

use crate::error::AppError;

/// Central application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    // ── SurrealDB ──────────────────────────────────────────────
    pub surrealdb_url: String,
    pub surrealdb_user: String,
    pub surrealdb_pass: String,
    pub surrealdb_namespace: String,
    pub platform_db_name: String,

    // ── JWT ────────────────────────────────────────────────────
    pub jwt_secret: String,
    pub jwt_access_lifetime_secs: u64,
    pub jwt_refresh_lifetime_secs: u64,

    // ── Server ─────────────────────────────────────────────────
    pub host: String,
    pub port: u16,

    // ── Tenant resolution ──────────────────────────────────────
    pub tenant_resolution_mode: String,
    pub base_domain: String,
    pub tenant_pool_max_size: usize,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// Calls `dotenvy::dotenv().ok()` first so that a `.env` file is picked up
    /// if present, then reads each variable (falling back to hard-coded defaults
    /// where one is specified).
    pub fn from_env() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            surrealdb_url: require_env("SURREALDB_URL")?,
            surrealdb_user: require_env("SURREALDB_USER")?,
            surrealdb_pass: require_env("SURREALDB_PASS")?,
            surrealdb_namespace: env_or("SURREALDB_NAMESPACE", "actuators"),
            platform_db_name: env_or("PLATFORM_DB_NAME", "platform"),

            jwt_secret: require_env("JWT_SECRET")?,
            jwt_access_lifetime_secs: env_or_parse("JWT_ACCESS_LIFETIME_SECS", 900)?,
            jwt_refresh_lifetime_secs: env_or_parse("JWT_REFRESH_LIFETIME_SECS", 2_592_000)?,

            host: env_or("HOST", "0.0.0.0"),
            port: env_or_parse("PORT", 8080)?,

            tenant_resolution_mode: env_or("TENANT_RESOLUTION_MODE", "both"),
            base_domain: env_or("BASE_DOMAIN", "localhost:8080"),
            tenant_pool_max_size: env_or_parse("TENANT_POOL_MAX_SIZE", 50)?,
        })
    }
}

// ── helpers ────────────────────────────────────────────────────────

/// Read a required environment variable or return `AppError::Config`.
fn require_env(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::Config(format!("missing required env var: {key}")))
}

/// Read an environment variable, falling back to `default` when absent.
fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Read an environment variable and parse it, falling back to `default`.
fn env_or_parse<T>(key: &str, default: T) -> Result<T, AppError>
where
    T: std::str::FromStr + ToString,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|e| AppError::Config(format!("invalid value for {key}: {e}"))),
        Err(_) => Ok(default),
    }
}
