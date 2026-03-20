use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::{Html, Json},
    routing::{delete, get, post, put},
};

async fn list_departments(State(s): State<AppState>) -> Result<Json<Vec<Department>>, DbError> {
    Ok(Json(repo::get_all_departments(&s).await?))
}
async fn create_department(
    State(s): State<AppState>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<Json<Department>, DbError> {
    Ok(Json(repo::create_department(&s, req).await?))
}
async fn get_department(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Department>, DbError> {
    Ok(Json(repo::get_department(&s, &id).await?))
}
async fn update_department(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>, DbError> {
    Ok(Json(repo::update_department(&s, &id, req).await?))
}
async fn delete_department(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_department(&s, &id).await?;
    Ok(Json(()))
}
async fn get_dept_employees(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, DbError> {
    Ok(Json(repo::get_department_employees(&s, &id).await?))
}
/// Returns department options as HTML for select dropdowns
async fn department_options(State(s): State<AppState>) -> Html<String> {
    let departments: Vec<Department> = repo::get_all_departments(&s)
        .await
        .unwrap_or_default();
    let options: String = departments
        .iter()
        .map(|d| {
            let id_str = d.id.as_ref().map(|t| t.id.to_string()).unwrap_or_default();
            format!(r#"<option value="{}">{}</option>"#, id_str, d.name)
        })
        .collect::<Vec<_>>()
        .join("\n");
    Html(options)
}

async fn list_positions(State(s): State<AppState>) -> Result<Json<Vec<Position>>, DbError> {
    Ok(Json(repo::get_all_positions(&s).await?))
}
async fn create_position(
    State(s): State<AppState>,
    Json(req): Json<CreatePositionRequest>,
) -> Result<Json<Position>, DbError> {
    Ok(Json(repo::create_position(&s, req).await?))
}
async fn get_position_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Position>, DbError> {
    Ok(Json(repo::get_position(&s, &id).await?))
}
async fn update_position(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePositionRequest>,
) -> Result<Json<Position>, DbError> {
    Ok(Json(repo::update_position(&s, &id, req).await?))
}
async fn delete_position(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_position(&s, &id).await?;
    Ok(Json(()))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/departments",
            get(list_departments).post(create_department),
        )
        .route("/departments/options", get(department_options))
        .route(
            "/departments/{id}",
            get(get_department)
                .put(update_department)
                .delete(delete_department),
        )
        .route("/departments/{id}/employees", get(get_dept_employees))
        .route("/positions", get(list_positions).post(create_position))
        .route(
            "/positions/{id}",
            get(get_position_handler)
                .put(update_position)
                .delete(delete_position),
        )
}
