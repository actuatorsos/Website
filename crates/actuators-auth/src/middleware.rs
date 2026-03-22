use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;

use actuators_core::{
    config::AppConfig,
    error::AppError,
    types::{PlatformAuthContext, TenantAuthContext},
};
use actuators_db::{platform::PlatformDb, pool::TenantConnectionPool};

use crate::jwt;

// ── Shared application state ──────────────────────────────────────────

/// Application state shared across all request handlers.
///
/// This struct is typically wrapped in `Arc` and installed via
/// `axum::Router::with_state`.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub platform_db: Arc<PlatformDb>,
    pub tenant_pool: Arc<TenantConnectionPool>,
}

// ── Platform auth extractor ───────────────────────────────────────────

/// Axum extractor that authenticates a platform-level request.
///
/// Reads the `act_platform_token` cookie, verifies the JWT, and yields a
/// `PlatformAuthContext`.  Returns 401 if the cookie is missing or the
/// token is invalid.
pub struct PlatformAuth(pub PlatformAuthContext);

impl<S> FromRequestParts<S> for PlatformAuth
where
    S: Send + Sync + 'static,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract shared state.
        let app_state = Arc::<AppState>::from_ref(state);

        // Read cookies from the request.
        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar
            .get("act_platform_token")
            .map(|c| c.value().to_owned())
            .ok_or_else(|| AppError::Unauthorized("missing platform auth token".into()))?;

        let claims = jwt::verify_platform_token(&token, &app_state.config.jwt_secret)?;

        tracing::debug!(
            account_id = %claims.sub,
            email = %claims.email,
            "platform auth verified"
        );

        Ok(PlatformAuth(PlatformAuthContext {
            account_id: claims.sub,
            email: claims.email,
            full_name: claims.full_name,
            is_super_admin: claims.is_super_admin,
        }))
    }
}

// ── Tenant auth extractor ─────────────────────────────────────────────

/// Axum extractor that authenticates a tenant-scoped request.
///
/// Reads the `act_tenant_token` cookie, verifies the JWT, and yields a
/// `TenantAuthContext`.  Returns 401 if the cookie is missing or the
/// token is invalid.
pub struct TenantAuth(pub TenantAuthContext);

impl<S> FromRequestParts<S> for TenantAuth
where
    S: Send + Sync + 'static,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar
            .get("act_tenant_token")
            .map(|c| c.value().to_owned())
            .ok_or_else(|| AppError::Unauthorized("missing tenant auth token".into()))?;

        let claims = jwt::verify_tenant_token(&token, &app_state.config.jwt_secret)?;

        tracing::debug!(
            account_id = %claims.sub,
            tenant_slug = %claims.tenant_slug,
            role = %claims.role,
            "tenant auth verified"
        );

        Ok(TenantAuth(TenantAuthContext {
            account_id: claims.sub,
            platform_id: claims.platform_id,
            tenant_slug: claims.tenant_slug,
            tenant_db: claims.tenant_db,
            role: claims.role,
            services: claims.services,
        }))
    }
}

// ── Optional platform auth extractor ──────────────────────────────────

/// Like `PlatformAuth`, but succeeds with `None` when no token is present
/// instead of returning 401.  Still returns an error if a token *is*
/// present but invalid.
pub struct OptionalPlatformAuth(pub Option<PlatformAuthContext>);

impl<S> FromRequestParts<S> for OptionalPlatformAuth
where
    S: Send + Sync + 'static,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);

        let jar = CookieJar::from_headers(&parts.headers);

        let Some(cookie) = jar.get("act_platform_token") else {
            return Ok(OptionalPlatformAuth(None));
        };

        let token = cookie.value();

        match jwt::verify_platform_token(token, &app_state.config.jwt_secret) {
            Ok(claims) => {
                tracing::debug!(
                    account_id = %claims.sub,
                    "optional platform auth verified"
                );
                Ok(OptionalPlatformAuth(Some(PlatformAuthContext {
                    account_id: claims.sub,
                    email: claims.email,
                    full_name: claims.full_name,
                    is_super_admin: claims.is_super_admin,
                })))
            }
            Err(e) => {
                tracing::warn!("invalid platform token in optional auth: {e}");
                // If a token is present but invalid, we still fail.
                Err(e)
            }
        }
    }
}
