use actuators_core::error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tracing::instrument;

// ── Record types used by downstream crates ─────────────────────────

/// An account-level permission override record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPermissionOverride {
    pub granted: bool,
}

/// A role-permission mapping record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissionRecord {
    pub granted: bool,
    pub scope: Option<String>,
}

// ── TenantDb ───────────────────────────────────────────────────────

/// Wraps a SurrealDB connection scoped to a single tenant database and
/// exposes CRUD helpers for every tenant-level entity.
#[derive(Debug, Clone)]
pub struct TenantDb {
    pub db: Surreal<Any>,
}

impl TenantDb {
    /// Wrap an already-connected client.
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Return a reference to the inner client.
    pub fn client(&self) -> &Surreal<Any> {
        &self.db
    }

    // ================================================================
    //  Accounts
    // ================================================================

    #[instrument(skip(self))]
    pub async fn create_account(
        &self,
        platform_id: &str,
        email: &str,
        display_name: &str,
        role: &str,
        preferred_lang: &str,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE account SET
                    platform_id    = $platform_id,
                    email          = $email,
                    display_name   = $display_name,
                    role           = $role,
                    status         = 'active',
                    preferred_lang = $preferred_lang,
                    created_at     = time::now(),
                    updated_at     = time::now()
                ;",
            )
            .bind(("platform_id", platform_id.to_owned()))
            .bind(("email", email.to_owned()))
            .bind(("display_name", display_name.to_owned()))
            .bind(("role", role.to_owned()))
            .bind(("preferred_lang", preferred_lang.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        account.ok_or_else(|| AppError::Database("failed to create tenant account".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_id(&self, id: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::thing('account', $id);")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_platform_id(
        &self,
        platform_id: &str,
    ) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM account WHERE platform_id = $platform_id LIMIT 1;")
            .bind(("platform_id", platform_id.to_owned()))
            .await
            .map_err(db_err)?;

        let account: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(account)
    }

    #[instrument(skip(self))]
    pub async fn list_accounts(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM account WHERE status != 'deleted' ORDER BY created_at DESC;",
            )
            .await
            .map_err(db_err)?;

        let accounts: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(accounts)
    }

    #[instrument(skip(self, fields))]
    pub async fn update_account(&self, id: &str, fields: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('account', $id) MERGE $fields SET updated_at = time::now();",
            )
            .bind(("id", id.to_owned()))
            .bind(("fields", fields))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("tenant account not found".into()))
    }

    /// Soft-delete an account by setting `status = 'deleted'`.
    #[instrument(skip(self))]
    pub async fn delete_account(&self, id: &str) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('account', $id) SET
                    status     = 'deleted',
                    updated_at = time::now()
                ;",
            )
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("tenant account not found".into()))
    }

    // ================================================================
    //  Employees
    // ================================================================

    #[instrument(skip(self, data))]
    pub async fn create_employee(&self, data: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE employee CONTENT $data SET
                    created_at = time::now(),
                    updated_at = time::now()
                ;",
            )
            .bind(("data", data))
            .await
            .map_err(db_err)?;

        let employee: Option<Value> = resp.take(0).map_err(db_err)?;
        employee.ok_or_else(|| AppError::Database("failed to create employee".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_employee_by_id(&self, id: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::thing('employee', $id);")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let employee: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(employee)
    }

    /// List employees with optional filters.
    ///
    /// - `department` -- filter by `department_id`
    /// - `status` -- filter by employment status
    /// - `search` -- case-insensitive match on first/last name (EN)
    /// - `limit` / `offset` -- pagination
    #[instrument(skip(self))]
    pub async fn list_employees(
        &self,
        department: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Value>, AppError> {
        let mut clauses: Vec<String> = Vec::new();

        if department.is_some() {
            clauses.push("department_id = $department".into());
        }
        if status.is_some() {
            clauses.push("status = $status".into());
        }
        if search.is_some() {
            clauses.push(
                "(string::lowercase(first_name_en) CONTAINS string::lowercase($search) \
                 OR string::lowercase(last_name_en) CONTAINS string::lowercase($search))"
                    .into(),
            );
        }

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT * FROM employee {where_clause} \
             ORDER BY created_at DESC LIMIT $limit START $offset;"
        );

        let mut query = self.db.query(&sql);

        if let Some(dept) = department {
            query = query.bind(("department", dept.to_owned()));
        }
        if let Some(st) = status {
            query = query.bind(("status", st.to_owned()));
        }
        if let Some(q) = search {
            query = query.bind(("search", q.to_owned()));
        }
        query = query.bind(("limit", limit));
        query = query.bind(("offset", offset));

        let mut resp = query.await.map_err(db_err)?;
        let employees: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(employees)
    }

    #[instrument(skip(self, fields))]
    pub async fn update_employee(&self, id: &str, fields: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('employee', $id) MERGE $fields SET updated_at = time::now();",
            )
            .bind(("id", id.to_owned()))
            .bind(("fields", fields))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("employee not found".into()))
    }

    #[instrument(skip(self))]
    pub async fn terminate_employee(&self, id: &str) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('employee', $id) SET
                    status           = 'terminated',
                    termination_date = time::now(),
                    updated_at       = time::now()
                ;",
            )
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("employee not found".into()))
    }

    // ================================================================
    //  Departments
    // ================================================================

    #[instrument(skip(self))]
    pub async fn create_department(
        &self,
        name_en: &str,
        name_ar: &str,
        code: &str,
        parent_id: Option<&str>,
        manager_id: Option<&str>,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE department SET
                    name_en    = $name_en,
                    name_ar    = $name_ar,
                    code       = $code,
                    parent_id  = $parent_id,
                    manager_id = $manager_id,
                    is_active  = true,
                    created_at = time::now(),
                    updated_at = time::now()
                ;",
            )
            .bind(("name_en", name_en.to_owned()))
            .bind(("name_ar", name_ar.to_owned()))
            .bind(("code", code.to_owned()))
            .bind(("parent_id", parent_id.map(|s| s.to_owned())))
            .bind(("manager_id", manager_id.map(|s| s.to_owned())))
            .await
            .map_err(db_err)?;

        let dept: Option<Value> = resp.take(0).map_err(db_err)?;
        dept.ok_or_else(|| AppError::Database("failed to create department".into()))
    }

    #[instrument(skip(self))]
    pub async fn get_department_by_id(&self, id: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::thing('department', $id);")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let dept: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(dept)
    }

    #[instrument(skip(self))]
    pub async fn list_departments(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM department ORDER BY name_en ASC;")
            .await
            .map_err(db_err)?;

        let departments: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(departments)
    }

    #[instrument(skip(self, fields))]
    pub async fn update_department(&self, id: &str, fields: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "UPDATE type::thing('department', $id) MERGE $fields SET updated_at = time::now();",
            )
            .bind(("id", id.to_owned()))
            .bind(("fields", fields))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("department not found".into()))
    }

    // ================================================================
    //  Job Titles
    // ================================================================

    #[instrument(skip(self))]
    pub async fn create_job_title(
        &self,
        name_en: &str,
        name_ar: &str,
        department_id: Option<&str>,
        level: Option<i32>,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE job_title SET
                    name_en       = $name_en,
                    name_ar       = $name_ar,
                    department_id = $department_id,
                    level         = $level,
                    is_active     = true,
                    created_at    = time::now()
                ;",
            )
            .bind(("name_en", name_en.to_owned()))
            .bind(("name_ar", name_ar.to_owned()))
            .bind(("department_id", department_id.map(|s| s.to_owned())))
            .bind(("level", level))
            .await
            .map_err(db_err)?;

        let title: Option<Value> = resp.take(0).map_err(db_err)?;
        title.ok_or_else(|| AppError::Database("failed to create job title".into()))
    }

    #[instrument(skip(self))]
    pub async fn list_job_titles(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM job_title ORDER BY name_en ASC;")
            .await
            .map_err(db_err)?;

        let titles: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(titles)
    }

    // ================================================================
    //  Contracts
    // ================================================================

    #[instrument(skip(self, data))]
    pub async fn create_contract(&self, data: Value) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE contract CONTENT $data SET
                    is_current = true,
                    created_at = time::now()
                ;",
            )
            .bind(("data", data))
            .await
            .map_err(db_err)?;

        let contract: Option<Value> = resp.take(0).map_err(db_err)?;
        contract.ok_or_else(|| AppError::Database("failed to create contract".into()))
    }

    #[instrument(skip(self))]
    pub async fn list_contracts_for_employee(
        &self,
        employee_id: &str,
    ) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM contract
                    WHERE employee_id = $employee_id
                    ORDER BY start_date DESC;",
            )
            .bind(("employee_id", employee_id.to_owned()))
            .await
            .map_err(db_err)?;

        let contracts: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(contracts)
    }

    // ================================================================
    //  Settings
    // ================================================================

    #[instrument(skip(self))]
    pub async fn get_setting(&self, key: &str) -> Result<Option<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM setting WHERE key = $key LIMIT 1;")
            .bind(("key", key.to_owned()))
            .await
            .map_err(db_err)?;

        let setting: Option<Value> = resp.take(0).map_err(db_err)?;
        Ok(setting)
    }

    /// Upsert a tenant setting. If the key already exists it is updated;
    /// otherwise a new record is created.
    #[instrument(skip(self))]
    pub async fn set_setting(
        &self,
        key: &str,
        value: &str,
        value_type: &str,
        updated_by: Option<&str>,
    ) -> Result<Value, AppError> {
        // Try update first.
        let mut resp = self
            .db
            .query(
                "UPDATE setting SET
                    value      = $value,
                    value_type = $value_type,
                    updated_by = $updated_by,
                    updated_at = time::now()
                WHERE key = $key;",
            )
            .bind(("key", key.to_owned()))
            .bind(("value", value.to_owned()))
            .bind(("value_type", value_type.to_owned()))
            .bind(("updated_by", updated_by.map(|s| s.to_owned())))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        if let Some(v) = updated {
            return Ok(v);
        }

        // Key didn't exist yet — create it.
        let mut resp = self
            .db
            .query(
                "CREATE setting SET
                    key        = $key,
                    value      = $value,
                    value_type = $value_type,
                    updated_by = $updated_by,
                    updated_at = time::now()
                ;",
            )
            .bind(("key", key.to_owned()))
            .bind(("value", value.to_owned()))
            .bind(("value_type", value_type.to_owned()))
            .bind(("updated_by", updated_by.map(|s| s.to_owned())))
            .await
            .map_err(db_err)?;

        let created: Option<Value> = resp.take(0).map_err(db_err)?;
        created.ok_or_else(|| AppError::Database("failed to upsert setting".into()))
    }

    #[instrument(skip(self))]
    pub async fn list_settings(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM setting ORDER BY key ASC;")
            .await
            .map_err(db_err)?;

        let settings: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(settings)
    }

    // ================================================================
    //  Audit Log
    // ================================================================

    #[instrument(skip(self, changes))]
    pub async fn log_audit(
        &self,
        actor_id: &str,
        actor_name: &str,
        action: &str,
        service: &str,
        entity_type: &str,
        entity_id: Option<&str>,
        changes: Option<Value>,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), AppError> {
        self.db
            .query(
                "CREATE audit_log SET
                    actor_id    = $actor_id,
                    actor_name  = $actor_name,
                    action      = $action,
                    service     = $service,
                    entity_type = $entity_type,
                    entity_id   = $entity_id,
                    changes     = $changes,
                    ip_address  = $ip,
                    user_agent  = $user_agent,
                    created_at  = time::now()
                ;",
            )
            .bind(("actor_id", actor_id.to_owned()))
            .bind(("actor_name", actor_name.to_owned()))
            .bind(("action", action.to_owned()))
            .bind(("service", service.to_owned()))
            .bind(("entity_type", entity_type.to_owned()))
            .bind(("entity_id", entity_id.map(|s| s.to_owned())))
            .bind(("changes", changes))
            .bind(("ip", ip.map(|s| s.to_owned())))
            .bind(("user_agent", user_agent.map(|s| s.to_owned())))
            .await
            .map_err(db_err)?;

        Ok(())
    }

    /// List audit logs with optional filters and pagination.
    #[instrument(skip(self))]
    pub async fn list_audit_logs(
        &self,
        service: Option<&str>,
        action: Option<&str>,
        actor_id: Option<&str>,
        date_from: Option<&str>,
        date_to: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Value>, AppError> {
        let mut clauses: Vec<String> = Vec::new();

        if service.is_some() {
            clauses.push("service = $service".into());
        }
        if action.is_some() {
            clauses.push("action = $action".into());
        }
        if actor_id.is_some() {
            clauses.push("actor_id = $actor_id".into());
        }
        if date_from.is_some() {
            clauses.push("created_at >= type::datetime($date_from)".into());
        }
        if date_to.is_some() {
            clauses.push("created_at <= type::datetime($date_to)".into());
        }

        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let sql = format!(
            "SELECT * FROM audit_log {where_clause} \
             ORDER BY created_at DESC LIMIT $limit START $offset;"
        );

        let mut query = self.db.query(&sql);

        if let Some(s) = service {
            query = query.bind(("service", s.to_owned()));
        }
        if let Some(a) = action {
            query = query.bind(("action", a.to_owned()));
        }
        if let Some(aid) = actor_id {
            query = query.bind(("actor_id", aid.to_owned()));
        }
        if let Some(df) = date_from {
            query = query.bind(("date_from", df.to_owned()));
        }
        if let Some(dt) = date_to {
            query = query.bind(("date_to", dt.to_owned()));
        }
        query = query.bind(("limit", limit));
        query = query.bind(("offset", offset));

        let mut resp = query.await.map_err(db_err)?;
        let logs: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(logs)
    }

    // ================================================================
    //  Notifications
    // ================================================================

    #[instrument(skip(self))]
    pub async fn create_notification(
        &self,
        account_id: &str,
        title_en: &str,
        title_ar: Option<&str>,
        body_en: Option<&str>,
        body_ar: Option<&str>,
        link: Option<&str>,
    ) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query(
                "CREATE notification SET
                    account_id = $account_id,
                    title_en   = $title_en,
                    title_ar   = $title_ar,
                    body_en    = $body_en,
                    body_ar    = $body_ar,
                    link       = $link,
                    is_read    = false,
                    created_at = time::now()
                ;",
            )
            .bind(("account_id", account_id.to_owned()))
            .bind(("title_en", title_en.to_owned()))
            .bind(("title_ar", title_ar.map(|s| s.to_owned())))
            .bind(("body_en", body_en.map(|s| s.to_owned())))
            .bind(("body_ar", body_ar.map(|s| s.to_owned())))
            .bind(("link", link.map(|s| s.to_owned())))
            .await
            .map_err(db_err)?;

        let notif: Option<Value> = resp.take(0).map_err(db_err)?;
        notif.ok_or_else(|| AppError::Database("failed to create notification".into()))
    }

    #[instrument(skip(self))]
    pub async fn list_notifications_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT * FROM notification
                    WHERE account_id = $account_id
                    ORDER BY created_at DESC;",
            )
            .bind(("account_id", account_id.to_owned()))
            .await
            .map_err(db_err)?;

        let notifications: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(notifications)
    }

    #[instrument(skip(self))]
    pub async fn mark_read(&self, id: &str) -> Result<Value, AppError> {
        let mut resp = self
            .db
            .query("UPDATE type::thing('notification', $id) SET is_read = true;")
            .bind(("id", id.to_owned()))
            .await
            .map_err(db_err)?;

        let updated: Option<Value> = resp.take(0).map_err(db_err)?;
        updated.ok_or_else(|| AppError::NotFound("notification not found".into()))
    }

    // ================================================================
    //  Permissions
    // ================================================================

    #[instrument(skip(self))]
    pub async fn list_permissions(&self) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query("SELECT * FROM permission ORDER BY key ASC;")
            .await
            .map_err(db_err)?;

        let perms: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(perms)
    }

    #[instrument(skip(self))]
    pub async fn get_permissions_for_role(&self, role: &str) -> Result<Vec<Value>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT
                    permission_id,
                    scope,
                    (SELECT key, name_en, name_ar, service_key, resource, action
                        FROM type::thing('permission', $parent.permission_id)
                    )[0] AS permission
                FROM role_permission
                WHERE role = $role;",
            )
            .bind(("role", role.to_owned()))
            .await
            .map_err(db_err)?;

        let perms: Vec<Value> = resp.take(0).map_err(db_err)?;
        Ok(perms)
    }

    /// Check whether an account (with its assigned role) holds a
    /// specific permission key. Account-level overrides take precedence.
    #[instrument(skip(self))]
    pub async fn check_permission(
        &self,
        account_id: &str,
        role: &str,
        permission_key: &str,
    ) -> Result<bool, AppError> {
        // 1. Check for an account-level override.
        if let Some(ovr) = self
            .get_account_permission_override(account_id, permission_key)
            .await?
        {
            return Ok(ovr.granted);
        }

        // 2. Fall back to role-based permission.
        if let Some(rp) = self.get_role_permission(role, permission_key).await? {
            return Ok(rp.granted);
        }

        // 3. Default deny.
        Ok(false)
    }

    // ── Backward-compatible permission helpers ─────────────────────

    /// Check for an account-level permission override.
    pub async fn get_account_permission_override(
        &self,
        account_id: &str,
        permission_key: &str,
    ) -> Result<Option<AccountPermissionOverride>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT granted FROM account_permission_override
                    WHERE account_id = $aid
                      AND permission_id = (
                          SELECT VALUE <string> id FROM permission WHERE key = $pkey LIMIT 1
                      )
                    LIMIT 1;",
            )
            .bind(("aid", account_id.to_owned()))
            .bind(("pkey", permission_key.to_owned()))
            .await
            .map_err(db_err)?;

        let raw: Option<Value> = resp.take(0).map_err(db_err)?;
        let record: Option<AccountPermissionOverride> = raw
            .map(|v| serde_json::from_value(v))
            .transpose()
            .map_err(|e| AppError::Database(format!("failed to deserialize AccountPermissionOverride: {e}")))?;
        Ok(record)
    }

    /// Check if a role has a specific permission.
    pub async fn get_role_permission(
        &self,
        role: &str,
        permission_key: &str,
    ) -> Result<Option<RolePermissionRecord>, AppError> {
        let mut resp = self
            .db
            .query(
                "SELECT true AS granted, scope FROM role_permission
                    WHERE role = $role
                      AND permission_id = (
                          SELECT VALUE <string> id FROM permission WHERE key = $pkey LIMIT 1
                      )
                    LIMIT 1;",
            )
            .bind(("role", role.to_owned()))
            .bind(("pkey", permission_key.to_owned()))
            .await
            .map_err(db_err)?;

        let raw: Option<Value> = resp.take(0).map_err(db_err)?;
        let record: Option<RolePermissionRecord> = raw
            .map(|v| serde_json::from_value(v))
            .transpose()
            .map_err(|e| AppError::Database(format!("failed to deserialize RolePermissionRecord: {e}")))?;
        Ok(record)
    }
}

// ── Helpers ────────────────────────────────────────────────────────

/// Convert a SurrealDB error into an `AppError::Database`.
fn db_err(e: surrealdb::Error) -> AppError {
    AppError::Database(e.to_string())
}
