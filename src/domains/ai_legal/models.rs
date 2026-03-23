//! AI Legal Models — نماذج المستشار القانوني السوري

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// Chat — المحادثة مع المستشار القانوني
// ============================================================================

/// Request to chat with the AI legal advisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub session_id: Option<String>,
    pub message: String,
    pub specialty: Option<String>,
}

/// AI legal advisor response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: String,
    pub citations: Vec<Citation>,
    pub confidence: f64,
    pub session_id: String,
    pub tokens_used: Option<i32>,
}

/// Legal citation reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub law: String,
    pub article: i32,
    pub text: String,
}

// ============================================================================
// Session — جلسات المحادثة
// ============================================================================

/// Chat session stored in DB — matches schema field names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalChatSession {
    pub id: Option<Thing>,
    #[serde(default)]
    pub user: Option<Thing>,
    pub title: Option<String>,
    #[serde(default = "default_specialty")]
    pub specialty: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message_count: i32,
    pub last_message_at: Option<String>,
    pub created_at: Option<String>,
}

fn default_specialty() -> String {
    "general".to_string()
}

/// Session response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: Option<String>,
    pub specialty: String,
    pub message_count: i32,
    pub created_at: String,
}

/// Chat message stored in DB — matches schema field names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<Thing>,
    #[serde(default)]
    pub session: Option<Thing>,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub citations: Option<Vec<Citation>>,
    pub confidence: Option<f64>,
    pub specialty_used: Option<String>,
    pub tokens_used: Option<i32>,
    pub response_ms: Option<i32>,
    pub feedback: Option<String>,
    pub created_at: Option<String>,
}

// ============================================================================
// Search — البحث في المواد القانونية
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub specialty: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub article_number: Option<i32>,
    pub text_ar: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    pub chapter: Option<String>,
    #[serde(default)]
    pub relevance: Option<f64>,
}

// ============================================================================
// Specialties — التخصصات القانونية
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialtyInfo {
    pub id: String,
    pub name_ar: String,
    pub description: String,
    pub suggested_questions: Vec<String>,
}

// ============================================================================
// Procedures — الإجراءات القانونية
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalProcedure {
    pub id: Option<Thing>,
    pub name_ar: Option<String>,
    pub category: Option<String>,
    pub specialty: Option<String>,
    pub steps: Option<Vec<serde_json::Value>>,
    pub required_docs: Option<Vec<String>>,
    pub fees: Option<serde_json::Value>,
    pub timeline: Option<String>,
    pub authority: Option<String>,
    pub legal_basis: Option<String>,
}
