//! Agent Policy Engine — محرك تقييم سياسات العمليات
//!
//! يقيّم ما إذا كان يحق للوكيل تنفيذ العملية فوراً أو يحتاج موافقة بشرية

use super::models::*;
use super::repository;
use crate::db::AppState;

/// Evaluate an agent's action request against the policy engine
pub async fn evaluate_action(
    state: &AppState,
    agent: &AgentAccount,
    request: &AgentActionRequest,
) -> Result<PolicyDecision, String> {
    // 1. Lookup policy for this action
    let policy = repository::get_policy_by_action(state, &request.action)
        .await
        .map_err(|e| format!("Policy lookup error: {}", e))?;

    let policy = match policy {
        Some(p) => p,
        None => {
            // No policy defined = require human approval by default
            return Ok(PolicyDecision {
                allowed: false,
                decision: "pending_approval".to_string(),
                message: format!(
                    "No policy defined for action '{}'. Creating approval request.",
                    request.action
                ),
                approval_id: create_approval_for_agent(state, agent, request, 24)
                    .await
                    .ok(),
            });
        }
    };

    // 2. Check if action is blocked entirely
    if policy.is_blocked {
        return Ok(PolicyDecision {
            allowed: false,
            decision: "blocked".to_string(),
            message: format!(
                "Action '{}' is permanently blocked for AI agents.",
                request.action
            ),
            approval_id: None,
        });
    }

    // 3. Check if agent has required scope
    if !agent.scopes.contains(&policy.scope_required) {
        return Ok(PolicyDecision {
            allowed: false,
            decision: "blocked".to_string(),
            message: format!(
                "Agent '{}' lacks required scope: '{}'",
                agent.name, policy.scope_required
            ),
            approval_id: None,
        });
    }

    // 4. Decision based on sensitivity
    match policy.sensitivity.as_str() {
        "routine" if !policy.requires_human => {
            // Auto-approve routine actions
            Ok(PolicyDecision {
                allowed: true,
                decision: "executed".to_string(),
                message: format!("Routine action '{}' auto-approved.", request.action),
                approval_id: None,
            })
        }
        "sensitive" | "routine" => {
            // Create approval request
            let approval_id =
                create_approval_for_agent(state, agent, request, policy.escalation_hours)
                    .await
                    .map_err(|e| format!("Failed to create approval: {}", e))?;
            Ok(PolicyDecision {
                allowed: false,
                decision: "pending_approval".to_string(),
                message: format!(
                    "Action '{}' requires human approval. Request created.",
                    request.action
                ),
                approval_id: Some(approval_id),
            })
        }
        "critical" => {
            // Always blocked for agents — must be done by human
            Ok(PolicyDecision {
                allowed: false,
                decision: "blocked".to_string(),
                message: format!(
                    "Critical action '{}' cannot be performed by AI agents.",
                    request.action
                ),
                approval_id: None,
            })
        }
        _ => Ok(PolicyDecision {
            allowed: false,
            decision: "blocked".to_string(),
            message: "Unknown sensitivity level.".to_string(),
            approval_id: None,
        }),
    }
}

/// Create an approval request for a given agent action
async fn create_approval_for_agent(
    state: &AppState,
    agent: &AgentAccount,
    request: &AgentActionRequest,
    escalation_hours: i32,
) -> Result<String, String> {
    let agent_id = agent
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .ok_or_else(|| "Agent has no ID".to_string())?;

    let priority = request.priority.as_deref().unwrap_or("normal");

    let approval = repository::create_approval(
        state,
        &agent_id,
        &request.action,
        &request.target_table,
        request.target_id.as_deref(),
        &request.payload,
        request.reason.as_deref(),
        priority,
        escalation_hours,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(approval.id.map(|t| t.id.to_raw()).unwrap_or_default())
}

/// Generate a random API key
pub fn generate_api_key() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    format!("drm_agent_{}", hex::encode(bytes))
}

/// Hash an API key using argon2
pub fn hash_api_key(key: &str) -> Result<String, String> {
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(key.as_bytes(), &salt)
        .map_err(|e| format!("Hash error: {}", e))?;
    Ok(hash.to_string())
}

/// Verify an API key against a stored hash
pub fn verify_api_key(key: &str, hash: &str) -> bool {
    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};

    match PasswordHash::new(hash) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(key.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}
