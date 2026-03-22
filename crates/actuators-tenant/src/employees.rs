use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use serde_json::{json, Value};

use actuators_auth::middleware::AppState;
use actuators_core::error::AppError;

use crate::auth::{TenantAuth, tenant_db};

// ── Query / Form structs ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EmployeeFilters {
    pub department_id: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeForm {
    pub employee_number: String,
    pub first_name_en: String,
    pub last_name_en: String,
    pub first_name_ar: Option<String>,
    pub last_name_ar: Option<String>,
    pub department_id: Option<String>,
    pub job_title_id: Option<String>,
    pub hire_date: Option<String>,
    pub employment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeForm {
    pub first_name_en: Option<String>,
    pub last_name_en: Option<String>,
    pub first_name_ar: Option<String>,
    pub last_name_ar: Option<String>,
    pub department_id: Option<String>,
    pub job_title_id: Option<String>,
    pub employment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractForm {
    pub contract_type: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub salary: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

// ── Template row types ───────────────────────────────────────────────

#[derive(Debug)]
struct EmployeeRow {
    id: String,
    employee_number: String,
    first_name_en: String,
    last_name_en: String,
    department_id: String,
    status: String,
}

#[derive(Debug)]
struct EmployeeDetail {
    id: String,
    employee_number: String,
    first_name_en: String,
    last_name_en: String,
    first_name_ar: String,
    last_name_ar: String,
    department_id: String,
    job_title_id: String,
    hire_date: String,
    employment_type: String,
    status: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct ContractRow {
    id: String,
    contract_type: String,
    start_date: String,
    end_date: String,
    salary: String,
    currency: String,
    is_current: bool,
}

#[derive(Debug)]
struct DepartmentOption {
    id: String,
    name_en: String,
}

#[derive(Debug)]
struct JobTitleOption {
    id: String,
    name_en: String,
}

// ── Templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Employees</title></head>
<body>
  <h1>Employees</h1>
  <a href="/app/hr/employees/new">+ New Employee</a>
  <form method="get" action="/app/hr/employees">
    <input type="text" name="search" placeholder="Search..." value="{{ search }}">
    <select name="status">
      <option value="">All statuses</option>
      <option value="active" {% if status_filter == "active" %}selected{% endif %}>Active</option>
      <option value="terminated" {% if status_filter == "terminated" %}selected{% endif %}>Terminated</option>
      <option value="on_leave" {% if status_filter == "on_leave" %}selected{% endif %}>On Leave</option>
    </select>
    <button type="submit">Filter</button>
  </form>
  <table>
    <thead>
      <tr><th>#</th><th>Name</th><th>Department</th><th>Status</th><th></th></tr>
    </thead>
    <tbody id="employee-rows">
    {% for emp in employees %}
      <tr>
        <td>{{ emp.employee_number }}</td>
        <td>{{ emp.first_name_en }} {{ emp.last_name_en }}</td>
        <td>{{ emp.department_id }}</td>
        <td>{{ emp.status }}</td>
        <td><a href="/app/hr/employees/{{ emp.id }}">View</a></td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <p>Page {{ page }} | {{ total }} results</p>
  <a href="/app">&larr; Dashboard</a>
</body>
</html>"#,
    ext = "html"
)]
struct EmployeeListTemplate {
    employees: Vec<EmployeeRow>,
    search: String,
    status_filter: String,
    page: i64,
    total: usize,
}

#[derive(Template)]
#[template(
    source = r#"{% for emp in employees %}
<tr>
  <td>{{ emp.employee_number }}</td>
  <td>{{ emp.first_name_en }} {{ emp.last_name_en }}</td>
  <td>{{ emp.department_id }}</td>
  <td>{{ emp.status }}</td>
  <td><a href="/app/hr/employees/{{ emp.id }}">View</a></td>
</tr>
{% endfor %}"#,
    ext = "html"
)]
struct EmployeeRowsPartialTemplate {
    employees: Vec<EmployeeRow>,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>New Employee</title></head>
<body>
  <h1>Create Employee</h1>
  <form method="post" action="/app/hr/employees">
    <label>Employee # <input type="text" name="employee_number" required></label><br>
    <label>First Name (EN) <input type="text" name="first_name_en" required></label><br>
    <label>Last Name (EN) <input type="text" name="last_name_en" required></label><br>
    <label>First Name (AR) <input type="text" name="first_name_ar"></label><br>
    <label>Last Name (AR) <input type="text" name="last_name_ar"></label><br>
    <label>Department
      <select name="department_id">
        <option value="">-- None --</option>
        {% for d in departments %}
        <option value="{{ d.id }}">{{ d.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Job Title
      <select name="job_title_id">
        <option value="">-- None --</option>
        {% for j in job_titles %}
        <option value="{{ j.id }}">{{ j.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Hire Date <input type="date" name="hire_date"></label><br>
    <label>Employment Type
      <select name="employment_type">
        <option value="full_time">Full Time</option>
        <option value="part_time">Part Time</option>
        <option value="contract">Contract</option>
        <option value="intern">Intern</option>
      </select>
    </label><br>
    <button type="submit">Create</button>
  </form>
  <a href="/app/hr/employees">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct CreateEmployeeTemplate {
    departments: Vec<DepartmentOption>,
    job_titles: Vec<JobTitleOption>,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>{{ emp.first_name_en }} {{ emp.last_name_en }}</title></head>
<body>
  <h1>{{ emp.first_name_en }} {{ emp.last_name_en }}</h1>
  <dl>
    <dt>Employee #</dt><dd>{{ emp.employee_number }}</dd>
    <dt>Name (AR)</dt><dd>{{ emp.first_name_ar }} {{ emp.last_name_ar }}</dd>
    <dt>Department</dt><dd>{{ emp.department_id }}</dd>
    <dt>Job Title</dt><dd>{{ emp.job_title_id }}</dd>
    <dt>Hire Date</dt><dd>{{ emp.hire_date }}</dd>
    <dt>Type</dt><dd>{{ emp.employment_type }}</dd>
    <dt>Status</dt><dd>{{ emp.status }}</dd>
  </dl>
  <a href="/app/hr/employees/{{ emp.id }}/edit">Edit</a>
  {% if emp.status != "terminated" %}
  <form method="post" action="/app/hr/employees/{{ emp.id }}/terminate" style="display:inline">
    <button type="submit" onclick="return confirm('Terminate this employee?')">Terminate</button>
  </form>
  {% endif %}
  <h2>Contracts</h2>
  <table>
    <thead>
      <tr><th>Type</th><th>Start</th><th>End</th><th>Salary</th><th>Current</th></tr>
    </thead>
    <tbody>
    {% for c in contracts %}
      <tr>
        <td>{{ c.contract_type }}</td>
        <td>{{ c.start_date }}</td>
        <td>{{ c.end_date }}</td>
        <td>{{ c.salary }} {{ c.currency }}</td>
        <td>{% if c.is_current %}Yes{% else %}No{% endif %}</td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <h3>Add Contract</h3>
  <form method="post" action="/app/hr/employees/{{ emp.id }}/contracts">
    <label>Type
      <select name="contract_type">
        <option value="permanent">Permanent</option>
        <option value="fixed_term">Fixed Term</option>
        <option value="probation">Probation</option>
      </select>
    </label><br>
    <label>Start Date <input type="date" name="start_date" required></label><br>
    <label>End Date <input type="date" name="end_date"></label><br>
    <label>Salary <input type="text" name="salary"></label><br>
    <label>Currency <input type="text" name="currency" value="SAR"></label><br>
    <label>Notes <textarea name="notes"></textarea></label><br>
    <button type="submit">Add Contract</button>
  </form>
  <a href="/app/hr/employees">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct EmployeeDetailTemplate {
    emp: EmployeeDetail,
    contracts: Vec<ContractRow>,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Edit — {{ emp.first_name_en }} {{ emp.last_name_en }}</title></head>
<body>
  <h1>Edit {{ emp.first_name_en }} {{ emp.last_name_en }}</h1>
  <form method="post" action="/app/hr/employees/{{ emp.id }}">
    <label>First Name (EN) <input type="text" name="first_name_en" value="{{ emp.first_name_en }}"></label><br>
    <label>Last Name (EN) <input type="text" name="last_name_en" value="{{ emp.last_name_en }}"></label><br>
    <label>First Name (AR) <input type="text" name="first_name_ar" value="{{ emp.first_name_ar }}"></label><br>
    <label>Last Name (AR) <input type="text" name="last_name_ar" value="{{ emp.last_name_ar }}"></label><br>
    <label>Department
      <select name="department_id">
        <option value="">-- None --</option>
        {% for d in departments %}
        <option value="{{ d.id }}" {% if d.id == emp.department_id %}selected{% endif %}>{{ d.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Job Title
      <select name="job_title_id">
        <option value="">-- None --</option>
        {% for j in job_titles %}
        <option value="{{ j.id }}" {% if j.id == emp.job_title_id %}selected{% endif %}>{{ j.name_en }}</option>
        {% endfor %}
      </select>
    </label><br>
    <label>Employment Type
      <select name="employment_type">
        <option value="full_time" {% if emp.employment_type == "full_time" %}selected{% endif %}>Full Time</option>
        <option value="part_time" {% if emp.employment_type == "part_time" %}selected{% endif %}>Part Time</option>
        <option value="contract" {% if emp.employment_type == "contract" %}selected{% endif %}>Contract</option>
        <option value="intern" {% if emp.employment_type == "intern" %}selected{% endif %}>Intern</option>
      </select>
    </label><br>
    <button type="submit">Save</button>
  </form>
  <a href="/app/hr/employees/{{ emp.id }}">&larr; Back</a>
</body>
</html>"#,
    ext = "html"
)]
struct EditEmployeeTemplate {
    emp: EmployeeDetail,
    departments: Vec<DepartmentOption>,
    job_titles: Vec<JobTitleOption>,
}

#[derive(Template)]
#[template(
    source = r#"<table>
  <thead>
    <tr><th>Type</th><th>Start</th><th>End</th><th>Salary</th><th>Current</th></tr>
  </thead>
  <tbody>
  {% for c in contracts %}
    <tr>
      <td>{{ c.contract_type }}</td>
      <td>{{ c.start_date }}</td>
      <td>{{ c.end_date }}</td>
      <td>{{ c.salary }} {{ c.currency }}</td>
      <td>{% if c.is_current %}Yes{% else %}No{% endif %}</td>
    </tr>
  {% endfor %}
  </tbody>
</table>"#,
    ext = "html"
)]
struct ContractsPartialTemplate {
    contracts: Vec<ContractRow>,
}

// ── RBAC helper ──────────────────────────────────────────────────────

fn require_hr_access(auth: &TenantAuth) -> Result<(), AppError> {
    match auth.0.role.as_str() {
        "admin" | "super_admin" | "manager" | "employee" => Ok(()),
        other => Err(AppError::Forbidden(format!(
            "role '{other}' cannot access HR module"
        ))),
    }
}

fn require_hr_write(auth: &TenantAuth) -> Result<(), AppError> {
    match auth.0.role.as_str() {
        "admin" | "super_admin" | "manager" => Ok(()),
        other => Err(AppError::Forbidden(format!(
            "role '{other}' cannot modify employee records"
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

fn value_to_row(v: &Value) -> EmployeeRow {
    EmployeeRow {
        id: val_str(v, "id"),
        employee_number: val_str(v, "employee_number"),
        first_name_en: val_str(v, "first_name_en"),
        last_name_en: val_str(v, "last_name_en"),
        department_id: val_str(v, "department_id"),
        status: val_str(v, "status"),
    }
}

fn value_to_detail(v: &Value) -> EmployeeDetail {
    EmployeeDetail {
        id: val_str(v, "id"),
        employee_number: val_str(v, "employee_number"),
        first_name_en: val_str(v, "first_name_en"),
        last_name_en: val_str(v, "last_name_en"),
        first_name_ar: val_str(v, "first_name_ar"),
        last_name_ar: val_str(v, "last_name_ar"),
        department_id: val_str(v, "department_id"),
        job_title_id: val_str(v, "job_title_id"),
        hire_date: val_str(v, "hire_date"),
        employment_type: val_str(v, "employment_type"),
        status: val_str(v, "status"),
    }
}

fn value_to_contract(v: &Value) -> ContractRow {
    ContractRow {
        id: val_str(v, "id"),
        contract_type: val_str(v, "contract_type"),
        start_date: val_str(v, "start_date"),
        end_date: val_str(v, "end_date"),
        salary: val_str(v, "salary"),
        currency: val_str(v, "currency"),
        is_current: val_bool(v, "is_current"),
    }
}

fn value_to_dept_option(v: &Value) -> DepartmentOption {
    DepartmentOption {
        id: val_str(v, "id"),
        name_en: val_str(v, "name_en"),
    }
}

fn value_to_jt_option(v: &Value) -> JobTitleOption {
    JobTitleOption {
        id: val_str(v, "id"),
        name_en: val_str(v, "name_en"),
    }
}

// ── Handlers ─────────────────────────────────────────────────────────

/// List employees with optional filters. Supports HTMX partial responses.
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    headers: HeaderMap,
    Query(filters): Query<EmployeeFilters>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_access(&auth)?;

    let db = tenant_db(&state, &auth).await?;

    let page = filters.page.unwrap_or(1).max(1);
    let per_page = filters.per_page.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let raw = db
        .list_employees(
            filters.department_id.as_deref(),
            filters.status.as_deref(),
            filters.search.as_deref(),
            per_page,
            offset,
        )
        .await?;

    let employees: Vec<EmployeeRow> = raw.iter().map(value_to_row).collect();
    let total = employees.len();

    // HTMX partial: return only the table rows.
    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some();

    if is_htmx {
        let tmpl = EmployeeRowsPartialTemplate { employees };
        return Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?));
    }

    let tmpl = EmployeeListTemplate {
        employees,
        search: filters.search.unwrap_or_default(),
        status_filter: filters.status.unwrap_or_default(),
        page,
        total,
    };

    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Render the create-employee form with department and job-title dropdowns.
pub async fn create_page(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw_depts = db.list_departments().await?;
    let raw_titles = db.list_job_titles().await?;

    let departments: Vec<DepartmentOption> = raw_depts.iter().map(value_to_dept_option).collect();
    let job_titles: Vec<JobTitleOption> = raw_titles.iter().map(value_to_jt_option).collect();

    let tmpl = CreateEmployeeTemplate {
        departments,
        job_titles,
    };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Create a new employee.
pub async fn create(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Form(form): Form<CreateEmployeeForm>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    if form.employee_number.is_empty() || form.first_name_en.is_empty() || form.last_name_en.is_empty() {
        return Err(AppError::Validation(
            "employee_number, first_name_en, and last_name_en are required".into(),
        ));
    }

    let db = tenant_db(&state, &auth).await?;

    let data = json!({
        "employee_number": form.employee_number,
        "first_name_en": form.first_name_en,
        "last_name_en": form.last_name_en,
        "first_name_ar": form.first_name_ar.unwrap_or_default(),
        "last_name_ar": form.last_name_ar.unwrap_or_default(),
        "department_id": form.department_id.unwrap_or_default(),
        "job_title_id": form.job_title_id.unwrap_or_default(),
        "hire_date": form.hire_date.unwrap_or_default(),
        "employment_type": form.employment_type.unwrap_or_else(|| "full_time".into()),
        "status": "active",
    });

    db.create_employee(data).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "create",
        "hr",
        "employee",
        None,
        Some(json!({ "employee_number": form.employee_number })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to("/app/hr/employees"))
}

/// Employee detail page with contracts.
pub async fn detail(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_access(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db
        .get_employee_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("employee '{id}' not found")))?;

    let emp = value_to_detail(&raw);

    // Self-check: employees can only view their own record.
    if auth.0.role == "employee" {
        let own_account = db.get_account_by_platform_id(&auth.0.platform_id).await?;
        let is_own = own_account
            .as_ref()
            .and_then(|a| a.get("employee_id"))
            .and_then(Value::as_str)
            .map(|eid| eid == id)
            .unwrap_or(false);
        if !is_own {
            return Err(AppError::Forbidden("employees can only view their own record".into()));
        }
    }

    let raw_contracts = db.list_contracts_for_employee(&id).await?;
    let contracts: Vec<ContractRow> = raw_contracts.iter().map(value_to_contract).collect();

    let tmpl = EmployeeDetailTemplate { emp, contracts };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Render the edit-employee form.
pub async fn edit_page(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db
        .get_employee_by_id(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("employee '{id}' not found")))?;

    let emp = value_to_detail(&raw);

    let raw_depts = db.list_departments().await?;
    let raw_titles = db.list_job_titles().await?;
    let departments: Vec<DepartmentOption> = raw_depts.iter().map(value_to_dept_option).collect();
    let job_titles: Vec<JobTitleOption> = raw_titles.iter().map(value_to_jt_option).collect();

    let tmpl = EditEmployeeTemplate {
        emp,
        departments,
        job_titles,
    };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Update an employee record.
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
    Form(form): Form<UpdateEmployeeForm>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;

    let mut fields = serde_json::Map::new();
    if let Some(ref v) = form.first_name_en {
        fields.insert("first_name_en".into(), json!(v));
    }
    if let Some(ref v) = form.last_name_en {
        fields.insert("last_name_en".into(), json!(v));
    }
    if let Some(ref v) = form.first_name_ar {
        fields.insert("first_name_ar".into(), json!(v));
    }
    if let Some(ref v) = form.last_name_ar {
        fields.insert("last_name_ar".into(), json!(v));
    }
    if let Some(ref v) = form.department_id {
        fields.insert("department_id".into(), json!(v));
    }
    if let Some(ref v) = form.job_title_id {
        fields.insert("job_title_id".into(), json!(v));
    }
    if let Some(ref v) = form.employment_type {
        fields.insert("employment_type".into(), json!(v));
    }

    db.update_employee(&id, Value::Object(fields.clone())).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "update",
        "hr",
        "employee",
        Some(&id),
        Some(Value::Object(fields)),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to(&format!("/app/hr/employees/{id}")))
}

/// Terminate an employee (set status to "terminated").
pub async fn terminate(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    db.terminate_employee(&id).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "terminate",
        "hr",
        "employee",
        Some(&id),
        None,
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to(&format!("/app/hr/employees/{id}")))
}

/// List contracts for an employee.
pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_access(&auth)?;

    let db = tenant_db(&state, &auth).await?;
    let raw = db.list_contracts_for_employee(&id).await?;
    let contracts: Vec<ContractRow> = raw.iter().map(value_to_contract).collect();

    let tmpl = ContractsPartialTemplate { contracts };
    Ok(Html(tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?))
}

/// Create a contract for an employee.
pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    auth: TenantAuth,
    Path(id): Path<String>,
    Form(form): Form<CreateContractForm>,
) -> Result<impl IntoResponse, AppError> {
    require_hr_write(&auth)?;

    if form.start_date.is_empty() {
        return Err(AppError::Validation("start_date is required".into()));
    }

    let db = tenant_db(&state, &auth).await?;

    let data = json!({
        "employee_id": id,
        "contract_type": form.contract_type,
        "start_date": form.start_date,
        "end_date": form.end_date.unwrap_or_default(),
        "salary": form.salary.unwrap_or_default(),
        "currency": form.currency.unwrap_or_else(|| "SAR".into()),
        "notes": form.notes.unwrap_or_default(),
    });

    db.create_contract(data).await?;

    db.log_audit(
        &auth.0.account_id,
        &auth.0.account_id,
        "create",
        "hr",
        "contract",
        Some(&id),
        Some(json!({ "contract_type": form.contract_type })),
        None,
        None,
    )
    .await
    .ok();

    Ok(Redirect::to(&format!("/app/hr/employees/{id}")))
}
