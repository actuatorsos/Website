use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{get, post, put},
};

async fn list_programs(State(s): State<AppState>) -> Result<Json<Vec<TrainingProgram>>, DbError> {
    Ok(Json(repo::get_all_training_programs(&s).await?))
}
async fn create_program(
    State(s): State<AppState>,
    Json(req): Json<CreateTrainingProgramRequest>,
) -> Result<Json<TrainingProgram>, DbError> {
    Ok(Json(repo::create_training_program(&s, req).await?))
}
async fn enroll(
    State(s): State<AppState>,
    Json(req): Json<EnrollRequest>,
) -> Result<Json<TrainingEnrollment>, DbError> {
    Ok(Json(repo::enroll_employee(&s, req).await?))
}
async fn complete_enrollment(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CompleteEnrollmentRequest>,
) -> Result<Json<TrainingEnrollment>, DbError> {
    Ok(Json(repo::complete_enrollment(&s, &id, req).await?))
}
async fn list_all_enrollments(
    State(s): State<AppState>,
) -> Result<Json<Vec<TrainingEnrollment>>, DbError> {
    Ok(Json(repo::get_all_enrollments(&s).await?))
}
async fn employee_enrollments(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<TrainingEnrollment>>, DbError> {
    Ok(Json(repo::get_employee_enrollments(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/programs",
            get(list_programs).post(create_program),
        )
        .route("/enroll", post(enroll))
        .route("/enrollments", get(list_all_enrollments))
        .route(
            "/enrollments/{id}/complete",
            put(complete_enrollment),
        )
        .route(
            "/enrollments/{employee_id}",
            get(employee_enrollments),
        )
}
