//! Agent Authentication Middleware — مصادقة وكلاء AI عبر API Key
//!
//! يستخرج مفتاح API من Header ويتحقق منه

use axum::{extract::State, http::Request, middleware::Next, response::Response};

use crate::db::AppState;
use crate::domains::agent::{repository, service};

/// Middleware that authenticates an AI agent via X-Agent-Key header
pub async fn require_agent_key(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, axum::response::Response> {
    // 1. Extract API key from header
    let api_key = req
        .headers()
        .get("X-Agent-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => {
            return Err(axum::response::IntoResponse::into_response((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": "Missing or empty X-Agent-Key header"
                })),
            )));
        }
    };

    // 2. Iterate through active agents and verify key
    let agents = repository::list_agents(&state).await.unwrap_or_default();

    let matched_agent = agents
        .into_iter()
        .find(|agent| agent.is_active && service::verify_api_key(&api_key, &agent.api_key_hash));

    let agent = match matched_agent {
        Some(a) => a,
        None => {
            return Err(axum::response::IntoResponse::into_response((
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({
                    "error": "Invalid or inactive API key"
                })),
            )));
        }
    };

    // 3. Update last_used_at
    if let Some(ref id) = agent.id {
        let _ = repository::touch_agent(&state, &id.id.to_raw()).await;
    }

    // 4. Inject agent into request extensions
    req.extensions_mut().insert(agent);

    Ok(next.run(req).await)
}
