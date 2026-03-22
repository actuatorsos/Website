//! Course Handlers — معالجات الكورسات والمسارات التعليمية

use axum::{
    Router,
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
};
use std::path::PathBuf;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use super::{models::*, repository as repo};

// ============================================================================
// Allowed file types for course attachments
// ============================================================================

const ALLOWED_ATTACHMENT_MIMES: &[&str] = &[
    "application/pdf",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/zip",
    "image/png",
    "image/jpeg",
];

const ALLOWED_ATTACHMENT_EXTS: &[&str] = &["pdf", "docx", "xlsx", "zip", "png", "jpg", "jpeg"];

/// Maximum attachment file size: 50MB
const MAX_ATTACHMENT_SIZE: i64 = 50 * 1024 * 1024;

/// Upload directory for course attachments
const COURSE_UPLOAD_DIR: &str = "static/uploads/courses";

// ============================================================================
// Admin routes (behind require_manager middleware)
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_courses).post(create_course))
        .route("/{id}", delete(delete_course))
        .route("/{id}/publish", put(toggle_publish))
        .route("/{id}/lessons", get(list_lessons).post(create_lesson))
        .route("/{id}/lessons/{lid}", delete(delete_lesson))
        .route("/{id}/lessons/{lid}/attachments", post(upload_attachment))
        .route("/{id}/enrollments", get(list_enrollments))
}

/// Public routes (no auth required for listing)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(public_list))
        .route("/{id}", get(public_detail))
}

/// Enrollment routes (requires auth but not manager)
pub fn enrollment_routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/enroll", post(enroll))
        .route("/my", get(my_enrollments))
}

// ============================================================================
// Handler implementations
// ============================================================================

/// List all courses for the current organization (admin view)
async fn list_courses(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<Course>>, StatusCode> {
    repo::get_all_courses(&s, user.organization_id.as_deref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Create a new course
async fn create_course(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateCourseRequest>,
) -> Result<Json<Course>, StatusCode> {
    // Extract the account ID portion (after 'account:')
    let uid = user.id.strip_prefix("account:").unwrap_or(&user.id);
    repo::create_course(&s, req, user.organization_id.as_deref(), uid)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Soft-delete a course
async fn delete_course(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    repo::delete_course(&s, &id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Toggle course publish status
async fn toggle_publish(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PublishRequest>,
) -> Result<StatusCode, StatusCode> {
    repo::toggle_publish(&s, &id, req.is_published)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// List lessons for a course
async fn list_lessons(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<CourseLesson>>, StatusCode> {
    repo::get_lessons(&s, &id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Create a lesson for a course
async fn create_lesson(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(mut req): Json<CreateLessonRequest>,
) -> Result<Json<CourseLesson>, StatusCode> {
    req.course_id = id.clone();
    let lesson = repo::create_lesson(&s, req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update total_lessons count on the course
    let _ = repo::update_course_lesson_count(&s, &id).await;

    Ok(Json(lesson))
}

/// Soft-delete a lesson
async fn delete_lesson(
    State(s): State<AppState>,
    Path((course_id, lid)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    repo::delete_lesson(&s, &lid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update total_lessons count on the course
    let _ = repo::update_course_lesson_count(&s, &course_id).await;

    Ok(StatusCode::OK)
}

/// Upload an attachment to a lesson
async fn upload_attachment(
    State(state): State<AppState>,
    Extension(_user): Extension<CurrentUser>,
    Path((course_id, lesson_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir = format!("{}/{}", COURSE_UPLOAD_DIR, course_id);
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        tracing::error!("Failed to create upload dir: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot create upload directory").into_response();
    }

    let mut file_name = String::new();
    let mut original_name = String::new();
    let mut mime_type = String::new();
    let mut file_size: i64 = 0;
    let mut saved_path: Option<String> = None;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "file" {
            if let Some(fname) = field.file_name() {
                original_name = fname.to_string();
            }
            if let Some(ct) = field.content_type() {
                mime_type = ct.to_string();
            }

            // Validate MIME type
            if !ALLOWED_ATTACHMENT_MIMES.contains(&mime_type.as_str()) {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Invalid file type '{}'. Allowed: PDF, DOCX, XLSX, ZIP, PNG, JPG", mime_type),
                ).into_response();
            }

            // Validate extension
            let ext = PathBuf::from(&original_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin")
                .to_lowercase();

            if !ALLOWED_ATTACHMENT_EXTS.contains(&ext.as_str()) {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Invalid file extension '.{}'", ext),
                ).into_response();
            }

            let unique_name = format!("{}.{}", Uuid::new_v4(), ext);
            let rel_path = format!("{}/{}", upload_dir, unique_name);
            let abs_path = PathBuf::from(&rel_path);
            file_name = unique_name;

            let mut file = match tokio::fs::File::create(&abs_path).await {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to create file: {e}");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create file").into_response();
                }
            };

            use tokio::io::AsyncWriteExt;
            while let Ok(Some(chunk)) = field.chunk().await {
                file_size += chunk.len() as i64;
                if file_size > MAX_ATTACHMENT_SIZE {
                    let _ = tokio::fs::remove_file(&abs_path).await;
                    return (StatusCode::PAYLOAD_TOO_LARGE, "File exceeds 50MB limit").into_response();
                }
                if let Err(e) = file.write_all(&chunk).await {
                    tracing::error!("Write error: {e}");
                    let _ = tokio::fs::remove_file(&abs_path).await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Disk write error").into_response();
                }
            }
            saved_path = Some(rel_path);
        }
    }

    let saved = match saved_path {
        Some(p) if file_size > 0 => p,
        _ => return (StatusCode::UNPROCESSABLE_ENTITY, "File is required").into_response(),
    };

    // Get current lesson attachments
    let lesson = match repo::get_lesson_by_id(&state, &lesson_id).await {
        Ok(l) => l,
        Err(_) => {
            let _ = tokio::fs::remove_file(&saved).await;
            return (StatusCode::NOT_FOUND, "Lesson not found").into_response();
        }
    };

    let mut attachments = lesson.attachments.unwrap_or_default();
    attachments.push(serde_json::json!({
        "name": original_name,
        "url": format!("/{}", saved),
        "size": file_size,
        "mime": mime_type,
        "file_name": file_name,
    }));

    match repo::update_lesson_attachments(&state, &lesson_id, attachments).await {
        Ok(_) => (StatusCode::OK, "Attachment uploaded successfully").into_response(),
        Err(e) => {
            tracing::error!("DB update failed: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// List enrollments for a course (admin view)
async fn list_enrollments(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<CourseEnrollment>>, StatusCode> {
    repo::get_enrollments(&s, &id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ============================================================================
// Public handlers
// ============================================================================

#[derive(Debug, serde::Deserialize)]
pub struct CourseFilterParams {
    pub category: Option<String>,
    pub level: Option<String>,
}

/// List published courses (public, no auth)
async fn public_list(
    State(s): State<AppState>,
    Query(params): Query<CourseFilterParams>,
) -> Result<Json<Vec<Course>>, StatusCode> {
    repo::get_published_courses(&s, params.category.as_deref(), params.level.as_deref())
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get course detail with lessons (public)
async fn public_detail(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let course = repo::get_course_by_id(&s, &id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let lessons = repo::get_lessons(&s, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "course": course,
        "lessons": lessons,
    })))
}

// ============================================================================
// Enrollment handlers
// ============================================================================

/// Enroll current user in a course
async fn enroll(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<Json<CourseEnrollment>, StatusCode> {
    let uid = user.id.strip_prefix("account:").unwrap_or(&user.id);
    repo::enroll(&s, uid, &id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// List current user's enrollments
async fn my_enrollments(
    State(s): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<CourseEnrollment>>, StatusCode> {
    let uid = user.id.strip_prefix("account:").unwrap_or(&user.id);
    repo::get_my_enrollments(&s, uid)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
