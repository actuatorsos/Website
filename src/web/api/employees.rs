//! Employees API Endpoints
//!
//! نقاط نهاية API للموظفين

use axum::{
    Json, Router,
    extract::{Extension, Path, State},
    routing::{delete, post},
};
use validator::Validate;

use crate::db::{self, AppState};
use crate::domains::hr::models::{CreateEmployeeRequest, Employee};
use crate::domains::hr::repository;
use crate::models::CurrentUser;

// ============================================================================
// Form Data
// ============================================================================

#[derive(serde::Deserialize, Validate)]
pub struct CreateEmployeeJson {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    #[validate(length(min = 5, max = 50))]
    pub phone: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub role: String,
    #[validate(length(max = 50))]
    pub national_id: Option<String>,
    pub hire_date: Option<String>,

    // --- Extended HR fields ---
    pub nationality: Option<String>,
    pub religion: Option<String>,
    pub marital_status: Option<String>,
    pub dependents: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_iban: Option<String>,
    pub emergency_name: Option<String>,
    pub emergency_phone: Option<String>,
    pub emergency_relation: Option<String>,
    pub base_salary: Option<f64>,
    pub housing_allowance: Option<f64>,
    pub transport_allowance: Option<f64>,
    pub employment_type: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_role(role: &str) -> String {
    if role.is_empty() { "other".to_string() } else { role.to_lowercase() }
}

// ============================================================================
// Handlers
// ============================================================================

async fn create_employee(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(payload): Json<CreateEmployeeJson>,
) -> axum::response::Result<Json<Employee>, crate::db::DbError> {
    if let Err(e) = payload.validate() {
        return Err(crate::db::DbError::Validation(e.to_string()));
    }

    // Default hire_date to today if not provided
    let hire_date = payload
        .hire_date
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let base_salary = payload.base_salary;
    let housing_allowance = payload.housing_allowance;
    let transport_allowance = payload.transport_allowance;
    let employment_type_val = payload.employment_type.clone();
    let request = CreateEmployeeRequest {
        name: payload.name,
        phone: payload.phone,
        email: payload.email,
        role: parse_role(&payload.role),
        national_id: payload.national_id,
        hire_date: hire_date.clone(),
        nationality: payload.nationality,
        religion: payload.religion,
        marital_status: payload.marital_status,
        dependents: payload.dependents,
        bank_name: payload.bank_name,
        bank_iban: payload.bank_iban,
        emergency_name: payload.emergency_name,
        emergency_phone: payload.emergency_phone,
        emergency_relation: payload.emergency_relation,
        base_salary,
        housing_allowance,
        transport_allowance,
        employment_type: payload.employment_type,
    };

    let employee = repository::create_employee(&state, request).await?;
    let emp_id_raw = employee
        .id
        .as_ref()
        .map(|t| crate::db::record_id_to_raw(t))
        .unwrap_or_default();

    // ════════════════════════════════════════════════════════════════
    // Hire-to-Pay Workflow — سير عمل التوظيف حتى الراتب
    // Each sub-step is non-blocking: failures are logged, not propagated.
    // ════════════════════════════════════════════════════════════════

    let mut workflow_errors: Vec<String> = Vec::new();

    // ── 1. Auto-create salary record ────────────────────────────────
    if let Some(salary_amount) = base_salary {
        if salary_amount > 0.0 {
            let salary_res: Result<Option<serde_json::Value>, _> = state
                .db
                .query(
                    "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $eid)); \
                     LET $acc = (SELECT VALUE id FROM account WHERE email = $emp_email[0]); \
                     IF $acc[0] != NONE THEN \
                         (CREATE salary SET \
                             employee = $acc[0], \
                             base_amount = $base, \
                             currency = 'SAR', \
                             effective_from = <datetime>$eff_from, \
                             notes = 'Auto-created on hire') \
                     ELSE \
                         (CREATE salary SET \
                             employee = type::thing('employee', $eid), \
                             base_amount = $base, \
                             currency = 'SAR', \
                             effective_from = <datetime>$eff_from, \
                             notes = 'Auto-created on hire (no account linked)') \
                     END",
                )
                .bind(("eid", emp_id_raw.clone()))
                .bind(("base", salary_amount))
                .bind(("eff_from", hire_date.clone()))
                .await
                .and_then(|mut r| r.take(2));

            if let Err(e) = salary_res {
                let msg = format!("Salary creation failed for employee {}: {}", emp_id_raw, e);
                tracing::warn!("{}", msg);
                workflow_errors.push(msg);
            } else {
                tracing::info!("Salary record auto-created for employee {}", emp_id_raw);
            }
        }
    }

    // ── 2. Auto-create employment contract ──────────────────────────
    {
        let emp_type = employment_type_val
            .as_deref()
            .unwrap_or("full_time");
        let contract_res: Result<Option<serde_json::Value>, _> = state
            .db
            .query(
                "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $eid)); \
                 LET $acc = (SELECT VALUE id FROM account WHERE email = $emp_email[0]); \
                 LET $account_id = IF $acc[0] != NONE THEN $acc[0] ELSE type::thing('employee', $eid) END; \
                 CREATE employment_contract SET \
                     employee = $account_id, \
                     position = 'pending_assignment', \
                     department = 'pending_assignment', \
                     employment_type = $emp_type, \
                     start_date = <datetime>$start, \
                     weekly_hours = 48, \
                     work_location = 'onsite', \
                     is_active = true",
            )
            .bind(("eid", emp_id_raw.clone()))
            .bind(("emp_type", emp_type.to_string()))
            .bind(("start", hire_date.clone()))
            .await
            .and_then(|mut r| r.take(3));

        if let Err(e) = contract_res {
            let msg = format!(
                "Employment contract creation failed for employee {}: {}",
                emp_id_raw, e
            );
            tracing::warn!("{}", msg);
            workflow_errors.push(msg);
        } else {
            tracing::info!(
                "Employment contract auto-created for employee {}",
                emp_id_raw
            );
        }
    }

    // ── 3. Auto-initialize leave balance ────────────────────────────
    {
        let current_year = chrono::Utc::now().format("%Y").to_string();
        let leave_res: Result<Option<serde_json::Value>, _> = state
            .db
            .query(
                "LET $emp_email = (SELECT VALUE email FROM type::thing('employee', $eid)); \
                 LET $acc = (SELECT VALUE id FROM account WHERE email = $emp_email[0]); \
                 LET $account_id = IF $acc[0] != NONE THEN $acc[0] ELSE type::thing('employee', $eid) END; \
                 CREATE leave_balance SET \
                     employee = $account_id, \
                     year = <int>$year, \
                     annual_total = 21, \
                     annual_used = 0, \
                     sick_total = 14, \
                     sick_used = 0, \
                     emergency_total = 5, \
                     emergency_used = 0, \
                     hajj_total = 15, \
                     hajj_used = 0, \
                     maternity_total = 70, \
                     maternity_used = 0, \
                     paternity_total = 3, \
                     paternity_used = 0, \
                     marriage_total = 5, \
                     marriage_used = 0, \
                     bereavement_total = 5, \
                     bereavement_used = 0, \
                     carry_over = 0",
            )
            .bind(("eid", emp_id_raw.clone()))
            .bind(("year", current_year.clone()))
            .await
            .and_then(|mut r| r.take(3));

        if let Err(e) = leave_res {
            let msg = format!(
                "Leave balance initialization failed for employee {}: {}",
                emp_id_raw, e
            );
            tracing::warn!("{}", msg);
            workflow_errors.push(msg);
        } else {
            tracing::info!(
                "Leave balance initialized for employee {} (year {})",
                emp_id_raw,
                current_year
            );
        }
    }

    // ── 4. Audit log — full hire event ──────────────────────────────
    let hire_summary = serde_json::json!({
        "action": "hire_workflow",
        "employee": serde_json::to_value(&employee).ok(),
        "sub_steps": {
            "salary_created": !workflow_errors.iter().any(|e| e.contains("Salary")),
            "contract_created": !workflow_errors.iter().any(|e| e.contains("contract")),
            "leave_balance_initialized": !workflow_errors.iter().any(|e| e.contains("Leave")),
        },
        "errors": &workflow_errors,
    });

    let _ = db::audit_log(
        &state.db,
        Some(&user.email),
        "hire",
        "employee",
        Some(&emp_id_raw),
        None,
        Some(hire_summary),
    )
    .await;

    // Broadcast to WebSocket clients
    let _ = state.notification_tx.send(serde_json::json!({
        "type": "notification",
        "action": "employee_hired",
        "data": {
            "id": &emp_id_raw,
            "name": &employee.name,
            "workflow_errors": &workflow_errors,
        }
    }).to_string());

    Ok(Json(employee))
}

async fn list_employees(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> axum::response::Result<Json<Vec<Employee>>, crate::db::DbError> {
    let employees: Vec<Employee> = repository::get_all_employees(&state, user.organization_id.as_deref()).await?;
    Ok(Json(employees))
}

async fn delete_employee(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> axum::response::Result<Json<serde_json::Value>, crate::db::DbError> {
    repository::delete_employee(&state, &id).await?;
    let _ = db::audit_log(
        &state.db,
        Some(&user.email),
        "delete",
        "employee",
        Some(&id),
        None,
        None,
    )
    .await;

    // Broadcast to WebSocket clients
    let _ = state.notification_tx.send(serde_json::json!({
        "type": "notification",
        "action": "employee_deleted",
        "data": { "id": &id }
    }).to_string());

    Ok(Json(serde_json::json!({ "success": true })))
}

// ============================================================================
// Routes
// ============================================================================

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_employee).get(list_employees))
        .route("/{id}", delete(delete_employee))
}
