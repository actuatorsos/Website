//! Organizations API Endpoints

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::db::AppState;

#[derive(Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub org_type: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Organization {
    pub id: Option<surrealdb::sql::Thing>,
    pub name: String,
    pub org_type: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub status: String,
}

async fn create_organization(
    State(state): State<AppState>,
    Json(req): Json<CreateOrgRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Name is required"})),
        );
    }

    let org_type = req.org_type.unwrap_or_else(|| "startup".to_string());

    let result: Result<Vec<Organization>, _> = state
        .db
        .query(
            "CREATE organization SET \
                name = $name, \
                org_type = $org_type, \
                email = $email, \
                phone = $phone, \
                description = $description, \
                status = 'pending', \
                created_at = time::now(), \
                updated_at = time::now()",
        )
        .bind(("name", req.name))
        .bind(("org_type", org_type))
        .bind(("email", req.email))
        .bind(("phone", req.phone))
        .bind(("description", req.description))
        .await
        .and_then(|mut r| r.take(0));

    match result {
        Ok(orgs) => {
            if let Some(org) = orgs.into_iter().next() {
                (axum::http::StatusCode::CREATED, Json(serde_json::json!(org)))
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create organization"})),
                )
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{}", e)})),
        ),
    }
}

async fn list_organizations(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result: Result<Vec<Organization>, _> = state
        .db
        .query("SELECT * FROM organization ORDER BY created_at DESC")
        .await
        .and_then(|mut r| r.take(0));

    match result {
        Ok(orgs) => Json(serde_json::json!(orgs)),
        Err(e) => Json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_organization))
        .route("/", get(list_organizations))
}
