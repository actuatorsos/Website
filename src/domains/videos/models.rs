// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// يمثّل فيديو تعليمي مخزّن في النظام.
/// يستخدم مع SurrealDB — الحقل `id` اختياري لأن DB تولّده عند الإنشاء.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    /// المعرّف الفريد — `None` قبل الحفظ، `Some` بعده (مثال: "educational_video:abc123")
    pub id: Option<surrealdb::sql::Thing>,
    /// عنوان الفيديو
    pub title: String,
    /// وصف اختياري للفيديو
    pub description: Option<String>,
    /// المسار النسبي للملف (مثال: "uploads/videos/abc123.mp4")
    pub file_path: String,
    /// اسم الملف الأصلي كما رُفع
    pub file_name: String,
    /// حجم الملف بالبايت
    pub file_size: i64,
    /// نوع الـ MIME
    pub mime_type: String,
    /// مدة الفيديو بالثواني (اختياري)
    pub duration_secs: Option<i64>,
    /// المستخدم الذي رفع الفيديو (مرجع لجدول account)
    pub uploaded_by: surrealdb::sql::Thing,
    /// تاريخ الرفع
    pub created_at: Option<DateTime<Utc>>,
}

/// يستخدم في قوالب Askama — نسخة مبسّطة خالية من أنواع SurrealDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoView {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub duration_secs: Option<i64>,
    pub uploaded_by: String,
    pub created_at: String,
}

/// البيانات المُرسَلة عند إنشاء فيديو جديد
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVideoPayload {
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub uploaded_by: surrealdb::sql::Thing,
}

impl VideoView {
    /// تحويل من `Video` (نموذج DB) إلى `VideoView` (للعرض في القوالب)
    pub fn from_video(v: Video) -> Self {
        let id = v
            .id
            .as_ref()
            .map(|t| format!("{}", t.id))
            .unwrap_or_default();

        let uploaded_by = format!("{}", v.uploaded_by.id);

        let created_at = v
            .created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "—".to_string());

        Self {
            id,
            title: v.title,
            description: v.description,
            file_path: v.file_path,
            file_name: v.file_name,
            file_size: v.file_size,
            mime_type: v.mime_type,
            duration_secs: v.duration_secs,
            uploaded_by,
            created_at,
        }
    }

    /// تنسيق حجم الملف للعرض (مثال: "24.5 MB")
    pub fn formatted_size(&self) -> String {
        let bytes = self.file_size as f64;
        if bytes < 1024.0 {
            format!("{} B", bytes)
        } else if bytes < 1024.0 * 1024.0 {
            format!("{:.1} KB", bytes / 1024.0)
        } else if bytes < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", bytes / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
