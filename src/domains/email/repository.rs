//! Email Repository — عمليات قاعدة البيانات للبريد الإلكتروني

use super::models::*;
use crate::db::{AppState, DbError};

// ============================================================================
// Email Config CRUD
// ============================================================================

/// Get the active SMTP configuration
pub async fn get_active_config(state: &AppState) -> Result<Option<EmailConfig>, DbError> {
    let config: Option<EmailConfig> = state
        .db
        .query("SELECT * FROM email_config WHERE is_active = true LIMIT 1")
        .await?
        .take(0)?;
    Ok(config)
}

/// Create or update SMTP config
pub async fn upsert_config(
    state: &AppState,
    req: UpsertEmailConfigRequest,
) -> Result<EmailConfig, DbError> {
    let use_tls = req.use_tls.unwrap_or(true);
    let config: Option<EmailConfig> = state
        .db
        .query(
            "UPSERT email_config SET \
             host = $host, port = $port, username = $username, \
             password = $password, from_email = $from_email, \
             from_name = $from_name, use_tls = $use_tls, \
             is_active = true, updated_at = time::now()",
        )
        .bind(("host", req.host))
        .bind(("port", req.port))
        .bind(("username", req.username))
        .bind(("password", req.password))
        .bind(("from_email", req.from_email))
        .bind(("from_name", req.from_name))
        .bind(("use_tls", use_tls))
        .await?
        .take(0)?;
    config.ok_or(DbError::NotFound)
}

// ============================================================================
// Email Template CRUD
// ============================================================================

/// List all templates
pub async fn list_templates(state: &AppState) -> Result<Vec<EmailTemplate>, DbError> {
    let templates: Vec<EmailTemplate> = state
        .db
        .query("SELECT * FROM email_template ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(templates)
}

/// Get template by ID
pub async fn get_template(state: &AppState, id: &str) -> Result<EmailTemplate, DbError> {
    let template: Option<EmailTemplate> = state.db.select(("email_template", id)).await?;
    template.ok_or(DbError::NotFound)
}

/// Create a new template
pub async fn create_template(
    state: &AppState,
    req: CreateTemplateRequest,
) -> Result<EmailTemplate, DbError> {
    let category = req.category.unwrap_or_else(|| "custom".to_string());
    let variables = req.variables.unwrap_or_default();
    let template: Option<EmailTemplate> = state
        .db
        .query(
            "CREATE email_template SET \
             name = $name, subject = $subject, body = $body, \
             category = $category, variables = $variables",
        )
        .bind(("name", req.name))
        .bind(("subject", req.subject))
        .bind(("body", req.body))
        .bind(("category", category))
        .bind(("variables", variables))
        .await?
        .take(0)?;
    template.ok_or(DbError::NotFound)
}

/// Update a template
pub async fn update_template(
    state: &AppState,
    id: &str,
    req: UpdateTemplateRequest,
) -> Result<EmailTemplate, DbError> {
    let template: Option<EmailTemplate> =
        state.db.update(("email_template", id)).merge(req).await?;
    template.ok_or(DbError::NotFound)
}

/// Delete a template
pub async fn delete_template(state: &AppState, id: &str) -> Result<(), DbError> {
    let _: Option<EmailTemplate> = state.db.delete(("email_template", id)).await?;
    Ok(())
}

// ============================================================================
// Email Log
// ============================================================================

/// List email logs with optional status filter
pub async fn list_logs(
    state: &AppState,
    status_filter: Option<&str>,
) -> Result<Vec<EmailLog>, DbError> {
    let logs: Vec<EmailLog> = match status_filter {
        Some(status) => state
            .db
            .query("SELECT * FROM email_log WHERE status = $status ORDER BY sent_at DESC LIMIT 100")
            .bind(("status", status.to_string()))
            .await?
            .take(0)?,
        None => state
            .db
            .query("SELECT * FROM email_log ORDER BY sent_at DESC LIMIT 100")
            .await?
            .take(0)?,
    };
    Ok(logs)
}

/// Record a new email log entry
pub async fn create_log(
    state: &AppState,
    recipient: &str,
    subject: &str,
    body: Option<&str>,
    status: &str,
    error_msg: Option<&str>,
) -> Result<EmailLog, DbError> {
    let log: Option<EmailLog> = state
        .db
        .query(
            "CREATE email_log SET \
             recipient = $recipient, subject = $subject, \
             body = $body, status = $status, error_msg = $error_msg",
        )
        .bind(("recipient", recipient.to_string()))
        .bind(("subject", subject.to_string()))
        .bind(("body", body.map(|b| b.to_string())))
        .bind(("status", status.to_string()))
        .bind(("error_msg", error_msg.map(|e| e.to_string())))
        .await?
        .take(0)?;
    log.ok_or(DbError::NotFound)
}
