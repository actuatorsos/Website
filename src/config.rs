// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Application Configuration
//!
//! إعدادات التطبيق من متغيرات البيئة

use std::env;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// SurrealDB WebSocket URL
    pub url: String,
    /// Database username
    pub user: String,
    /// Database password
    pub pass: String,
    /// SurrealDB namespace
    pub namespace: String,
    /// SurrealDB database name
    pub database: String,
}

/// Server configuration  
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind host address
    pub host: String,
    /// Bind port
    pub port: u16,
}

/// Admin authentication credentials (legacy Basic Auth fallback)
#[derive(Debug, Clone)]
pub struct AdminConfig {
    /// Admin username
    pub user: String,
    /// Admin password
    pub pass: String,
}

/// JWT authentication configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// HMAC secret key for signing tokens
    pub secret: String,
    /// Token expiry duration in hours
    pub expiry_hours: i64,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Database settings
    pub db: DbConfig,
    /// Server settings
    pub server: ServerConfig,
    /// Legacy admin credentials
    pub admin: AdminConfig,
    /// JWT settings
    pub jwt: JwtConfig,
}

/// Default (insecure) JWT secret — used only when JWT_SECRET env var is not set.
const DEFAULT_JWT_SECRET: &str = "CHANGE_ME_IN_PRODUCTION_64_CHARS_MINIMUM_SECRET_KEY_HERE_NOW";

/// Helper: try primary env var, then fallback, then default
fn env_chain(primary: &str, fallback: &str, default: &str) -> String {
    env::var(primary)
        .or_else(|_| env::var(fallback))
        .unwrap_or_else(|_| default.to_string())
}

/// Helper: try primary then fallback, error if neither set
fn env_require(primary: &str, fallback: &str) -> Result<String, env::VarError> {
    env::var(primary).or_else(|_| env::var(fallback))
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Supports both native names (SURREAL_*) and Digital Ocean names (DATABASE_URL, DB_*).
    pub fn from_env() -> Result<Self, env::VarError> {
        let config = Self {
            db: DbConfig {
                url: env_chain("SURREAL_URL", "DATABASE_URL", "ws://127.0.0.1:8000"),
                user: env_require("SURREAL_USER", "DB_USER")?,
                pass: env_require("SURREAL_PASS", "DB_PASS")?,
                namespace: env_chain("SURREAL_NS", "DB_NS", "actuators"),
                database: env_chain("SURREAL_DB", "DB_NAME", "platform"),
            },
            server: ServerConfig {
                host: env_chain("SERVER_HOST", "HOST", "0.0.0.0"),
                port: env::var("SERVER_PORT")
                    .or_else(|_| env::var("PORT"))
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            admin: AdminConfig {
                user: env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string()),
                pass: env::var("ADMIN_PASS").unwrap_or_else(|_| "admin".to_string()),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                    DEFAULT_JWT_SECRET.to_string()
                }),
                expiry_hours: env::var("JWT_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
        };

        Ok(config)
    }
}
