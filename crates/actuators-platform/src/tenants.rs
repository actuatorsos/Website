use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use actuators_auth::jwt::{self, TenantClaims};
use actuators_auth::middleware::{AppState, PlatformAuth};
use actuators_core::error::AppError;
use actuators_db::provision;
use actuators_middleware::csrf::generate_csrf_token;

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "platform/create_tenant.html")]
pub struct CreateTenantTemplate {
    pub lang: String,
    pub error: Option<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "platform/select_plan.html")]
pub struct SelectPlanTemplate {
    pub lang: String,
    pub plans: Vec<Value>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "platform/invitation.html")]
pub struct InvitationTemplate {
    pub lang: String,
    pub invitation: Value,
    pub error: Option<String>,
    pub csrf_token: String,
}

// ── Form types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTenantForm {
    pub name: String,
    pub slug: String,
    pub org_type: String,
    pub plan_name: String,
    #[serde(default)]
    pub _csrf_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SlugCheck {
    pub slug: String,
}

// ── Handlers ─────────────────────────────────────────────────────────

/// Render the "create new tenant" form.
pub async fn create_page(
    PlatformAuth(_auth): PlatformAuth,
) -> Result<Response, AppError> {
    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let template = CreateTenantTemplate {
        lang: "en".to_owned(),
        error: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Handle the "create tenant" form submission.
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
    jar: CookieJar,
    Form(form): Form<CreateTenantForm>,
) -> Result<Response, AppError> {
    let name = form.name.trim().to_owned();
    let slug = form.slug.trim().to_lowercase();
    let org_type = form.org_type.trim().to_owned();
    let plan_name = form.plan_name.trim().to_owned();

    // ── Validate ───────────────────────────────────────────────────
    if name.is_empty() || slug.is_empty() {
        return render_create_error("Name and slug are required.", &jar);
    }

    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') || slug.starts_with('-') {
        return render_create_error(
            "Slug may only contain lowercase letters, digits, and hyphens.",
            &jar,
        );
    }

    // Check that the slug is not already taken.
    let existing = state.platform_db.get_tenant_by_slug(&slug).await?;
    if existing.is_some() {
        return render_create_error("This slug is already taken.", &jar);
    }

    // Look up the plan.
    let plan = state
        .platform_db
        .get_plan_by_name(&plan_name)
        .await?
        .ok_or_else(|| AppError::BadRequest(format!("unknown plan: {plan_name}")))?;

    let plan_id = plan["id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("plan missing id".into()))?;

    // ── Create tenant record ───────────────────────────────────────
    let tenant = state
        .platform_db
        .create_tenant(&name, &slug, &org_type, &auth.account_id, plan_id)
        .await?;

    let tenant_id = tenant["id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("tenant missing id".into()))?;

    let db_name = format!("tenant_{slug}");

    info!(slug = %slug, tenant_id = %tenant_id, "tenant record created");

    // ── Provision tenant database ──────────────────────────────────
    provision::provision_tenant(&state.platform_db, &state.tenant_pool, &slug).await?;

    // ── Create founder account in tenant DB ────────────────────────
    provision::create_founder_account(
        &state.tenant_pool,
        &db_name,
        &auth.account_id,
        &auth.email,
        &auth.full_name,
    )
    .await?;

    // ── Create membership ──────────────────────────────────────────
    state
        .platform_db
        .create_membership(&auth.account_id, tenant_id, "admin")
        .await?;

    info!(slug = %slug, "tenant fully provisioned");

    Ok(Redirect::to(&format!("/tenants/{slug}/enter")).into_response())
}

/// Render the plan selection page.
pub async fn select_plan_page(
    State(state): State<Arc<AppState>>,
    PlatformAuth(_auth): PlatformAuth,
) -> Result<Response, AppError> {
    let plans = state.platform_db.list_plans().await?;

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let template = SelectPlanTemplate {
        lang: "en".to_owned(),
        plans,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// HTMX endpoint that checks whether a slug is available.
pub async fn check_slug(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Form(form): Form<SlugCheck>,
) -> Result<Response, AppError> {
    let slug = form.slug.trim().to_lowercase();

    if slug.is_empty() {
        return Ok(Html("").into_response());
    }

    let existing = state.platform_db.get_tenant_by_slug(&slug).await?;

    let is_htmx = headers
        .get("HX-Request")
        .is_some();

    let fragment = if existing.is_some() {
        r#"<span class="text-red-600">This slug is already taken.</span>"#
    } else {
        r#"<span class="text-green-600">Available!</span>"#
    };

    if is_htmx {
        Ok(Html(fragment).into_response())
    } else {
        Ok(axum::Json(serde_json::json!({
            "available": existing.is_none(),
            "slug": slug,
        }))
        .into_response())
    }
}

/// Issue a tenant JWT for the authenticated user and redirect them into
/// the tenant application.
pub async fn enter_tenant(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
    jar: CookieJar,
    Path(slug): Path<String>,
) -> Result<Response, AppError> {
    // ── Look up the tenant ─────────────────────────────────────────
    let tenant = state
        .platform_db
        .get_tenant_by_slug(&slug)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("tenant not found: {slug}")))?;

    let tenant_id = tenant
        .id
        .as_ref()
        .map(|t| t.to_string())
        .ok_or_else(|| AppError::Internal("tenant missing id".into()))?;

    // ── Verify membership ──────────────────────────────────────────
    let membership = state
        .platform_db
        .get_membership(&auth.account_id, &tenant_id)
        .await?
        .ok_or_else(|| {
            AppError::Forbidden("you are not a member of this organization".into())
        })?;

    let role = membership["role"]
        .as_str()
        .unwrap_or("employee")
        .to_owned();

    // ── Fetch enabled services ─────────────────────────────────────
    let services = state
        .platform_db
        .get_tenant_services(&tenant_id)
        .await?;

    // ── Issue tenant JWT ───────────────────────────────────────────
    let now = Utc::now().timestamp() as usize;
    let claims = TenantClaims {
        sub: auth.account_id.clone(),
        platform_id: auth.account_id,
        tenant_slug: slug.clone(),
        tenant_db: tenant.db_name.clone(),
        role,
        services,
        iat: now,
        exp: now + state.config.jwt_access_lifetime_secs as usize,
    };

    let tenant_token = jwt::create_tenant_token(&claims, &state.config.jwt_secret)?;

    let cookie = Cookie::build(("act_tenant_token", tenant_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(
            state.config.jwt_access_lifetime_secs as i64,
        ))
        .build();

    let jar = jar.add(cookie);

    info!(slug = %slug, "entering tenant");

    Ok((jar, Redirect::to("/app")).into_response())
}

/// Render the invitation details page.
pub async fn invitation_page(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    let invitation = state
        .platform_db
        .get_invitation_by_token(&token)
        .await?
        .ok_or_else(|| AppError::NotFound("invitation not found or expired".into()))?;

    let csrf_token = generate_csrf_token();

    let cookie = Cookie::build(("act_csrf", csrf_token.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let template = InvitationTemplate {
        lang: "en".to_owned(),
        invitation,
        error: None,
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((CookieJar::new().add(cookie), Html(html)).into_response())
}

/// Accept an invitation, create the tenant membership, and redirect to
/// the tenant.
pub async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    PlatformAuth(auth): PlatformAuth,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    // ── Look up the invitation ─────────────────────────────────────
    let invitation = state
        .platform_db
        .get_invitation_by_token(&token)
        .await?
        .ok_or_else(|| AppError::NotFound("invitation not found or expired".into()))?;

    let tenant_id = invitation["tenant_id"]
        .as_str()
        .ok_or_else(|| AppError::Internal("invitation missing tenant_id".into()))?;

    let role = invitation["role"]
        .as_str()
        .unwrap_or("employee");

    let invitation_email = invitation["email"]
        .as_str()
        .unwrap_or("");

    // Verify the invitation is for this user.
    if !invitation_email.is_empty() && invitation_email != auth.email {
        return Err(AppError::Forbidden(
            "this invitation was sent to a different email address".into(),
        ));
    }

    // ── Accept the invitation ──────────────────────────────────────
    state.platform_db.accept_invitation(&token).await?;

    // ── Create membership ──────────────────────────────────────────
    state
        .platform_db
        .create_membership(&auth.account_id, tenant_id, role)
        .await?;

    // ── Create tenant account if needed ────────────────────────────
    let mut resp = state
        .platform_db
        .client()
        .query("SELECT * FROM type::thing('tenant', $id) LIMIT 1;")
        .bind(("id", tenant_id.to_owned()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let tenant_record: Option<Value> = resp
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(tr) = tenant_record {
        let db_name = tr["db_name"].as_str().unwrap_or("");
        let slug = tr["slug"].as_str().unwrap_or("");

        if !db_name.is_empty() {
            let _ = provision::create_founder_account(
                &state.tenant_pool,
                db_name,
                &auth.account_id,
                &auth.email,
                &auth.full_name,
            )
            .await;
        }

        if !slug.is_empty() {
            info!(
                slug = %slug,
                account_id = %auth.account_id,
                "invitation accepted"
            );
            return Ok(Redirect::to(&format!("/tenants/{slug}/enter")).into_response());
        }
    }

    // Fallback: redirect to dashboard.
    Ok(Redirect::to("/dashboard").into_response())
}

// ── Private helpers ──────────────────────────────────────────────────

/// Re-render the create-tenant form with an error message.
fn render_create_error(message: &str, jar: &CookieJar) -> Result<Response, AppError> {
    let csrf_token = jar
        .get("act_csrf")
        .map(|c| c.value().to_owned())
        .unwrap_or_else(generate_csrf_token);

    let template = CreateTenantTemplate {
        lang: "en".to_owned(),
        error: Some(message.to_owned()),
        csrf_token,
    };

    let html = template.render().map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Html(html).into_response())
}
