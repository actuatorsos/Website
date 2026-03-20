//! Email Domain Models — نماذج بيانات البريد الإلكتروني

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// Email Config — إعدادات SMTP
// ============================================================================

/// SMTP server configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// Unique identifier
    pub id: Option<Thing>,
    /// SMTP host (e.g. smtp.gmail.com)
    pub host: String,
    /// SMTP port (e.g. 587, 465)
    pub port: i32,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
    /// Sender email address
    pub from_email: String,
    /// Sender display name
    pub from_name: String,
    /// Use TLS encryption
    #[serde(default = "default_true")]
    pub use_tls: bool,
    /// Is this config active
    #[serde(default = "default_true")]
    pub is_active: bool,
    /// Last updated
    pub updated_at: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Request to create/update SMTP config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertEmailConfigRequest {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: Option<bool>,
}

// ============================================================================
// Email Template — قوالب البريد الإلكتروني
// ============================================================================

/// Email template stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Template name (unique)
    pub name: String,
    /// Email subject line
    pub subject: String,
    /// Email body (HTML or plain text)
    pub body: String,
    /// Category
    pub category: String,
    /// Template variables (e.g. ["name", "date"])
    #[serde(default)]
    pub variables: Vec<String>,
    /// Is active
    #[serde(default = "default_true")]
    pub is_active: bool,
    /// Creation date
    pub created_at: Option<String>,
}

/// Request to create a new template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub subject: String,
    pub body: String,
    pub category: Option<String>,
    pub variables: Option<Vec<String>>,
}

/// Request to update a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub category: Option<String>,
    pub variables: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Email Log — سجل الرسائل
// ============================================================================

/// Email send log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLog {
    /// Unique identifier
    pub id: Option<Thing>,
    /// Recipient email
    pub recipient: String,
    /// Email subject
    pub subject: String,
    /// Email body
    pub body: Option<String>,
    /// Template used (if any)
    pub template: Option<Thing>,
    /// Send status
    pub status: String,
    /// Error message if failed
    pub error_msg: Option<String>,
    /// Who sent it
    pub sent_by: Option<Thing>,
    /// When it was sent
    pub sent_at: Option<String>,
}

// ============================================================================
// Send Email Request — طلب إرسال بريد
// ============================================================================

/// Request to send an email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    /// List of recipient emails
    pub recipients: Vec<String>,
    /// Email subject
    pub subject: String,
    /// Email body (HTML)
    pub body: String,
    /// Optional template ID to use
    pub template_id: Option<String>,
    /// Variables to substitute in template
    pub variables: Option<std::collections::HashMap<String, String>>,
}

/// Response for send email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailResponse {
    /// Number of emails sent successfully
    pub sent: usize,
    /// Number of emails that failed
    pub failed: usize,
    /// Details per recipient
    pub results: Vec<SendResult>,
}

/// Per-recipient send result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub recipient: String,
    pub status: String,
    pub error: Option<String>,
}
