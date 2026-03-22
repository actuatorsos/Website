-- ╔══════════════════════════════════════════════════════════════════╗
-- ║  Actuators ERP — Platform Schema v001                          ║
-- ╚══════════════════════════════════════════════════════════════════╝

-- ── platform_account ─────────────────────────────────────────────
DEFINE TABLE platform_account SCHEMAFULL;
DEFINE FIELD email                      ON platform_account TYPE string;
DEFINE FIELD password_hash              ON platform_account TYPE string;
DEFINE FIELD full_name                  ON platform_account TYPE string;
DEFINE FIELD phone                      ON platform_account TYPE option<string>;
DEFINE FIELD avatar_url                 ON platform_account TYPE option<string>;
DEFINE FIELD preferred_lang             ON platform_account TYPE string DEFAULT 'ar';
DEFINE FIELD email_verified             ON platform_account TYPE bool DEFAULT false;
DEFINE FIELD verification_token         ON platform_account TYPE option<string>;
DEFINE FIELD verification_token_expires ON platform_account TYPE option<datetime>;
DEFINE FIELD is_super_admin             ON platform_account TYPE bool DEFAULT false;
DEFINE FIELD status                     ON platform_account TYPE string DEFAULT 'active';
DEFINE FIELD created_at                 ON platform_account TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at                 ON platform_account TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_platform_account_email ON platform_account FIELDS email UNIQUE;

-- ── tenant ───────────────────────────────────────────────────────
DEFINE TABLE tenant SCHEMAFULL;
DEFINE FIELD name              ON tenant TYPE string;
DEFINE FIELD slug              ON tenant TYPE string;
DEFINE FIELD db_name           ON tenant TYPE string;
DEFINE FIELD org_type          ON tenant TYPE string DEFAULT 'startup';
DEFINE FIELD owner_id          ON tenant TYPE string;
DEFINE FIELD plan_name         ON tenant TYPE string DEFAULT 'free';
DEFINE FIELD status            ON tenant TYPE string DEFAULT 'active';
DEFINE FIELD commercial_reg    ON tenant TYPE option<string>;
DEFINE FIELD logo_url          ON tenant TYPE option<string>;
DEFINE FIELD domain            ON tenant TYPE option<string>;
DEFINE FIELD max_users         ON tenant TYPE int DEFAULT 3;
DEFINE FIELD storage_limit_mb  ON tenant TYPE int DEFAULT 512;
DEFINE FIELD trial_ends_at     ON tenant TYPE option<datetime>;
DEFINE FIELD created_at        ON tenant TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at        ON tenant TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_tenant_slug   ON tenant FIELDS slug UNIQUE;

-- ── plan ─────────────────────────────────────────────────────────
DEFINE TABLE plan SCHEMAFULL;
DEFINE FIELD name       ON plan TYPE string;
DEFINE FIELD name_ar    ON plan TYPE string;
DEFINE FIELD price_eur  ON plan TYPE float;
DEFINE FIELD max_users  ON plan TYPE int;
DEFINE FIELD storage_mb ON plan TYPE int;
DEFINE FIELD services   ON plan TYPE array;
DEFINE FIELD is_active  ON plan TYPE bool DEFAULT true;
DEFINE FIELD created_at ON plan TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_plan_name ON plan FIELDS name UNIQUE;

-- ── tenant_service ───────────────────────────────────────────────
DEFINE TABLE tenant_service SCHEMAFULL;
DEFINE FIELD tenant_id   ON tenant_service TYPE string;
DEFINE FIELD service_key ON tenant_service TYPE string;
DEFINE FIELD enabled     ON tenant_service TYPE bool DEFAULT true;
DEFINE FIELD enabled_at  ON tenant_service TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_ts_tenant_svc ON tenant_service FIELDS tenant_id, service_key UNIQUE;

-- ── tenant_membership ────────────────────────────────────────────
DEFINE TABLE tenant_membership SCHEMAFULL;
DEFINE FIELD platform_account_id ON tenant_membership TYPE string;
DEFINE FIELD tenant_id           ON tenant_membership TYPE string;
DEFINE FIELD role                ON tenant_membership TYPE string DEFAULT 'employee';
DEFINE FIELD status              ON tenant_membership TYPE string DEFAULT 'active';
DEFINE FIELD joined_at           ON tenant_membership TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_tm_account_tenant ON tenant_membership FIELDS platform_account_id, tenant_id UNIQUE;

-- ── service_catalog ──────────────────────────────────────────────
DEFINE TABLE service_catalog SCHEMAFULL;
DEFINE FIELD key           ON service_catalog TYPE string;
DEFINE FIELD name_en       ON service_catalog TYPE string;
DEFINE FIELD name_ar       ON service_catalog TYPE string;
DEFINE FIELD description_en ON service_catalog TYPE string;
DEFINE FIELD description_ar ON service_catalog TYPE string;
DEFINE FIELD icon          ON service_catalog TYPE string;
DEFINE FIELD category      ON service_catalog TYPE string;
DEFINE FIELD is_core       ON service_catalog TYPE bool DEFAULT false;
DEFINE FIELD sort_order    ON service_catalog TYPE int DEFAULT 0;
DEFINE INDEX idx_sc_key ON service_catalog FIELDS key UNIQUE;

-- ── platform_audit_log ───────────────────────────────────────────
DEFINE TABLE platform_audit_log SCHEMAFULL;
DEFINE FIELD actor_id    ON platform_audit_log TYPE option<string>;
DEFINE FIELD action      ON platform_audit_log TYPE string;
DEFINE FIELD entity_type ON platform_audit_log TYPE string;
DEFINE FIELD entity_id   ON platform_audit_log TYPE string;
DEFINE FIELD details     ON platform_audit_log TYPE option<object>;
DEFINE FIELD ip_address  ON platform_audit_log TYPE option<string>;
DEFINE FIELD user_agent  ON platform_audit_log TYPE option<string>;
DEFINE FIELD created_at  ON platform_audit_log TYPE datetime DEFAULT time::now();

-- ── platform_refresh_token ───────────────────────────────────────
DEFINE TABLE platform_refresh_token SCHEMAFULL;
DEFINE FIELD account_id  ON platform_refresh_token TYPE string;
DEFINE FIELD token_hash  ON platform_refresh_token TYPE string;
DEFINE FIELD device_info ON platform_refresh_token TYPE option<string>;
DEFINE FIELD expires_at  ON platform_refresh_token TYPE datetime;
DEFINE FIELD created_at  ON platform_refresh_token TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_prt_hash ON platform_refresh_token FIELDS token_hash UNIQUE;

-- ── invitation ───────────────────────────────────────────────────
DEFINE TABLE invitation SCHEMAFULL;
DEFINE FIELD tenant_id   ON invitation TYPE string;
DEFINE FIELD email       ON invitation TYPE string;
DEFINE FIELD role        ON invitation TYPE string DEFAULT 'employee';
DEFINE FIELD token       ON invitation TYPE string;
DEFINE FIELD invited_by  ON invitation TYPE string;
DEFINE FIELD status      ON invitation TYPE string DEFAULT 'pending';
DEFINE FIELD expires_at  ON invitation TYPE datetime;
DEFINE FIELD created_at  ON invitation TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_inv_token ON invitation FIELDS token UNIQUE;

-- ── platform_metric ──────────────────────────────────────────────
DEFINE TABLE platform_metric SCHEMAFULL;
DEFINE FIELD metric_key   ON platform_metric TYPE string;
DEFINE FIELD metric_value ON platform_metric TYPE float;
DEFINE FIELD period       ON platform_metric TYPE string;
DEFINE FIELD recorded_at  ON platform_metric TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_pm_key_period ON platform_metric FIELDS metric_key, period UNIQUE;
