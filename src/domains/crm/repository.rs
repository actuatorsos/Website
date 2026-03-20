//! CRM Repository — عمليات قاعدة البيانات للعملاء والفرص

use super::models::*;
use crate::db::{AppState, DbError};
use f64;

pub async fn create_contact(
    state: &AppState,
    req: CreateContactRequest,
) -> Result<Contact, DbError> {
    let client_id = req.client_id;
    let name = req.name;
    let title = req.title;
    let department = req.department;
    let email = req.email;
    let phone = req.phone;
    let mobile = req.mobile;
    let is_primary = req.is_primary.unwrap_or(false);

    let contact: Option<Contact> = state
        .db
        .query(
            "CREATE contact SET \
             client = type::thing('client', $client_id), \
             name = $name, title = $title, \
             department = $dept, email = $email, phone = $phone, mobile = $mobile, \
             is_primary = $is_primary, is_archived = false",
        )
        .bind(("client_id", client_id))
        .bind(("name", name))
        .bind(("title", title))
        .bind(("dept", department))
        .bind(("email", email))
        .bind(("phone", phone))
        .bind(("mobile", mobile))
        .bind(("is_primary", is_primary))
        .await?
        .take(0)?;
    contact.ok_or(DbError::NotFound)
}

pub async fn get_contacts_by_client(
    state: &AppState,
    client_id: &str,
) -> Result<Vec<Contact>, DbError> {
    let id = client_id.to_string();
    let contacts: Vec<Contact> = state.db
        .query("SELECT * FROM contact WHERE client = type::thing('client', $id) AND (is_archived = false OR is_archived = NONE)")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(contacts)
}

pub async fn delete_contact(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "contact", id).await?;
    Ok(())
}

pub async fn create_interaction(
    state: &AppState,
    req: CreateInteractionRequest,
) -> Result<Interaction, DbError> {
    let client_id = req.client_id;
    let channel = req.channel;
    let direction = req.direction;
    let subject = req.subject;
    let notes = req.notes;
    let outcome = req.outcome;
    let priority = req.priority;
    let next_action = req.next_action;
    let next_action_date = req.next_action_date;
    let duration_min = req.duration_min;

    let interaction: Option<Interaction> = state
        .db
        .query(
            "CREATE interaction SET \
             client = type::thing('client', $client_id), \
             channel = $channel, direction = $dir, subject = $subject, \
             notes = $notes, outcome = $outcome, priority = $priority, \
             next_action = $next_action, next_action_date = $next_action_date, \
             duration_min = $duration_min, is_archived = false",
        )
        .bind(("client_id", client_id))
        .bind(("channel", channel))
        .bind(("dir", direction))
        .bind(("subject", subject))
        .bind(("notes", notes))
        .bind(("outcome", outcome))
        .bind(("priority", priority))
        .bind(("next_action", next_action))
        .bind(("next_action_date", next_action_date))
        .bind(("duration_min", duration_min))
        .await?
        .take(0)?;
    interaction.ok_or(DbError::NotFound)
}

pub async fn get_interactions_by_client(
    state: &AppState,
    client_id: &str,
) -> Result<Vec<Interaction>, DbError> {
    let id = client_id.to_string();
    let interactions: Vec<Interaction> = state.db
        .query("SELECT * FROM interaction WHERE client = type::thing('client', $id) ORDER BY created_at DESC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(interactions)
}

pub async fn create_opportunity(
    state: &AppState,
    req: CreateOpportunityRequest,
) -> Result<Opportunity, DbError> {
    let client_id = req.client_id;
    let title = req.title;
    let description = req.description;
    let value = req.value.unwrap_or(0.0);
    let probability = req.probability.unwrap_or(50_i64);
    let weighted_value = value * (probability as f64) / 100.0;
    let stage = req.stage.unwrap_or_else(|| "prospecting".to_string());
    let source = req.source;
    let expected_close = req.expected_close;
    let assigned_to_id = req.assigned_to_id;

    let opp: Option<Opportunity> = state
        .db
        .query(
            "CREATE opportunity SET \
             client = type::thing('client', $client_id), \
             title = $title, description = $desc, value = $value, \
             probability = $probability, weighted_value = $weighted, \
             stage = $stage, source = $source, \
             expected_close = $expected_close, \
             is_archived = false",
        )
        .bind(("client_id", client_id))
        .bind(("title", title))
        .bind(("desc", description))
        .bind(("value", value))
        .bind(("probability", probability))
        .bind(("weighted", weighted_value))
        .bind(("stage", stage))
        .bind(("source", source))
        .bind(("expected_close", expected_close))
        .await?
        .take(0)?;
    let _ = assigned_to_id;
    opp.ok_or(DbError::NotFound)
}

pub async fn get_all_opportunities(state: &AppState) -> Result<Vec<Opportunity>, DbError> {
    let opps: Vec<Opportunity> = state.db
        .query("SELECT * FROM opportunity WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?.take(0)?;
    Ok(opps)
}

pub async fn get_opportunity(state: &AppState, id: &str) -> Result<Opportunity, DbError> {
    let id = id.to_string();
    let opp: Option<Opportunity> = state.db.select(("opportunity", id)).await?;
    opp.ok_or(DbError::NotFound)
}

pub async fn update_opportunity_stage(
    state: &AppState,
    id: &str,
    req: UpdateOpportunityStageRequest,
) -> Result<Opportunity, DbError> {
    let id = id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let new_stage = req.stage.clone();
    let mut data = serde_json::json!({
        "stage": req.stage,
        "probability": req.probability,
        "lost_reason": req.lost_reason,
    });

    if new_stage == "won" {
        data["won_date"] = serde_json::Value::String(now);
    } else if new_stage == "lost" {
        data["lost_date"] = serde_json::Value::String(now);
    }
    let opp: Option<Opportunity> = state.db.update(("opportunity", id)).merge(data).await?;
    opp.ok_or(DbError::NotFound)
}

pub async fn delete_opportunity(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "opportunity", id).await?;
    Ok(())
}

pub async fn create_quotation(
    state: &AppState,
    req: CreateQuotationRequest,
) -> Result<Quotation, DbError> {
    let client_id = req.client_id;
    let title = req.title;
    let items = req.items;
    let discount_rate = req.discount_rate.unwrap_or(0.0);
    let tax_rate = req.tax_rate.unwrap_or(15.0);
    let payment_terms = req.payment_terms;
    let delivery_terms = req.delivery_terms;
    let valid_until = req.valid_until;
    let notes = req.notes;

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM quotation GROUP ALL")
        .await?
        .take(0)
        .unwrap_or_default();
    let n = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    let quote_number = format!("QT-{:04}", n);

    let subtotal: f64 = items.iter().map(|item| item.total).sum();
    let discount_amount = subtotal * discount_rate / 100.0;
    let after_discount = subtotal - discount_amount;
    let tax_amount = after_discount * tax_rate / 100.0;
    let total = after_discount + tax_amount;
    let items_json = serde_json::to_value(&items).unwrap_or_default();

    let quot: Option<Quotation> = state
        .db
        .query(
            "CREATE quotation SET \
             client = type::thing('client', $client_id), \
             quote_number = $quote_num, title = $title, \
             items = $items, subtotal = $subtotal, \
             discount_rate = $disc_rate, discount_amount = $disc_amount, \
             tax_rate = $tax_rate, tax_amount = $tax_amount, total = $total, \
             payment_terms = $payment_terms, delivery_terms = $delivery_terms, \
             valid_until = $valid_until, notes = $notes, \
             status = 'draft', version = 1",
        )
        .bind(("client_id", client_id))
        .bind(("quote_num", quote_number))
        .bind(("title", title))
        .bind(("items", items_json))
        .bind(("subtotal", subtotal))
        .bind(("disc_rate", discount_rate))
        .bind(("disc_amount", discount_amount))
        .bind(("tax_rate", tax_rate))
        .bind(("tax_amount", tax_amount))
        .bind(("total", total))
        .bind(("payment_terms", payment_terms))
        .bind(("delivery_terms", delivery_terms))
        .bind(("valid_until", valid_until))
        .bind(("notes", notes))
        .await?
        .take(0)?;
    quot.ok_or(DbError::NotFound)
}

pub async fn get_all_quotations(state: &AppState) -> Result<Vec<Quotation>, DbError> {
    let quots: Vec<Quotation> = state
        .db
        .query("SELECT * FROM quotation ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(quots)
}

pub async fn get_quotation(state: &AppState, id: &str) -> Result<Quotation, DbError> {
    let id = id.to_string();
    let quot: Option<Quotation> = state.db.select(("quotation", id)).await?;
    quot.ok_or(DbError::NotFound)
}
