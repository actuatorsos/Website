//! HR Org Repository — Database operations for departments and positions

use super::models::*;
use crate::db::{AppState, DbError};

// ══════════════════════════════════════════════════════════════════
// Departments
// ══════════════════════════════════════════════════════════════════

pub async fn create_department(
    state: &AppState,
    req: CreateDepartmentRequest,
) -> Result<Department, DbError> {
    let code = req.code;
    let name = req.name;
    let dept: Option<Department> = state.db
        .query("CREATE department SET code = $code, name = $name, is_active = true")
        .bind(("code", code))
        .bind(("name", name))
        .await?
        .take(0)?;
    dept.ok_or(DbError::NotFound)
}

pub async fn get_all_departments(state: &AppState) -> Result<Vec<Department>, DbError> {
    let depts: Vec<Department> = state.db
        .query("SELECT * FROM department WHERE is_archived = false OR is_archived = NONE ORDER BY code ASC")
        .await?
        .take(0)?;
    Ok(depts)
}

pub async fn get_department(state: &AppState, id: &str) -> Result<Department, DbError> {
    let id = id.to_string();
    let dept: Option<Department> = state.db.select(("department", id)).await?;
    dept.ok_or(DbError::NotFound)
}

pub async fn update_department(
    state: &AppState,
    id: &str,
    req: UpdateDepartmentRequest,
) -> Result<Department, DbError> {
    let id = id.to_string();
    let dept: Option<Department> = state
        .db
        .update(("department", id))
        .merge(serde_json::json!({
            "name": req.name,
            "is_active": req.is_active,
        }))
        .await?;
    dept.ok_or(DbError::NotFound)
}

pub async fn delete_department(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "department", id).await?;
    Ok(())
}

pub async fn get_department_employees(
    state: &AppState,
    dept_id: &str,
) -> Result<Vec<serde_json::Value>, DbError> {
    let dept_id = dept_id.to_string();
    let employees: Vec<serde_json::Value> = state.db
        .query("SELECT id, name, role, status FROM employee WHERE department = type::thing('department', $id) AND (is_archived = false OR is_archived = NONE)")
        .bind(("id", dept_id))
        .await?
        .take(0)?;
    Ok(employees)
}

// ══════════════════════════════════════════════════════════════════
// Positions
// ══════════════════════════════════════════════════════════════════

pub async fn create_position(
    state: &AppState,
    req: CreatePositionRequest,
) -> Result<Position, DbError> {
    let code = req.code;
    let title = req.title;
    let grade = req.grade;
    let dept_ref = req
        .department_id
        .map(|id| format!("type::thing('department', '{}')", id))
        .unwrap_or_else(|| "NONE".to_string());

    let pos: Option<Position> = state
        .db
        .query(format!(
            "CREATE position SET code = $code, title = $title, \
             grade = $grade, department = {}, is_active = true",
            dept_ref
        ))
        .bind(("code", code))
        .bind(("title", title))
        .bind(("grade", grade))
        .await?
        .take(0)?;
    pos.ok_or(DbError::NotFound)
}

pub async fn get_all_positions(state: &AppState) -> Result<Vec<Position>, DbError> {
    let positions: Vec<Position> = state.db
        .query("SELECT * FROM position WHERE is_archived = false OR is_archived = NONE ORDER BY code ASC")
        .await?
        .take(0)?;
    Ok(positions)
}

pub async fn get_position(state: &AppState, id: &str) -> Result<Position, DbError> {
    let id = id.to_string();
    let pos: Option<Position> = state.db.select(("position", id)).await?;
    pos.ok_or(DbError::NotFound)
}

pub async fn update_position(
    state: &AppState,
    id: &str,
    req: UpdatePositionRequest,
) -> Result<Position, DbError> {
    let id = id.to_string();
    let pos: Option<Position> = state
        .db
        .update(("position", id))
        .merge(serde_json::json!({
            "title": req.title,
            "grade": req.grade,
            "is_active": req.is_active,
        }))
        .await?;
    pos.ok_or(DbError::NotFound)
}

pub async fn delete_position(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "position", id).await?;
    Ok(())
}
