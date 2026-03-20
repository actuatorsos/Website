//! Database Module
//!
//! إدارة الاتصال بقاعدة البيانات SurrealDB

pub mod live;
pub mod seed;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::config::AppConfig;

use crate::i18n::I18n;
use crate::models::{
    Account, Asset, AssetStatus, AssignAssetRequest, CreateAssetRequest, SignupRequest,
};

/// Database error types — أنواع أخطاء قاعدة البيانات
#[derive(Error, Debug)]
pub enum DbError {
    /// خطأ في الاتصال أو الاستعلام
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),

    /// السجل غير موجود
    #[error("Record not found")]
    NotFound,

    /// قيمة مكررة (مثل: بريد مسجل مسبقاً)
    #[error("Duplicate: {field} = {value}")]
    Duplicate { field: String, value: String },

    /// خطأ في التحقق من البيانات
    #[error("Validation error: {0}")]
    Validation(String),

    /// غير مصرح
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// تعارض منطقي (مثل: حذف فاتورة مدفوعة)
    #[error("Conflict: {0}")]
    Conflict(String),
}

/// Unified API error response — استجابة خطأ موحدة
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for DbError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            DbError::Database(e) => {
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("assert")
                    || err_str.contains("field")
                    || err_str.contains("index")
                    || err_str.contains("validation")
                {
                    (
                        StatusCode::BAD_REQUEST,
                        "VALIDATION_ERROR".to_string(),
                        format!("Data validation failed: {}", e),
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "DATABASE_ERROR".to_string(),
                        format!("Internal database error: {}", e),
                    )
                }
            }
            DbError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND".to_string(),
                "Record not found".to_string(),
            ),
            DbError::Duplicate { field, value } => (
                StatusCode::CONFLICT,
                "DUPLICATE".to_string(),
                format!("Duplicate {}: {}", field, value),
            ),
            DbError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                msg.clone(),
            ),
            DbError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, "FORBIDDEN".to_string(), msg.clone())
            }
            DbError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT".to_string(), msg.clone()),
        };
        (
            status,
            Json(ApiErrorResponse {
                code,
                message,
                details: None,
            }),
        )
            .into_response()
    }
}

// ============================================================================
// Pagination — ترقيم الصفحات
// ============================================================================

/// Pagination query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedQuery {
    /// Number of items per page (default: 25, max: 100)
    pub limit: Option<u32>,
    /// Page number (1-indexed, default: 1)
    pub page: Option<u32>,
    /// Search/filter term
    pub search: Option<String>,
}

impl PaginatedQuery {
    /// Get the validated limit
    pub fn get_limit(&self) -> u32 {
        self.limit.unwrap_or(25).min(100).max(1)
    }

    /// Get the offset for SQL
    pub fn get_offset(&self) -> u32 {
        (self.get_page() - 1) * self.get_limit()
    }

    /// Get the validated page number
    pub fn get_page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }
}

/// Paginated result wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T: Serialize> {
    /// Result data
    pub data: Vec<T>,
    /// Current page
    pub page: u32,
    /// Items per page
    pub limit: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub total_pages: u32,
}

impl<T: Serialize> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(data: Vec<T>, page: u32, limit: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            data,
            page,
            limit,
            total,
            total_pages,
        }
    }

    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there's a previous page
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

// ============================================================================
// Audit Trail — سجل التدقيق
// ============================================================================

/// Record an audit log entry
pub async fn audit_log(
    db: &Surreal<Client>,
    actor_id: Option<&str>,
    action: &str,
    table_name: &str,
    record_id: Option<&str>,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
) -> Result<(), DbError> {
    let _ = db
        .query(
            "CREATE audit_log SET \
            actor = $actor, action = $action, \
            table_name = $table, record_id = $rid, \
            old_data = $old, new_data = $new, \
            timestamp = time::now()",
        )
        .bind(("actor", actor_id.map(|a| format!("account:{}", a))))
        .bind(("action", action.to_string()))
        .bind(("table", table_name.to_string()))
        .bind(("rid", record_id.map(|r| r.to_string())))
        .bind(("old", old_data))
        .bind(("new", new_data))
        .await;
    Ok(())
}

// ============================================================================
// Soft Delete helper — الحذف المنطقي
// ============================================================================

/// Archive a record (soft delete) instead of physical delete
pub async fn soft_delete(db: &Surreal<Client>, table: &str, id: &str) -> Result<(), DbError> {
    // type::thing() requires the table name as a literal, not a bind variable
    let query = format!(
        "UPDATE type::thing('{}', $id) SET is_archived = true",
        table
    );
    db.query(&query).bind(("id", id.to_string())).await?;
    Ok(())
}

/// In-memory cache for dashboard statistics
pub struct StatsCache {
    pub data: HashMap<String, i64>,
    pub last_updated: Instant,
}

/// Application state containing database connection, i18n, and JWT config
#[derive(Clone)]
pub struct AppState {
    /// SurrealDB connection
    pub db: Surreal<Client>,
    /// Internationalization
    pub i18n: Arc<I18n>,
    /// JWT HMAC secret key
    pub jwt_secret: String,
    /// JWT token expiry in hours
    pub jwt_expiry_hours: i64,
    /// Broadcast channel for realtime board events (SSE)
    pub board_events: broadcast::Sender<BoardEvent>,
    /// In-memory cache for dashboard stats
    pub stats_cache: Arc<RwLock<StatsCache>>,
}

/// Event sent to SSE clients when board changes
#[derive(Clone, Debug, Serialize)]
pub struct BoardEvent {
    pub board_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
}

impl AppState {
    /// Initialize database connection and application state
    pub async fn new(config: &AppConfig) -> Result<Self, DbError> {
        tracing::info!("Connecting to SurrealDB at {}", config.db.url);

        let mut attempts = 0;
        let max_attempts = 10;

        loop {
            match Surreal::new::<Ws>(&config.db.url).await {
                Ok(db) => {
                    // Authenticate
                    if let Err(e) = db
                        .signin(Root {
                            username: &config.db.user,
                            password: &config.db.pass,
                        })
                        .await
                    {
                        tracing::warn!(
                            "Authentication failed (attempt {}/{}): {}",
                            attempts + 1,
                            max_attempts,
                            e
                        );
                    } else if let Err(e) = db
                        .use_ns(&config.db.namespace)
                        .use_db(&config.db.database)
                        .await
                    {
                        tracing::warn!(
                            "Selecting DB failed (attempt {}/{}): {}",
                            attempts + 1,
                            max_attempts,
                            e
                        );
                    } else {
                        tracing::info!("Connected to SurrealDB successfully");
                        let i18n = Arc::new(I18n::new());
                        // Create broadcast channel for SSE (capacity 100)
                        let (tx, _) = broadcast::channel(100);
                        let state = Self {
                            db,
                            i18n,
                            jwt_secret: config.jwt.secret.clone(),
                            jwt_expiry_hours: config.jwt.expiry_hours,
                            board_events: tx,
                            stats_cache: Arc::new(RwLock::new(StatsCache {
                                data: HashMap::new(),
                                last_updated: Instant::now(),
                            })),
                        };

                        // Apply database schema
                        match Self::apply_schema(&state.db).await {
                            Ok(()) => tracing::info!("📐 Database schema applied successfully"),
                            Err(e) => tracing::warn!("⚠️ Schema application had issues: {}", e),
                        }

                        // Seed default admin account if not exists
                        let admin_email = &config.admin.user;
                        if state.get_account_by_email(admin_email).await.is_err() {
                            tracing::info!(
                                "Admin account '{}' not found, seeding from config...",
                                admin_email
                            );
                            let password = config.admin.pass.clone();
                            let hashed = tokio::task::spawn_blocking(move || {
                                crate::models::hash_password(&password).unwrap_or_default()
                            })
                            .await
                            .unwrap_or_default();

                            if !hashed.is_empty() {
                                let _ = state.db.query("CREATE account SET email = $email, password_hash = $hash, role = 'admin', is_active = true, auth_methods = [], created_at = time::now()")
                                    .bind(("email", admin_email.clone()))
                                    .bind(("hash", hashed))
                                    .await;
                                tracing::info!("Admin account seeded successfully.");
                            }
                        }

                        return Ok(state);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Connection failed (attempt {}/{}): {}",
                        attempts + 1,
                        max_attempts,
                        e
                    );
                }
            }

            attempts += 1;
            if attempts >= max_attempts {
                tracing::error!(
                    "Failed to connect to SurrealDB after {} attempts",
                    max_attempts
                );
                return Err(DbError::Database(surrealdb::Error::Api(
                    surrealdb::error::Api::Query("Failed to connect to database".into()),
                )));
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    /// Apply the database schema from schema.surql
    async fn apply_schema(db: &Surreal<Client>) -> Result<(), DbError> {
        let schema_paths = [
            std::path::PathBuf::from("src/db/schema.surql"),
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("schema.surql")))
                .unwrap_or_default(),
        ];

        let mut schema_content = None;
        for path in &schema_paths {
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        tracing::info!("📄 Loading schema from: {}", path.display());
                        schema_content = Some(content);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read schema from {}: {}", path.display(), e);
                    }
                }
            }
        }

        let content = match schema_content {
            Some(c) => c,
            None => {
                tracing::warn!("No schema.surql found, skipping schema application");
                return Ok(());
            }
        };

        match db.query(&content).await {
            Ok(mut response) => {
                let errors = response.take_errors();
                if errors.is_empty() {
                    tracing::info!("✅ Schema applied: all statements executed successfully");
                } else {
                    for (idx, err) in &errors {
                        tracing::warn!("⚠️ Schema statement {} had issue: {}", idx, err);
                    }
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("❌ Schema application failed: {}", e);
                Err(DbError::Database(e))
            }
        }
    }
}

// ============================================================================
// Stats Cache
// ============================================================================

impl AppState {
    /// Get a cached row count for a table, querying the DB on cache miss or expiry.
    pub async fn get_cached_count(&self, table: &str, ttl_secs: u64) -> i64 {
        // Check cache first
        {
            let cache = self.stats_cache.read().await;
            if cache.last_updated.elapsed().as_secs() < ttl_secs {
                if let Some(&count) = cache.data.get(table) {
                    return count;
                }
            }
        }

        // Allowlist to prevent injection
        const ALLOWED: &[&str] = &["organization", "event", "employee", "project", "asset", "machine"];
        if !ALLOWED.contains(&table) {
            return 0;
        }

        // Cache miss — query DB
        #[derive(serde::Deserialize)]
        struct CountResult {
            count: i64,
        }

        let query = format!("SELECT count() as count FROM {}", table);
        let count: i64 = self
            .db
            .query(query)
            .await
            .ok()
            .and_then(|mut r| r.take::<Vec<CountResult>>(0).ok())
            .and_then(|v| v.first().map(|r| r.count))
            .unwrap_or(0);

        // Update cache
        {
            let mut cache = self.stats_cache.write().await;
            cache.data.insert(table.to_string(), count);
            // Reset timer only if cache was stale (all entries expired together)
            if cache.last_updated.elapsed().as_secs() >= ttl_secs {
                cache.last_updated = Instant::now();
            }
        }

        count
    }
}

// ============================================================================
// Account / Auth DAL
// ============================================================================

impl AppState {
    /// Create a new account with hashed password
    pub async fn create_account(
        &self,
        req: &SignupRequest,
        password_hash: &str,
    ) -> Result<Account, DbError> {
        let result: Option<Account> = self
            .db
            .query(
                "CREATE account SET \
                    email = $email, \
                    password_hash = $hash, \
                    role = 'applicant', \
                    is_active = true, \
                    email_verified = false, \
                    full_name = $full_name, \
                    phone = $phone, \
                    user_type = $user_type, \
                    auth_methods = [], \
                    avatar = None, \
                    created_at = time::now()",
            )
            .bind(("email", req.email.clone()))
            .bind(("hash", password_hash.to_string()))
            .bind(("full_name", req.full_name.clone()))
            .bind(("phone", req.phone.clone()))
            .bind(("user_type", req.user_type.clone()))
            .await
            .map_err(DbError::Database)?
            .take(0)
            .map_err(DbError::Database)?;

        result.ok_or(DbError::NotFound)
    }

    /// Get account by email address
    pub async fn get_account_by_email(&self, email: &str) -> Result<Account, DbError> {
        let mut result = self
            .db
            .query("SELECT * FROM account WHERE email = $email LIMIT 1")
            .bind(("email", email.to_string()))
            .await
            .map_err(DbError::Database)?;

        let accounts: Vec<Account> = result.take(0).map_err(DbError::Database)?;
        accounts.into_iter().next().ok_or(DbError::NotFound)
    }

    /// Get account by record ID
    pub async fn get_account_by_id(&self, id: &str) -> Result<Account, DbError> {
        let result: Option<Account> = self
            .db
            .select(("account", id))
            .await
            .map_err(DbError::Database)?;

        result.ok_or(DbError::NotFound)
    }

    /// Update last_login timestamp
    pub async fn update_last_login(&self, id: &str) -> Result<(), DbError> {
        self.db
            .query("UPDATE account SET last_login = time::now() WHERE id = $id")
            .bind(("id", format!("account:{id}")))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// List all accounts (admin only)
    pub async fn get_all_accounts(&self) -> Result<Vec<Account>, DbError> {
        let mut result = self
            .db
            .query("SELECT * FROM account ORDER BY created_at DESC")
            .await
            .map_err(DbError::Database)?;

        let accounts: Vec<Account> = result.take(0).map_err(DbError::Database)?;
        Ok(accounts)
    }

    /// Update account role
    pub async fn update_account_role(&self, id: &str, role: &str) -> Result<(), DbError> {
        self.db
            .query("UPDATE type::thing('account', $id) SET role = $role")
            .bind(("id", id.to_string()))
            .bind(("role", role.to_string()))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// Enable/disable account
    pub async fn set_account_active(&self, id: &str, active: bool) -> Result<(), DbError> {
        self.db
            .query("UPDATE type::thing('account', $id) SET is_active = $active")
            .bind(("id", id.to_string()))
            .bind(("active", active))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// Update account password
    pub async fn update_account_password(
        &self,
        id: &str,
        hashed_password: &str,
    ) -> Result<(), DbError> {
        self.db
            .query("UPDATE type::thing('account', $id) SET password_hash = $hash")
            .bind(("id", id.to_string()))
            .bind(("hash", hashed_password.to_string()))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// Update account avatar
    pub async fn update_account_avatar(&self, id: &str, avatar_url: &str) -> Result<(), DbError> {
        self.db
            .query("UPDATE type::thing('account', $id) SET avatar = $avatar")
            .bind(("id", id.to_string()))
            .bind(("avatar", avatar_url.to_string()))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// Create an auth token (email verification or password reset)
    pub async fn create_auth_token(
        &self,
        account_id: &str,
        token: &str,
        token_type: &str,
        expires_minutes: i64,
    ) -> Result<(), DbError> {
        // Delete existing tokens of same type for this account
        self.db
            .query("DELETE auth_token WHERE account = type::thing('account', $id) AND token_type = $type")
            .bind(("id", account_id.to_string()))
            .bind(("type", token_type.to_string()))
            .await
            .map_err(DbError::Database)?;

        self.db
            .query(
                "CREATE auth_token SET \
                    account = type::thing('account', $id), \
                    token = $token, \
                    token_type = $type, \
                    expires_at = time::now() + type::duration($duration), \
                    used = false, \
                    created_at = time::now()",
            )
            .bind(("id", account_id.to_string()))
            .bind(("token", token.to_string()))
            .bind(("type", token_type.to_string()))
            .bind(("duration", format!("{}m", expires_minutes)))
            .await
            .map_err(DbError::Database)?;

        Ok(())
    }

    /// Validate and consume an auth token, returns account ID
    pub async fn validate_auth_token(
        &self,
        token: &str,
        token_type: &str,
    ) -> Result<String, DbError> {
        let mut result = self.db
            .query(
                "SELECT *, account.id AS account_id FROM auth_token \
                 WHERE token = $token AND token_type = $type AND used = false AND expires_at > time::now() \
                 LIMIT 1",
            )
            .bind(("token", token.to_string()))
            .bind(("type", token_type.to_string()))
            .await
            .map_err(DbError::Database)?;

        #[derive(serde::Deserialize)]
        struct TokenResult {
            account: surrealdb::sql::Thing,
        }

        let tokens: Vec<TokenResult> = result.take(0).map_err(DbError::Database)?;
        let token_record = tokens.into_iter().next().ok_or(DbError::NotFound)?;

        // Mark token as used
        self.db
            .query("UPDATE auth_token SET used = true WHERE token = $token")
            .bind(("token", token.to_string()))
            .await
            .map_err(DbError::Database)?;

        Ok(token_record.account.id.to_raw())
    }

    /// Set email_verified flag on account
    pub async fn set_email_verified(&self, id: &str, verified: bool) -> Result<(), DbError> {
        self.db
            .query("UPDATE type::thing('account', $id) SET email_verified = $verified")
            .bind(("id", id.to_string()))
            .bind(("verified", verified))
            .await
            .map_err(DbError::Database)?;
        Ok(())
    }
}

// ============================================================================

// ============================================================================
// Asset DAL
// ============================================================================

impl AppState {
    pub async fn create_asset(&self, req: CreateAssetRequest) -> Result<Asset, DbError> {
        let asset: Option<Asset> = self
            .db
            .create::<Option<Asset>>("asset")
            .content(Asset {
                id: None,
                created_at: None,
                name: req.name,
                category: req.category,
                serial_number: req.serial_number,
                purchase_date: req.purchase_date,
                value: req.value,
                status: AssetStatus::Available,
                location: req.location,
                assigned_to: None,
                assigned_employee_name: None,
            })
            .await?
            .into_iter()
            .next();
        asset.ok_or(DbError::NotFound)
    }

    pub async fn get_all_assets(&self) -> Result<Vec<Asset>, DbError> {
        Ok(self.db.select("asset").await?)
    }

    pub async fn get_asset(&self, id: &str) -> Result<Asset, DbError> {
        let asset: Option<Asset> = self.db.select(("asset", id)).await?;
        asset.ok_or(DbError::NotFound)
    }

    pub async fn assign_asset(&self, id: &str, req: AssignAssetRequest) -> Result<Asset, DbError> {
        let asset: Option<Asset> = self
            .db
            .update(("asset", id))
            .merge(serde_json::json!({
                "assigned_to": req.employee_id,
                "location": req.location,
                "status": AssetStatus::InUse,
            }))
            .await?;
        asset.ok_or(DbError::NotFound)
    }
}

// ============================================================================
