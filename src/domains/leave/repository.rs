//! Leave Repository — عمليات الإجازات في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_leave_request(
    state: &AppState,
    req: CreateLeaveRequest,
) -> Result<LeaveRequest, DbError> {
    let emp_id = req.employee_id;
    let leave_type = req.leave_type;
    let start_date = req.start_date;
    let end_date = req.end_date;
    let reason = req.reason;
    let days = calculate_days(&start_date, &end_date);

    let leave: Option<LeaveRequest> = state
        .db
        .query(
            "CREATE leave_request SET \
             employee = type::thing('employee', $emp_id), \
             leave_type = $leave_type, start_date = $start_date, \
             end_date = $end_date, days = $days, \
             reason = $reason, status = 'pending'",
        )
        .bind(("emp_id", emp_id))
        .bind(("leave_type", leave_type))
        .bind(("start_date", start_date))
        .bind(("end_date", end_date))
        .bind(("days", days))
        .bind(("reason", reason))
        .await?
        .take(0)?;
    leave.ok_or(DbError::NotFound)
}

pub async fn get_all_leave_requests(state: &AppState, status: Option<&str>) -> Result<Vec<LeaveRequest>, DbError> {
    let leaves: Vec<LeaveRequest> = match status {
        Some(s) => {
            state.db
                .query("SELECT * FROM leave_request WHERE (is_archived = false OR is_archived = NONE) AND status = $status ORDER BY created_at DESC")
                .bind(("status", s.to_string()))
                .await?.take(0)?
        }
        None => {
            state.db
                .query("SELECT * FROM leave_request WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
                .await?.take(0)?
        }
    };
    Ok(leaves)
}

pub async fn get_leave_requests_by_employee(
    state: &AppState,
    employee_id: &str,
) -> Result<Vec<LeaveRequest>, DbError> {
    let id = employee_id.to_string();
    let leaves: Vec<LeaveRequest> = state.db
        .query("SELECT * FROM leave_request WHERE employee = type::thing('employee', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY created_at DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(leaves)
}

pub async fn approve_leave(
    state: &AppState,
    id: &str,
    req: ApproveLeaveRequest,
) -> Result<LeaveRequest, DbError> {
    let id = id.to_string();
    let approved_by = req.approved_by_id;
    let now = chrono::Utc::now().to_rfc3339();
    let leave: Option<LeaveRequest> = state
        .db
        .update(("leave_request", id))
        .merge(serde_json::json!({
            "status": "approved",
            "approved_by": format!("employee:{}", approved_by),
            "approved_at": now,
        }))
        .await?;
    leave.ok_or(DbError::NotFound)
}

pub async fn reject_leave(
    state: &AppState,
    id: &str,
    req: RejectLeaveRequest,
) -> Result<LeaveRequest, DbError> {
    let id = id.to_string();
    let rejection_reason = req.rejection_reason;
    let leave: Option<LeaveRequest> = state
        .db
        .update(("leave_request", id))
        .merge(serde_json::json!({
            "status": "rejected",
            "rejection_reason": rejection_reason,
        }))
        .await?;
    leave.ok_or(DbError::NotFound)
}

pub async fn get_leave_balance(
    state: &AppState,
    employee_id: &str,
) -> Result<Option<LeaveBalance>, DbError> {
    let id = employee_id.to_string();
    let year = chrono::Utc::now()
        .format("%Y")
        .to_string()
        .parse::<i64>()
        .unwrap_or(2025);
    let balance: Option<LeaveBalance> = state.db
        .query("SELECT * FROM leave_balance WHERE employee = type::thing('employee', $id) AND year = $year LIMIT 1")
        .bind(("id", id))
        .bind(("year", year))
        .await?.take(0)?;
    Ok(balance)
}

fn calculate_days(start: &str, end: &str) -> i64 {
    if let (Ok(s), Ok(e)) = (
        chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d"),
        chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d"),
    ) {
        (e - s).num_days() + 1
    } else {
        1
    }
}
