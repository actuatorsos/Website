use serde::{Deserialize, Serialize};

// ── Newtypes ───────────────────────────────────────────────────────

/// A tenant's URL-safe slug (e.g. `acme-corp`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantSlug(pub String);

impl TenantSlug {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for an account (user) on the platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub String);

impl AccountId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ── Context structs ────────────────────────────────────────────────

/// Resolved tenant information attached to a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub slug: String,
    pub db_name: String,
    pub plan_name: String,
    pub services: Vec<String>,
    pub max_users: i32,
    pub storage_limit_mb: i32,
}

/// Authentication context for platform-level (super-admin) requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAuthContext {
    pub account_id: String,
    pub email: String,
    pub full_name: String,
    pub is_super_admin: bool,
}

/// Authentication context for tenant-scoped requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAuthContext {
    pub account_id: String,
    pub platform_id: String,
    pub tenant_slug: String,
    pub tenant_db: String,
    pub role: String,
    pub services: Vec<String>,
}
