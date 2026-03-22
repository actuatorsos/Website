use axum::{extract::Request, middleware::Next, response::Response};

use actuators_core::{error::AppError, types::TenantContext};

/// Services that are always available regardless of the tenant's plan.
const CORE_SERVICES: &[&str] = &["auth", "settings", "audit", "projects", "documents"];

/// Create a middleware that gates access to a specific service.
///
/// The returned function checks the current request's [`TenantContext`]
/// (inserted by the tenant-resolver middleware) and verifies that
/// `service_key` appears in the tenant's enabled services list.
///
/// Core services (auth, settings, audit, projects, documents) are always
/// allowed.
///
/// # Usage
///
/// ```ignore
/// use axum::{Router, middleware};
/// use actuators_middleware::service_gate::require_service;
///
/// let hr_routes = Router::new()
///     .route("/employees", get(list_employees))
///     .route_layer(middleware::from_fn(require_service("hr")));
/// ```
pub fn require_service(
    service_key: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>>
       + Clone
       + Send
       + 'static {
    move |req: Request, next: Next| {
        Box::pin(async move {
            // Core services always pass.
            if CORE_SERVICES.contains(&service_key) {
                return Ok(next.run(req).await);
            }

            let ctx = req
                .extensions()
                .get::<TenantContext>()
                .ok_or_else(|| {
                    tracing::warn!(
                        service_key,
                        "service gate: no TenantContext in request extensions"
                    );
                    AppError::Internal("tenant context not available".into())
                })?
                .clone();

            if ctx.services.iter().any(|s| s == service_key) {
                tracing::debug!(
                    service_key,
                    tenant_slug = %ctx.slug,
                    "service gate: access granted"
                );
                Ok(next.run(req).await)
            } else {
                tracing::warn!(
                    service_key,
                    tenant_slug = %ctx.slug,
                    "service gate: service not available on tenant's plan"
                );
                Err(AppError::Forbidden(
                    "Service not available on your plan".into(),
                ))
            }
        })
    }
}
