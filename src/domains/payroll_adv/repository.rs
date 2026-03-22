//! Payroll Advanced Repository — عمليات الرواتب في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};
use surrealdb::types::{RecordId, SurrealValue};

/// Minimal employee struct for payroll generation
#[derive(Debug, Clone, serde::Deserialize, SurrealValue)]
struct EmployeeRef {
    pub id: Option<RecordId>,
    pub name: Option<String>,
    pub email: Option<String>,
}

pub async fn get_employee_salary(
    state: &AppState,
    employee_id: &str,
) -> Result<Option<Salary>, DbError> {
    // Look up employee email, then find salary via linked account
    let salary: Option<Salary> = state
        .db
        .query(
            "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $id)); \
             SELECT * FROM salary WHERE employee IN \
             (SELECT VALUE id FROM account WHERE email = $emp_email[0]) \
             ORDER BY effective_from DESC LIMIT 1",
        )
        .bind(("id", employee_id.to_string()))
        .await
        .map_err(DbError::Database)?
        .take(1)
        .map_err(DbError::Database)?;
    Ok(salary)
}

pub async fn update_salary(
    state: &AppState,
    employee_id: &str,
    req: UpdateSalaryRequest,
) -> Result<Salary, DbError> {
    let base = req.base_amount.unwrap_or(0.0);
    let currency = req.currency.unwrap_or_else(|| "SYP".to_string());
    let eff_from = req.effective_from;

    // Find the account linked to this employee
    let salary: Option<Salary> = state
        .db
        .query(
            "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $eid)); \
             LET $acc = (SELECT VALUE id FROM account WHERE email = $emp_email[0]); \
             UPSERT salary SET \
             employee = $acc[0], \
             base_amount = $base, \
             currency = $currency, \
             effective_from = <datetime>$eff_from",
        )
        .bind(("eid", employee_id.to_string()))
        .bind(("base", base))
        .bind(("currency", currency))
        .bind(("eff_from", eff_from.unwrap_or_default()))
        .await
        .map_err(DbError::Database)?
        .take(2)
        .map_err(DbError::Database)?;
    salary.ok_or(DbError::NotFound)
}

pub async fn generate_payroll(
    state: &AppState,
    req: GeneratePayrollRequest,
) -> Result<Vec<PayrollRecord>, DbError> {
    let month = req.month;

    // Get all active employees
    let employees: Vec<EmployeeRef> = state
        .db
        .query(
            "SELECT id, name, email FROM employee WHERE status = 'active' AND (is_archived = false OR is_archived = NONE)",
        )
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;

    let mut records = Vec::new();
    for emp in &employees {
        let _emp_id_str = match &emp.id {
            Some(thing) => crate::db::record_id_to_raw(thing),
            None => continue,
        };
        let _emp_name = emp.name.clone().unwrap_or_default();
        let emp_email = emp.email.clone().unwrap_or_default();

        if emp_email.is_empty() {
            continue;
        }

        // Find salary via account email link
        let salary: Option<Salary> = state
            .db
            .query(
                "SELECT * FROM salary WHERE employee IN \
                 (SELECT VALUE id FROM account WHERE email = $email) \
                 ORDER BY effective_from DESC LIMIT 1",
            )
            .bind(("email", emp_email.clone()))
            .await
            .map_err(DbError::Database)?
            .take(0)
            .map_err(DbError::Database)?;

        let base = salary.as_ref().and_then(|s| s.base_amount).unwrap_or(0.0);

        // Simple payroll calculation
        let housing = base * 0.25; // 25% housing allowance
        let transport = base * 0.1; // 10% transport
        let gross = base + housing + transport;
        let gosi = base * 0.0975; // 9.75% GOSI employee share
        let total_deductions = gosi;
        let net = gross - total_deductions;

        // Parse month string "YYYY-MM" into year and month
        let parts: Vec<&str> = month.split('-').collect();
        let p_year: i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(2025);
        let p_month: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

        // Find the account ID for this employee (payroll.employee = record<account>)
        let acc_result: Option<EmployeeRef> = state
            .db
            .query("SELECT id FROM account WHERE email = $email LIMIT 1")
            .bind(("email", emp_email.clone()))
            .await
            .map_err(DbError::Database)?
            .take(0)
            .map_err(DbError::Database)?;

        let acc_id = match acc_result.and_then(|a| a.id) {
            Some(thing) => crate::db::record_id_to_raw(&thing),
            None => continue,
        };

        // Create payroll record — using SCHEMA field names
        let record: Option<PayrollRecord> = state
            .db
            .query(
                "CREATE payroll SET \
                 employee = type::thing('account', $acc_id), \
                 period_month = $p_month, \
                 period_year = $p_year, \
                 base_salary = $basic, \
                 housing_allowance = $housing, \
                 transport_allowance = $transport, \
                 other_allowances = 0.0, \
                 gosi_employee = $gosi, \
                 gosi_employer = $gosi_er, \
                 deductions = $total_ded, \
                 net_salary = $net, \
                 status = 'draft'",
            )
            .bind(("acc_id", acc_id))
            .bind(("p_month", p_month))
            .bind(("p_year", p_year))
            .bind(("basic", base))
            .bind(("housing", housing))
            .bind(("transport", transport))
            .bind(("gosi", gosi))
            .bind(("gosi_er", base * 0.12))
            .bind(("total_ded", total_deductions))
            .bind(("net", net))
            .await
            .map_err(DbError::Database)?
            .take(0)
            .map_err(DbError::Database)?;

        if let Some(r) = record {
            records.push(r);
        }
    }

    Ok(records)
}

pub async fn get_payroll_by_month(
    state: &AppState,
    month: &str,
    scope_account_id: Option<&str>,
) -> Result<Vec<PayrollRecord>, DbError> {
    // Parse "YYYY-MM" into year and month
    let parts: Vec<&str> = month.split('-').collect();
    let p_year: i32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(2025);
    let p_month: i32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);

    let records: Vec<PayrollRecord> = match scope_account_id {
        Some(uid) => {
            state
                .db
                .query(
                    "SELECT * FROM payroll WHERE period_year = $p_year AND period_month = $p_month \
                     AND (is_archived = false OR is_archived = NONE) \
                     AND employee = type::thing('account', $uid)",
                )
                .bind(("p_year", p_year))
                .bind(("p_month", p_month))
                .bind(("uid", uid.to_string()))
                .await
                .map_err(DbError::Database)?
                .take(0)
                .map_err(DbError::Database)?
        }
        None => {
            state
                .db
                .query(
                    "SELECT *, employee.full_name AS employee_name FROM payroll \
                     WHERE period_year = $p_year AND period_month = $p_month \
                     AND (is_archived = false OR is_archived = NONE)",
                )
                .bind(("p_year", p_year))
                .bind(("p_month", p_month))
                .await
                .map_err(DbError::Database)?
                .take(0)
                .map_err(DbError::Database)?
        }
    };
    Ok(records)
}

pub async fn approve_payroll(state: &AppState, id: &str) -> Result<PayrollRecord, DbError> {
    let clean_id = id.strip_prefix("payroll:").unwrap_or(id).to_string();
    let record: Option<PayrollRecord> = state
        .db
        .query("UPDATE type::thing('payroll', $id) SET status = 'approved', payment_date = time::now()")
        .bind(("id", clean_id))
        .await
        .map_err(DbError::Database)?
        .take(0)
        .map_err(DbError::Database)?;
    record.ok_or(DbError::NotFound)
}
