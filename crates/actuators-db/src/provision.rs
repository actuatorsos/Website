use actuators_core::error::AppError;
use serde_json::Value;
use tracing::{info, instrument};

use crate::platform::PlatformDb;
use crate::pool::TenantConnectionPool;

// ═══════════════════════════════════════════════════════════════════
//  Tenant schema migration SQL
// ═══════════════════════════════════════════════════════════════════

/// Full SCHEMAFULL migration applied to every newly provisioned tenant
/// database. This defines the canonical table structure for Phase 1.
const TENANT_SCHEMA_SQL: &str = r#"
-- ╔══════════════════════════════════════════════════════════════════╗
-- ║  Actuators ERP — Tenant Schema (Phase 1)                       ║
-- ╚══════════════════════════════════════════════════════════════════╝

-- ── account ────────────────────────────────────────────────────────
DEFINE TABLE account SCHEMAFULL;
DEFINE FIELD platform_id     ON account TYPE string;
DEFINE FIELD email            ON account TYPE string;
DEFINE FIELD display_name     ON account TYPE string;
DEFINE FIELD role             ON account TYPE string DEFAULT 'employee';
DEFINE FIELD status           ON account TYPE string DEFAULT 'active';
DEFINE FIELD preferred_lang   ON account TYPE string DEFAULT 'en';
DEFINE FIELD last_login       ON account TYPE option<datetime>;
DEFINE FIELD created_at       ON account TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at       ON account TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_account_platform_id ON account FIELDS platform_id UNIQUE;
DEFINE INDEX idx_account_email       ON account FIELDS email UNIQUE;

-- ── department ─────────────────────────────────────────────────────
DEFINE TABLE department SCHEMAFULL;
DEFINE FIELD name_en     ON department TYPE string;
DEFINE FIELD name_ar     ON department TYPE string;
DEFINE FIELD code        ON department TYPE string;
DEFINE FIELD parent_id   ON department TYPE option<string>;
DEFINE FIELD manager_id  ON department TYPE option<string>;
DEFINE FIELD is_active   ON department TYPE bool DEFAULT true;
DEFINE FIELD created_at  ON department TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at  ON department TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_department_code ON department FIELDS code UNIQUE;

-- ── job_title ──────────────────────────────────────────────────────
DEFINE TABLE job_title SCHEMAFULL;
DEFINE FIELD name_en        ON job_title TYPE string;
DEFINE FIELD name_ar        ON job_title TYPE string;
DEFINE FIELD department_id  ON job_title TYPE option<string>;
DEFINE FIELD level          ON job_title TYPE option<int>;
DEFINE FIELD is_active      ON job_title TYPE bool DEFAULT true;
DEFINE FIELD created_at     ON job_title TYPE datetime DEFAULT time::now();

-- ── employee ───────────────────────────────────────────────────────
DEFINE TABLE employee SCHEMAFULL;
DEFINE FIELD account_id       ON employee TYPE option<string>;
DEFINE FIELD employee_number  ON employee TYPE string;
DEFINE FIELD first_name_en    ON employee TYPE string;
DEFINE FIELD last_name_en     ON employee TYPE string;
DEFINE FIELD first_name_ar    ON employee TYPE option<string>;
DEFINE FIELD last_name_ar     ON employee TYPE option<string>;
DEFINE FIELD national_id      ON employee TYPE option<string>;
DEFINE FIELD date_of_birth    ON employee TYPE option<datetime>;
DEFINE FIELD gender           ON employee TYPE option<string>;
DEFINE FIELD nationality      ON employee TYPE option<string>;
DEFINE FIELD phone            ON employee TYPE option<string>;
DEFINE FIELD personal_email   ON employee TYPE option<string>;
DEFINE FIELD department_id    ON employee TYPE option<string>;
DEFINE FIELD job_title_id     ON employee TYPE option<string>;
DEFINE FIELD manager_id       ON employee TYPE option<string>;
DEFINE FIELD hire_date        ON employee TYPE option<datetime>;
DEFINE FIELD employment_type  ON employee TYPE string DEFAULT 'full_time';
DEFINE FIELD status           ON employee TYPE string DEFAULT 'active';
DEFINE FIELD termination_date ON employee TYPE option<datetime>;
DEFINE FIELD created_at       ON employee TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at       ON employee TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_employee_number  ON employee FIELDS employee_number UNIQUE;
DEFINE INDEX idx_employee_account ON employee FIELDS account_id;

-- ── contract ───────────────────────────────────────────────────────
DEFINE TABLE contract SCHEMAFULL;
DEFINE FIELD employee_id            ON contract TYPE string;
DEFINE FIELD contract_type          ON contract TYPE string;
DEFINE FIELD start_date             ON contract TYPE datetime;
DEFINE FIELD end_date               ON contract TYPE option<datetime>;
DEFINE FIELD base_salary            ON contract TYPE float;
DEFINE FIELD currency               ON contract TYPE string DEFAULT 'SAR';
DEFINE FIELD working_hours_per_week ON contract TYPE int DEFAULT 40;
DEFINE FIELD is_current             ON contract TYPE bool DEFAULT true;
DEFINE FIELD notes                  ON contract TYPE option<string>;
DEFINE FIELD created_at             ON contract TYPE datetime DEFAULT time::now();

-- ── permission ─────────────────────────────────────────────────────
DEFINE TABLE permission SCHEMAFULL;
DEFINE FIELD key          ON permission TYPE string;
DEFINE FIELD name_en      ON permission TYPE string;
DEFINE FIELD name_ar      ON permission TYPE string;
DEFINE FIELD service_key  ON permission TYPE string;
DEFINE FIELD resource     ON permission TYPE string;
DEFINE FIELD action       ON permission TYPE string;
DEFINE INDEX idx_permission_key ON permission FIELDS key UNIQUE;

-- ── role_permission ────────────────────────────────────────────────
DEFINE TABLE role_permission SCHEMAFULL;
DEFINE FIELD role           ON role_permission TYPE string;
DEFINE FIELD permission_id  ON role_permission TYPE string;
DEFINE FIELD scope          ON role_permission TYPE string DEFAULT 'all';
DEFINE INDEX idx_role_perm ON role_permission FIELDS role, permission_id UNIQUE;

-- ── account_permission_override ────────────────────────────────────
DEFINE TABLE account_permission_override SCHEMAFULL;
DEFINE FIELD account_id     ON account_permission_override TYPE string;
DEFINE FIELD permission_id  ON account_permission_override TYPE string;
DEFINE FIELD granted        ON account_permission_override TYPE bool;
DEFINE FIELD created_at     ON account_permission_override TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_acct_perm_override ON account_permission_override FIELDS account_id, permission_id UNIQUE;

-- ── setting ────────────────────────────────────────────────────────
DEFINE TABLE setting SCHEMAFULL;
DEFINE FIELD key          ON setting TYPE string;
DEFINE FIELD value        ON setting TYPE string;
DEFINE FIELD value_type   ON setting TYPE string DEFAULT 'string';
DEFINE FIELD updated_by   ON setting TYPE option<string>;
DEFINE FIELD updated_at   ON setting TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_setting_key ON setting FIELDS key UNIQUE;

-- ── audit_log ──────────────────────────────────────────────────────
DEFINE TABLE audit_log SCHEMAFULL;
DEFINE FIELD actor_id     ON audit_log TYPE string;
DEFINE FIELD actor_name   ON audit_log TYPE string;
DEFINE FIELD action       ON audit_log TYPE string;
DEFINE FIELD service      ON audit_log TYPE string;
DEFINE FIELD entity_type  ON audit_log TYPE string;
DEFINE FIELD entity_id    ON audit_log TYPE option<string>;
DEFINE FIELD changes      ON audit_log TYPE option<object>;
DEFINE FIELD ip_address   ON audit_log TYPE option<string>;
DEFINE FIELD user_agent   ON audit_log TYPE option<string>;
DEFINE FIELD created_at   ON audit_log TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_audit_created ON audit_log FIELDS created_at;

-- ── notification ───────────────────────────────────────────────────
DEFINE TABLE notification SCHEMAFULL;
DEFINE FIELD account_id  ON notification TYPE string;
DEFINE FIELD title_en    ON notification TYPE string;
DEFINE FIELD title_ar    ON notification TYPE option<string>;
DEFINE FIELD body_en     ON notification TYPE option<string>;
DEFINE FIELD body_ar     ON notification TYPE option<string>;
DEFINE FIELD link        ON notification TYPE option<string>;
DEFINE FIELD is_read     ON notification TYPE bool DEFAULT false;
DEFINE FIELD created_at  ON notification TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_notif_account ON notification FIELDS account_id;

-- ── session ────────────────────────────────────────────────────────
DEFINE TABLE session SCHEMAFULL;
DEFINE FIELD account_id   ON session TYPE string;
DEFINE FIELD token_hash   ON session TYPE string;
DEFINE FIELD device_info  ON session TYPE option<string>;
DEFINE FIELD ip_address   ON session TYPE option<string>;
DEFINE FIELD expires_at   ON session TYPE datetime;
DEFINE FIELD created_at   ON session TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_session_token ON session FIELDS token_hash UNIQUE;

-- ── i18n_override ──────────────────────────────────────────────────
DEFINE TABLE i18n_override SCHEMAFULL;
DEFINE FIELD key         ON i18n_override TYPE string;
DEFINE FIELD lang        ON i18n_override TYPE string;
DEFINE FIELD value       ON i18n_override TYPE string;
DEFINE FIELD updated_at  ON i18n_override TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_i18n_key_lang ON i18n_override FIELDS key, lang UNIQUE;

-- ── attachment ─────────────────────────────────────────────────────
DEFINE TABLE attachment SCHEMAFULL;
DEFINE FIELD entity_type  ON attachment TYPE string;
DEFINE FIELD entity_id    ON attachment TYPE string;
DEFINE FIELD file_name    ON attachment TYPE string;
DEFINE FIELD file_path    ON attachment TYPE string;
DEFINE FIELD mime_type    ON attachment TYPE string;
DEFINE FIELD size_bytes   ON attachment TYPE int;
DEFINE FIELD uploaded_by  ON attachment TYPE string;
DEFINE FIELD created_at   ON attachment TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_attachment_entity ON attachment FIELDS entity_type, entity_id;
"#;

// ═══════════════════════════════════════════════════════════════════
//  Seed data
// ═══════════════════════════════════════════════════════════════════

/// Seed permissions for Phase 1 services (HR, Settings, Audit).
const SEED_PERMISSIONS_SQL: &str = r#"
-- HR Employees
CREATE permission SET key='hr.employees.create', name_en='Create Employees', name_ar='إنشاء موظفين', service_key='hr', resource='employees', action='create';
CREATE permission SET key='hr.employees.read',   name_en='View Employees',   name_ar='عرض الموظفين',   service_key='hr', resource='employees', action='read';
CREATE permission SET key='hr.employees.update', name_en='Update Employees', name_ar='تعديل الموظفين', service_key='hr', resource='employees', action='update';
CREATE permission SET key='hr.employees.delete', name_en='Delete Employees', name_ar='حذف الموظفين',   service_key='hr', resource='employees', action='delete';

-- HR Departments
CREATE permission SET key='hr.departments.create', name_en='Create Departments', name_ar='إنشاء أقسام', service_key='hr', resource='departments', action='create';
CREATE permission SET key='hr.departments.read',   name_en='View Departments',   name_ar='عرض الأقسام',   service_key='hr', resource='departments', action='read';
CREATE permission SET key='hr.departments.update', name_en='Update Departments', name_ar='تعديل الأقسام', service_key='hr', resource='departments', action='update';
CREATE permission SET key='hr.departments.delete', name_en='Delete Departments', name_ar='حذف الأقسام',   service_key='hr', resource='departments', action='delete';

-- Settings
CREATE permission SET key='settings.read',   name_en='View Settings',   name_ar='عرض الإعدادات',   service_key='settings', resource='settings', action='read';
CREATE permission SET key='settings.update', name_en='Update Settings', name_ar='تعديل الإعدادات', service_key='settings', resource='settings', action='update';

-- Audit
CREATE permission SET key='audit.read', name_en='View Audit Logs', name_ar='عرض سجل المراجعة', service_key='audit', resource='audit_log', action='read';
"#;

/// Seed default role-to-permission mappings.
const SEED_ROLE_PERMISSIONS_SQL: &str = r#"
-- ═══ admin — all permissions, scope = all ═══
LET $perms = (SELECT id, key FROM permission);
FOR $p IN $perms {
    CREATE role_permission SET role = 'admin', permission_id = <string> $p.id, scope = 'all';
};

-- ═══ manager — read/create/update on hr.*, plus settings.read and audit.read ═══
FOR $p IN (SELECT id FROM permission WHERE action IN ['read', 'create', 'update'] AND service_key = 'hr') {
    CREATE role_permission SET role = 'manager', permission_id = <string> $p.id, scope = 'department';
};
FOR $p IN (SELECT id FROM permission WHERE key IN ['settings.read', 'audit.read']) {
    CREATE role_permission SET role = 'manager', permission_id = <string> $p.id, scope = 'all';
};

-- ═══ employee — limited read ═══
FOR $p IN (SELECT id FROM permission WHERE key = 'hr.employees.read') {
    CREATE role_permission SET role = 'employee', permission_id = <string> $p.id, scope = 'self';
};
FOR $p IN (SELECT id FROM permission WHERE key = 'hr.departments.read') {
    CREATE role_permission SET role = 'employee', permission_id = <string> $p.id, scope = 'all';
};

-- ═══ intern — minimal read ═══
FOR $p IN (SELECT id FROM permission WHERE key = 'hr.employees.read') {
    CREATE role_permission SET role = 'intern', permission_id = <string> $p.id, scope = 'self';
};

-- ═══ ai_agent — read on everything, scope = all ═══
FOR $p IN (SELECT id FROM permission WHERE action = 'read') {
    CREATE role_permission SET role = 'ai_agent', permission_id = <string> $p.id, scope = 'all';
};
"#;

/// Seed default tenant settings.
const SEED_SETTINGS_SQL: &str = r#"
CREATE setting SET key='timezone',          value='Asia/Riyadh', value_type='string';
CREATE setting SET key='language',          value='ar',          value_type='string';
CREATE setting SET key='currency',          value='SAR',         value_type='string';
CREATE setting SET key='date_format',       value='yyyy-MM-dd',  value_type='string';
CREATE setting SET key='time_format',       value='HH:mm',       value_type='string';
CREATE setting SET key='week_start',        value='sunday',      value_type='string';
CREATE setting SET key='fiscal_year_start', value='01',          value_type='string';
"#;

// ═══════════════════════════════════════════════════════════════════
//  Public API
// ═══════════════════════════════════════════════════════════════════

/// Provision a brand-new tenant database: create its schema, seed
/// permissions, role mappings, and default settings.
#[instrument(skip(platform_db, pool))]
pub async fn provision_tenant(
    platform_db: &PlatformDb,
    pool: &TenantConnectionPool,
    slug: &str,
) -> Result<(), AppError> {
    let db_name = format!("tenant_{slug}");

    // Get a raw connection so we can run arbitrary DDL.
    let conn = pool.get_raw(&db_name).await?;

    // platform_db is available for future cross-db lookups during
    // provisioning (e.g. fetching plan limits). Suppress warning.
    let _ = platform_db;

    info!(db_name = %db_name, "running tenant schema migration");
    conn.query(TENANT_SCHEMA_SQL)
        .await
        .map_err(|e| AppError::Database(format!("schema migration failed: {e}")))?;

    info!("seeding permissions");
    conn.query(SEED_PERMISSIONS_SQL)
        .await
        .map_err(|e| AppError::Database(format!("permission seeding failed: {e}")))?;

    info!("seeding role-permission mappings");
    conn.query(SEED_ROLE_PERMISSIONS_SQL)
        .await
        .map_err(|e| AppError::Database(format!("role-permission seeding failed: {e}")))?;

    info!("seeding default settings");
    conn.query(SEED_SETTINGS_SQL)
        .await
        .map_err(|e| AppError::Database(format!("settings seeding failed: {e}")))?;

    info!(db_name = %db_name, "tenant provisioned successfully");
    Ok(())
}

/// Create the founding admin account inside a freshly provisioned tenant
/// database. This account is linked back to the platform-level account
/// via `platform_id`.
#[instrument(skip(pool))]
pub async fn create_founder_account(
    pool: &TenantConnectionPool,
    db_name: &str,
    platform_id: &str,
    email: &str,
    display_name: &str,
) -> Result<Value, AppError> {
    let conn = pool.get_raw(db_name).await?;

    let mut resp = conn
        .query(
            "CREATE account SET
                platform_id    = $platform_id,
                email          = $email,
                display_name   = $display_name,
                role           = 'admin',
                status         = 'active',
                preferred_lang = 'en',
                created_at     = time::now(),
                updated_at     = time::now()
            ;",
        )
        .bind(("platform_id", platform_id.to_owned()))
        .bind(("email", email.to_owned()))
        .bind(("display_name", display_name.to_owned()))
        .await
        .map_err(|e| AppError::Database(format!("failed to create founder account: {e}")))?;

    let account: Option<Value> = resp
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    account.ok_or_else(|| AppError::Database("failed to create founder account".into()))
}
