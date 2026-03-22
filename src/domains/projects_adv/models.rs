//! Projects Advanced Models — نماذج إدارة المشاريع (Trello Style)

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Project {
    pub id: Option<RecordId>,
    pub title: String,
    pub description: Option<String>,
    pub client: Option<RecordId>,
    pub department: Option<RecordId>,
    pub manager: Option<RecordId>,
    pub budget: Option<f64>,
    pub spent: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,   // active, on_hold, completed, cancelled
    pub priority: Option<String>, // low, medium, high
    pub visibility: Option<String>,
    pub progress_percent: Option<i64>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateProjectRequest {
    pub title: String,
    pub description: Option<String>,
    pub client_id: Option<String>,
    pub department_id: Option<String>,
    pub manager_id: Option<String>,
    pub budget: Option<f64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub priority: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Board {
    pub id: Option<RecordId>,
    pub project: RecordId,
    pub title: String,
    pub background: Option<String>,
    pub is_default: Option<bool>,
    pub position: Option<i64>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateBoardRequest {
    pub project_id: String,
    pub title: String,
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct BoardList {
    pub id: Option<RecordId>,
    pub board: RecordId,
    pub title: String,
    pub position: i64,
    pub wip_limit: Option<i64>,
    pub is_closed: Option<bool>,
    pub color: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateBoardListRequest {
    pub board_id: String,
    pub title: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Card {
    pub id: Option<RecordId>,
    pub board_list: RecordId,
    pub title: String,
    pub description: Option<String>,
    pub position: i64,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub is_complete: Option<bool>,
    pub cover_color: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateCardRequest {
    pub board_list_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MoveCardRequest {
    pub target_list_id: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CardComment {
    pub id: Option<RecordId>,
    pub card: RecordId,
    pub author: RecordId,
    pub content: String,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateCardCommentRequest {
    pub card_id: String,
    pub author_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ChecklistItem {
    pub id: Option<RecordId>,
    pub checklist: RecordId,
    pub title: String,
    pub is_checked: Option<bool>,
    pub position: i64,
    pub due_date: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CardChecklist {
    pub id: Option<RecordId>,
    pub card: RecordId,
    pub title: String,
    pub position: i64,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

/// A board list with its cards embedded — used by the frontend Kanban view
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct BoardListWithCards {
    pub id: Option<RecordId>,
    pub board: RecordId,
    pub title: String,
    pub position: i64,
    pub color: Option<String>,
    pub is_closed: Option<bool>,
    pub cards: Vec<Card>,
}

/// Request to update an existing card
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateCardRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub estimated_hours: Option<f64>,
}

// ============================================================================
// Project Members — أعضاء المشروع
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProjectMember {
    pub id: Option<RecordId>,
    pub project: RecordId,
    pub member: RecordId,
    pub role: String, // owner, admin, member, viewer
    pub joined_at: Option<String>,
    pub is_archived: Option<bool>,
}

/// Project member with account details (email) for display
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProjectMemberWithAccount {
    pub id: Option<RecordId>,
    pub project: RecordId,
    pub member: RecordId,
    pub role: String,
    pub joined_at: Option<String>,
    pub member_email: Option<String>,
    pub member_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct AddMemberRequest {
    pub member_id: String,
    pub role: Option<String>, // defaults to "member"
}
