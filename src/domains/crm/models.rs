//! CRM Domain Models — نماذج إدارة علاقات العملاء

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ══════════════════════════════════════════════════════════════════
// Contact — جهة اتصال
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<Thing>,
    pub client: Thing,
    pub name: String,
    pub title: Option<String>,
    pub department: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub linkedin: Option<String>,
    pub is_primary: Option<bool>,
    pub preferred_language: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactRequest {
    pub client_id: String,
    pub name: String,
    pub title: Option<String>,
    pub department: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub is_primary: Option<bool>,
}

// ══════════════════════════════════════════════════════════════════
// Interaction — سجل تفاعل
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Option<Thing>,
    pub client: Thing,
    pub contact: Option<Thing>,
    pub employee: Option<Thing>,
    pub opportunity: Option<Thing>,
    pub channel: String,
    pub direction: Option<String>,
    pub subject: String,
    pub notes: Option<String>,
    pub outcome: Option<String>,
    pub duration_min: Option<i64>,
    pub priority: Option<String>,
    pub is_completed: Option<bool>,
    pub next_action: Option<String>,
    pub next_action_date: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInteractionRequest {
    pub client_id: String,
    pub contact_id: Option<String>,
    pub channel: String,
    pub direction: Option<String>,
    pub subject: String,
    pub notes: Option<String>,
    pub outcome: Option<String>,
    pub duration_min: Option<i64>,
    pub priority: Option<String>,
    pub next_action: Option<String>,
    pub next_action_date: Option<String>,
}

// ══════════════════════════════════════════════════════════════════
// Opportunity — فرصة بيع
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Option<Thing>,
    pub client: Thing,
    pub title: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: Option<String>,
    pub probability: Option<i64>,
    pub weighted_value: Option<f64>,
    pub source: Option<String>,
    pub competitor: Option<String>,
    pub expected_close: Option<String>,
    pub won_date: Option<String>,
    pub lost_date: Option<String>,
    pub lost_reason: Option<String>,
    pub next_action: Option<String>,
    pub next_action_date: Option<String>,
    pub products: Option<serde_json::Value>,
    pub assigned_to: Option<Thing>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOpportunityRequest {
    pub client_id: String,
    pub title: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub stage: Option<String>,
    pub probability: Option<i64>,
    pub source: Option<String>,
    pub expected_close: Option<String>,
    pub assigned_to_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOpportunityStageRequest {
    pub stage: String,
    pub probability: Option<i64>,
    pub lost_reason: Option<String>,
}

// ══════════════════════════════════════════════════════════════════
// Quotation — عرض سعر
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotationItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub total: f64,
    pub tax_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quotation {
    pub id: Option<Thing>,
    pub client: Thing,
    pub opportunity: Option<Thing>,
    pub quote_number: String,
    pub title: Option<String>,
    pub items: Option<Vec<QuotationItem>>,
    pub subtotal: Option<f64>,
    pub discount_rate: Option<f64>,
    pub discount_amount: Option<f64>,
    pub tax_rate: Option<f64>,
    pub tax_amount: Option<f64>,
    pub total: Option<f64>,
    pub currency: Option<String>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub valid_until: Option<String>,
    pub version: Option<i64>,
    pub status: Option<String>,
    pub prepared_by: Option<Thing>,
    pub converted_to: Option<Thing>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuotationRequest {
    pub client_id: String,
    pub opportunity_id: Option<String>,
    pub title: Option<String>,
    pub items: Vec<QuotationItem>,
    pub discount_rate: Option<f64>,
    pub tax_rate: Option<f64>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
}
