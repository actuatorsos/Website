//! Payroll Advanced Models — نماذج الرواتب

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Salary {
    pub id: Option<Thing>,
    pub employee: Thing,
    pub basic: Option<f64>,
    pub housing: Option<f64>,
    pub transport: Option<f64>,
    pub phone: Option<f64>,
    pub other_allowances: Option<f64>,
    pub total: Option<f64>,
    pub effective_date: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSalaryRequest {
    pub basic: Option<f64>,
    pub housing: Option<f64>,
    pub transport: Option<f64>,
    pub phone: Option<f64>,
    pub other_allowances: Option<f64>,
    pub effective_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollRecord {
    pub id: Option<Thing>,
    pub employee: Thing,
    pub month: String, // YYYY-MM
    pub basic: Option<f64>,
    pub housing: Option<f64>,
    pub transport: Option<f64>,
    pub other_allowances: Option<f64>,
    pub overtime_amount: Option<f64>,
    pub gross: Option<f64>,
    pub gosi_deduction: Option<f64>,
    pub absence_deduction: Option<f64>,
    pub advance_deduction: Option<f64>,
    pub penalty_deduction: Option<f64>,
    pub other_deductions: Option<f64>,
    pub total_deductions: Option<f64>,
    pub net: Option<f64>,
    pub status: Option<String>, // draft, approved, paid
    pub payment_date: Option<String>,
    pub payment_method: Option<String>,
    pub bank_reference: Option<String>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePayrollRequest {
    pub month: String,                     // YYYY-MM
    pub employee_ids: Option<Vec<String>>, // None = generate for all
}
