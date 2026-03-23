// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! قاعدة بيانات الفيديوهات التعليمية

use crate::db::{AppState, DbError};
use super::models::{CreateVideoPayload, Video};

/// جلب جميع الفيديوهات التعليمية مرتّبة من الأحدث إلى الأقدم
pub async fn get_all_videos(state: &AppState) -> Result<Vec<Video>, DbError> {
    let db = &state.db;
    let videos: Vec<Video> = db
        .query("SELECT * FROM educational_video ORDER BY created_at DESC LIMIT 200")
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    Ok(videos)
}

/// جلب فيديو واحد بمعرّفه
pub async fn get_video_by_id(state: &AppState, id: &str) -> Result<Video, DbError> {
    let db = &state.db;
    let record_id = id.replace("educational_video:", "");

    let video: Option<Video> = db
        .select(("educational_video", record_id.as_str()))
        .await
        .map_err(DbError::Database)?;

    video.ok_or(DbError::NotFound)
}

/// إنشاء فيديو جديد — يستخدم SurrealQL مدمجاً لتمرير record<account> بالشكل الصحيح
pub async fn create_video(state: &AppState, payload: CreateVideoPayload) -> Result<Video, DbError> {
    let db = &state.db;

    // نبني صيغة معرّف المستخدم لـ SurrealDB (مثال: account:abc123)
    let uploaded_by_str = format!("{}:{}", payload.uploaded_by.tb, payload.uploaded_by.id);

    // ننسخ القيم كـ String لتتجاوز قيود 'static لـ bind()
    let title       = payload.title.clone();
    let description = payload.description.clone();
    let file_path   = payload.file_path.clone();
    let file_name   = payload.file_name.clone();
    let file_size   = payload.file_size;
    let mime_type   = payload.mime_type.clone();

    // الاستعلام المضمّن — الـ uploaded_by يُوضع مباشرة في الـ SurrealQL لتجنب مشاكل Serialization
    let query = format!(
        r#"CREATE educational_video CONTENT {{
            title: $title,
            description: $description,
            file_path: $file_path,
            file_name: $file_name,
            file_size: $file_size,
            mime_type: $mime_type,
            uploaded_by: {uploaded_by_str}
        }}"#
    );

    let created: Option<Video> = db
        .query(query)
        .bind(("title",       title))
        .bind(("description", description))
        .bind(("file_path",   file_path))
        .bind(("file_name",   file_name))
        .bind(("file_size",   file_size))
        .bind(("mime_type",   mime_type))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;

    created.ok_or(DbError::NotFound)
}

/// حذف فيديو بمعرّفه
pub async fn delete_video(state: &AppState, id: &str) -> Result<(), DbError> {
    let db = &state.db;
    let record_id = id.replace("educational_video:", "");

    let _: Option<Video> = db
        .delete(("educational_video", record_id.as_str()))
        .await
        .map_err(DbError::Database)?;

    Ok(())
}
