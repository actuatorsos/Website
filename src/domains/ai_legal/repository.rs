//! AI Legal Repository — عمليات قاعدة البيانات للمستشار القانوني

use super::models::*;
use crate::db::{AppState, DbError};

// ============================================================================
// Sessions — جلسات المحادثة
// ============================================================================

pub async fn create_session(
    state: &AppState,
    user_id: &str,
    specialty: &str,
) -> Result<LegalChatSession, DbError> {
    let session: Option<LegalChatSession> = state
        .db
        .query(
            "CREATE legal_chat_session SET \
             user = type::thing('account', $uid), \
             specialty = $specialty, \
             status = 'active', \
             message_count = 0, \
             created_at = time::now()",
        )
        .bind(("uid", user_id.to_string()))
        .bind(("specialty", specialty.to_string()))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    session.ok_or(DbError::NotFound)
}

pub async fn get_sessions(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<LegalChatSession>, DbError> {
    let sessions: Vec<LegalChatSession> = state
        .db
        .query(
            "SELECT * FROM legal_chat_session \
             WHERE user = type::thing('account', $uid) \
             ORDER BY created_at DESC LIMIT 50",
        )
        .bind(("uid", user_id.to_string()))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    Ok(sessions)
}

pub async fn get_session(
    state: &AppState,
    session_id: &str,
) -> Result<LegalChatSession, DbError> {
    let session: Option<LegalChatSession> = state
        .db
        .query("SELECT * FROM type::thing('legal_chat_session', $sid)")
        .bind(("sid", session_id.to_string()))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    session.ok_or(DbError::NotFound)
}

pub async fn delete_session(state: &AppState, session_id: &str) -> Result<(), DbError> {
    state
        .db
        .query(
            "DELETE legal_chat_message WHERE session = type::thing('legal_chat_session', $sid)",
        )
        .bind(("sid", session_id.to_string()))
        .await
        .map_err(DbError::Database)?;
    state
        .db
        .query("DELETE type::thing('legal_chat_session', $sid)")
        .bind(("sid", session_id.to_string()))
        .await
        .map_err(DbError::Database)?;
    Ok(())
}

pub async fn increment_message_count(
    state: &AppState,
    session_id: &str,
) -> Result<(), DbError> {
    state
        .db
        .query(
            "UPDATE type::thing('legal_chat_session', $sid) SET \
             message_count += 1, \
             last_message_at = time::now()",
        )
        .bind(("sid", session_id.to_string()))
        .await
        .map_err(DbError::Database)?;
    Ok(())
}

pub async fn update_session_title(
    state: &AppState,
    session_id: &str,
    title: &str,
) -> Result<(), DbError> {
    state
        .db
        .query("UPDATE type::thing('legal_chat_session', $sid) SET title = $title")
        .bind(("sid", session_id.to_string()))
        .bind(("title", title.to_string()))
        .await
        .map_err(DbError::Database)?;
    Ok(())
}

// ============================================================================
// Messages — رسائل المحادثة
// ============================================================================

pub async fn get_messages(
    state: &AppState,
    session_id: &str,
) -> Result<Vec<ChatMessage>, DbError> {
    let messages: Vec<ChatMessage> = state
        .db
        .query(
            "SELECT * FROM legal_chat_message \
             WHERE session = type::thing('legal_chat_session', $sid) \
             ORDER BY created_at ASC",
        )
        .bind(("sid", session_id.to_string()))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    Ok(messages)
}

pub async fn save_message(
    state: &AppState,
    session_id: &str,
    role: &str,
    content: &str,
    citations: Option<&Vec<Citation>>,
    confidence: Option<f64>,
    tokens_used: Option<i32>,
    response_ms: Option<i32>,
) -> Result<ChatMessage, DbError> {
    let citations_json = serde_json::to_value(citations.cloned().unwrap_or_default())
        .unwrap_or(serde_json::Value::Array(vec![]));

    let msg: Option<ChatMessage> = state
        .db
        .query(
            "CREATE legal_chat_message SET \
             session = type::thing('legal_chat_session', $sid), \
             role = $role, \
             content = $content, \
             citations = $citations, \
             confidence = $confidence, \
             tokens_used = $tokens, \
             response_ms = $ms, \
             created_at = time::now()",
        )
        .bind(("sid", session_id.to_string()))
        .bind(("role", role.to_string()))
        .bind(("content", content.to_string()))
        .bind(("citations", citations_json))
        .bind(("confidence", confidence))
        .bind(("tokens", tokens_used))
        .bind(("ms", response_ms))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    msg.ok_or(DbError::NotFound)
}

// ============================================================================
// Legal Articles — البحث في المواد القانونية
// ============================================================================

pub async fn search_articles(
    state: &AppState,
    query: &str,
    _specialty: Option<&str>,
    limit: i32,
) -> Result<Vec<SearchResult>, DbError> {
    // Simple keyword search using string::contains
    let results: Vec<SearchResult> = state
        .db
        .query(
            "SELECT article_number, text_ar, chapter \
             FROM legal_article \
             WHERE string::contains(string::lowercase(text_ar), string::lowercase($query)) \
             LIMIT $lim",
        )
        .bind(("query", query.to_string()))
        .bind(("lim", limit))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    Ok(results)
}

// ============================================================================
// Specialties — التخصصات
// ============================================================================

pub fn get_specialties() -> Vec<SpecialtyInfo> {
    vec![
        SpecialtyInfo {
            id: "litigation".to_string(),
            name_ar: "المحاماة والتقاضي".to_string(),
            description: "الدعاوى المدنية والجزائية والشرعية، إجراءات التقاضي والطعن بالأحكام".to_string(),
            suggested_questions: vec![
                "كيف أرفع دعوى مدنية أمام محكمة البداية؟".to_string(),
                "ما هي مدة الطعن بالاستئناف؟".to_string(),
                "ما هي شروط التحكيم التجاري في سوريا؟".to_string(),
            ],
        },
        SpecialtyInfo {
            id: "entrepreneurship".to_string(),
            name_ar: "ريادة الأعمال وتأسيس الشركات".to_string(),
            description: "تأسيس الشركات والسجل التجاري والعقود التجارية وقوانين الاستثمار".to_string(),
            suggested_questions: vec![
                "كيف أؤسس شركة محدودة المسؤولية في سوريا؟".to_string(),
                "ما هي حوافز قانون الاستثمار رقم 18/2021؟".to_string(),
                "ما الفرق بين الشركة التضامنية والمحدودة المسؤولية؟".to_string(),
            ],
        },
        SpecialtyInfo {
            id: "patents".to_string(),
            name_ar: "براءات الاختراع والملكية الفكرية".to_string(),
            description: "تسجيل براءات الاختراع والعلامات التجارية وحماية حقوق المؤلف".to_string(),
            suggested_questions: vec![
                "كيف أسجل براءة اختراع في سوريا؟".to_string(),
                "ما هي مدة حماية العلامة التجارية؟".to_string(),
                "كيف أحمي حقوق المؤلف لبرنامج حاسوبي؟".to_string(),
            ],
        },
        SpecialtyInfo {
            id: "legal_accounting".to_string(),
            name_ar: "المحاسبة القانونية والضرائب".to_string(),
            description: "النظام الضريبي والالتزامات المحاسبية والتأمينات الاجتماعية".to_string(),
            suggested_questions: vec![
                "كيف أحسب ضريبة الدخل على أرباحي؟".to_string(),
                "ما هي نسبة اشتراكات التأمينات الاجتماعية؟".to_string(),
                "ما هي الإعفاءات الضريبية المتاحة للمنشآت الصغيرة؟".to_string(),
            ],
        },
        SpecialtyInfo {
            id: "general".to_string(),
            name_ar: "استشارة قانونية عامة".to_string(),
            description: "استشارات قانونية عامة في جميع فروع القانون السوري".to_string(),
            suggested_questions: vec![
                "ما هي حقوقي كمستأجر في سوريا؟".to_string(),
                "كيف أحصل على حكم نفقة؟".to_string(),
                "ما هي عقوبة إصدار شيك بدون رصيد؟".to_string(),
            ],
        },
    ]
}

// ============================================================================
// Procedures — الإجراءات القانونية
// ============================================================================

pub async fn get_procedures(
    state: &AppState,
    specialty: Option<&str>,
) -> Result<Vec<LegalProcedure>, DbError> {
    let procedures: Vec<LegalProcedure> = match specialty {
        Some(spec) => {
            state
                .db
                .query(
                    "SELECT * FROM legal_procedure WHERE specialty = $spec ORDER BY name_ar ASC",
                )
                .bind(("spec", spec.to_string()))
                .await
                .map_err(DbError::Database)?
                .take(0)
                .map_err(DbError::Database)?
        }
        None => {
            state
                .db
                .query("SELECT * FROM legal_procedure ORDER BY name_ar ASC")
                .await
                .map_err(DbError::Database)?
                .take(0)
                .map_err(DbError::Database)?
        }
    };
    Ok(procedures)
}

// ============================================================================
// Search Logging
// ============================================================================

pub async fn log_search(
    state: &AppState,
    user_id: &str,
    query: &str,
    specialty: Option<&str>,
    results_count: i32,
) -> Result<(), DbError> {
    let _ = state
        .db
        .query(
            "CREATE legal_search_log SET \
             user = type::thing('account', $uid), \
             query = $query, \
             specialty = $spec, \
             results_count = $count, \
             created_at = time::now()",
        )
        .bind(("uid", user_id.to_string()))
        .bind(("query", query.to_string()))
        .bind(("spec", specialty.map(|s| s.to_string())))
        .bind(("count", results_count))
        .await;
    Ok(())
}
