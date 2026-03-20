//! Attendance API Endpoints
//!
//! نقاط نهاية API للدوام

use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    response::Html,
    routing::{delete, get, post},
};
use std::collections::HashMap;
use tower_cookies::Cookies;
use validator::Validate;

use crate::db::AppState;
use crate::domains::hr::models::{Attendance, CheckInRequest, Employee, PersonType, Trainee};
use crate::domains::hr::repository;
use crate::i18n::Language;

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "fragments/attendance_row.html")]
pub struct AttendanceRowTemplate {
    pub attendance: Attendance,
    pub t: HashMap<String, String>,
}

#[derive(Template)]
#[template(path = "fragments/attendance_list.html")]
pub struct AttendanceListTemplate {
    pub records: Vec<Attendance>,
    pub t: HashMap<String, String>,
}

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CheckInForm {
    #[validate(length(min = 1, max = 100))]
    pub person_id: String,
    #[validate(length(min = 1, max = 50))]
    pub person_type: String,
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct DateQuery {
    pub date: Option<String>,
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

fn parse_person_type(pt: &str) -> PersonType {
    match pt {
        "trainee" => PersonType::Trainee,
        _ => PersonType::Employee,
    }
}

// ============================================================================
// Handlers
// ============================================================================

async fn check_in(
    State(state): State<AppState>,
    cookies: Cookies,
    Form(form): Form<CheckInForm>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    if let Err(e) = form.validate() {
        return Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Validation Error: {}</td></tr>"#,
            e
        ));
    }

    let person_type = parse_person_type(&form.person_type);

    // Lookup person name
    let person_name = repository::get_person_name(&state, &form.person_id, &person_type).await;

    let request = CheckInRequest {
        person_id: form.person_id,
        person_type,
        person_name: Some(person_name),
        notes: form.notes,
    };

    match repository::check_in(&state, request).await {
        Ok(attendance) => {
            let template = AttendanceRowTemplate { attendance, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn check_out(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<String>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    match repository::check_out(&state, &id).await {
        Ok(attendance) => {
            let template = AttendanceRowTemplate { attendance, t };
            Html(
                template
                    .render()
                    .unwrap_or_else(|e| format!("Error: {}", e)),
            )
        }
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

async fn list_attendance(
    State(state): State<AppState>,
    cookies: Cookies,
    Query(query): Query<DateQuery>,
) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let records = match query.date {
        Some(ref date) if !date.is_empty() => repository::get_attendance_by_date(&state, date)
            .await
            .unwrap_or_else(|_| Vec::<Attendance>::new()),
        _ => repository::get_today_attendance(&state)
            .await
            .unwrap_or_else(|_| Vec::<Attendance>::new()),
    };

    let template = AttendanceListTemplate { records, t };
    Html(
        template
            .render()
            .unwrap_or_else(|e| format!("Error: {}", e)),
    )
}

async fn delete_attendance(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    match repository::delete_attendance(&state, &id).await {
        Ok(_) => Html(String::new()),
        Err(e) => Html(format!(
            r#"<tr class="bg-red-100"><td colspan="7" class="p-4 text-red-600">Error: {}</td></tr>"#,
            e
        )),
    }
}

/// Returns employee and trainee options as HTML for select dropdown
async fn people_options(State(state): State<AppState>, cookies: Cookies) -> Html<String> {
    let lang = resolve_language(&cookies);
    let t = state.i18n.get_dictionary(lang.as_str());

    let employees: Vec<Employee> = repository::get_all_employees(&state)
        .await
        .unwrap_or_default();
    let trainees: Vec<Trainee> = repository::get_all_trainees(&state)
        .await
        .unwrap_or_default();

    let emp_label = t
        .get("type_employee")
        .cloned()
        .unwrap_or_else(|| "موظفين".to_string());
    let trainee_label = t
        .get("type_trainee")
        .cloned()
        .unwrap_or_else(|| "متدربين".to_string());

    let select_label = t
        .get("form_select_person")
        .cloned()
        .unwrap_or_else(|| "اختر الشخص".to_string());

    let mut html = format!(r#"<option value="">-- {} --</option>"#, select_label);

    html.push_str(&format!(r#"<optgroup label="{}">"#, emp_label));
    for emp in &employees {
        let eid = emp
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();
        html.push_str(&format!(
            r#"<option value="{}" data-type="employee">{}</option>"#,
            eid, emp.name
        ));
    }
    html.push_str("</optgroup>");

    html.push_str(&format!(r#"<optgroup label="{}">"#, trainee_label));
    for trainee in &trainees {
        let tid = trainee
            .id
            .as_ref()
            .map(|thing| thing.id.to_string())
            .unwrap_or_default();
        html.push_str(&format!(
            r#"<option value="{}" data-type="trainee">{}</option>"#,
            tid, trainee.name
        ));
    }
    html.push_str("</optgroup>");

    Html(html)
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/check-in", post(check_in))
        .route("/check-out/{id}", post(check_out))
        .route("/", get(list_attendance))
        .route("/{id}", delete(delete_attendance))
        .route("/people-options", get(people_options))
}
