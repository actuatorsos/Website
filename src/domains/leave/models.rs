//! Leave Management Models — نماذج الإجازات

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveRequest {
    pub id: Option<Thing>,
    pub employee: Thing,
    pub leave_type: String,
    pub start_date: String,
    pub end_date: String,
    pub days: Option<i64>,
    pub reason: Option<String>,
    pub status: Option<String>, // pending, approved, rejected
    pub approved_by: Option<Thing>,
    pub approved_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeaveRequest {
    pub employee_id: String,
    pub leave_type: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveLeaveRequest {
    pub approved_by_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectLeaveRequest {
    pub rejection_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveBalance {
    pub id: Option<Thing>,
    pub employee: Thing,
    pub year: i64,
    pub annual_total: Option<i64>,
    pub annual_used: Option<i64>,
    pub annual_remaining: Option<i64>,
    pub sick_total: Option<i64>,
    pub sick_used: Option<i64>,
    pub hajj_total: Option<i64>,
    pub hajj_used: Option<i64>,
    pub maternity_total: Option<i64>,
    pub maternity_used: Option<i64>,
    pub created_at: Option<String>,
}
