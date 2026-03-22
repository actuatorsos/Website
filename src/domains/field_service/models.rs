//! Field Service Models — نماذج الخدمة الميدانية

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceTicket {
    pub id: Option<Thing>,
    pub title: String,
    pub description: Option<String>,
    pub client: Option<Thing>,
    pub assigned_to: Option<Thing>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub scheduled_date: Option<String>,
    pub completed_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    // Enriched fields from FETCH
    pub client_name: Option<String>,
    pub assigned_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub title: String,
    pub description: Option<String>,
    pub client_id: Option<String>,
    pub assigned_to: Option<String>,
    pub priority: Option<String>,
    pub scheduled_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketStatus {
    pub status: String,
    pub notes: Option<String>,
}
