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

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, env::VarError> {
        let config = Self {
            db: DbConfig {
                url: env::var("SURREAL_URL").unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string()),
                user: env::var("SURREAL_USER")?,
                pass: env::var("SURREAL_PASS")?,
                namespace: env::var("SURREAL_NS").unwrap_or_else(|_| "dr_machine".to_string()),
                database: env::var("SURREAL_DB").unwrap_or_else(|_| "main".to_string()),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
            },
            admin: AdminConfig {
                user: env::var("ADMIN_USER")?,
                pass: env::var("ADMIN_PASS")?,
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

        // Warn if using default JWT secret (insecure for production)
        if config.jwt.secret == DEFAULT_JWT_SECRET || config.jwt.secret.len() < 32 {
            eprintln!("⚠️  WARNING: JWT_SECRET is using the default/weak value!");
            eprintln!("   This is acceptable for development, but MUST be changed in production.");
            eprintln!("   Set a strong random JWT_SECRET of at least 64 characters.");

            // Block startup in production (when RUST_LOG is not debug)
            let log_level = env::var("RUST_LOG").unwrap_or_default();
            if !log_level.contains("debug") && env::var("ALLOW_INSECURE_JWT").is_err() {
                eprintln!("   Set ALLOW_INSECURE_JWT=1 to bypass this check (NOT recommended).");
                return Err(env::VarError::NotPresent);
            }
        }

        Ok(config)
    }
}
