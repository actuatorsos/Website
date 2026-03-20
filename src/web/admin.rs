//! Admin Routes
//!
//! لوحة التحكم المحمية بـ JWT

use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use std::collections::HashMap;
use tower_cookies::{Cookie, Cookies};

use crate::db::AppState;
use crate::i18n::Language;
use crate::middleware::auth::decode_token;

// ============================================================================
// Stats struct for dashboard
// ============================================================================

/// Statistics displayed on the dashboard
#[derive(Default)]
pub struct DashboardStats {
    /// Total number of organizations (startups + volunteer teams)
    pub organizations: usize,
    /// Total number of events
    pub events: usize,
    /// Total number of employees
    pub employees: usize,
    /// Number of active projects
    pub projects: usize,
}

// ============================================================================
// Templates
// ============================================================================

/// Template for the admin dashboard
#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct DashboardTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub stats: DashboardStats,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the organizations management page
#[derive(Template)]
#[template(path = "admin/organizations.html")]
pub struct OrganizationsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the events management page
#[derive(Template)]
#[template(path = "admin/events.html")]
pub struct EventsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the assets management page
#[derive(Template)]
#[template(path = "admin/assets.html")]
pub struct AssetsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the user profile management page
#[derive(Template)]
#[template(path = "admin/profile.html")]
pub struct ProfileTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
    pub account: crate::models::Account,
}

/// Template for the accounts management page
#[derive(Template)]
#[template(path = "admin/accounts.html")]
pub struct AccountsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
    pub accounts: Vec<crate::models::Account>,
}

/// Template for the public verify page
#[derive(Template)]
#[template(path = "verify.html")]
pub struct VerifyTemplate {}

/// Template for verify result
#[derive(Template)]
#[template(path = "fragments/verify_result.html")]
pub struct VerifyResultTemplate {
    pub certificate: crate::domains::finance::models::Certificate,
}

/// Template for the organization registration request page
#[derive(Template)]
#[template(path = "admin/org_register.html")]
pub struct OrgRegisterTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
}

/// Template for the login page
#[derive(Template)]
#[template(path = "admin/login.html")]
pub struct LoginTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
}

/// Template for the register page
#[derive(Template)]
#[template(path = "admin/register.html")]
pub struct RegisterTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
}

/// Template for forgot password page
#[derive(Template)]
#[template(path = "admin/forgot_password.html")]
pub struct ForgotPasswordTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
}

/// Template for reset password page
#[derive(Template)]
#[template(path = "admin/reset_password.html")]
pub struct ResetPasswordTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub token: String,
}

/// Template for email verification page
#[derive(Template)]
#[template(path = "admin/verify_email.html")]
pub struct VerifyEmailTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub token: String,
}

// ============================================================================
// JWT Auth Check — redirects to login page if invalid
// ============================================================================

/// Authenticated user info extracted from JWT
pub struct AuthInfo {
    pub email: String,
    pub role: String,
    pub avatar: Option<String>,
}

/// Check JWT cookie and return user info, or redirect to login page
pub async fn check_jwt_auth(
    cookies: &Cookies,
    jwt_secret: &str,
    state: &AppState,
) -> Result<AuthInfo, Response> {
    let token = cookies.get("auth_token").map(|c| c.value().to_string());

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => return Err(Redirect::to("/admin/login").into_response()),
    };

    match decode_token(&token, jwt_secret) {
        Ok(claims) => {
            // Fetch user account to get avatar
            let avatar = match state.get_account_by_email(&claims.email).await {
                Ok(acc) => acc.avatar,
                Err(_) => None,
            };

            Ok(AuthInfo {
                email: claims.email,
                role: claims.role,
                avatar,
            })
        }
        Err(_) => {
            // Clear invalid cookie
            cookies.remove(Cookie::from("auth_token"));
            Err(Redirect::to("/admin/login").into_response())
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Query parameter for language selection
#[derive(serde::Deserialize)]
pub struct LangParam {
    pub lang: Option<String>,
}

pub fn resolve_language(cookies: &Cookies, query: Option<String>) -> Language {
    if let Some(l) = query {
        let lang = Language::from_str(&l);
        cookies.add(Cookie::new("lang", lang.as_str().to_string()));
        return lang;
    }
    if let Some(cookie) = cookies.get("lang") {
        return Language::from_str(cookie.value());
    }
    Language::Arabic
}

// ============================================================================
// Dashboard helpers
// ============================================================================

/// Get total count from a SurrealDB table (used for dashboard stats).
async fn get_table_count(state: &AppState, table: &str) -> usize {
    // Allowlist to prevent injection
    const ALLOWED: &[&str] = &["organization", "event", "employee", "project", "asset", "machine"];
    if !ALLOWED.contains(&table) {
        return 0;
    }

    #[derive(serde::Deserialize)]
    struct CountResult {
        count: i64,
    }

    let query = format!("SELECT count() as count FROM {}", table);
    let result: Vec<CountResult> = state
        .db
        .query(query)
        .await
        .ok()
        .and_then(|mut r| r.take(0).ok())
        .unwrap_or_default();

    result.first().map(|r| r.count as usize).unwrap_or(0)
}

// ============================================================================
// Handlers
// ============================================================================

/// Login page (public — no auth)
async fn login_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = LoginTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
    };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn register_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let template = RegisterTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
    };
    Html(template.render().unwrap_or_default()).into_response()
}

async fn forgot_password_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let template = ForgotPasswordTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
    };
    Html(template.render().unwrap_or_default()).into_response()
}

#[derive(serde::Deserialize)]
struct TokenParam {
    token: Option<String>,
    lang: Option<String>,
}

async fn reset_password_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<TokenParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let template = ResetPasswordTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        token: params.token.unwrap_or_default(),
    };
    Html(template.render().unwrap_or_default()).into_response()
}

async fn verify_email_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<TokenParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let template = VerifyEmailTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        token: params.token.unwrap_or_default(),
    };
    Html(template.render().unwrap_or_default()).into_response()
}

async fn dashboard(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    // Use cached counts with 60-second TTL for dashboard stats
    let stats = DashboardStats {
        organizations: state.get_cached_count("organization", 60).await as usize,
        events: state.get_cached_count("event", 60).await as usize,
        employees: state.get_cached_count("employee", 60).await as usize,
        projects: state.get_cached_count("project", 60).await as usize,
    };

    let template = DashboardTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "dashboard".to_string(),
        stats,
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn organizations_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = OrganizationsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "organizations".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn events_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = EventsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "events".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn assets_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = AssetsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "assets".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn accounts_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    // Only admin can view accounts
    if auth.role != "admin" {
        return Redirect::to("/admin").into_response();
    }

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let accounts = state.get_all_accounts().await.unwrap_or_default();

    let template = AccountsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "accounts".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
        accounts,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

async fn profile_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    // Fetch user account to pass into the template (for avatar/details)
    // AuthInfo only contains email/role. We need ID to get the account, but check_jwt_auth doesn't return ID.
    // Wait! check_jwt_auth only returns email and role!
    // We can fetch the account by email using the DB state.
    let account = state
        .get_account_by_email(&auth.email)
        .await
        .unwrap_or_else(|_| crate::models::Account {
            id: None,
            email: auth.email.clone(),
            password_hash: String::new(),
            role: crate::models::AccountRole::Applicant,
            is_active: false,
            email_verified: false,
            full_name: None,
            phone: None,
            user_type: "member".to_string(),
            organization: None,
            auth_methods: vec![],
            created_at: None,
            last_login: None,
            avatar: None,
        });

    let template = ProfileTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: String::new(), // Not active on sidebar
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
        account,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

/// Public verify page (no auth)
async fn verify_page() -> Response {
    let template = VerifyTemplate {};
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

/// Public verify check (HTMX)
#[derive(serde::Deserialize)]
struct VerifyParams {
    credential_id: Option<String>,
}

async fn verify_check(
    State(state): State<AppState>,
    Query(params): Query<VerifyParams>,
) -> Html<String> {
    let credential_id = params
        .credential_id
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    if credential_id.is_empty() {
        return Html(
            r#"<div class="text-center text-gray-500 py-4">Please enter a Credential ID</div>"#
                .to_string(),
        );
    }

    match crate::domains::finance::repository::get_certificate_by_credential_id(&state, &credential_id).await {
        Ok(certificate) => {
            if certificate.status == crate::domains::finance::models::CertificateStatus::Revoked {
                return Html(r#"<div class="bg-red-50 border border-red-200 rounded-2xl p-6 text-center"><div class="w-12 h-12 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-3"><svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg></div><h3 class="text-lg font-bold text-red-700">⚠ Certificate Revoked / شهادة ملغاة</h3></div>"#.to_string());
            }
            let tmpl = VerifyResultTemplate { certificate };
            Html(tmpl.render().unwrap_or_else(|e| format!("<div class='text-red-500'>{}</div>", e)))
        }
        Err(_) => {
            Html(r#"<div class="bg-yellow-50 border border-yellow-200 rounded-2xl p-6 text-center"><div class="w-12 h-12 bg-yellow-100 rounded-full flex items-center justify-center mx-auto mb-3"><svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"/></svg></div><h3 class="text-lg font-bold text-yellow-700">✗ Not Found / غير موجودة</h3><p class="text-yellow-600 text-sm mt-1">No certificate matches this Credential ID</p></div>"#.to_string())
        }
    }
}

// ============================================================================
// New ERP Module Templates
// ============================================================================

/// HR Organization page — Departments & Positions
#[derive(Template)]
#[template(path = "admin/hr_org.html")]
pub struct HrOrgTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Leave Management page
#[derive(Template)]
#[template(path = "admin/leave.html")]
pub struct LeaveTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Payroll page
#[derive(Template)]
#[template(path = "admin/payroll.html")]
pub struct PayrollTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// HR Compliance page — Warnings, EOS, Overtime
#[derive(Template)]
#[template(path = "admin/compliance.html")]
pub struct ComplianceTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Training page
#[derive(Template)]
#[template(path = "admin/training.html")]
pub struct TrainingTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// CRM page — Contacts, Opportunities, Quotations
#[derive(Template)]
#[template(path = "admin/crm.html")]
pub struct CrmTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Product & Service Catalog page
#[derive(Template)]
#[template(path = "admin/catalog.html")]
pub struct CatalogTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Inventory page
#[derive(Template)]
#[template(path = "admin/inventory.html")]
pub struct InventoryTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Manufacturing / BOM page
#[derive(Template)]
#[template(path = "admin/manufacturing.html")]
pub struct ManufacturingTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Clients management page
#[derive(Template)]
#[template(path = "admin/customers.html")]
pub struct ClientsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Projects Board (Trello) page
#[derive(Template)]
#[template(path = "admin/projects_board.html")]
pub struct ProjectsBoardTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Email management page
#[derive(Template)]
#[template(path = "admin/email.html")]
pub struct EmailPageTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// AI Agents management page
#[derive(Template)]
#[template(path = "admin/agents.html")]
pub struct AgentsPageTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// E-Commerce Store management page
#[derive(Template)]
#[template(path = "admin/store.html")]
pub struct StorePageTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Educational Videos page
#[derive(Template)]
#[template(path = "admin/videos.html")]
pub struct VideosTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
    pub videos: Vec<crate::domains::videos::models::VideoView>,
}

/// Notifications center page
#[derive(Template)]
#[template(path = "admin/notifications.html")]
pub struct NotificationsPageTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Document Viewer page
#[derive(Template)]
#[template(path = "admin/document_view.html")]
pub struct DocumentViewTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
    pub entity_type: String,
    pub entity_id: String,
}

pub async fn document_view_page(
    State(state): State<AppState>,
    cookies: Cookies,
    axum::extract::Path((entity_type, entity_id)): axum::extract::Path<(String, String)>,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let template = DocumentViewTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: entity_type.clone(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
        entity_type,
        entity_id,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

// ============================================================================
// New Module Handlers — macro to reduce boilerplate
// ============================================================================

macro_rules! simple_admin_page {
    ($fn_name:ident, $tmpl:ident, $active:literal) => {
        async fn $fn_name(
            State(state): State<AppState>,
            cookies: Cookies,
            Query(params): Query<LangParam>,
        ) -> Response {
            let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
                Ok(info) => info,
                Err(redirect) => return redirect,
            };
            let lang = resolve_language(&cookies, params.lang);
            let t = state.i18n.get_dictionary(lang.as_str());
            let template = $tmpl {
                lang: lang.as_str().to_string(),
                dir: lang.dir().to_string(),
                t,
                active_page: $active.to_string(),
                user_email: auth.email,
                user_role: auth.role,
                user_avatar: auth.avatar,
            };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
            .into_response()
        }
    };
}

simple_admin_page!(hr_org_page, HrOrgTemplate, "hr_org");
simple_admin_page!(leave_page, LeaveTemplate, "leave");
simple_admin_page!(payroll_page, PayrollTemplate, "payroll");
simple_admin_page!(compliance_page, ComplianceTemplate, "compliance");
simple_admin_page!(training_page, TrainingTemplate, "training");
simple_admin_page!(clients_page, ClientsTemplate, "clients");
simple_admin_page!(crm_page, CrmTemplate, "crm");
simple_admin_page!(catalog_page, CatalogTemplate, "catalog");
simple_admin_page!(inventory_page, InventoryTemplate, "inventory");
simple_admin_page!(manufacturing_page, ManufacturingTemplate, "manufacturing");
simple_admin_page!(projects_board_page, ProjectsBoardTemplate, "projects_board");
simple_admin_page!(email_mgmt_page, EmailPageTemplate, "email");
simple_admin_page!(agents_mgmt_page, AgentsPageTemplate, "agents");
simple_admin_page!(store_mgmt_page, StorePageTemplate, "store");
simple_admin_page!(
    notifications_page,
    NotificationsPageTemplate,
    "notifications"
);

/// صفحة الفيديوهات التعليمية — تجلب قائمة الفيديوهات من DB
pub async fn videos_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    let videos: Vec<crate::domains::videos::models::VideoView> =
        crate::domains::videos::repository::get_all_videos(&state)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(crate::domains::videos::models::VideoView::from_video)
            .collect();

    let template = VideosTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "videos".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
        videos,
    };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

// ============================================================================
// Audit Log
// ============================================================================

/// A single audit log entry for display
#[derive(serde::Deserialize, Clone)]
pub struct AuditLogEntry {
    pub timestamp: String,
    pub actor: Option<String>,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<String>,
}

/// Template for the audit log page
#[derive(Template)]
#[template(path = "admin/audit_log.html")]
pub struct AuditLogTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
    pub entries: Vec<AuditLogEntry>,
}

/// Audit log viewer page — fetches recent audit log entries from DB
async fn audit_log_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let auth = match check_jwt_auth(&cookies, &state.jwt_secret, &state).await {
        Ok(info) => info,
        Err(redirect) => return redirect,
    };

    // Only admin can view audit log
    if auth.role != "admin" {
        return Redirect::to("/admin").into_response();
    }

    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());

    // Fetch recent audit log entries
    #[derive(serde::Deserialize)]
    struct RawAuditEntry {
        timestamp: Option<String>,
        actor: Option<String>,
        action: Option<String>,
        table_name: Option<String>,
        record_id: Option<String>,
    }

    let raw_entries: Vec<RawAuditEntry> = state
        .db
        .query("SELECT timestamp, actor, action, table_name, record_id FROM audit_log ORDER BY timestamp DESC LIMIT 200")
        .await
        .ok()
        .and_then(|mut r| r.take::<Vec<RawAuditEntry>>(0).ok())
        .unwrap_or_default();

    let entries: Vec<AuditLogEntry> = raw_entries
        .into_iter()
        .map(|e| AuditLogEntry {
            timestamp: e.timestamp.unwrap_or_default(),
            actor: e.actor,
            action: e.action.unwrap_or_default(),
            table_name: e.table_name.unwrap_or_default(),
            record_id: e.record_id,
        })
        .collect();

    let template = AuditLogTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "audit_log".to_string(),
        user_email: auth.email,
        user_role: auth.role,
        user_avatar: auth.avatar,
        entries,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
    .into_response()
}

// ============================================================================
// Routes
// ============================================================================

/// Configures routes for the admin panel (JWT protected)
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        .route("/", get(dashboard))
        .route("/organizations", get(organizations_page))
        .route("/events", get(events_page))
        .route("/assets", get(assets_page))
        .route("/accounts", get(accounts_page))
        .route("/profile", get(profile_page))
        .route("/documents/{type}/{id}", get(document_view_page))
        // ── New ERP Modules ──────────────────────────────────
        .route("/hr", get(hr_org_page))
        .route("/leave", get(leave_page))
        .route("/payroll", get(payroll_page))
        .route("/compliance", get(compliance_page))
        .route("/training", get(training_page))
        .route("/clients", get(clients_page))
        .route("/crm", get(crm_page))
        .route("/catalog", get(catalog_page))
        .route("/inventory", get(inventory_page))
        .route("/manufacturing", get(manufacturing_page))
        .route("/projects-board", get(projects_board_page))
        .route("/email", get(email_mgmt_page))
        .route("/agents", get(agents_mgmt_page))
        .route("/store", get(store_mgmt_page))
        .route("/notifications", get(notifications_page))
        .route("/videos", get(videos_page))
        .route("/audit-log", get(audit_log_page))
}

/// Organization registration request page (public — no auth)
async fn org_register_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<LangParam>,
) -> Response {
    let lang = resolve_language(&cookies, params.lang);
    let t = state.i18n.get_dictionary(lang.as_str());
    let template = OrgRegisterTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
    };
    Html(template.render().unwrap_or_default()).into_response()
}

/// Public routes (no auth required)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/verify", get(verify_page))
        .route("/verify/check", get(verify_check))
        .route("/forgot-password", get(forgot_password_page))
        .route("/reset-password", get(reset_password_page))
        .route("/verify-email", get(verify_email_page))
        .route("/org-register", get(org_register_page))
}
