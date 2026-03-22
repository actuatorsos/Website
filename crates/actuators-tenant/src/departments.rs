use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::{json, Value};

use actuators_auth::middleware::AppState;
use actuators_core::error::AppError;

use crate::auth::{TenantAuth, tenant_db};

// ── Form structs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentForm {
    pub name_en: String,
    pub name_ar: String,
    pub code: String,
    pub parent_id: Option<String>,
    pub manager_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentForm {
    pub name_en: Option<String>,
    pub name_ar: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub manager_id: Option<String>,
    pub is_active: Option<String>,
}

// ── Template row types ───────────────────────────────────────────────

#[derive(Debug)]
struct DepartmentRow {
    id: String,
    name_en: String,
    name_ar: String,
    code: String,
    is_active: bool,
}

#[derive(Debug)]
struct DepartmentDetail {
    id: String,
    name_en: String,
    name_ar: String,
    code: String,
    parent_id: String,
    manager_id: String,
    is_active: bool,
}

#[derive(Debug)]
struct EmployeeRow {
    id: String,
    first_name_en: String,
    last_name_en: String,
    employee_number: String,
}

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Departments</title></head>
<body>
  <h1>Departments</h1>
  <a href="/app/hr/departments/new">+ New Department</a>
  <table>
    <thead>
      <tr><th>Code</th><th>Name (EN)</th><th>Name (AR)</th><th>Active</th><th></th></tr>
    </thead>
    <tbody>
    {% for dept in departments %}
      <tr>
        <td>{{ dept.code }}</td>
        <td>{{ dept.name_en }}</td>
        <td>{{ dept.name_ar }}</td>
        <td>{% if dept.is_active %}Yes{% else %}No{% endif %}</td>
        <td><a href="/app/hr/departments/{{ dept.id }}">View</a></td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <a href="/app">&larr; Dashboard</a>
</body>
</html>"#,
    ext = "html"
)]
struct DepartmentListTemplate {
    departments: Vec<DepartmentRow>,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>New Department</title></head>
<body>
  <h1>Create Department</h1>
  <form method="post" action="/app/hr/departments">
    <label>Name (EN) <input type="text" name="name_en" required></label><br>
    <label>Name (AR) <input type="text" name="name_ar" required></label><br>
    <label>Code <input type="text" name="code" required></label><br>
    <label>Parent Department
      <select name="parent_id">
        <option value="">-- None --</option>
        {% for d in existing_departments %}
        <option value="{{ d.id }}">{{ d.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Manager ID <input type="text" name="manager_id"></label><br>
    <button type="submit">Create</button>
  </form>
  <a href="/app/hr/departments">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct CreateDepartmentTemplate {
    existing_departments: Vec<DepartmentRow>,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>{{ dept.name_en }}</title></head>
<body>
  <h1>{{ dept.name_en }} ({{ dept.code }})</h1>
  <form method="post" action="/app/hr/departments/{{ dept.id }}">
    <label>Name (EN) <input type="text" name="name_en" value="{{ dept.name_en }}"></label><br>
    <label>Name (AR) <input type="text" name="name_ar" value="{{ dept.name_ar }}"></label><br>
    <label>Code <input type="text" name="code" value="{{ dept.code }}"></label><br>
    <label>Parent Department
      <select name="parent_id">
        <option value="">-- None --</option>
        {% for d in all_departments %}
        <option value="{{ d.id }}" {% if d.id == dept.parent_id %}selected{% endif %}>{{ d.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Manager ID <input type="text" name="manager_id" value="{{ dept.manager_id }}"></label><br>
    <label>Active
      <select name="is_active">
        <option value="true" {% if dept.is_active %}selected{% endif %}>Yes</option>
        <option value="false" {% if !dept.is_active %}selected{% endif %}>No</option>
      </select>
    </label><br>
    <button type="submit">Update</button>
  </form>
  <h2>Employees in Department</h2>
  <table>
    <thead>
      <tr><th>#</th><th>Name</th><th></th></tr>
    </thead>
    <tbody>
    {% for emp in employees %}
      <tr>
        <td>{{ emp.employee_number }}</td>
        <td>{{ emp.first_name_en }} {{ emp.last_name_en }}</td>
        <td><a href="/app/hr/employees/{{ emp.id }}">View</a></td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <a href="/app/hr/departments">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct DepartmentDetailTemplate {
    dept: DepartmentDetail,
    employees: Vec<EmployeeRow>,
    all_departments: Vec<DepartmentRow>,
}

// ── RBAC helper ──────────────────────────────────────────────────────

fn require_dept_write(auth: &TenantAuth) -> Result<(), AppError> {
    match auth.0.role.as_str() {
        "admin" | "super_admin" | "manager" => Ok(()),
        other => Err(AppError::Forbidden(format!(
            "role '{other}' cannot manage departments"
        ))),
    }
}

// ── Value helpers ────────────────────────────────────────────────────

fn val_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn val_bool(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn value_to_row(v: &Value) -> DepartmentRow {
    DepartmentRow {
        id: val_str(v, "id"),
        name_en: val_str(v, "name_en"),
        name_ar: val_str(v, "name_ar"),
        code: val_str(v, "code"),
        is_active: val_bool(v, "is_active"),
    }
}

fn value_to_detail(v: &Value) -> DepartmentDetail {
    DepartmentDetail {
        id: val_str(v, "id"),
        name_en: val_str(v, "name_en"),
        name_ar: val_str(v, "name_ar"),
        code: val_str(v, "code"),
        parent_id: val_str(v, "parent_id"),
        manager_id: val_str(v, "manager_id"),
        is_active: val_bool(v, "is_active"),
    }
}

fn value_to_employee_row(v: &Value) -> EmployeeRow {
    EmployeeRow {
        id: val_str(v, "id"),
        first_name_en: val_str(v, "first_name_en"),
        last_name_en: val_str(v, "last_name_en"),
        employee_number: val_str(v, "employee_number"),
    }
}

// ── Handlers ─────────────────────────────────────────────────────────

/// List all departments.
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = tenant_db(&state, &auth).await?;
    let raw = db.list_departments().await?;
    let departments: Vec<DepartmentRow> = raw.iter().map(value_to_row).collect();

    let tmpl = DepartmentListTemplate { departments };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Render the create-department form.
pub async fn create_page(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    require_dept_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db.list_departments().await?;
    let existing_departments: Vec<DepartmentRow> = raw.iter().map(value_to_row).collect();

    let tmpl = CreateDepartmentTemplate {
        existing_departments,
    };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Create a new department.
pub async fn create(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Form(form): Form<CreateDepartmentForm>,
) -> Result<impl IntoResponse, AppError> {
    require_dept_write(&auth)?;

    if form.name_en.is_empty() || form.code.is_empty() {
        return Err(AppError::Validation("name_en and code are required".into()));
    }

    let db = tenant_db(&state, &auth).await?;

    // Filter out empty strings for optional fields.
    let parent_id = form.parent_id.as_deref().filter(|s| !s.is_empty());
    let manager_id = form.manager_id.as_deref().filter(|s| !s.is_empty());

    db.create_department(&form.name_en, &form.name_ar, &form.code, parent_id, manager_id)
        .await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "create",
        "hr",
        "department",
        None,
        Some(json!({ "code": form.code, "name_en": form.name_en })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to("/app/hr/departments"))
}

/// Department detail with employees and edit form.
pub async fn detail(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = tenant_db(&state, &auth).await?;

    let raw = db
        .get_department_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("department '{id}' not found")))?;

    let dept = value_to_detail(&raw);

    // Fetch employees in this department.
    let raw_employees = db
        .list_employees(Some(&id), None, None, 100, 0)
        .await?;
    let employees: Vec<EmployeeRow> = raw_employees.iter().map(value_to_employee_row).collect();

    // Fetch all departments for parent dropdown.
    let raw_all = db.list_departments().await?;
    let all_departments: Vec<DepartmentRow> = raw_all
        .iter()
        .filter(|v| val_str(v, "id") != id)
        .map(value_to_row)
        .collect();

    let tmpl = DepartmentDetailTemplate {
        dept,
        employees,
        all_departments,
    };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Update a department.
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
    Form(form): Form<UpdateDepartmentForm>,
) -> Result<impl IntoResponse, AppError> {
    require_dept_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;

    let mut fields = serde_json::Map::new();
    if let Some(ref v) = form.name_en {
        fields.insert("name_en".into(), json!(v));
    }
    if let Some(ref v) = form.name_ar {
        fields.insert("name_ar".into(), json!(v));
    }
    if let Some(ref v) = form.code {
        fields.insert("code".into(), json!(v));
    }
    if let Some(ref v) = form.parent_id {
        let val = if v.is_empty() { Value::Null } else { json!(v) };
        fields.insert("parent_id".into(), val);
    }
    if let Some(ref v) = form.manager_id {
        let val = if v.is_empty() { Value::Null } else { json!(v) };
        fields.insert("manager_id".into(), val);
    }
    if let Some(ref v) = form.is_active {
        fields.insert("is_active".into(), json!(v == "true"));
    }

    db.update_department(&id, Value::Object(fields.clone())).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "update",
        "hr",
        "department",
        Some(&id),
        Some(Value::Object(fields)),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to(&format!("/app/hr/departments/{id}")))
}
