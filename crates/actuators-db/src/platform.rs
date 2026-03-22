use actuators_core::config::AppConfig;
use actuators_core::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::{info, instrument};

// ── Record types used by downstream crates ─────────────────────────

/// A tenant record stored in the platform database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRecord {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub db_name: String,
    pub status: String,
    pub plan_name: String,
    pub max_users: i32,
    pub storage_limit_mb: i32,
}

/// A tenant-service link stored in the platform database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantServiceRecord {
    pub service_key: String,
}

// ── PlatformDb ─────────────────────────────────────────────────────

/// Handles all platform-level database operations against the shared
/// platform namespace/database in SurrealDB.
#[derive(Debug, Clone)]
pub struct PlatformDb {
    pub db: Surreal<Any>,
}

impl PlatformDb {
    // ── Constructors ───────────────────────────────────────────────

    /// Wrap an already-connected client.
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Connect to the SurrealDB cloud instance and select the platform
    /// namespace + database.
    #[instrument(skip_all)]
    pub async fn connect(config: &AppConfig) -> Result<Self, AppError> {
        let db = surrealdb::engine::any::connect(&config.surrealdb_url)
            .await
            .map_err(|e| AppError::Database(format!("failed to connect to SurrealDB: {e}")))?;

        db.signin(Root {
            username: config.surrealdb_user.clone(),
            password: config.surrealdb_pass.clone(),
        })
        .await
        .map_err(|e| AppError::Database(format!("failed to sign in: {e}")))?;

        db.use_ns(&config.surrealdb_namespace)
            .use_db(&config.platform_db_name)
            .await
            .map_err(|e| AppError::Database(format!("failed to select ns/db: {e}")))?;

        info!(
            ns = %config.surrealdb_namespace,
            db = %config.platform_db_name,
            "platform database connected"
        );

        Ok(Self { db })
    }

    /// Return a reference to the inner Surreal client.
    pub fn client(&self) -> &Surreal<Any> {
        &self.db
    }

    // ── Platform Account CRUD ──────────────────────────────────────

    #[instrument(skip(self, password_hash))]
    pub async fn create_account(
        &self,
        email: &str,
        password_hash: &str,
        full_name: &str,
        preferred_lang: &str,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE platform_account SET
                    email              = $email,
                    password_hash      = $password_hash,
                    full_name          = $full_name,
                    preferred_lang     = $preferred_lang,
                    is_super_admin     = false,
                    email_verified     = false,
                    verification_token = rand::uuid(),
                    status             = 'active',
                    created_at         = time::now(),
                    updated_at         = time::now()
                ;",
            )
            .bind(("email", email.to_owned()))
            .bind(("password_hash", password_hash.to_owned()))
            .bind(("full_name", full_name.to_owned()))
            .bind(("preferred_lang", preferred_lang.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        account.ok_or_else(|| AppError::Database("failed to create account".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_email(&self, email: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM platform_account WHERE email = $email LIMIT 1;")
            .bind(("email", email.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_id(&self, id: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::thing('platform_account', $id);")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(account)
    }

    #[instrument(skip(self, fields))]
    pub async fn update_account(&self, id: &str, fields: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('platform_account', $id) MERGE $fields SET updated_at = time::now();",
            )
            .bind(("id", id.to_owned()))
            .bind(("fields", fields))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("account not found".into()))
    }

    #[instrument(skip(self))]
    pub async fn verify_email(&self, token: &str) -> Result<bool, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE platform_account
                    SET email_verified     = true,
                        verification_token = NONE,
                        updated_at         = time::now()
                    WHERE verification_token = $token;",
            )
            .bind(("token", token.to_owned()))
            .await
            .map_err(db_err)?;

        let updated: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(!updated.is_empty())
    }

    // ── Tenant CRUD ────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn create_tenant(
        &self,
        name: &str,
        slug: &str,
        org_type: &str,
        owner_id: &str,
        plan_id: &str,
    ) -> Result<Value, AppError> {
        let db_name = format!("tenant_{slug}");

        let mut resp = self
            .db
            .query(
                "CREATE tenant SET
                    name       = $name,
                    slug       = $slug,
                    org_type   = $org_type,
                    owner_id   = $owner_id,
                    plan_id    = $plan_id,
                    db_name    = $db_name,
                    status     = 'active',
                    created_at = time::now(),
                    updated_at = time::now()
                ;",
            )
            .bind(("name", name.to_owned()))
            .bind(("slug", slug.to_owned()))
            .bind(("org_type", org_type.to_owned()))
            .bind(("owner_id", owner_id.to_owned()))
            .bind(("plan_id", plan_id.to_owned()))
            .bind(("db_name", db_name))
            .await
            .map_err(db_err)?;

        let tenant: Option<Value> = resp.take(0).map_err(db_err)?;
        tenant.ok_or_else(|| AppError::Database("failed to create tenant".into()))
    }

    /// Look up a tenant by slug. Returns a typed [`TenantRecord`] that
    /// downstream middleware relies on.
    #[instrument(skip(self))]
    pub async fn get_tenant_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<TenantRecord>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM tenant WHERE slug = $slug AND status = 'active' LIMIT 1;")
            .bind(("slug", slug.to_owned()))
            .await
            .map_err(db_err)?;

        let raw: Option<Value> = resp.take(0).map_err(db_err)?;
        let record: Option<TenantRecord> = raw
            .map(|v| serde_json::from_value(v))
            .transpose()
            .map_err(|e| AppError::Database(format!("failed to deserialize TenantRecord: {e}")))?;
        Ok(record)
    }

    #[instrument(skip(self))]
    pub async fn list_tenants_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT tenant_id.* AS tenant
                    FROM tenant_membership
                    WHERE platform_account_id = $account_id;",
            )
            .bind(("account_id", account_id.to_owned()))
            .await
            .map_err(db_err)?;

        let tenants: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(tenants)
    }

    #[instrument(skip(self, fields))]
    pub async fn update_tenant(&self, id: &str, fields: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('tenant', $id) MERGE $fields SET updated_at = time::now();",
            )
            .bind(("id", id.to_owned()))
            .bind(("fields", fields))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("tenant not found".into()))
    }

    // ── Membership ─────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn create_membership(
        &self,
        platform_account_id: &str,
        tenant_id: &str,
        role: &str,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE tenant_membership SET
                    platform_account_id = $platform_account_id,
                    tenant_id           = $tenant_id,
                    role                = $role,
                    created_at          = time::now()
                ;",
            )
            .bind(("platform_account_id", platform_account_id.to_owned()))
            .bind(("tenant_id", tenant_id.to_owned()))
            .bind(("role", role.to_owned()))
            .await
            .map_err(db_err)?;

        let membership: Option<Value> = resp.take(0).map_err(db_err)?;
        membership.ok_or_else(|| AppError::Database("failed to create membership".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_membership(
        &self,
        platform_account_id: &str,
        tenant_id: &str,
    ) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM tenant_membership
                    WHERE platform_account_id = $platform_account_id
                      AND tenant_id           = $tenant_id
                    LIMIT 1;",
            )
            .bind(("platform_account_id", platform_account_id.to_owned()))
            .bind(("tenant_id", tenant_id.to_owned()))
            .await
            .map_err(db_err)?;

        let membership: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(membership)
    }

    // ── Plans ──────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn get_plan_by_name(&self, name: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM plan WHERE name = $name LIMIT 1;")
            .bind(("name", name.to_owned()))
            .await
            .map_err(db_err)?;

        let plan: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(plan)
    }

    #[instrument(skip(self))]
    pub async fn get_plan_by_id(&self, id: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::thing('plan', $id);")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let plan: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(plan)
    }

    #[instrument(skip(self))]
    pub async fn list_plans(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM plan ORDER BY created_at ASC;")
            .await
            .map_err(db_err)?;

        let plans: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(plans)
    }

    #[instrument(skip(self))]
    pub async fn create_plan(
        &self,
        name: &str,
        display_name: &str,
        max_users: i32,
        storage_limit_mb: i32,
        price_monthly: f64,
        currency: &str,
        services: &[String],
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE plan SET
                    name             = $name,
                    display_name     = $display_name,
                    max_users        = $max_users,
                    storage_limit_mb = $storage_limit_mb,
                    price_monthly    = $price_monthly,
                    currency         = $currency,
                    services         = $services,
                    is_active        = true,
                    created_at       = time::now()
                ;",
            )
            .bind(("name", name.to_owned()))
            .bind(("display_name", display_name.to_owned()))
            .bind(("max_users", max_users))
            .bind(("storage_limit_mb", storage_limit_mb))
            .bind(("price_monthly", price_monthly))
            .bind(("currency", currency.to_owned()))
            .bind(("services", services.to_owned()))
            .await
            .map_err(db_err)?;

        let plan: Option<Value> = resp.take(0).map_err(db_err)?;
        plan.ok_or_else(|| AppError::Database("failed to create plan".into()))
    }

    // ── Service Catalog ────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn list_services(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM service_catalog ORDER BY key ASC;")
            .await
            .map_err(db_err)?;

        let services: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(services)
    }

    #[instrument(skip(self))]
    pub async fn create_service(
        &self,
        key: &str,
        name_en: &str,
        name_ar: &str,
        description_en: &str,
        description_ar: &str,
        is_default: bool,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE service_catalog SET
                    key            = $key,
                    name_en        = $name_en,
                    name_ar        = $name_ar,
                    description_en = $description_en,
                    description_ar = $description_ar,
                    is_default     = $is_default,
                    is_active      = true,
                    created_at     = time::now()
                ;",
            )
            .bind(("key", key.to_owned()))
            .bind(("name_en", name_en.to_owned()))
            .bind(("name_ar", name_ar.to_owned()))
            .bind(("description_en", description_en.to_owned()))
            .bind(("description_ar", description_ar.to_owned()))
            .bind(("is_default", is_default))
            .await
            .map_err(db_err)?;

        let service: Option<Value> = resp.take(0).map_err(db_err)?;
        service.ok_or_else(|| AppError::Database("failed to create service".into()))
    }

    // ── Tenant Services ────────────────────────────────────────────

    /// Get the list of service keys enabled for a tenant.
    #[instrument(skip(self))]
    pub async fn get_tenant_services(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<String>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT service_key FROM tenant_service
                    WHERE tenant = $tid
                      AND is_enabled = true;",
            )
            .bind(("tid", tenant_id.to_owned()))
            .await
            .map_err(db_err)?;

        let raw: Vec<Value> = resp.take(0).map_err(db_err)?;
        let records: Vec<TenantServiceRecord> = raw
            .into_iter()
            .map(|v| serde_json::from_value(v))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("failed to deserialize TenantServiceRecord: {e}")))?;
        Ok(records.into_iter().map(|r| r.service_key).collect())
    }

    // ── Refresh Tokens ─────────────────────────────────────────────

    #[instrument(skip(self, token_hash))]
    pub async fn store_refresh_token(
        &self,
        account_id: &str,
        token_hash: &str,
        device_info: &str,
        expires_at: &str,
    ) -> Result<(), AppError> {
        self.db
            .query(
                "CREATE refresh_token SET
                    account_id  = $account_id,
                    token_hash  = $token_hash,
                    device_info = $device_info,
                    expires_at  = type::datetime($expires_at),
                    created_at  = time::now()
                ;",
            )
            .bind(("account_id", account_id.to_owned()))
            .bind(("token_hash", token_hash.to_owned()))
            .bind(("device_info", device_info.to_owned()))
            .bind(("expires_at", expires_at.to_owned()))
            .await
            .map_err(db_err)?;

        Ok(())
    }

    #[instrument(skip(self, token_hash))]
    pub async fn get_refresh_token(&self, token_hash: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM refresh_token
                    WHERE token_hash = $token_hash
                    LIMIT 1;",
            )
            .bind(("token_hash", token_hash.to_owned()))
            .await
            .map_err(db_err)?;

        let token: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(token)
    }

    #[instrument(skip(self, token_hash))]
    pub async fn delete_refresh_token(&self, token_hash: &str) -> Result<(), AppError> {
        self.db
            .query("DELETE FROM refresh_token WHERE token_hash = $token_hash;")
            .bind(("token_hash", token_hash.to_owned()))
            .await
            .map_err(db_err)?;

        Ok(())
    }

    // ── Audit ──────────────────────────────────────────────────────

    #[instrument(skip(self, details))]
    pub async fn log_platform_audit(
        &self,
        actor_id: &str,
        action: &str,
        entity_type: &str,
        entity_id: &str,
        details: Value,
        ip: &str,
        user_agent: &str,
    ) -> Result<(), AppError> {
        self.db
            .query(
                "CREATE platform_audit_log SET
                    actor_id    = $actor_id,
                    action      = $action,
                    entity_type = $entity_type,
                    entity_id   = $entity_id,
                    details     = $details,
                    ip_address  = $ip,
                    user_agent  = $user_agent,
                    created_at  = time::now()
                ;",
            )
            .bind(("actor_id", actor_id.to_owned()))
            .bind(("action", action.to_owned()))
            .bind(("entity_type", entity_type.to_owned()))
            .bind(("entity_id", entity_id.to_owned()))
            .bind(("details", details))
            .bind(("ip", ip.to_owned()))
            .bind(("user_agent", user_agent.to_owned()))
            .await
            .map_err(db_err)?;

        Ok(())
    }

    // ── Metrics ────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn record_metric(
        &self,
        metric_key: &str,
        metric_value: f64,
        period: &str,
    ) -> Result<(), AppError> {
        self.db
            .query(
                "CREATE platform_metric SET
                    metric_key   = $metric_key,
                    metric_value = $metric_value,
                    period       = $period,
                    created_at   = time::now()
                ;",
            )
            .bind(("metric_key", metric_key.to_owned()))
            .bind(("metric_value", metric_value))
            .bind(("period", period.to_owned()))
            .await
            .map_err(db_err)?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_metrics(&self, period: &str) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM platform_metric
                    WHERE period = $period
                    ORDER BY created_at DESC;",
            )
            .bind(("period", period.to_owned()))
            .await
            .map_err(db_err)?;

        let metrics: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(metrics)
    }

    // ── Invitations ────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn create_invitation(
        &self,
        tenant_id: &str,
        email: &str,
        role: &str,
        token: &str,
        invited_by: &str,
        expires_at: &str,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE invitation SET
                    tenant_id  = $tenant_id,
                    email      = $email,
                    role       = $role,
                    token      = $token,
                    invited_by = $invited_by,
                    status     = 'pending',
                    expires_at = type::datetime($expires_at),
                    created_at = time::now()
                ;",
            )
            .bind(("tenant_id", tenant_id.to_owned()))
            .bind(("email", email.to_owned()))
            .bind(("role", role.to_owned()))
            .bind(("token", token.to_owned()))
            .bind(("invited_by", invited_by.to_owned()))
            .bind(("expires_at", expires_at.to_owned()))
            .await
            .map_err(db_err)?;

        let invitation: Option<Value> = resp.take(0).map_err(db_err)?;
        invitation.ok_or_else(|| AppError::Database("failed to create invitation".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_invitation_by_token(&self, token: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM invitation
                    WHERE token  = $token
                      AND status = 'pending'
                    LIMIT 1;",
            )
            .bind(("token", token.to_owned()))
            .await
            .map_err(db_err)?;

        let invitation: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(invitation)
    }

    #[instrument(skip(self))]
    pub async fn accept_invitation(&self, token: &str) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE invitation SET
                    status      = 'accepted',
                    accepted_at = time::now()
                WHERE token  = $token
                  AND status = 'pending';",
            )
            .bind(("token", token.to_owned()))
            .await
            .map_err(db_err)?;

        let invitation: Option<Value> = resp.take(0).map_err(db_err)?;
        invitation
            .ok_or_else(|| AppError::NotFound("invitation not found or already used".into()))
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Convert a SurrealDB error into an `AppError::Database`.
fn db_err(e: surrealdb::Error) -> AppError {
    AppError::Database(e.to_string())
}
