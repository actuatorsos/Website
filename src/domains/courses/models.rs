//! Course Models — نماذج الكورسات والمسارات التعليمية

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    pub id: Option<Thing>,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub instructor_name: Option<String>,
    pub category: Option<String>,
    pub level: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub is_published: Option<bool>,
    pub is_archived: Option<bool>,
    pub total_lessons: Option<i64>,
    pub organization: Option<Thing>,
    pub created_by: Option<Thing>,
    pub created_at: Option<String>,
    // Enriched
    pub enrollment_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCourseRequest {
    pub title: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub instructor_name: Option<String>,
    pub category: Option<String>,
    pub level: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseLesson {
    pub id: Option<Thing>,
    pub course: Option<Thing>,
    pub title: String,
    pub description: Option<String>,
    pub lesson_order: Option<i64>,
    pub video: Option<Thing>,
    pub video_url: Option<String>,
    pub duration_mins: Option<i64>,
    pub attachments: Option<Vec<serde_json::Value>>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
    // Enriched from FETCH
    pub video_title: Option<String>,
    pub video_file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLessonRequest {
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub lesson_order: Option<i64>,
    pub video_id: Option<String>,
    pub video_url: Option<String>,
    pub duration_mins: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CourseEnrollment {
    pub id: Option<Thing>,
    pub account: Option<Thing>,
    pub course: Option<Thing>,
    pub progress: Option<f64>,
    pub status: Option<String>,
    pub enrolled_at: Option<String>,
    pub completed_at: Option<String>,
    // Enriched
    pub course_title: Option<String>,
    pub account_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    pub course_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub is_published: bool,
}
