//! Customer Domain Handlers
//!
//! إدارة مسارات العملاء في لوحة التحكم

use askama::Template;
use axum::{
    Router,
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::collections::HashMap;
use tower_cookies::Cookies;

use crate::db::AppState;
use crate::web::admin::{LangParam, check_jwt_auth, resolve_language};

/// Template for the customers management page
#[derive(Template)]
#[template(path = "admin/customers.html")]
pub struct CustomersTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Handler for the customers management page
pub async fn customers_page(
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

    let template = CustomersTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "customers".to_string(),
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

/// Router for customer UI pages
pub fn routes() -> Router<AppState> {
    Router::new().route("/customers", get(customers_page))
}
