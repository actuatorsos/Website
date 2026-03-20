//! Agent Domain Models — نماذج البنية التحتية لوكلاء AI

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// Agent Account — هوية الوكيل
// ============================================================================

/// AI agent account with API key and scoped permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAccount {
    pub id: Option<Thing>,
    pub name: String,
    pub description: Option<String>,
    pub api_key_hash: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: i32,
    #[serde(default = "default_true")]
    pub is_active: bool,
    pub created_by: Option<Thing>,
    pub created_at: Option<String>,
    pub last_used_at: Option<String>,
}

fn default_rate_limit() -> i32 {
    100
}
fn default_true() -> bool {
    true
}

/// Request to create a new agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limit: Option<i32>,
}

/// Response after creating agent (includes raw API key — shown once)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentResponse {
    pub agent: AgentAccount,
    /// Raw API key — shown only once at creation
    pub api_key: String,
}

/// Request to update agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub rate_limit: Option<i32>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Action Policy — سياسة تصنيف العمليات
// ============================================================================

/// Policy that classifies an operation as routine, sensitive, or critical
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPolicy {
    pub id: Option<Thing>,
    pub action: String,
    pub scope_required: String,
    pub sensitivity: String,
    pub auto_approve_conditions: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub requires_human: bool,
    #[serde(default)]
    pub is_blocked: bool,
    #[serde(default = "default_escalation")]
    pub escalation_hours: i32,
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_escalation() -> i32 {
    24
}

/// Request to create a policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    pub action: String,
    pub scope_required: String,
    pub sensitivity: Option<String>,
    pub auto_approve_conditions: Option<serde_json::Value>,
    pub requires_human: Option<bool>,
    pub is_blocked: Option<bool>,
    pub escalation_hours: Option<i32>,
    pub description: Option<String>,
}

// ============================================================================
// Approval Request — طلبات الموافقة
// ============================================================================

/// Approval request from an AI agent for a sensitive action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Option<Thing>,
    pub agent: Option<Thing>,
    pub action: String,
    pub target_table: String,
    pub target_id: Option<String>,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
    pub status: String,
    pub priority: String,
    pub reviewed_by: Option<Thing>,
    pub review_note: Option<String>,
    pub created_at: Option<String>,
    pub reviewed_at: Option<String>,
    pub expires_at: Option<String>,
    pub executed_at: Option<String>,
}

/// Agent's request to perform an action (evaluated by policy engine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentActionRequest {
    pub action: String,
    pub target_table: String,
    pub target_id: Option<String>,
    pub payload: serde_json::Value,
    pub reason: Option<String>,
    pub priority: Option<String>,
}

/// Result of policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub decision: String, // "executed", "pending_approval", "blocked"
    pub message: String,
    pub approval_id: Option<String>,
}

/// Human review of an approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewApprovalRequest {
    pub approved: bool,
    pub note: Option<String>,
}

// ============================================================================
// Agent Usage Log — سجل الاستخدام
// ============================================================================

/// Usage log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageLog {
    pub id: Option<Thing>,
    pub agent: Option<Thing>,
    pub endpoint: String,
    pub method: String,
    pub action: Option<String>,
    pub status_code: i32,
    pub response_ms: Option<i32>,
    pub was_approved: Option<bool>,
    pub was_blocked: Option<bool>,
    pub ip_address: Option<String>,
    pub timestamp: Option<String>,
}

// ============================================================================
// Notification — إشعارات
// ============================================================================

/// Notification model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Option<Thing>,
    pub recipient: Option<Thing>,
    pub title: String,
    pub body: String,
    #[serde(rename = "type")]
    pub notif_type: String,
    pub related_table: Option<String>,
    pub related_id: Option<String>,
    #[serde(default)]
    pub is_read: bool,
    pub channel: Option<String>,
    pub created_at: Option<String>,
}
