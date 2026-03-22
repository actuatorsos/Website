use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use actuators_core::error::AppError;
use actuators_db::tenant::TenantDb;

use crate::middleware::{AppState, TenantAuth};

// ── Permission checks ─────────────────────────────────────────────────

/// Check whether an account has a specific permission.
///
/// Resolution order:
/// 1. Look for an account-level override in `account_permission_override`.
///    If present, its `granted` flag takes precedence.
/// 2. Otherwise, check the role-level permission in `role_permission`.
/// 3. If neither record exists the permission is denied.
pub async fn check_permission(
    tenant_db: &TenantDb,
    account_id: &str,
    role: &str,
    permission_key: &str,
) -> Result<bool, AppError> {
    // 1. Account-level override
    if let Some(ovr) = tenant_db
        .get_account_permission_override(account_id, permission_key)
        .await?
    {
        tracing::debug!(
            account_id,
            permission_key,
            granted = ovr.granted,
            "account permission override found"
        );
        return Ok(ovr.granted);
    }

    // 2. Role-level permission
    if let Some(rp) = tenant_db.get_role_permission(role, permission_key).await? {
        tracing::debug!(
            role,
            permission_key,
            granted = rp.granted,
            "role permission found"
        );
        return Ok(rp.granted);
    }

    // 3. Default deny
    tracing::debug!(
        account_id,
        role,
        permission_key,
        "no permission record found — denied"
    );
    Ok(false)
}

/// Retrieve the scope for a role's permission (e.g. `"all"`, `"department"`,
/// `"self"`).
///
/// Returns `None` if the role has no record for the given permission key.
pub async fn get_permission_scope(
    tenant_db: &TenantDb,
    role: &str,
    permission_key: &str,
) -> Result<Option<String>, AppError> {
    let record = tenant_db.get_role_permission(role, permission_key).await?;

    match record {
        Some(rp) => {
            tracing::debug!(role, permission_key, scope = ?rp.scope, "permission scope resolved");
            Ok(rp.scope)
        }
        None => {
            tracing::debug!(role, permission_key, "no permission scope found");
            Ok(None)
        }
    }
}

// ── Role-checking middleware ──────────────────────────────────────────

/// Create an Axum middleware that rejects requests whose authenticated
/// tenant role is not in `allowed_roles`.
///
/// Usage:
/// ```ignore
/// use axum::middleware;
/// use actuators_auth::rbac::require_role;
///
/// let router = Router::new()
///     .route("/admin/settings", get(handler))
///     .route_layer(middleware::from_fn_with_state(
///         state.clone(),
///         require_role(&["admin", "super_admin"]),
///     ));
/// ```
///
/// Note: Because `from_fn` requires a concrete function signature rather
/// than a closure, we return a boxed middleware.  The caller should use
/// `axum::middleware::from_fn_with_state` with the returned function.
pub fn require_role(
    allowed_roles: &'static [&'static str],
) -> impl Fn(
    State<Arc<AppState>>,
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>>
       + Clone
       + Send
       + 'static {
    move |State(_state): State<Arc<AppState>>, req: Request, next: Next| {
        let allowed = allowed_roles;
        Box::pin(async move {
            // Extract the tenant auth context from request extensions.
            let auth = req
                .extensions()
                .get::<TenantAuth>()
                .or_else(|| {
                    // Try the inner context directly — middleware further up the
                    // chain may have inserted it.
                    None
                });

            // Also check for a raw TenantAuthContext inserted by middleware.
            let role = req
                .extensions()
                .get::<actuators_core::types::TenantAuthContext>()
                .map(|ctx| ctx.role.as_str());

            let role = match (auth, role) {
                (Some(TenantAuth(ctx)), _) => ctx.role.clone(),
                (_, Some(r)) => r.to_owned(),
                _ => {
                    tracing::warn!("require_role: no tenant auth context in request");
                    return Err(AppError::Unauthorized(
                        "authentication required".into(),
                    ));
                }
            };

            if allowed.contains(&role.as_str()) {
                tracing::debug!(role = %role, "role check passed");
                Ok(next.run(req).await)
            } else {
                tracing::warn!(
                    role = %role,
                    allowed = ?allowed,
                    "role check failed"
                );
                Err(AppError::Forbidden(format!(
                    "role '{role}' is not allowed for this resource"
                )))
            }
        })
    }
}
