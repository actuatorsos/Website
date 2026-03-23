//! Projects API Endpoints
//!
//! نقاط نهاية API للمشاريع

use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    response::Html,
    routing::{get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::machinery::models::{CreateProjectRequest, Project, ProjectStatus};
use crate::domains::machinery::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/project_row.html")]
pub struct ProjectRowTemplate {
    pub project: Project,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/project_list.html")]
pub struct ProjectListTemplate {
    pub projects: Vec<Project>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateProjectForm {
    #[validate(length(min = 1, max = 100))]
    pub customer_id: String,
    #[validate(length(min = 2, max = 200))]
    pub title: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(length(min = 8, max = 50))]
    pub start_date: String,
    #[validate(length(max = 50))]
    pub end_date: Option<String>,
    #[validate(range(min = 0.0))]
    pub budget: Option<f64>,
}

#[derive(serde::Deserialize)]
pub struct UpdateProjectStatusForm {
    pub status: String,
}

// ============================================================================
// Helpers
// ============================================================================

fn resolve_language(cookies: &Cookies) -> Language {
    if let Some(cookie) = cookies.get("lang") {
        return Language::from_str(cookie.value());
    }
    Language::Arabic
}

fn parse_status(status: &str) -> ProjectStatus {
    match status {
        "new" => ProjectStatus::New,
        "in_progress" => ProjectStatus::InProgress,
        "on_hold" => ProjectStatus::OnHold,
        "completed" => ProjectStatus::Completed,
        "cancelled" => ProjectStatus::Cancelled,
        _ => ProjectStatus::New,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_project(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateProjectForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="5" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let request = CreateProjectRequest {
        customer_id: form.customer_id,
        title: form.title,
        description: form.description,
        start_date: form.start_date,
        end_date: form.end_date,
        budget: form.budget,
    };

    match repository::create_project(&state, request).await {
        Ok(project) => {
            let template = ProjectRowTemplate { project, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="5" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn list_projects(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let projects: Vec<Project> = repository::get_all_projects(&state)
        .await
        .unwrap_or_default();
    let template = ProjectListTemplate { projects, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UpdateProjectStatusForm>,
) -> Html<String> {
    let status = parse_status(&form.status);
    match repository::update_project_status(&state, &id, status).await {
        Ok(_) => Html(r#"<span class="text-green-600">✓</span>"#.to_string()),
        Err(e) => Html(format!("<span class='text-red-600'>Error: {}</span>", e)),
    }
}

async fn get_projects_json(State(state): State<AppState>) -> Json<Vec<Project>> {
    let projects: Vec<Project> = repository::get_all_projects(&state)
        .await
        .unwrap_or_default();
    Json(projects)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_project))
        .route("/", get(list_projects))
        .route("/json", get(get_projects_json))
        .route("/{id}/status", post(update_status))
}
