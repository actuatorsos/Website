//! Payroll Advanced Repository — عمليات الرواتب في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};
use f64;

pub async fn get_employee_salary(
    state: &AppState,
    employee_id: &str,
) -> Result<Option<Salary>, DbError> {
    let id = employee_id.to_string();
    let salary: Option<Salary> = state.db
        .query("SELECT * FROM salary WHERE employee = type::thing('employee', $id) ORDER BY created_at DESC LIMIT 1")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(salary)
}

pub async fn update_salary(
    state: &AppState,
    employee_id: &str,
    req: UpdateSalaryRequest,
) -> Result<Salary, DbError> {
    let emp_id = employee_id.to_string();
    let basic = req.basic.unwrap_or(0.0);
    let housing = req.housing.unwrap_or(0.0);
    let transport = req.transport.unwrap_or(0.0);
    let phone = req.phone.unwrap_or(0.0);
    let other = req.other_allowances.unwrap_or(0.0);
    let total = basic + housing + transport + phone + other;
    let eff_date = req.effective_date;

    let salary: Option<Salary> = state
        .db
        .query(
            "UPSERT salary SET \
             employee = type::thing('employee', $emp_id), \
             basic = $basic, housing = $housing, transport = $transport, \
             phone = $phone, other_allowances = $other, total = $total, \
             effective_date = $eff_date",
        )
        .bind(("emp_id", emp_id))
        .bind(("basic", basic))
        .bind(("housing", housing))
        .bind(("transport", transport))
        .bind(("phone", phone))
        .bind(("other", other))
        .bind(("total", total))
        .bind(("eff_date", eff_date))
        .await?
        .take(0)?;
    salary.ok_or(DbError::NotFound)
}

pub async fn generate_payroll(
    state: &AppState,
    req: GeneratePayrollRequest,
) -> Result<Vec<PayrollRecord>, DbError> {
    let month = req.month;
    let employee_ids = req.employee_ids;

    let query = match &employee_ids {
        Some(ids) if !ids.is_empty() => {
            let id_list = ids.iter()
                .map(|id| format!("employee:{}", id))
                .collect::<Vec<_>>()
                .join(", ");
            format!("SELECT * FROM employee WHERE id IN [{}] AND (is_archived = false OR is_archived = NONE)", id_list)
        }
        _ => "SELECT * FROM employee WHERE (is_archived = false OR is_archived = NONE) AND status = 'active'".to_string(),
    };

    let employees: Vec<serde_json::Value> = state.db.query(query).await?.take(0)?;

    let mut records = Vec::new();
    for emp in &employees {
        let emp_id = emp
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if emp_id.is_empty() {
            continue;
        }

        let salary: Option<Salary> = state.db
            .query("SELECT * FROM salary WHERE employee = type::thing('employee', $id) ORDER BY created_at DESC LIMIT 1")
            .bind(("id", emp_id.clone()))
            .await?.take(0)?;

        let (basic, housing, transport, other) = salary
            .map(|s| {
                (
                    s.basic.unwrap_or(0.0),
                    s.housing.unwrap_or(0.0),
                    s.transport.unwrap_or(0.0),
                    s.other_allowances.unwrap_or(0.0),
                )
            })
            .unwrap_or((0.0, 0.0, 0.0, 0.0));

        let gross = basic + housing + transport + other;
        let gosi = basic * 0.1;
        let total_deductions = gosi;
        let net = gross - total_deductions;
        let month_clone = month.clone();

        let record: Option<PayrollRecord> = state
            .db
            .query(
                "CREATE payroll SET \
                 employee = type::thing('employee', $emp_id), \
                 month = $month, basic = $basic, housing = $housing, \
                 transport = $transport, other_allowances = $other, \
                 gross = $gross, gosi_deduction = $gosi, \
                 total_deductions = $total_ded, net = $net, \
                 status = 'draft'",
            )
            .bind(("emp_id", emp_id))
            .bind(("month", month_clone))
            .bind(("basic", basic))
            .bind(("housing", housing))
            .bind(("transport", transport))
            .bind(("other", other))
            .bind(("gross", gross))
            .bind(("gosi", gosi))
            .bind(("total_ded", total_deductions))
            .bind(("net", net))
            .await?
            .take(0)?;

        if let Some(r) = record {
            records.push(r);
        }
    }

    Ok(records)
}

pub async fn get_payroll_by_month(
    state: &AppState,
    month: &str,
) -> Result<Vec<PayrollRecord>, DbError> {
    let month = month.to_string();
    let records: Vec<PayrollRecord> = state.db
        .query("SELECT * FROM payroll WHERE month = $month AND (is_archived = false OR is_archived = NONE)")
        .bind(("month", month))
        .await?.take(0)?;
    Ok(records)
}

pub async fn approve_payroll(state: &AppState, id: &str) -> Result<PayrollRecord, DbError> {
    let id = id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let record: Option<PayrollRecord> = state
        .db
        .update(("payroll", id))
        .merge(serde_json::json!({ "status": "approved", "payment_date": now }))
        .await?;
    record.ok_or(DbError::NotFound)
}
