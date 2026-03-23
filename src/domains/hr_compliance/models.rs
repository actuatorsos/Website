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
    #[serde(default)]
    pub employee: Option<Thing>,
    #[serde(default)]
    pub issued_by: Option<Thing>,
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
    #[serde(default)]
    pub deduction_days: Option<i64>,
    #[serde(default)]
    pub date: Option<serde_json::Value>,
    #[serde(default)]
    pub expiry_date: Option<serde_json::Value>,
    #[serde(default)]
    pub employee_ack: Option<bool>,
    #[serde(default)]
    pub ack_date: Option<serde_json::Value>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub attachments: Option<Vec<String>>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<serde_json::Value>,
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
    pub termination_type: Option<String>,
    #[serde(default)]
    pub hire_date: Option<serde_json::Value>,
    #[serde(default)]
    pub end_date: Option<serde_json::Value>,
    #[serde(default)]
    pub total_years: Option<f64>,
    #[serde(default)]
    pub last_salary: Option<f64>,
    #[serde(default)]
    pub half_month_years: Option<f64>,
    #[serde(default)]
    pub full_month_years: Option<f64>,
    #[serde(default)]
    pub gross_amount: Option<f64>,
    #[serde(default)]
    pub deductions: Option<f64>,
    #[serde(default)]
    pub net_amount: Option<f64>,
    #[serde(default)]
    pub leave_balance_payout: Option<f64>,
    #[serde(default)]
    pub total_payout: Option<f64>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub approved_by: Option<Thing>,
    #[serde(default)]
    pub payment_date: Option<serde_json::Value>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculateEosRequest {
    pub employee_id: String,
    pub end_date: String,
    pub termination_type: String,
    #[serde(default)]
    pub annual_leave_days_remaining: Option<i64>,
    #[serde(default)]
    pub deductions: Option<f64>,
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
    pub date: Option<serde_json::Value>,
    #[serde(default)]
    pub hours: Option<f64>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub rate_multiplier: Option<f64>,
    #[serde(default)]
    pub calculated_amount: Option<f64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub approved_by: Option<Thing>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOvertimeRequest {
    pub employee_id: String,
    pub date: String,
    pub hours: f64,
    #[serde(default)]
    pub reason: Option<String>,
}
