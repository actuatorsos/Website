use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post, put},
};

async fn list_contacts(State(s): State<AppState>) -> Result<Json<Vec<Contact>>, DbError> {
    // Return all contacts
    let contacts: Vec<Contact> = s.db.query("SELECT * FROM contact WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC").await?.take(0)?;
    Ok(Json(contacts))
}
async fn create_contact(
    State(s): State<AppState>,
    Json(req): Json<CreateContactRequest>,
) -> Result<Json<Contact>, DbError> {
    Ok(Json(repo::create_contact(&s, req).await?))
}
async fn get_client_contacts(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Contact>>, DbError> {
    Ok(Json(repo::get_contacts_by_client(&s, &id).await?))
}
async fn delete_contact(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_contact(&s, &id).await?;
    Ok(Json(()))
}
async fn create_interaction(
    State(s): State<AppState>,
    Json(req): Json<CreateInteractionRequest>,
) -> Result<Json<Interaction>, DbError> {
    Ok(Json(repo::create_interaction(&s, req).await?))
}
async fn get_client_interactions(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Interaction>>, DbError> {
    Ok(Json(repo::get_interactions_by_client(&s, &id).await?))
}
async fn list_opportunities(State(s): State<AppState>) -> Result<Json<Vec<Opportunity>>, DbError> {
    Ok(Json(repo::get_all_opportunities(&s).await?))
}
async fn create_opportunity(
    State(s): State<AppState>,
    Json(req): Json<CreateOpportunityRequest>,
) -> Result<Json<Opportunity>, DbError> {
    Ok(Json(repo::create_opportunity(&s, req).await?))
}
async fn get_opportunity(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Opportunity>, DbError> {
    Ok(Json(repo::get_opportunity(&s, &id).await?))
}
async fn update_opportunity_stage(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOpportunityStageRequest>,
) -> Result<Json<Opportunity>, DbError> {
    Ok(Json(repo::update_opportunity_stage(&s, &id, req).await?))
}
async fn delete_opportunity(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_opportunity(&s, &id).await?;
    Ok(Json(()))
}
async fn list_quotations(State(s): State<AppState>) -> Result<Json<Vec<Quotation>>, DbError> {
    Ok(Json(repo::get_all_quotations(&s).await?))
}
async fn create_quotation(
    State(s): State<AppState>,
    Json(req): Json<CreateQuotationRequest>,
) -> Result<Json<Quotation>, DbError> {
    Ok(Json(repo::create_quotation(&s, req).await?))
}
async fn get_quotation(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Quotation>, DbError> {
    Ok(Json(repo::get_quotation(&s, &id).await?))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/contacts", get(list_contacts).post(create_contact))
        .route("/contacts/{id}", delete(delete_contact))
        .route("/clients/{id}/contacts", get(get_client_contacts))
        .route("/interactions", post(create_interaction))
        .route("/clients/{id}/interactions", get(get_client_interactions))
        .route(
            "/opportunities",
            get(list_opportunities).post(create_opportunity),
        )
        .route(
            "/opportunities/{id}",
            get(get_opportunity).delete(delete_opportunity),
        )
        .route("/opportunities/{id}/stage", put(update_opportunity_stage))
        .route("/quotations", get(list_quotations).post(create_quotation))
        .route("/quotations/{id}", get(get_quotation))
}
