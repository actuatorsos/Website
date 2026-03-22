use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use actuators_auth::jwt;
use actuators_auth::middleware::AppState;
use actuators_core::{error::AppError, types::TenantContext};

/// Axum middleware that resolves the current tenant from the incoming request
/// and inserts a [`TenantContext`] into the request extensions.
///
/// Resolution strategies (tried in order according to config):
///
/// 1. **Subdomain** — extract slug from `Host` header
///    (e.g. `acme.app.example.com` -> `acme`).
/// 2. **Header** — read the `X-Tenant-Slug` header.
/// 3. **JWT** — if an `act_tenant_token` cookie is present, decode it and
///    read `tenant_slug` from the claims.
///
/// The configured `tenant_resolution_mode` controls which strategies are
/// attempted:
/// - `"subdomain"` — only strategy 1
/// - `"header"` — only strategy 2
/// - `"both"` (default) — all three strategies in order
pub async fn resolve_tenant(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let mode = state.config.tenant_resolution_mode.as_str();
    let slug = resolve_slug(&state, &req, mode);

    let slug = match slug {
        Some(s) => s,
        None => {
            // If this is an /app/* route a tenant is required.
            let path = req.uri().path();
            if path.starts_with("/app/") || path.starts_with("/app") {
                tracing::warn!(path, "tenant required but could not be resolved");
                return Err(AppError::NotFound("tenant not found".into()));
            }
            // For non-app routes (landing page, platform admin, etc.) proceed
            // without a tenant context.
            return Ok(next.run(req).await);
        }
    };

    tracing::debug!(slug = %slug, "resolved tenant slug");

    // Look up the tenant in the platform database.
    let tenant = state
        .platform_db
        .get_tenant_by_slug(&slug)
        .await?
        .ok_or_else(|| {
            tracing::warn!(slug = %slug, "no active tenant found for slug");
            AppError::NotFound(format!("tenant '{slug}' not found"))
        })?;

    // Verify the tenant is active.
    if tenant.status != "active" {
        tracing::warn!(slug = %slug, status = %tenant.status, "tenant is not active");
        return Err(AppError::Forbidden(format!(
            "tenant '{slug}' is not active"
        )));
    }

    // Fetch enabled services.
    let tenant_id_str = tenant
        .id
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_default();

    let services = state
        .platform_db
        .get_tenant_services(&tenant_id_str)
        .await?;

    let ctx = TenantContext {
        tenant_id: tenant_id_str,
        slug: tenant.slug,
        db_name: tenant.db_name,
        plan_name: tenant.plan_name,
        services,
        max_users: tenant.max_users,
        storage_limit_mb: tenant.storage_limit_mb,
    };

    tracing::debug!(
        tenant_id = %ctx.tenant_id,
        slug = %ctx.slug,
        db_name = %ctx.db_name,
        "tenant context attached to request"
    );

    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}

// ── Private helpers ───────────────────────────────────────────────────

/// Try each configured resolution strategy and return the first slug found.
fn resolve_slug(state: &AppState, req: &Request, mode: &str) -> Option<String> {
    match mode {
        "subdomain" => resolve_from_subdomain(req, &state.config.base_domain),
        "header" => resolve_from_header(req),
        // "both" or any other value: try all strategies in order.
        _ => resolve_from_subdomain(req, &state.config.base_domain)
            .or_else(|| resolve_from_header(req))
            .or_else(|| resolve_from_jwt(req, &state.config.jwt_secret)),
    }
}

/// Strategy 1: extract slug from the Host header's subdomain.
///
/// Given `base_domain = "app.example.com"` and Host `acme.app.example.com`,
/// extracts `"acme"`.
fn resolve_from_subdomain(req: &Request, base_domain: &str) -> Option<String> {
    let host = req
        .headers()
        .get(axum::http::header::HOST)?
        .to_str()
        .ok()?;

    // Strip optional port from base_domain for comparison.
    let base_no_port = base_domain.split(':').next().unwrap_or(base_domain);
    let host_no_port = host.split(':').next().unwrap_or(host);

    // The host must end with the base domain and have at least one extra label.
    if host_no_port == base_no_port {
        return None; // Exact match — no subdomain.
    }

    let suffix = format!(".{base_no_port}");
    if !host_no_port.ends_with(&suffix) {
        return None;
    }

    let slug = &host_no_port[..host_no_port.len() - suffix.len()];

    // A valid slug is a single DNS label (no dots).
    if slug.is_empty() || slug.contains('.') {
        return None;
    }

    tracing::debug!(slug, "tenant slug resolved from subdomain");
    Some(slug.to_owned())
}

/// Strategy 2: read slug from the `X-Tenant-Slug` header.
fn resolve_from_header(req: &Request) -> Option<String> {
    let value = req
        .headers()
        .get("X-Tenant-Slug")?
        .to_str()
        .ok()?
        .trim()
        .to_owned();

    if value.is_empty() {
        return None;
    }

    tracing::debug!(slug = %value, "tenant slug resolved from X-Tenant-Slug header");
    Some(value)
}

/// Strategy 3: decode the tenant JWT cookie and extract `tenant_slug`.
fn resolve_from_jwt(req: &Request, jwt_secret: &str) -> Option<String> {
    let jar = axum_extra::extract::CookieJar::from_headers(req.headers());

    let cookie = jar.get("act_tenant_token")?;
    let token = cookie.value();

    match jwt::verify_tenant_token(token, jwt_secret) {
        Ok(claims) => {
            tracing::debug!(
                slug = %claims.tenant_slug,
                "tenant slug resolved from JWT cookie"
            );
            Some(claims.tenant_slug)
        }
        Err(e) => {
            tracing::warn!("failed to verify tenant JWT for slug resolution: {e}");
            None
        }
    }
}
