//! HR Compliance Repository — عمليات الإنذارات ونهاية الخدمة

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_warning(
    state: &AppState,
    req: CreateWarningRequest,
) -> Result<Warning, DbError> {
    let emp_id = req.employee_id;
    let wtype = req.warning_type.unwrap_or_else(|| "verbal".to_string());
    let desc = req.description.unwrap_or_default();
    let violation = req.violation.unwrap_or_else(|| desc.clone());
    let deduction = req.deduction_amount.unwrap_or(0.0);
    let suspension = req.suspension_days.unwrap_or(0);

    let w: Option<Warning> = state
        .db
        .query(
            "CREATE warning SET \
             employee = type::thing('employee', $emp_id), \
             issued_by = (SELECT VALUE id FROM account LIMIT 1)[0], \
             warning_type = $wtype, violation = $violation, \
             description = $desc, deduction_amount = $deduction, \
             deduction_days = 0, \
             suspension_days = $suspension, \
             date = time::now(), \
             employee_ack = false",
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

pub async fn get_all_warnings(state: &AppState, org_id: Option<&str>) -> Result<Vec<Warning>, DbError> {
    let warnings: Vec<Warning> = if let Some(org) = org_id {
        state
            .db
            .query("SELECT * FROM warning WHERE (is_archived = false OR is_archived = NONE) AND organization = type::record($org) ORDER BY created_at DESC")
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query("SELECT * FROM warning WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
            .await?
            .take(0)?
    };
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
    let end_date = req.end_date;
    let termination_type = req.termination_type;

    // Get hire_date and base_salary from employee table
    let emp: Option<serde_json::Value> = state
        .db
        .query(
            "SELECT hire_date, base_salary FROM type::thing('employee', $id)",
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
        .unwrap_or(0.0);

    let hire_date = chrono::NaiveDate::parse_from_str(&hire_date_str, "%Y-%m-%d")
        .unwrap_or(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    let term_date = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .unwrap_or(chrono::Utc::now().date_naive());

    let total_days = (term_date - hire_date).num_days();
    let total_years = (total_days as f64) / 365.0;

    let five_years = 5.0;
    let half_month = base_salary / 2.0;
    let full_month = base_salary;

    let half_month_years = if total_years <= five_years { total_years } else { five_years };
    let full_month_years = if total_years > five_years { total_years - five_years } else { 0.0 };

    let first_five = half_month_years * half_month;
    let remaining = full_month_years * full_month;

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
    let leave_balance_payout = daily_rate * (leave_days as f64);
    let deductions = req.deductions.unwrap_or(0.0);
    let net = gross + leave_balance_payout - deductions;

    let eos: Option<EndOfService> = state
        .db
        .query(
            "CREATE end_of_service SET \
             employee = type::thing('employee', $emp_id), \
             termination_type = $term_type, \
             hire_date = <datetime>$hire_date, \
             end_date = <datetime>$end_date, \
             total_years = $total_years, \
             last_salary = $base_salary, \
             half_month_years = $half_month_years, \
             full_month_years = $full_month_years, \
             gross_amount = $gross, \
             deductions = $deductions, \
             net_amount = $net, \
             leave_balance_payout = $leave_payout, \
             total_payout = $net, \
             status = 'calculated'",
        )
        .bind(("emp_id", emp_id))
        .bind(("hire_date", hire_date_str))
        .bind(("end_date", end_date))
        .bind(("term_type", termination_type))
        .bind(("total_years", total_years))
        .bind(("base_salary", base_salary))
        .bind(("half_month_years", half_month_years))
        .bind(("full_month_years", full_month_years))
        .bind(("gross", gross))
        .bind(("deductions", deductions))
        .bind(("net", net))
        .bind(("leave_payout", leave_balance_payout))
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
    let reason = req.reason.unwrap_or_else(|| "عمل إضافي".to_string());

    // Get base salary directly from employee record
    let emp_data: Option<serde_json::Value> = state
        .db
        .query(
            "SELECT base_salary FROM type::thing('employee', $id)",
        )
        .bind(("id", emp_id.clone()))
        .await?
        .take(0)?;

    let monthly = emp_data
        .and_then(|e| e.get("base_salary").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);

    let hourly = monthly / (30.0 * 8.0);
    let rate = 1.5;
    let calculated = hourly * rate * hours;

    let ot: Option<OvertimeRequest> = state
        .db
        .query(
            "CREATE overtime_request SET \
             employee = type::thing('employee', $emp_id), \
             date = <datetime>$date, hours = $hours, reason = $reason, \
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

pub async fn get_all_overtime(state: &AppState, org_id: Option<&str>) -> Result<Vec<OvertimeRequest>, DbError> {
    let ots: Vec<OvertimeRequest> = if let Some(org) = org_id {
        state
            .db
            .query("SELECT * FROM overtime_request WHERE (is_archived = false OR is_archived = NONE) AND organization = type::record($org) ORDER BY date DESC")
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query("SELECT * FROM overtime_request WHERE is_archived = false OR is_archived = NONE ORDER BY date DESC")
            .await?
            .take(0)?
    };
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
    let clean_id = id.strip_prefix("overtime_request:").unwrap_or(id).to_string();
    let ot: Option<OvertimeRequest> = state
        .db
        .query("UPDATE type::thing('overtime_request', $id) SET status = 'approved'")
        .bind(("id", clean_id))
        .await?
        .take(0)?;
    ot.ok_or(DbError::NotFound)
}
