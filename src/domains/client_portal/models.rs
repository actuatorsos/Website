//! Client Portal Models — نماذج بوابة العميل

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct SupportTicket {
    pub id: Option<RecordId>,
    pub subject: String,
    pub description: Option<String>,
    pub client: Option<RecordId>,
    pub submitted_by: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<RecordId>,
    pub resolution: Option<String>,
    pub rating: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub closed_at: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub description: Option<String>,
    pub client_id: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct TicketReply {
    pub id: Option<RecordId>,
    pub ticket: Option<RecordId>,
    pub author: Option<String>,
    pub author_role: Option<String>,
    pub message: String,
    pub created_at: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateReplyRequest {
    pub message: String,
    pub author_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateTicketStatus {
    pub status: String,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CloseTicketRequest {
    pub resolution: Option<String>,
    pub rating: Option<i64>,
}
