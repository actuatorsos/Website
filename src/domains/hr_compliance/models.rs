//! HR Compliance Models — نماذج الامتثال (إنذارات، نهاية الخدمة، الإضافي)

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ══════════════════════════════════════════════════════════════════
// Warning — إنذار رسمي
// ══════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    #[serde(default)]
    pub id: Option<Thing>,
    pub employee: Option<Thing>,
    #[serde(default)]
    pub issued_by: Option<Thing>,
    #[serde(default)]
    pub warning_type: Option<String>, // verbal, written_1, written_2, final, termination
    #[serde(default)]
    pub violation: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deduction_amount: Option<f64>,
    #[serde(default)]
    pub suspension_days: Option<i64>,
    #[serde(default)]
    pub is_acknowledged: Option<bool>,
    #[serde(default)]
    pub acknowledged_at: Option<String>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWarningRequest {
    pub employee_id: String,
    #[serde(default)]
    pub issued_by_id: Option<String>,
    #[serde(default)]
    pub warning_type: Option<String>,
    #[serde(default)]
    pub violation: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub deduction_amount: Option<f64>,
    #[serde(default)]
    pub suspension_days: Option<i64>,
}

// ══════════════════════════════════════════════════════════════════
// End of Service — نهاية الخدمة
// ══════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndOfService {
    #[serde(default)]
    pub id: Option<Thing>,
    #[serde(default)]
    pub employee: Option<Thing>,
    #[serde(default)]
    pub termination_date: Option<String>,
    #[serde(default)]
    pub termination_type: Option<String>, // resignation, termination, retirement, contract_end
    #[serde(default)]
    pub total_years: Option<f64>,
    #[serde(default)]
    pub total_months: Option<i64>,
    #[serde(default)]
    pub last_basic_salary: Option<f64>,
    #[serde(default)]
    pub first_five_years_amount: Option<f64>,
    #[serde(default)]
    pub remaining_years_amount: Option<f64>,
    #[serde(default)]
    pub gross_amount: Option<f64>,
    #[serde(default)]
    pub annual_leave_encashment: Option<f64>,
    #[serde(default)]
    pub other_deductions: Option<f64>,
    #[serde(default)]
    pub net_amount: Option<f64>,
    #[serde(default)]
    pub total_payout: Option<f64>,
    #[serde(default)]
    pub calculation_notes: Option<String>,
    #[serde(default)]
    pub status: Option<String>, // draft, finalized, paid
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateEosRequest {
    pub employee_id: String,
    pub termination_date: String,
    pub termination_type: String,
    #[serde(default)]
    pub annual_leave_days_remaining: Option<i64>,
    #[serde(default)]
    pub other_deductions: Option<f64>,
}

// ══════════════════════════════════════════════════════════════════
// Overtime Request — طلب عمل إضافي
// ══════════════════════════════════════════════════════════════════
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvertimeRequest {
    #[serde(default)]
    pub id: Option<Thing>,
    #[serde(default)]
    pub employee: Option<Thing>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub hours: Option<f64>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub rate_multiplier: Option<f64>,
    #[serde(default)]
    pub calculated_amount: Option<f64>,
    #[serde(default)]
    pub status: Option<String>, // pending, approved, rejected
    #[serde(default)]
    pub approved_by: Option<Thing>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOvertimeRequest {
    pub employee_id: String,
    pub date: String,
    pub hours: f64,
    #[serde(default)]
    pub reason: Option<String>,
}
