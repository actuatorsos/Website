//! Projects Advanced Models — نماذج إدارة المشاريع (Trello Style)

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<Thing>,
    pub title: String,
    pub description: Option<String>,
    pub client: Option<Thing>,
    pub department: Option<Thing>,
    pub manager: Option<Thing>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: Option<Thing>,
    pub project: Thing,
    pub title: String,
    pub background: Option<String>,
    pub is_default: Option<bool>,
    pub position: Option<i64>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardRequest {
    pub project_id: String,
    pub title: String,
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardList {
    pub id: Option<Thing>,
    pub board: Thing,
    pub title: String,
    pub position: i64,
    pub wip_limit: Option<i64>,
    pub is_closed: Option<bool>,
    pub color: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBoardListRequest {
    pub board_id: String,
    pub title: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Option<Thing>,
    pub board_list: Thing,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub board_list_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCardRequest {
    pub target_list_id: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardComment {
    pub id: Option<Thing>,
    pub card: Thing,
    pub author: Thing,
    pub content: String,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardCommentRequest {
    pub card_id: String,
    pub author_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Option<Thing>,
    pub checklist: Thing,
    pub title: String,
    pub is_checked: Option<bool>,
    pub position: i64,
    pub due_date: Option<String>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardChecklist {
    pub id: Option<Thing>,
    pub card: Thing,
    pub title: String,
    pub position: i64,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

/// A board list with its cards embedded — used by the frontend Kanban view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardListWithCards {
    pub id: Option<Thing>,
    pub board: Thing,
    pub title: String,
    pub position: i64,
    pub color: Option<String>,
    pub is_closed: Option<bool>,
    pub cards: Vec<Card>,
}

/// Request to update an existing card
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMember {
    pub id: Option<Thing>,
    pub project: Thing,
    pub member: Thing,
    pub role: String, // owner, admin, member, viewer
    pub joined_at: Option<String>,
    pub is_archived: Option<bool>,
}

/// Project member with account details (email) for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMemberWithAccount {
    pub id: Option<Thing>,
    pub project: Thing,
    pub member: Thing,
    pub role: String,
    pub joined_at: Option<String>,
    pub member_email: Option<String>,
    pub member_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub member_id: String,
    pub role: Option<String>, // defaults to "member"
}
