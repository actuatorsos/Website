//! Training Repository — عمليات التدريب في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn create_training_program(
    state: &AppState,
    req: CreateTrainingProgramRequest,
) -> Result<TrainingProgram, DbError> {
    let title = req.title;
    let ptype = req.program_type;
    let cat = req.category;
    let provider = req.provider;
    let desc = req.description;
    let loc = req.location;
    let start_date = req.start_date;
    let end_date = req.end_date;
    let hours = req.total_hours;
    let max = req.max_participants;
    let cost = req.cost;
    let cert = req.certificate_provided.unwrap_or(false);

    let prog: Option<TrainingProgram> = state
        .db
        .query(
            "CREATE training_program SET \
             title = $title, program_type = $ptype, category = $cat, \
             provider = $provider, description = $desc, location = $loc, \
             start_date = $start_date, end_date = $end_date, \
             total_hours = $hours, max_participants = $max, \
             cost = $cost, certificate_provided = $cert, \
             status = 'planned'",
        )
        .bind(("title", title))
        .bind(("ptype", ptype))
        .bind(("cat", cat))
        .bind(("provider", provider))
        .bind(("desc", desc))
        .bind(("loc", loc))
        .bind(("start_date", start_date))
        .bind(("end_date", end_date))
        .bind(("hours", hours))
        .bind(("max", max))
        .bind(("cost", cost))
        .bind(("cert", cert))
        .await?
        .take(0)?;
    prog.ok_or(DbError::NotFound)
}

pub async fn get_all_training_programs(state: &AppState) -> Result<Vec<TrainingProgram>, DbError> {
    let programs: Vec<TrainingProgram> = state.db
        .query("SELECT * FROM training_program WHERE is_archived = false OR is_archived = NONE ORDER BY start_date DESC")
        .await?.take(0)?;
    Ok(programs)
}

pub async fn enroll_employee(
    state: &AppState,
    req: EnrollRequest,
) -> Result<TrainingEnrollment, DbError> {
    let emp_id = req.employee_id;
    let prog_id = req.program_id;
    let notes = req.notes;

    let enr: Option<TrainingEnrollment> = state
        .db
        .query(
            "CREATE training_enrollment SET \
             employee = type::thing('employee', $emp_id), \
             program = type::thing('training_program', $prog_id), \
             enrolled_at = time::now(), \
             status = 'enrolled', notes = $notes",
        )
        .bind(("emp_id", emp_id))
        .bind(("prog_id", prog_id))
        .bind(("notes", notes))
        .await?
        .take(0)?;
    enr.ok_or(DbError::NotFound)
}

pub async fn complete_enrollment(
    state: &AppState,
    id: &str,
    req: CompleteEnrollmentRequest,
) -> Result<TrainingEnrollment, DbError> {
    let id = id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let enr: Option<TrainingEnrollment> = state
        .db
        .update(("training_enrollment", id))
        .merge(serde_json::json!({
            "status": "completed",
            "score": req.score,
            "certificate_number": req.certificate_number,
            "completed_at": now,
            "notes": req.notes,
        }))
        .await?;
    enr.ok_or(DbError::NotFound)
}

pub async fn get_all_enrollments(state: &AppState) -> Result<Vec<TrainingEnrollment>, DbError> {
    let enrs: Vec<TrainingEnrollment> = state.db
        .query("SELECT * FROM training_enrollment WHERE is_archived = false OR is_archived = NONE ORDER BY enrolled_at DESC")
        .await?.take(0)?;
    Ok(enrs)
}

pub async fn get_employee_enrollments(
    state: &AppState,
    employee_id: &str,
) -> Result<Vec<TrainingEnrollment>, DbError> {
    let id = employee_id.to_string();
    let enrs: Vec<TrainingEnrollment> = state.db
        .query("SELECT * FROM training_enrollment WHERE employee = type::thing('employee', $id) AND (is_archived = false OR is_archived = NONE)")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(enrs)
}
