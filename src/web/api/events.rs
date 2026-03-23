//! Events API Endpoints

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::db::AppState;

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub event_type: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: Option<surrealdb::sql::Thing>,
    pub title: String,
    pub event_type: String,
    pub start_date: String,
    pub end_date: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub status: String,
}

async fn create_event(
    State(state): State<AppState>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    if req.title.trim().is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Title is required"})),
        );
    }

    let event_type = req.event_type.unwrap_or_else(|| "meetup".to_string());

    let result: Result<Vec<Event>, _> = state
        .db
        .query(
            "CREATE event SET \
                title = $title, \
                event_type = $event_type, \
                start_date = <datetime>$start_date, \
                end_date = <datetime>$end_date, \
                location = $location, \
                description = $description, \
                status = 'draft', \
                registered_count = 0, \
                is_free = true, \
                price = 0.0, \
                created_at = time::now(), \
                updated_at = time::now()",
        )
        .bind(("title", req.title))
        .bind(("event_type", event_type))
        .bind(("start_date", req.start_date))
        .bind(("end_date", req.end_date))
        .bind(("location", req.location))
        .bind(("description", req.description))
        .await
        .and_then(|mut r| r.take(0));

    match result {
        Ok(events) => {
            if let Some(ev) = events.into_iter().next() {
                (axum::http::StatusCode::CREATED, Json(serde_json::json!(ev)))
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Failed to create event"})),
                )
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("{}", e)})),
        ),
    }
}

async fn list_events(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result: Result<Vec<Event>, _> = state
        .db
        .query("SELECT * FROM event ORDER BY start_date DESC")
        .await
        .and_then(|mut r| r.take(0));

    match result {
        Ok(events) => Json(serde_json::json!(events)),
        Err(e) => Json(serde_json::json!({"error": format!("{}", e)})),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_event))
        .route("/", get(list_events))
}
