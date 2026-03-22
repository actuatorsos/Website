//! Payroll Advanced Models — نماذج الرواتب

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

/// Salary record — matches schema
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Salary {
    pub id: Option<RecordId>,
    pub employee: Option<RecordId>,
    pub base_amount: Option<f64>,
    pub currency: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateSalaryRequest {
    pub base_amount: Option<f64>,
    pub currency: Option<String>,
    pub effective_from: Option<String>,
}

/// Payroll record — matches ACTUAL schema fields
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PayrollRecord {
    pub id: Option<RecordId>,
    pub employee: Option<RecordId>,
    pub period_month: Option<i32>,
    pub period_year: Option<i32>,
    pub base_salary: Option<f64>,
    pub housing_allowance: Option<f64>,
    pub transport_allowance: Option<f64>,
    pub other_allowances: Option<f64>,
    pub overtime_hours: Option<f64>,
    pub overtime_amount: Option<f64>,
    pub deductions: Option<f64>,
    pub deduction_notes: Option<String>,
    pub gosi_employee: Option<f64>,
    pub gosi_employer: Option<f64>,
    pub net_salary: Option<f64>,
    pub currency: Option<String>,
    pub payment_method: Option<String>,
    pub payment_date: Option<String>,
    pub status: Option<String>,
    pub approved_by: Option<RecordId>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
    // Computed field from join
    #[serde(default)]
    pub employee_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct GeneratePayrollRequest {
    pub month: String,
    pub employee_ids: Option<Vec<String>>,
}
