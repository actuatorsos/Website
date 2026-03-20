//! HR Compliance Repository — عمليات الإنذارات ونهاية الخدمة

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_warning(
    state: &AppState,
    req: CreateWarningRequest,
) -> Result<Warning, DbError> {
    let emp_id = req.employee_id;
    let wtype = req.warning_type.unwrap_or_else(|| "verbal".to_string());
    let violation = req.violation;
    let desc = req.description.unwrap_or_default();
    let deduction = req.deduction_amount.unwrap_or(0.0);
    let suspension = req.suspension_days.unwrap_or(0);

    let w: Option<Warning> = state
        .db
        .query(
            "CREATE warning SET \
             employee = type::thing('employee', $emp_id), \
             warning_type = $wtype, violation = $violation, \
             description = $desc, deduction_amount = $deduction, \
             suspension_days = $suspension, is_acknowledged = false",
        )
        .bind(("emp_id", emp_id))
        .bind(("wtype", wtype))
        .bind(("violation", violation))
        .bind(("desc", desc))
        .bind(("deduction", deduction))
        .bind(("suspension", suspension))
        .await?
        .take(0)?;
    w.ok_or(DbError::NotFound)
}

pub async fn get_all_warnings(state: &AppState) -> Result<Vec<Warning>, DbError> {
    let warnings: Vec<Warning> = state
        .db
        .query("SELECT * FROM warning WHERE (is_archived = false OR is_archived = NONE) ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(warnings)
}

pub async fn get_employee_warnings(
    state: &AppState,
    employee_id: &str,
) -> Result<Vec<Warning>, DbError> {
    let id = employee_id.to_string();
    let warnings: Vec<Warning> = state.db
        .query("SELECT * FROM warning WHERE employee = type::thing('employee', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY created_at DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(warnings)
}

pub async fn calculate_eos(
    state: &AppState,
    req: CalculateEosRequest,
) -> Result<EndOfService, DbError> {
    let emp_id = req.employee_id;
    let termination_date = req.termination_date;
    let termination_type = req.termination_type;

    let emp: Option<serde_json::Value> = state
        .db
        .query(
            "SELECT hire_date, base_salary FROM employee WHERE id = type::thing('employee', $id)",
        )
        .bind(("id", emp_id.clone()))
        .await?
        .take(0)?;

    let emp = emp.ok_or(DbError::NotFound)?;
    let hire_date_str = emp
        .get("hire_date")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let base_salary = emp
        .get("base_salary")
        .and_then(|v| v.as_f64())
        .map(|f| f)
        .unwrap_or(0.0);

    let hire_date = chrono::NaiveDate::parse_from_str(&hire_date_str, "%Y-%m-%d")
        .unwrap_or(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    let term_date = chrono::NaiveDate::parse_from_str(&termination_date, "%Y-%m-%d")
        .unwrap_or(chrono::Utc::now().date_naive());

    let total_days = (term_date - hire_date).num_days();
    let total_years = (total_days as f64) / 365.0;
    let total_months = total_days / 30;

    let five_years = 5.0;
    let half_month = base_salary / 2.0;
    let full_month = base_salary;

    let (first_five, remaining) = if total_years <= five_years {
        (total_years * half_month, 0.0)
    } else {
        let rem = total_years - five_years;
        (five_years * half_month, rem * full_month)
    };

    let (first_five_adj, remaining_adj) = if termination_type == "resignation" {
        if total_years < five_years {
            (first_five / 2.0, remaining)
        } else if total_years < 10.0 {
            (first_five * 0.667, remaining)
        } else {
            (first_five, remaining)
        }
    } else {
        (first_five, remaining)
    };

    let gross = first_five_adj + remaining_adj;
    let leave_days = req.annual_leave_days_remaining.unwrap_or(0);
    let daily_rate = base_salary / 30.0;
    let leave_encashment = daily_rate * (leave_days as f64);
    let other_ded = req.other_deductions.unwrap_or(0.0);
    let net = gross + leave_encashment - other_ded;

    let eos: Option<EndOfService> = state
        .db
        .query(
            "CREATE end_of_service SET \
             employee = type::thing('employee', $emp_id), \
             termination_date = $term_date, termination_type = $term_type, \
             total_years = $total_years, total_months = $total_months, \
             last_basic_salary = $base_salary, \
             first_five_years_amount = $first_five, \
             remaining_years_amount = $remaining, \
             gross_amount = $gross, \
             annual_leave_encashment = $leave_enc, \
             other_deductions = $other_ded, \
             net_amount = $net, total_payout = $net, \
             status = 'draft'",
        )
        .bind(("emp_id", emp_id))
        .bind(("term_date", termination_date))
        .bind(("term_type", termination_type))
        .bind(("total_years", total_years))
        .bind(("total_months", total_months))
        .bind(("base_salary", base_salary))
        .bind(("first_five", first_five_adj))
        .bind(("remaining", remaining_adj))
        .bind(("gross", gross))
        .bind(("leave_enc", leave_encashment))
        .bind(("other_ded", other_ded))
        .bind(("net", net))
        .await?
        .take(0)?;

    eos.ok_or(DbError::NotFound)
}

pub async fn create_overtime_request(
    state: &AppState,
    req: CreateOvertimeRequest,
) -> Result<OvertimeRequest, DbError> {
    let emp_id = req.employee_id;
    let date = req.date;
    let hours = req.hours;
    let reason = req.reason;

    let emp_salary: Option<serde_json::Value> = state
        .db
        .query("SELECT base_salary FROM employee WHERE id = type::thing('employee', $id)")
        .bind(("id", emp_id.clone()))
        .await?
        .take(0)?;

    let monthly = emp_salary
        .and_then(|e| e.get("base_salary").and_then(|v| v.as_f64()))
        .map(|f| f)
        .unwrap_or(0.0);

    let hourly = monthly / (30.0 * 8.0);
    let rate = 1.5;
    let calculated = hourly * rate * hours;

    let ot: Option<OvertimeRequest> = state
        .db
        .query(
            "CREATE overtime_request SET \
             employee = type::thing('employee', $emp_id), \
             date = $date, hours = $hours, reason = $reason, \
             rate_multiplier = $rate, calculated_amount = $calc, \
             status = 'pending'",
        )
        .bind(("emp_id", emp_id))
        .bind(("date", date))
        .bind(("hours", hours))
        .bind(("reason", reason))
        .bind(("rate", rate))
        .bind(("calc", calculated))
        .await?
        .take(0)?;
    ot.ok_or(DbError::NotFound)
}

pub async fn get_all_overtime(state: &AppState) -> Result<Vec<OvertimeRequest>, DbError> {
    let ots: Vec<OvertimeRequest> = state
        .db
        .query("SELECT * FROM overtime_request WHERE (is_archived = false OR is_archived = NONE) ORDER BY date DESC")
        .await?
        .take(0)?;
    Ok(ots)
}

pub async fn get_employee_overtime(
    state: &AppState,
    employee_id: &str,
) -> Result<Vec<OvertimeRequest>, DbError> {
    let id = employee_id.to_string();
    let ots: Vec<OvertimeRequest> = state.db
        .query("SELECT * FROM overtime_request WHERE employee = type::thing('employee', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY date DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(ots)
}

pub async fn approve_overtime(state: &AppState, id: &str) -> Result<OvertimeRequest, DbError> {
    let id = id.to_string();
    let ot: Option<OvertimeRequest> = state
        .db
        .update(("overtime_request", id))
        .merge(serde_json::json!({ "status": "approved" }))
        .await?;
    ot.ok_or(DbError::NotFound)
}
