//! Machinery Domain Database Logic
//!
//! فصل العمليات المتعلقة بقواعد البيانات للآلات والمشاريع وعمليات الإصلاح

use super::models::{
    CreateMachineRequest, CreateProjectRequest, CreateRepairRequest, Machine, MachineStatus,
    Project, ProjectStatus, RepairOperation, RepairStatus, UpdateRepairRequest,
};
use crate::db::{AppState, DbError};

// ============================================================================
// Machine Repository
// ============================================================================

pub async fn create_machine(
    state: &AppState,
    req: CreateMachineRequest,
) -> Result<Machine, DbError> {
    let machine: Option<Machine> = state
        .db
        .create::<Option<Machine>>("machine")
        .content(Machine {
            id: None,
            created_at: None,
            customer_id: req.customer_id,
            customer_name: None,
            serial_number: req.serial_number,
            model: req.model,
            manufacturer: req.manufacturer,
            purchase_date: req.purchase_date,
            status: MachineStatus::Working,
        })
        .await?
        .into_iter()
        .next();
    let created = machine.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "machine",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_machines(state: &AppState) -> Result<Vec<Machine>, DbError> {
    let machines: Vec<Machine> = state.db
        .query("SELECT * FROM machine WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(machines)
}

pub async fn get_customer_machines(
    state: &AppState,
    customer_id: &str,
) -> Result<Vec<Machine>, DbError> {
    let cid = customer_id.to_string();
    let machines: Vec<Machine> = state.db
        .query("SELECT * FROM machine WHERE customer_id = $cid AND (is_archived = false OR is_archived = NONE)")
        .bind(("cid", cid))
        .await?
        .take::<Vec<Machine>>(0)?;
    Ok(machines)
}

pub async fn get_machine(state: &AppState, id: &str) -> Result<Machine, DbError> {
    let machine: Option<Machine> = state.db.select(("machine", id)).await?;
    machine.ok_or(DbError::NotFound)
}

pub async fn update_machine_status(
    state: &AppState,
    id: &str,
    status: MachineStatus,
) -> Result<Machine, DbError> {
    let machine: Option<Machine> = state
        .db
        .update(("machine", id))
        .merge(serde_json::json!({ "status": status }))
        .await?;
    machine.ok_or(DbError::NotFound)
}

// ============================================================================
// Project Repository
// ============================================================================

pub async fn create_project(
    state: &AppState,
    req: CreateProjectRequest,
) -> Result<Project, DbError> {
    let project: Option<Project> = state
        .db
        .create::<Option<Project>>("project")
        .content(Project {
            id: None,
            created_at: None,
            customer_id: req.customer_id,
            customer_name: None,
            title: req.title,
            description: req.description,
            start_date: req.start_date,
            end_date: req.end_date,
            status: ProjectStatus::New,
            budget: req.budget,
        })
        .await?
        .into_iter()
        .next();
    let created = project.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "project",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_projects(state: &AppState) -> Result<Vec<Project>, DbError> {
    let projects: Vec<Project> = state.db
        .query("SELECT * FROM project WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(projects)
}

pub async fn get_project(state: &AppState, id: &str) -> Result<Project, DbError> {
    let project: Option<Project> = state.db.select(("project", id)).await?;
    project.ok_or(DbError::NotFound)
}

pub async fn update_project_status(
    state: &AppState,
    id: &str,
    status: ProjectStatus,
) -> Result<Project, DbError> {
    let project: Option<Project> = state
        .db
        .update(("project", id))
        .merge(serde_json::json!({ "status": status }))
        .await?;
    project.ok_or(DbError::NotFound)
}

// ============================================================================
// Repair Operation Repository
// ============================================================================

pub async fn create_repair(
    state: &AppState,
    req: CreateRepairRequest,
) -> Result<RepairOperation, DbError> {
    let now = chrono::Utc::now().to_rfc3339();
    let repair: Option<RepairOperation> = state
        .db
        .create::<Option<RepairOperation>>("repair_operation")
        .content(RepairOperation {
            id: None,
            created_at: None,
            machine_id: req.machine_id,
            machine_serial: None,
            project_id: req.project_id,
            project_title: None,
            employee_id: req.employee_id,
            employee_name: None,
            customer_name: None,
            description: req.description,
            diagnosis: req.diagnosis,
            parts_used: None,
            cost: None,
            start_time: now,
            end_time: None,
            status: RepairStatus::New,
        })
        .await?
        .into_iter()
        .next();
    let created = repair.ok_or(DbError::NotFound)?;
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "repair_operation",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_repairs(state: &AppState) -> Result<Vec<RepairOperation>, DbError> {
    let repairs: Vec<RepairOperation> = state.db
        .query("SELECT * FROM repair_operation WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(repairs)
}

pub async fn get_repair(state: &AppState, id: &str) -> Result<RepairOperation, DbError> {
    let repair: Option<RepairOperation> = state.db.select(("repair_operation", id)).await?;
    repair.ok_or(DbError::NotFound)
}

pub async fn update_repair(
    state: &AppState,
    id: &str,
    req: UpdateRepairRequest,
) -> Result<RepairOperation, DbError> {
    let mut update = serde_json::json!({ "status": req.status });
    if let Some(d) = req.diagnosis {
        update["diagnosis"] = serde_json::json!(d);
    }
    if let Some(p) = req.parts_used {
        update["parts_used"] = serde_json::json!(p);
    }
    if let Some(c) = req.cost {
        update["cost"] = serde_json::json!(c);
    }
    if req.status == RepairStatus::Completed {
        update["end_time"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    }

    let repair: Option<RepairOperation> = state
        .db
        .update(("repair_operation", id))
        .merge(update)
        .await?;
    repair.ok_or(DbError::NotFound)
}

pub async fn get_machine_repairs(
    state: &AppState,
    machine_id: &str,
) -> Result<Vec<RepairOperation>, DbError> {
    let mid = machine_id.to_string();
    let repairs: Vec<RepairOperation> = state.db
        .query("SELECT * FROM repair_operation WHERE machine_id = $mid AND (is_archived = false OR is_archived = NONE) ORDER BY start_time DESC")
        .bind(("mid", mid))
        .await?
        .take::<Vec<RepairOperation>>(0)?;
    Ok(repairs)
}
