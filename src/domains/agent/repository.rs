//! Agent Repository — عمليات قاعدة البيانات للوكلاء

use super::models::*;
use crate::db::{AppState, DbError};

// ============================================================================
// Agent Account
// ============================================================================

/// Create a new agent
pub async fn create_agent(
    state: &AppState,
    name: &str,
    description: Option<&str>,
    api_key_hash: &str,
    scopes: &[String],
    rate_limit: i32,
) -> Result<AgentAccount, DbError> {
    let agent: Option<AgentAccount> = state.db
        .query("CREATE agent_account SET name = $name, description = $desc, api_key_hash = $hash, scopes = $scopes, rate_limit = $rl")
        .bind(("name", name.to_string()))
        .bind(("desc", description.map(|d| d.to_string())))
        .bind(("hash", api_key_hash.to_string()))
        .bind(("scopes", scopes.to_vec()))
        .bind(("rl", rate_limit))
        .await?.take(0)?;
    agent.ok_or(DbError::NotFound)
}

/// List all agents
pub async fn list_agents(state: &AppState) -> Result<Vec<AgentAccount>, DbError> {
    let agents: Vec<AgentAccount> = state
        .db
        .query("SELECT * FROM agent_account ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(agents)
}

/// Get agent by ID
pub async fn get_agent(state: &AppState, id: &str) -> Result<AgentAccount, DbError> {
    let agent: Option<AgentAccount> = state.db.select(("agent_account", id)).await?;
    agent.ok_or(DbError::NotFound)
}

/// Find agent by API key hash
pub async fn find_agent_by_key_hash(
    state: &AppState,
    hash: &str,
) -> Result<Option<AgentAccount>, DbError> {
    let agent: Option<AgentAccount> = state
        .db
        .query(
            "SELECT * FROM agent_account WHERE api_key_hash = $hash AND is_active = true LIMIT 1",
        )
        .bind(("hash", hash.to_string()))
        .await?
        .take(0)?;
    Ok(agent)
}

/// Update last_used_at
pub async fn touch_agent(state: &AppState, id: &str) -> Result<(), DbError> {
    state.db
        .query("UPDATE agent_account SET last_used_at = time::now() WHERE id = type::thing('agent_account', $id)")
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

/// Update agent
pub async fn update_agent(
    state: &AppState,
    id: &str,
    req: UpdateAgentRequest,
) -> Result<AgentAccount, DbError> {
    let agent: Option<AgentAccount> = state.db.update(("agent_account", id)).merge(req).await?;
    agent.ok_or(DbError::NotFound)
}

/// Delete agent
pub async fn delete_agent(state: &AppState, id: &str) -> Result<(), DbError> {
    let _: Option<AgentAccount> = state.db.delete(("agent_account", id)).await?;
    Ok(())
}

// ============================================================================
// Action Policy
// ============================================================================

/// List all policies
pub async fn list_policies(state: &AppState) -> Result<Vec<ActionPolicy>, DbError> {
    let policies: Vec<ActionPolicy> = state
        .db
        .query("SELECT * FROM action_policy WHERE is_active = true ORDER BY action ASC")
        .await?
        .take(0)?;
    Ok(policies)
}

/// Get policy by action name
pub async fn get_policy_by_action(
    state: &AppState,
    action: &str,
) -> Result<Option<ActionPolicy>, DbError> {
    let policy: Option<ActionPolicy> = state
        .db
        .query("SELECT * FROM action_policy WHERE action = $action AND is_active = true LIMIT 1")
        .bind(("action", action.to_string()))
        .await?
        .take(0)?;
    Ok(policy)
}

/// Create policy
pub async fn create_policy(
    state: &AppState,
    req: CreatePolicyRequest,
) -> Result<ActionPolicy, DbError> {
    let sensitivity = req.sensitivity.unwrap_or_else(|| "sensitive".to_string());
    let requires_human = req.requires_human.unwrap_or(true);
    let is_blocked = req.is_blocked.unwrap_or(false);
    let hours = req.escalation_hours.unwrap_or(24);

    let policy: Option<ActionPolicy> = state.db
        .query("CREATE action_policy SET action = $action, scope_required = $scope, sensitivity = $sens, auto_approve_conditions = $cond, requires_human = $human, is_blocked = $blocked, escalation_hours = $hours, description = $desc")
        .bind(("action", req.action))
        .bind(("scope", req.scope_required))
        .bind(("sens", sensitivity))
        .bind(("cond", req.auto_approve_conditions))
        .bind(("human", requires_human))
        .bind(("blocked", is_blocked))
        .bind(("hours", hours))
        .bind(("desc", req.description))
        .await?.take(0)?;
    policy.ok_or(DbError::NotFound)
}

// ============================================================================
// Approval Request
// ============================================================================

/// Create an approval request
pub async fn create_approval(
    state: &AppState,
    agent_id: &str,
    action: &str,
    target_table: &str,
    target_id: Option<&str>,
    payload: &serde_json::Value,
    reason: Option<&str>,
    priority: &str,
    escalation_hours: i32,
) -> Result<ApprovalRequest, DbError> {
    let approval: Option<ApprovalRequest> = state.db
        .query("CREATE approval_request SET agent = type::thing('agent_account', $agent_id), action = $action, target_table = $table, target_id = $tid, payload = $payload, reason = $reason, priority = $priority, expires_at = time::now() + type::duration($dur)")
        .bind(("agent_id", agent_id.to_string()))
        .bind(("action", action.to_string()))
        .bind(("table", target_table.to_string()))
        .bind(("tid", target_id.map(|s| s.to_string())))
        .bind(("payload", payload.clone()))
        .bind(("reason", reason.map(|s| s.to_string())))
        .bind(("priority", priority.to_string()))
        .bind(("dur", format!("{}h", escalation_hours)))
        .await?.take(0)?;
    approval.ok_or(DbError::NotFound)
}

/// List pending approvals
pub async fn list_pending_approvals(state: &AppState) -> Result<Vec<ApprovalRequest>, DbError> {
    let approvals: Vec<ApprovalRequest> = state
        .db
        .query("SELECT * FROM approval_request WHERE status = 'pending' ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(approvals)
}

/// List all approvals
pub async fn list_approvals(
    state: &AppState,
    status: Option<&str>,
) -> Result<Vec<ApprovalRequest>, DbError> {
    let approvals: Vec<ApprovalRequest> = match status {
        Some(s) => state.db
            .query("SELECT * FROM approval_request WHERE status = $status ORDER BY created_at DESC LIMIT 100")
            .bind(("status", s.to_string()))
            .await?.take(0)?,
        None => state.db
            .query("SELECT * FROM approval_request ORDER BY created_at DESC LIMIT 100")
            .await?.take(0)?,
    };
    Ok(approvals)
}

/// Review (approve/reject) a request
pub async fn review_approval(
    state: &AppState,
    id: &str,
    approved: bool,
    reviewer_id: Option<&str>,
    note: Option<&str>,
) -> Result<ApprovalRequest, DbError> {
    let new_status = if approved { "approved" } else { "rejected" };
    let approval: Option<ApprovalRequest> = state.db
        .query("UPDATE type::thing('approval_request', $id) SET status = $status, review_note = $note, reviewed_at = time::now() RETURN AFTER")
        .bind(("id", id.to_string()))
        .bind(("status", new_status.to_string()))
        .bind(("note", note.map(|s| s.to_string())))
        .await?.take(0)?;
    approval.ok_or(DbError::NotFound)
}

// ============================================================================
// Notification
// ============================================================================

/// Create a notification
pub async fn create_notification(
    state: &AppState,
    recipient_id: &str,
    title: &str,
    body: &str,
    notif_type: &str,
    channel: &str,
) -> Result<Notification, DbError> {
    let notif: Option<Notification> = state.db
        .query("CREATE notification SET recipient = type::thing('account', $rid), title = $title, body = $body, type = $ntype, channel = $ch")
        .bind(("rid", recipient_id.to_string()))
        .bind(("title", title.to_string()))
        .bind(("body", body.to_string()))
        .bind(("ntype", notif_type.to_string()))
        .bind(("ch", channel.to_string()))
        .await?.take(0)?;
    notif.ok_or(DbError::NotFound)
}

/// List unread notifications for a user
pub async fn list_notifications(
    state: &AppState,
    user_id: &str,
    unread_only: bool,
) -> Result<Vec<Notification>, DbError> {
    let notifs: Vec<Notification> = if unread_only {
        state.db
            .query("SELECT * FROM notification WHERE recipient = type::thing('account', $uid) AND is_read = false ORDER BY created_at DESC")
            .bind(("uid", user_id.to_string()))
            .await?.take(0)?
    } else {
        state.db
            .query("SELECT * FROM notification WHERE recipient = type::thing('account', $uid) ORDER BY created_at DESC LIMIT 50")
            .bind(("uid", user_id.to_string()))
            .await?.take(0)?
    };
    Ok(notifs)
}

/// Mark notification as read
pub async fn mark_read(state: &AppState, id: &str) -> Result<(), DbError> {
    state
        .db
        .query("UPDATE type::thing('notification', $id) SET is_read = true")
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

// ============================================================================
// Agent Usage Log
// ============================================================================

/// Log an agent API usage
pub async fn log_usage(
    state: &AppState,
    agent_id: &str,
    endpoint: &str,
    method: &str,
    action: Option<&str>,
    status_code: i32,
    response_ms: Option<i32>,
    was_approved: Option<bool>,
    was_blocked: Option<bool>,
) -> Result<(), DbError> {
    state.db
        .query("CREATE agent_usage_log SET agent = type::thing('agent_account', $aid), endpoint = $ep, method = $method, action = $action, status_code = $code, response_ms = $ms, was_approved = $approved, was_blocked = $blocked")
        .bind(("aid", agent_id.to_string()))
        .bind(("ep", endpoint.to_string()))
        .bind(("method", method.to_string()))
        .bind(("action", action.map(|s| s.to_string())))
        .bind(("code", status_code))
        .bind(("ms", response_ms))
        .bind(("approved", was_approved))
        .bind(("blocked", was_blocked))
        .await?;
    Ok(())
}
