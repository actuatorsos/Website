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

    // emp_id could be from employee table (e1) or account table (emp1)
    // Find the account via employee email lookup
    let leave: Option<LeaveRequest> = state
        .db
        .query(
            "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $eid)); \
             LET $acc = (SELECT VALUE id FROM account WHERE email = $emp_email[0]); \
             LET $account_id = IF $acc[0] != NONE THEN $acc[0] ELSE type::thing('account', $eid) END; \
             CREATE leave_request SET \
             employee = $account_id, \
             leave_type = $leave_type, start_date = <datetime>$start_date, \
             end_date = <datetime>$end_date, days = $days, \
             reason = $reason, status = 'pending'",
        )
        .bind(("eid", emp_id))
        .bind(("leave_type", leave_type))
        .bind(("start_date", start_date))
        .bind(("end_date", end_date))
        .bind(("days", days))
        .bind(("reason", reason))
        .await?
        .take(3)?;
    leave.ok_or(DbError::NotFound)
}

pub async fn get_all_leave_requests(state: &AppState, status: Option<&str>, scope_account_id: Option<&str>, org_id: Option<&str>) -> Result<Vec<LeaveRequest>, DbError> {
    let mut query = String::from(
        "SELECT *, employee.full_name AS employee_name, employee.email AS employee_email FROM leave_request WHERE (is_archived = false OR is_archived = NONE)"
    );

    if let Some(_) = status {
        query.push_str(" AND status = $status");
    }
    if let Some(_) = scope_account_id {
        query.push_str(" AND employee = type::thing('account', $uid)");
    }
    if let Some(_) = org_id {
        query.push_str(" AND employee.organization = type::record($org)");
    }
    query.push_str(" ORDER BY created_at DESC");

    let leaves: Vec<LeaveRequest> = state.db
        .query(&query)
        .bind(("status", status.unwrap_or_default().to_string()))
        .bind(("uid", scope_account_id.unwrap_or_default().to_string()))
        .bind(("org", org_id.unwrap_or_default().to_string()))
        .await?.take(0)?;
    Ok(leaves)
}

pub async fn get_leave_requests_by_employee(
    state: &AppState,
    employee_id: &str,
) -> Result<Vec<LeaveRequest>, DbError> {
    let id = employee_id.to_string();
    let leaves: Vec<LeaveRequest> = state.db
        .query("SELECT * FROM leave_request WHERE employee = type::thing('account', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY created_at DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(leaves)
}

pub async fn approve_leave(
    state: &AppState,
    id: &str,
    req: ApproveLeaveRequest,
) -> Result<LeaveRequest, DbError> {
    let clean_id = id.strip_prefix("leave_request:").unwrap_or(id).to_string();
    let approved_by = req.approved_by_id;
    let leave: Option<LeaveRequest> = state
        .db
        .query("UPDATE type::thing('leave_request', $id) SET status = 'approved', approved_by = type::thing('account', $by)")
        .bind(("id", clean_id))
        .bind(("by", approved_by))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    leave.ok_or(DbError::NotFound)
}

pub async fn reject_leave(
    state: &AppState,
    id: &str,
    _req: RejectLeaveRequest,
) -> Result<LeaveRequest, DbError> {
    let clean_id = id.strip_prefix("leave_request:").unwrap_or(id).to_string();
    let leave: Option<LeaveRequest> = state
        .db
        .query("UPDATE type::thing('leave_request', $id) SET status = 'rejected'")
        .bind(("id", clean_id))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
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
        .query("SELECT * FROM leave_balance WHERE employee = type::thing('account', $id) AND year = $year LIMIT 1")
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
