// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Handlers للفيديوهات التعليمية
//!
//! يوفّر هذا الموديول ثلاثة نقاط دخول رئيسية:
//! - `upload_video`   — استقبال multipart/form-data وحفظ الفيديو
//! - `stream_video`   — بث الفيديو مع دعم HTTP Range (seek)
//! - `delete_video`   — حذف فيديو

use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{error, info};
use uuid::Uuid;

use crate::db::{AppState, DbError};

use super::{
    models::CreateVideoPayload,
    repository,
};

/// مجلد التخزين للفيديوهات (نسبي من جذر المشروع)
const VIDEO_UPLOAD_DIR: &str = "static/uploads/videos";

/// Allowed MIME types for video uploads
const ALLOWED_VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-msvideo",
    "video/x-matroska",
];

/// Allowed file extensions for video uploads
const ALLOWED_VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "avi", "mkv"];

/// Maximum video file size (200MB)
const MAX_VIDEO_SIZE: i64 = 200 * 1024 * 1024;

// ============================================================================
// Upload Handler
// ============================================================================

/// استقبال فيديو عبر multipart/form-data وحفظه على القرص وفي DB.
///
/// الحقول المُرسَلة في الـ Form:
/// - `title`       (text, إلزامي)
/// - `description` (text, اختياري)
/// - `video`       (file, إلزامي)
///
/// يعيد HTML جزئي (HTMX response) لتحديث القائمة بعد الرفع.
pub async fn upload_video(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    // --- استخراج هوية المستخدم من JWT cookie ---
    let uploaded_by_thing = match extract_user_thing(&headers, &state).await {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        }
    };

    let mut title = String::new();
    let mut description: Option<String> = None;
    let mut original_name = String::from("video.mp4");
    let mut mime_type = String::from("video/mp4");
    let mut file_size = 0i64;
    let mut file_path = None;

    // --- توليد اسم ملف مبدئي لحفظ التيار ---
    let unique_id = Uuid::new_v4().to_string();
    
    // إنشاء مسار رفع الملفات مبكراً
    let _ = tokio::fs::create_dir_all(VIDEO_UPLOAD_DIR).await;

    // --- قراءة حقول الـ multipart ---
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                match field.name() {
                    Some("title") => {
                        title = field.text().await.unwrap_or_default();
                    }
                    Some("description") => {
                        let val = field.text().await.unwrap_or_default();
                        if !val.is_empty() {
                            description = Some(val);
                        }
                    }
                    Some("video") => {
                        // نحفظ اسم الملف الأصلي
                        if let Some(fname) = field.file_name() {
                            original_name = fname.to_string();
                        }
                        // نحفظ الـ content-type
                        if let Some(ct) = field.content_type() {
                            mime_type = ct.to_string();
                        }

                        // --- التحقق من نوع الملف ---
                        if !ALLOWED_VIDEO_TYPES.contains(&mime_type.as_str()) {
                            return (StatusCode::UNPROCESSABLE_ENTITY, format!(
                                "Invalid video type '{}'. Allowed: mp4, webm, mov, avi, mkv",
                                mime_type
                            )).into_response();
                        }

                        // استخراج الصيغة والتحقق منها
                        let ext = PathBuf::from(&original_name)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("mp4")
                            .to_lowercase();

                        if !ALLOWED_VIDEO_EXTENSIONS.contains(&ext.as_str()) {
                            return (StatusCode::UNPROCESSABLE_ENTITY, format!(
                                "Invalid file extension '.{}'. Allowed: {}",
                                ext,
                                ALLOWED_VIDEO_EXTENSIONS.join(", ")
                            )).into_response();
                        }
                        
                        let unique_name = format!("{}.{}", unique_id, ext);
                        let rel_path = format!("{VIDEO_UPLOAD_DIR}/{unique_name}");
                        let abs_path = PathBuf::from(&rel_path);
                        file_path = Some(rel_path);

                        // فتح ملف للكتابة
                        let mut file = match tokio::fs::File::create(&abs_path).await {
                            Ok(f) => f,
                            Err(e) => {
                                error!("Failed to create file on disk: {e}");
                                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create file").into_response();
                            }
                        };

                        use tokio::io::AsyncWriteExt;
                        // قراءة الملف كتيار من الـ stream لتجنب تجاوز حد الذاكرة
                        while let Ok(Some(chunk)) = field.chunk().await {
                            file_size += chunk.len() as i64;

                            // فحص حد الحجم أثناء الرفع
                            if file_size > MAX_VIDEO_SIZE {
                                let _ = tokio::fs::remove_file(&abs_path).await;
                                return (StatusCode::PAYLOAD_TOO_LARGE, format!(
                                    "Video exceeds maximum size of {}MB",
                                    MAX_VIDEO_SIZE / (1024 * 1024)
                                )).into_response();
                            }

                            if let Err(e) = file.write_all(&chunk).await {
                                error!("Failed to write chunk to disk: {e}");
                                let _ = tokio::fs::remove_file(&abs_path).await;
                                return (StatusCode::INTERNAL_SERVER_ERROR, "Disk write error").into_response();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(None) => break, // نهاية الفورم
            Err(e) => {
                error!("Error parsing multipart stream: {e}");
                return (StatusCode::BAD_REQUEST, "Upload error. File might exceed limits or connection dropped.").into_response();
            }
        }
    }

    // --- التحقق من الإدخالات ---
    if title.trim().is_empty() {
        if let Some(path) = &file_path {
            let _ = tokio::fs::remove_file(path).await;
        }
        return (StatusCode::UNPROCESSABLE_ENTITY, "Title is required").into_response();
    }

    let rel_path = match file_path {
        Some(p) if file_size > 0 => p,
        _ => {
            return (StatusCode::UNPROCESSABLE_ENTITY, "Video file is required").into_response();
        }
    };

    // --- حفظ السجل في قاعدة البيانات ---
    let payload = CreateVideoPayload {
        title: title.trim().to_string(),
        description,
        file_path: rel_path.clone(),
        file_name: original_name,
        file_size,
        mime_type,
        uploaded_by: uploaded_by_thing,
    };

    match repository::create_video(&state, payload).await {
        Ok(video) => {
            info!("Video uploaded: {:?}", video.id);
            // نعيد HTML جزئي لـ HTMX يُظهر رسالة نجاح
            let body = format!(
                r#"<div class="alert alert-success" role="alert" id="upload-feedback">
                    <span class="font-bold">تم الرفع بنجاح!</span> سيظهر الفيديو في القائمة بعد إعادة التحميل.
                   </div>
                   <script>
                     setTimeout(() => {{ document.getElementById('upload-feedback')?.remove(); }}, 4000);
                     htmx.trigger('#video-list-container', 'refresh');
                   </script>"#
            );
            (StatusCode::OK, [("Content-Type", "text/html; charset=utf-8")], body)
                .into_response()
        }
        Err(e) => {
            // احذف الملف المحفوظ في حالة فشل DB لتجنب الملفات اليتيمة
            let _ = tokio::fs::remove_file(&rel_path).await;
            error!("DB insert failed: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ============================================================================
// Stream Handler (HTTP Range Requests)
// ============================================================================

/// بث فيديو بدعم كامل لـ HTTP Range Requests للسماح بعملية Seek.
///
/// يقرأ الترويسة `Range` لتحديد القطعة المطلوبة،
/// ويعيد `206 Partial Content` أو `200 OK` وفقاً لذلك.
pub async fn stream_video(
    State(state): State<AppState>,
    Path(id): Path<String>,
    req_headers: HeaderMap,
) -> Response {
    // جلب بيانات الفيديو من DB
    let video = match repository::get_video_by_id(&state, &id).await {
        Ok(v) => v,
        Err(DbError::NotFound) => {
            return (StatusCode::NOT_FOUND, "Video not found").into_response();
        }
        Err(e) => {
            error!("DB error fetching video {id}: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Server error").into_response();
        }
    };

    let file_path = PathBuf::from(&video.file_path);

    // التحقق من وجود الملف
    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(m) => m,
        Err(_) => {
            return (StatusCode::NOT_FOUND, "File not found on disk").into_response();
        }
    };

    let total_size = metadata.len();
    let mime = video.mime_type.clone();

    // --- تحليل ترويسة Range ---
    let range_header = req_headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !range_header.is_empty() && range_header.starts_with("bytes=") {
        // استخراج start و end
        let range_str = &range_header["bytes=".len()..];
        let parts: Vec<&str> = range_str.split('-').collect();

        let start = parts
            .first()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let end = parts
            .get(1)
            .and_then(|s| s.parse::<u64>().ok())
            // إذا لم يُحدَّد النهاية، استخدم chunk بحجم 1MB أو نهاية الملف
            .unwrap_or_else(|| {
                let chunk = 1024 * 1024; // 1MB
                (start + chunk).min(total_size - 1)
            });

        if start >= total_size || start > end {
            return (
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{total_size}"))],
                "",
            )
                .into_response();
        }

        let chunk_size = end - start + 1;

        // فتح الملف والقفز إلى الموضع المطلوب
        let mut file = match File::open(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open video file: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot open file").into_response();
            }
        };

        // نقل موضع القراءة بـ seek
        use tokio::io::AsyncSeekExt;
        if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
            error!("Seek error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Seek error").into_response();
        }

        // قراءة الـ chunk المطلوب فقط باستخدام tokio::io::AsyncReadExt
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; chunk_size as usize];
        match file.read_exact(&mut buf).await {
            Ok(_) => {}
            Err(e) => {
                // في حالة EOF قبل نهاية الـ chunk (مقبول)
                error!("Read error (may be EOF): {e}");
            }
        }

        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, mime)
            .header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{total_size}"),
            )
            .header(header::CONTENT_LENGTH, chunk_size.to_string())
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from(buf))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    } else {
        // طلب كامل — نبث الملف بالكامل كـ stream
        let file = match File::open(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open video file: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot open file").into_response();
            }
        };

        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CONTENT_LENGTH, total_size.to_string())
            .header(header::ACCEPT_RANGES, "bytes")
            .body(body)
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
}

// ============================================================================
// Delete Handler
// ============================================================================

/// حذف فيديو من DB والقرص
pub async fn delete_video_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    // جلب مسار الملف قبل الحذف من DB
    let video = match repository::get_video_by_id(&state, &id).await {
        Ok(v) => v,
        Err(DbError::NotFound) => {
            return (StatusCode::NOT_FOUND, "Video not found").into_response();
        }
        Err(e) => {
            error!("DB error: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Server error").into_response();
        }
    };

    let file_path = video.file_path.clone();

    // حذف من DB أولاً
    if let Err(e) = repository::delete_video(&state, &id).await {
        error!("Failed to delete video from DB: {e:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response();
    }

    // ثم حذف الملف الفعلي (نتجاهل الخطأ إذا لم يكن موجوداً)
    let _ = tokio::fs::remove_file(&file_path).await;

    info!("Video deleted: {id}");

    // HTMX: نعيد HTTP 200 مع تعليمة لإزالة العنصر من DOM
    (
        StatusCode::OK,
        [("HX-Trigger", "videoDeleted"), ("Content-Type", "text/html")],
        "",
    )
        .into_response()
}

// ============================================================================
// Helper
// ============================================================================

/// استخراج هوية المستخدم (Thing) من JWT cookie
async fn extract_user_thing(
    headers: &HeaderMap,
    state: &AppState,
) -> Option<surrealdb::sql::Thing> {
    // استخراج الـ cookie يدوياً من الترويسة (tower-cookies غير متاحة هنا)
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    let token_value = cookie_header
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("auth_token=")
        })?
        .to_string();

    let claims = crate::middleware::auth::decode_token(&token_value, &state.jwt_secret).ok()?;

    // جلب الحساب من DB للحصول على الـ ID الفعلي
    let account = state.get_account_by_email(&claims.email).await.ok()?;

    // account.id هو بالفعل Option<Thing>، نرجعه رأساً
    Some(account.id?)
}

// ============================================================================
// Routes
// ============================================================================

/// تسجيل مسارات الفيديوهات ضمن `/admin`
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/videos/upload", post(upload_video))
        .route("/videos/stream/{id}", get(stream_video))
        .route("/videos/{id}", delete(delete_video_handler))
}
