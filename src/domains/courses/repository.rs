//! Course Repository — عمليات الكورسات في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_course(
    state: &AppState,
    req: CreateCourseRequest,
    org_id: Option<&str>,
    user_id: &str,
) -> Result<Course, DbError> {
    let title = req.title;
    let desc = req.description;
    let thumbnail = req.thumbnail;
    let instructor = req.instructor_name;
    let cat = req.category.unwrap_or_else(|| "other".to_string());
    let level = req.level.unwrap_or_else(|| "beginner".to_string());
    let price = req.price.unwrap_or(0.0);
    let currency = req.currency.unwrap_or_else(|| "USD".to_string());

    let course: Option<Course> = if let Some(org) = org_id {
        state.db
            .query(
                "CREATE course SET \
                 title = $title, description = $desc, thumbnail = $thumbnail, \
                 instructor_name = $instructor, category = $cat, level = $level, \
                 price = $price, currency = $currency, \
                 organization = type::record($org), \
                 created_by = type::thing('account', $uid)"
            )
            .bind(("title", title))
            .bind(("desc", desc))
            .bind(("thumbnail", thumbnail))
            .bind(("instructor", instructor))
            .bind(("cat", cat))
            .bind(("level", level))
            .bind(("price", price))
            .bind(("currency", currency))
            .bind(("org", org.to_string()))
            .bind(("uid", user_id.to_string()))
            .await?
            .take(0)?
    } else {
        state.db
            .query(
                "CREATE course SET \
                 title = $title, description = $desc, thumbnail = $thumbnail, \
                 instructor_name = $instructor, category = $cat, level = $level, \
                 price = $price, currency = $currency, \
                 created_by = type::thing('account', $uid)"
            )
            .bind(("title", title))
            .bind(("desc", desc))
            .bind(("thumbnail", thumbnail))
            .bind(("instructor", instructor))
            .bind(("cat", cat))
            .bind(("level", level))
            .bind(("price", price))
            .bind(("currency", currency))
            .bind(("uid", user_id.to_string()))
            .await?
            .take(0)?
    };
    course.ok_or(DbError::NotFound)
}

pub async fn get_all_courses(
    state: &AppState,
    org_id: Option<&str>,
) -> Result<Vec<Course>, DbError> {
    let courses: Vec<Course> = if let Some(org) = org_id {
        state.db
            .query(
                "SELECT *, \
                 (SELECT count() FROM course_enrollment WHERE course = $parent.id GROUP ALL)[0].count AS enrollment_count \
                 FROM course WHERE (is_archived = false OR is_archived = NONE) \
                 AND organization = type::record($org) ORDER BY created_at DESC"
            )
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state.db
            .query(
                "SELECT *, \
                 (SELECT count() FROM course_enrollment WHERE course = $parent.id GROUP ALL)[0].count AS enrollment_count \
                 FROM course WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC"
            )
            .await?
            .take(0)?
    };
    Ok(courses)
}

pub async fn get_published_courses(
    state: &AppState,
    category: Option<&str>,
    level: Option<&str>,
) -> Result<Vec<Course>, DbError> {
    let mut query_str = String::from(
        "SELECT *, \
         (SELECT count() FROM course_enrollment WHERE course = $parent.id GROUP ALL)[0].count AS enrollment_count \
         FROM course WHERE is_published = true AND (is_archived = false OR is_archived = NONE)"
    );

    if category.is_some() {
        query_str.push_str(" AND category = $cat");
    }
    if level.is_some() {
        query_str.push_str(" AND level = $lvl");
    }
    query_str.push_str(" ORDER BY created_at DESC");

    let courses: Vec<Course> = state.db
        .query(&query_str)
        .bind(("cat", category.unwrap_or("").to_string()))
        .bind(("lvl", level.unwrap_or("").to_string()))
        .await?
        .take(0)?;
    Ok(courses)
}

pub async fn get_course_by_id(state: &AppState, id: &str) -> Result<Course, DbError> {
    let course: Option<Course> = state.db
        .query(
            "SELECT *, \
             (SELECT count() FROM course_enrollment WHERE course = $parent.id GROUP ALL)[0].count AS enrollment_count \
             FROM type::thing('course', $id)"
        )
        .bind(("id", id.to_string()))
        .await?
        .take(0)?;
    course.ok_or(DbError::NotFound)
}

pub async fn toggle_publish(state: &AppState, id: &str, publish: bool) -> Result<(), DbError> {
    state.db
        .query("UPDATE type::thing('course', $id) SET is_published = $pub")
        .bind(("id", id.to_string()))
        .bind(("pub", publish))
        .await?;
    Ok(())
}

pub async fn delete_course(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "course", id).await
}

pub async fn create_lesson(
    state: &AppState,
    req: CreateLessonRequest,
) -> Result<CourseLesson, DbError> {
    let cid = req.course_id;
    let title = req.title;
    let desc = req.description;
    let order = req.lesson_order.unwrap_or(0);
    let video_url = req.video_url;
    let duration = req.duration_mins;

    let lesson: Option<CourseLesson> = if let Some(vid) = req.video_id {
        state.db
            .query(
                "CREATE course_lesson SET \
                 course = type::thing('course', $cid), title = $title, \
                 description = $desc, lesson_order = $order, \
                 video = type::thing('educational_video', $vid), \
                 video_url = $vurl, duration_mins = $dur"
            )
            .bind(("cid", cid))
            .bind(("title", title))
            .bind(("desc", desc))
            .bind(("order", order))
            .bind(("vid", vid))
            .bind(("vurl", video_url))
            .bind(("dur", duration))
            .await?
            .take(0)?
    } else {
        state.db
            .query(
                "CREATE course_lesson SET \
                 course = type::thing('course', $cid), title = $title, \
                 description = $desc, lesson_order = $order, \
                 video_url = $vurl, duration_mins = $dur"
            )
            .bind(("cid", cid))
            .bind(("title", title))
            .bind(("desc", desc))
            .bind(("order", order))
            .bind(("vurl", video_url))
            .bind(("dur", duration))
            .await?
            .take(0)?
    };
    lesson.ok_or(DbError::NotFound)
}

pub async fn get_lessons(
    state: &AppState,
    course_id: &str,
) -> Result<Vec<CourseLesson>, DbError> {
    let lessons: Vec<CourseLesson> = state.db
        .query(
            "SELECT *, video.title AS video_title, video.file_path AS video_file_path \
             FROM course_lesson \
             WHERE course = type::thing('course', $cid) AND (is_archived = false OR is_archived = NONE) \
             ORDER BY lesson_order ASC"
        )
        .bind(("cid", course_id.to_string()))
        .await?
        .take(0)?;
    Ok(lessons)
}

pub async fn delete_lesson(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "course_lesson", id).await
}

pub async fn update_lesson_attachments(
    state: &AppState,
    id: &str,
    attachments: Vec<serde_json::Value>,
) -> Result<(), DbError> {
    state.db
        .query("UPDATE type::thing('course_lesson', $id) SET attachments = $att")
        .bind(("id", id.to_string()))
        .bind(("att", attachments))
        .await?;
    Ok(())
}

pub async fn get_lesson_by_id(
    state: &AppState,
    id: &str,
) -> Result<CourseLesson, DbError> {
    let lesson: Option<CourseLesson> = state.db
        .query("SELECT * FROM type::thing('course_lesson', $id)")
        .bind(("id", id.to_string()))
        .await?
        .take(0)?;
    lesson.ok_or(DbError::NotFound)
}

pub async fn enroll(
    state: &AppState,
    account_id: &str,
    course_id: &str,
) -> Result<CourseEnrollment, DbError> {
    let enr: Option<CourseEnrollment> = state.db
        .query(
            "CREATE course_enrollment SET \
             account = type::thing('account', $aid), \
             course = type::thing('course', $cid)"
        )
        .bind(("aid", account_id.to_string()))
        .bind(("cid", course_id.to_string()))
        .await?
        .take(0)?;
    enr.ok_or(DbError::NotFound)
}

pub async fn get_enrollments(
    state: &AppState,
    course_id: &str,
) -> Result<Vec<CourseEnrollment>, DbError> {
    let enrs: Vec<CourseEnrollment> = state.db
        .query(
            "SELECT *, account.email AS account_email, course.title AS course_title \
             FROM course_enrollment WHERE course = type::thing('course', $cid)"
        )
        .bind(("cid", course_id.to_string()))
        .await?
        .take(0)?;
    Ok(enrs)
}

pub async fn get_my_enrollments(
    state: &AppState,
    account_id: &str,
) -> Result<Vec<CourseEnrollment>, DbError> {
    let enrs: Vec<CourseEnrollment> = state.db
        .query(
            "SELECT *, course.title AS course_title \
             FROM course_enrollment WHERE account = type::thing('account', $aid)"
        )
        .bind(("aid", account_id.to_string()))
        .await?
        .take(0)?;
    Ok(enrs)
}

pub async fn update_course_lesson_count(
    state: &AppState,
    course_id: &str,
) -> Result<(), DbError> {
    state.db
        .query(
            "UPDATE type::thing('course', $cid) SET total_lessons = \
             (SELECT count() FROM course_lesson WHERE course = type::thing('course', $cid) AND (is_archived = false OR is_archived = NONE) GROUP ALL)[0].count ?? 0"
        )
        .bind(("cid", course_id.to_string()))
        .await?;
    Ok(())
}
