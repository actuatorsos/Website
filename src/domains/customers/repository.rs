//! Customer Domain Database Logic
//!
//! فصل العمليات المتعلقة بقواعد البيانات للعملاء

use super::models::{Client, ClientStatus, CreateClientRequest};
use crate::db::{AppState, DbError};

pub async fn create_client(state: &AppState, req: CreateClientRequest) -> Result<Client, DbError> {
    let client: Option<Client> = state
        .db
        .create("client")
        .content(req)
        .await?
        .unwrap_or(None);
    let created = client.ok_or(DbError::NotFound)?;
    // Audit log
    crate::db::audit_log(
        &state.db,
        None,
        "create",
        "client",
        created.id.as_ref().map(|t| t.id.to_raw()).as_deref(),
        None,
        None,
    )
    .await?;
    Ok(created)
}

pub async fn get_all_clients(state: &AppState) -> Result<Vec<Client>, DbError> {
    let clients: Vec<Client> = state.db
        .query("SELECT * FROM client WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
        .await?
        .take(0)?;
    Ok(clients)
}

pub async fn get_client(state: &AppState, id: &str) -> Result<Client, DbError> {
    let client: Option<Client> = state.db.select::<Option<Client>>(("client", id)).await?;
    client.ok_or(DbError::NotFound)
}

/// Soft delete — أرشفة العميل بدلاً من حذفه
pub async fn delete_client(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "client", id).await?;
    crate::db::audit_log(&state.db, None, "delete", "client", Some(id), None, None).await?;
    Ok(())
}

pub async fn update_client_status(
    state: &AppState,
    id: &str,
    status: ClientStatus,
) -> Result<Client, DbError> {
    let client: Option<Client> = state
        .db
        .update::<Option<Client>>(("client", id))
        .merge(serde_json::json!({ "status": status }))
        .await?;
    client.ok_or(DbError::NotFound)
}
