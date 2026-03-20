//! Machinery Domain Handlers
//!
//! إدارة مسارات الآلات، المشاريع، والإصلاحات في لوحة التحكم

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

/// Template for the machines management page
#[derive(Template)]
#[template(path = "admin/machines.html")]
pub struct MachinesTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the projects management page
#[derive(Template)]
#[template(path = "admin/projects.html")]
pub struct ProjectsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the repairs management page
#[derive(Template)]
#[template(path = "admin/repairs.html")]
pub struct RepairsTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

pub async fn machines_page(
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

    let template = MachinesTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "machines".to_string(),
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

pub async fn projects_page(
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

    let template = ProjectsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "projects".to_string(),
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

pub async fn repairs_page(
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

    let template = RepairsTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "repairs".to_string(),
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

/// Router for machinery UI pages
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/machines", get(machines_page))
        .route("/projects", get(projects_page))
        .route("/repairs", get(repairs_page))
}
