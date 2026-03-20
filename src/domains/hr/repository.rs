//! HR Domain Database Logic
//!
//! فصل العمليات المتعلقة بقواعد البيانات للموارد البشرية

use super::models::{
    Attendance, CheckInRequest, CreateEmployeeRequest, CreateTraineeRequest, Employee,
    EmployeeStatus, PersonType, Trainee, TraineeStatus,
};
use crate::db::{AppState, DbError};

// ============================================================================
// Employee Repository
// ============================================================================

pub async fn create_employee(
    state: &AppState,
    req: CreateEmployeeRequest,
) -> Result<Employee, DbError> {
    let emp: Option<Employee> = state
        .db
        .create::<Option<Employee>>("employee")
        .content(Employee {
            id: None,
            created_at: None,
            name: req.name,
            phone: req.phone,
            email: req.email,
            role: req.role,
            national_id: req.national_id,
            hire_date: req.hire_date,
            status: EmployeeStatus::Active,
            nationality: req.nationality,
            religion: req.religion,
            marital_status: req.marital_status,
            dependents: req.dependents,
            bank_name: req.bank_name,
            bank_iban: req.bank_iban,
            emergency_name: req.emergency_name,
            emergency_phone: req.emergency_phone,
            emergency_relation: req.emergency_relation,
            base_salary: req.base_salary,
            housing_allowance: req.housing_allowance,
            transport_allowance: req.transport_allowance,
            employment_type: req.employment_type.or(Some("full_time".to_string())),
        })
        .await?
        .into_iter()
        .next();
    let created = emp.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "employee",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_employees(state: &AppState) -> Result<Vec<Employee>, DbError> {
    let employees: Vec<Employee> = state.db
        .query("SELECT * FROM employee WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(employees)
}

pub async fn get_employee(state: &AppState, id: &str) -> Result<Employee, DbError> {
    let emp: Option<Employee> = state.db.select(("employee", id)).await?;
    emp.ok_or(DbError::NotFound)
}

/// Soft delete — أرشفة الموظف
pub async fn delete_employee(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "employee", id).await?;
    crate::db::audit_log(&state.db, None, "delete", "employee", Some(id), None, None).await?;
    Ok(())
}

// ============================================================================
// Trainee Repository
// ============================================================================

pub async fn create_trainee(
    state: &AppState,
    req: CreateTraineeRequest,
) -> Result<Trainee, DbError> {
    let trainee: Option<Trainee> = state
        .db
        .create::<Option<Trainee>>("trainee")
        .content(Trainee {
            id: None,
            created_at: None,
            name: req.name,
            phone: req.phone,
            email: req.email,
            institution: req.institution,
            start_date: req.start_date,
            end_date: req.end_date,
            status: TraineeStatus::Active,
        })
        .await?
        .into_iter()
        .next();
    let created = trainee.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "trainee",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_trainees(state: &AppState) -> Result<Vec<Trainee>, DbError> {
    let trainees: Vec<Trainee> = state.db
        .query("SELECT * FROM trainee WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(trainees)
}

pub async fn get_trainee(state: &AppState, id: &str) -> Result<Trainee, DbError> {
    let trainee: Option<Trainee> = state.db.select(("trainee", id)).await?;
    trainee.ok_or(DbError::NotFound)
}

/// Soft delete — أرشفة المتدرب
pub async fn delete_trainee(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "trainee", id).await?;
    crate::db::audit_log(&state.db, None, "delete", "trainee", Some(id), None, None).await?;
    Ok(())
}

// ============================================================================
// Attendance Repository
// ============================================================================

pub async fn check_in(state: &AppState, req: CheckInRequest) -> Result<Attendance, DbError> {
    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let attendance: Option<Attendance> = state
        .db
        .create::<Option<Attendance>>("attendance")
        .content(Attendance {
            id: None,
            person_id: req.person_id,
            person_type: req.person_type,
            person_name: req.person_name,
            check_in: now.to_rfc3339(),
            check_out: None,
            date: Some(today),
            notes: req.notes,
        })
        .await?
        .into_iter()
        .next();
    attendance.ok_or(DbError::NotFound)
}

pub async fn check_out(state: &AppState, id: &str) -> Result<Attendance, DbError> {
    let now = chrono::Utc::now().to_rfc3339();
    let attendance: Option<Attendance> = state
        .db
        .update(("attendance", id))
        .merge(serde_json::json!({ "check_out": now }))
        .await?;
    attendance.ok_or(DbError::NotFound)
}

pub async fn get_today_attendance(state: &AppState) -> Result<Vec<Attendance>, DbError> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let result: Vec<Attendance> = state
        .db
        .query("SELECT * FROM attendance WHERE date = $date ORDER BY check_in DESC")
        .bind(("date", today))
        .await?
        .take(0)?;
    Ok(result)
}

pub async fn get_attendance_by_date(
    state: &AppState,
    date: &str,
) -> Result<Vec<Attendance>, DbError> {
    let result: Vec<Attendance> = state
        .db
        .query("SELECT * FROM attendance WHERE date = $date ORDER BY check_in DESC")
        .bind(("date", date.to_string()))
        .await?
        .take(0)?;
    Ok(result)
}

/// Soft delete — أرشفة سجل الحضور
pub async fn delete_attendance(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "attendance", id).await?;
    Ok(())
}

pub async fn get_person_name(
    state: &AppState,
    person_id: &str,
    person_type: &PersonType,
) -> String {
    match person_type {
        PersonType::Employee => get_employee(state, person_id)
            .await
            .map(|e| e.name)
            .unwrap_or_else(|_| person_id.to_string()),
        PersonType::Trainee => get_trainee(state, person_id)
            .await
            .map(|t| t.name)
            .unwrap_or_else(|_| person_id.to_string()),
    }
}
