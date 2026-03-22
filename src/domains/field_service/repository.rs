//! Field Service Repository — عمليات الخدمة الميدانية في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn get_all_tickets(state: &AppState, org_id: Option<&str>) -> Result<Vec<ServiceTicket>, DbError> {
    let tickets: Vec<ServiceTicket> = if let Some(org) = org_id {
        state
            .db
            .query("SELECT *, client.company_name AS client_name, assigned_to.name AS assigned_name FROM service_ticket WHERE is_archived = false AND organization = type::record($org) ORDER BY created_at DESC")
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query("SELECT *, client.company_name AS client_name, assigned_to.name AS assigned_name FROM service_ticket WHERE is_archived = false ORDER BY created_at DESC")
            .await?
            .take(0)?
    };
    Ok(tickets)
}

pub async fn create_ticket(
    state: &AppState,
    req: CreateTicketRequest,
    org_id: Option<&str>,
) -> Result<ServiceTicket, DbError> {
    let title = req.title;
    let description = req.description;
    let priority = req.priority.unwrap_or_else(|| "medium".to_string());
    let scheduled_date = req.scheduled_date;
    let location = req.location;
    let notes = req.notes;
    let client_id = req.client_id;
    let assigned_to = req.assigned_to;

    let mut query_str = String::from(
        "CREATE service_ticket SET \
         title = $title, \
         description = $description, \
         priority = $priority, \
         scheduled_date = $scheduled_date, \
         location = $location, \
         notes = $notes",
    );

    if org_id.is_some() {
        query_str.push_str(", organization = type::record($org_id)");
    }
    let org_id_val = org_id.map(|s| s.to_string());

    if client_id.is_some() {
        query_str.push_str(", client = type::thing('client', $client_id)");
    }
    if assigned_to.is_some() {
        query_str.push_str(", assigned_to = type::thing('employee', $assigned_to), status = 'assigned'");
    }

    let ticket: Option<ServiceTicket> = state
        .db
        .query(&query_str)
        .bind(("title", title))
        .bind(("description", description))
        .bind(("priority", priority))
        .bind(("scheduled_date", scheduled_date))
        .bind(("location", location))
        .bind(("notes", notes))
        .bind(("client_id", client_id))
        .bind(("assigned_to", assigned_to))
        .bind(("org_id", org_id_val))
        .await?
        .take(0)?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn update_ticket_status(
    state: &AppState,
    id: &str,
    status: &str,
    notes: Option<String>,
) -> Result<ServiceTicket, DbError> {
    let mut query_str = format!(
        "UPDATE type::thing('service_ticket', $id) SET status = $status"
    );
    if status == "completed" {
        query_str.push_str(", completed_date = time::now()");
    }
    if notes.is_some() {
        query_str.push_str(", notes = $notes");
    }

    let ticket: Option<ServiceTicket> = state
        .db
        .query(&query_str)
        .bind(("id", id.to_string()))
        .bind(("status", status.to_string()))
        .bind(("notes", notes))
        .await?
        .take(0)?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn delete_ticket(state: &AppState, id: &str) -> Result<ServiceTicket, DbError> {
    let ticket: Option<ServiceTicket> = state
        .db
        .query("UPDATE type::thing('service_ticket', $id) SET is_archived = true")
        .bind(("id", id.to_string()))
        .await?
        .take(0)?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn get_tickets_by_employee(
    state: &AppState,
    emp_id: &str,
) -> Result<Vec<ServiceTicket>, DbError> {
    let tickets: Vec<ServiceTicket> = state
        .db
        .query(
            "SELECT * FROM service_ticket WHERE assigned_to = type::thing('employee', $emp_id) \
             AND is_archived = false ORDER BY created_at DESC",
        )
        .bind(("emp_id", emp_id.to_string()))
        .await?
        .take(0)?;
    Ok(tickets)
}
