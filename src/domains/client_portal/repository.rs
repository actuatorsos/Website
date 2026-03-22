//! Client Portal Repository — عمليات بوابة العميل في قاعدة البيانات

use super::models::*;
use crate::db::{AppState, DbError};

pub async fn get_all_tickets(state: &AppState, org_id: Option<&str>) -> Result<Vec<SupportTicket>, DbError> {
    let tickets: Vec<SupportTicket> = if let Some(org) = org_id {
        state
            .db
            .query(
                "SELECT * FROM support_ticket WHERE (is_archived = false OR is_archived = NONE) AND organization = type::record($org) ORDER BY created_at DESC",
            )
            .bind(("org", org.to_string()))
            .await?
            .take(0)?
    } else {
        state
            .db
            .query(
                "SELECT * FROM support_ticket WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC",
            )
            .await?
            .take(0)?
    };
    Ok(tickets)
}

pub async fn get_ticket_by_id(
    state: &AppState,
    id: &str,
) -> Result<SupportTicket, DbError> {
    let ticket: Option<SupportTicket> = state
        .db
        .query("SELECT * FROM type::thing('support_ticket', $id)")
        .bind(("id", id.to_string()))
        .await?
        .take(0)?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn create_ticket(
    state: &AppState,
    req: CreateTicketRequest,
    submitted_by: &str,
) -> Result<SupportTicket, DbError> {
    let subject = req.subject;
    let desc = req.description;
    let client_id = req.client_id;
    let category = req.category.unwrap_or_else(|| "general".to_string());
    let priority = req.priority.unwrap_or_else(|| "medium".to_string());
    let submitted = submitted_by.to_string();

    let query = if let Some(ref cid) = client_id {
        let _ = cid;
        "CREATE support_ticket SET \
         subject = $subject, description = $desc, \
         client = type::thing('client', $client_id), \
         submitted_by = $submitted_by, \
         category = $category, priority = $priority, \
         status = 'new'"
    } else {
        "CREATE support_ticket SET \
         subject = $subject, description = $desc, \
         submitted_by = $submitted_by, \
         category = $category, priority = $priority, \
         status = 'new'"
    };

    let ticket: Option<SupportTicket> = state
        .db
        .query(query)
        .bind(("subject", subject))
        .bind(("desc", desc))
        .bind(("client_id", client_id.unwrap_or_default()))
        .bind(("submitted_by", submitted))
        .bind(("category", category))
        .bind(("priority", priority))
        .await?
        .take(0)?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn add_reply(
    state: &AppState,
    ticket_id: &str,
    message: &str,
    author: &str,
    author_role: &str,
) -> Result<TicketReply, DbError> {
    let tid = ticket_id.to_string();
    let msg = message.to_string();
    let auth = author.to_string();
    let role = author_role.to_string();

    let reply: Option<TicketReply> = state
        .db
        .query(
            "CREATE ticket_reply SET \
             ticket = type::thing('support_ticket', $tid), \
             author = $author, author_role = $role, \
             message = $msg",
        )
        .bind(("tid", tid))
        .bind(("author", auth))
        .bind(("role", role))
        .bind(("msg", msg))
        .await?
        .take(0)?;
    reply.ok_or(DbError::NotFound)
}

pub async fn update_ticket_status(
    state: &AppState,
    id: &str,
    status: &str,
    resolution: Option<String>,
) -> Result<SupportTicket, DbError> {
    let now = chrono::Utc::now().to_rfc3339();
    let ticket: Option<SupportTicket> = state
        .db
        .update(("support_ticket", id))
        .merge(serde_json::json!({
            "status": status,
            "resolution": resolution,
            "updated_at": now,
        }))
        .await?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn close_ticket(
    state: &AppState,
    id: &str,
    resolution: Option<String>,
    rating: Option<i64>,
) -> Result<SupportTicket, DbError> {
    let now = chrono::Utc::now().to_rfc3339();
    let ticket: Option<SupportTicket> = state
        .db
        .update(("support_ticket", id))
        .merge(serde_json::json!({
            "status": "closed",
            "resolution": resolution,
            "rating": rating,
            "updated_at": now,
            "closed_at": now,
        }))
        .await?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn delete_ticket(state: &AppState, id: &str) -> Result<SupportTicket, DbError> {
    let ticket: Option<SupportTicket> = state
        .db
        .update(("support_ticket", id))
        .merge(serde_json::json!({
            "is_archived": true,
        }))
        .await?;
    ticket.ok_or(DbError::NotFound)
}

pub async fn get_ticket_replies(
    state: &AppState,
    ticket_id: &str,
) -> Result<Vec<TicketReply>, DbError> {
    let tid = ticket_id.to_string();
    let replies: Vec<TicketReply> = state
        .db
        .query(
            "SELECT * FROM ticket_reply WHERE ticket = type::thing('support_ticket', $tid) AND (is_archived = false OR is_archived = NONE) ORDER BY created_at ASC",
        )
        .bind(("tid", tid))
        .await?
        .take(0)?;
    Ok(replies)
}
