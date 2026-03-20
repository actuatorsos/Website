//! HR Domain Handlers
//!
//! إدارة مسارات الموارد البشرية (موظفين، متدربين، حضور) في لوحة التحكم

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

/// Template for the employees management page
#[derive(Template)]
#[template(path = "admin/employees.html")]
pub struct EmployeesTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the trainees management page
#[derive(Template)]
#[template(path = "admin/trainees.html")]
pub struct TraineesTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

/// Template for the attendance management page
#[derive(Template)]
#[template(path = "admin/attendance.html")]
pub struct AttendanceTemplate {
    pub lang: String,
    pub dir: String,
    pub t: HashMap<String, String>,
    pub active_page: String,
    pub user_email: String,
    pub user_role: String,
    pub user_avatar: Option<String>,
}

pub async fn employees_page(
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

    let template = EmployeesTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "employees".to_string(),
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

pub async fn trainees_page(
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

    let template = TraineesTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "trainees".to_string(),
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

pub async fn attendance_page(
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

    let template = AttendanceTemplate {
        lang: lang.as_str().to_string(),
        dir: lang.dir().to_string(),
        t,
        active_page: "attendance".to_string(),
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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/employees", get(employees_page))
        .route("/trainees", get(trainees_page))
        .route("/attendance", get(attendance_page))
}
