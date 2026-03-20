//! Trainees API Endpoints
//!
//! نقاط نهاية API للمتدربين

use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Html,
    routing::{delete, get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::hr::models::{CreateTraineeRequest, Trainee};
use crate::domains::hr::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/trainee_row.html")]
pub struct TraineeRowTemplate {
    pub trainee: Trainee,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/trainee_list.html")]
pub struct TraineeListTemplate {
    pub trainees: Vec<Trainee>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateTraineeForm {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    #[validate(length(min = 5, max = 50))]
    pub phone: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 100))]
    pub institution: String,
    #[validate(length(min = 8, max = 50))]
    pub start_date: String,
    #[validate(length(min = 8, max = 50))]
    pub end_date: String,
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

// ============================================================================
// Handlers
// ============================================================================

async fn create_trainee(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CreateTraineeForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="5" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let request = CreateTraineeRequest {
        name: form.name,
        phone: form.phone,
        email: form.email,
        institution: form.institution,
        start_date: form.start_date,
        end_date: form.end_date,
    };

    match repository::create_trainee(&state, request).await {
        Ok(trainee) => {
            let template = TraineeRowTemplate { trainee, t };
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

async fn list_trainees(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let trainees: Vec<Trainee> = repository::get_all_trainees(&state)
        .await
        .unwrap_or_default();
    let template = TraineeListTemplate { trainees, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn delete_trainee(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    match repository::delete_trainee(&state, &id).await {
        Ok(_) => Html(String::new()),
        Err(e) => Html(format!("Error: {}", e)),
    }
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_trainee))
        .route("/", get(list_trainees))
        .route("/{id}", delete(delete_trainee))
}
