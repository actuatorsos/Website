use super::models::*;
use super::repository as repo;
use crate::db::AppState;
use crate::db::DbError;
use crate::models::CurrentUser;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{
    Router,
    extract::{Path, Request, State},
    response::Json,
    routing::{delete as axum_delete, get, post, put},
};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

async fn list_projects(
    State(s): State<AppState>,
    request: Request,
) -> Result<Json<Vec<Project>>, DbError> {
    let user = request.extensions().get::<CurrentUser>().cloned();
    let (user_id, user_role) = match &user {
        Some(u) => (u.id.as_str(), format!("{}", u.role)),
        None => ("", "admin".to_string()),
    };
    Ok(Json(repo::get_all_projects(&s, user_id, &user_role).await?))
}
async fn create_project(
    State(s): State<AppState>,
    request: Request,
) -> Result<Json<serde_json::Value>, DbError> {
    let user = request.extensions().get::<CurrentUser>().cloned();
    let creator_id = user.as_ref().map(|u| u.id.clone()).unwrap_or_default();

    // Parse body manually since we already consumed the request for extensions
    let body = axum::body::to_bytes(request.into_body(), 1024 * 64)
        .await
        .map_err(|_| DbError::Validation("Invalid request body".to_string()))?;
    let req: CreateProjectRequest = serde_json::from_slice(&body)
        .map_err(|e| DbError::Validation(format!("Invalid JSON: {}", e)))?;

    let project = repo::create_project(&s, req, &creator_id).await?;
    // Auto-create default board + 4 lists
    let project_id = project
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .unwrap_or_default();
    if !project_id.is_empty() {
        let _ = repo::create_default_board(&s, &project_id).await;
    }
    Ok(Json(serde_json::to_value(&project).unwrap_or_default()))
}
async fn get_project(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, DbError> {
    Ok(Json(
        repo::get_project(&s, id.trim_start_matches("project:")).await?,
    ))
}
async fn delete_project(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    repo::delete_project(&s, id.trim_start_matches("project:")).await?;
    Ok(Json(()))
}
async fn get_project_boards(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Board>>, DbError> {
    Ok(Json(
        repo::get_project_boards(&s, id.trim_start_matches("project:")).await?,
    ))
}
async fn create_board(
    State(s): State<AppState>,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<Board>, DbError> {
    Ok(Json(repo::create_board(&s, req).await?))
}

// Returns board lists WITH embedded cards — the main endpoint the Kanban board uses
async fn get_board_lists_with_cards(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<BoardListWithCards>>, DbError> {
    Ok(Json(
        repo::get_board_lists_with_cards(&s, id.trim_start_matches("board:")).await?,
    ))
}

// Helper to fire board events
async fn fire_board_event(state: &AppState, board_id: &str, event_type: &str) {
    if !board_id.is_empty() {
        let _ = state.board_events.send(crate::db::BoardEvent {
            board_id: board_id.to_string(),
            event_type: event_type.to_string(),
            payload: serde_json::json!({ "timestamp": chrono::Utc::now().to_rfc3339() }),
        });
    }
}

// Create list via POST /boards/{id}/lists (matches frontend JS)
async fn create_board_list_for_board(
    State(s): State<AppState>,
    Path(board_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<BoardList>, DbError> {
    let clean_board_id = board_id.trim_start_matches("board:");
    let title = req
        .get("name")
        .or_else(|| req.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let color = req
        .get("color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let list = repo::create_board_list(
        &s,
        CreateBoardListRequest {
            board_id: clean_board_id.to_string(),
            title,
            color,
        },
    )
    .await?;
    fire_board_event(&s, clean_board_id, "list_created").await;
    Ok(Json(list))
}

async fn create_board_list(
    State(s): State<AppState>,
    Json(req): Json<CreateBoardListRequest>,
) -> Result<Json<BoardList>, DbError> {
    let clean_board_id = req.board_id.trim_start_matches("board:").to_string();
    let mut req = req;
    req.board_id = clean_board_id.clone();
    let list = repo::create_board_list(&s, req).await?;
    fire_board_event(&s, &clean_board_id, "list_created").await;
    Ok(Json(list))
}
async fn get_list_cards(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Card>>, DbError> {
    Ok(Json(
        repo::get_list_cards(&s, id.trim_start_matches("board_list:")).await?,
    ))
}

// Create card via POST /board-lists/{id}/cards (matches frontend JS)
async fn create_card_for_list(
    State(s): State<AppState>,
    Path(list_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<Card>, DbError> {
    let clean_list_id = list_id.trim_start_matches("board_list:");
    let title = req
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = req
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let priority = req
        .get("priority")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let due_date = req
        .get("due_date")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let estimated_hours = req.get("estimated_hours").and_then(|v| v.as_f64());
    let board_id = repo::get_board_id_from_list(&s, clean_list_id).await?;
    let card = repo::create_card(
        &s,
        CreateCardRequest {
            board_list_id: clean_list_id.to_string(),
            title,
            description,
            priority,
            due_date,
            estimated_hours,
        },
    )
    .await?;
    fire_board_event(&s, &board_id, "card_created").await;
    Ok(Json(card))
}

async fn create_card(
    State(s): State<AppState>,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<Card>, DbError> {
    let list_id = req.board_list_id.clone();
    let card = repo::create_card(&s, req).await?;
    if let Ok(board_id) = repo::get_board_id_from_list(&s, &list_id).await {
        fire_board_event(&s, &board_id, "card_created").await;
    }
    Ok(Json(card))
}

// GET single card
async fn get_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Card>, DbError> {
    Ok(Json(
        repo::get_card(&s, id.trim_start_matches("card:")).await?,
    ))
}

// PUT update card
async fn update_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<Card>, DbError> {
    let clean_id = id.trim_start_matches("card:");
    let board_id = repo::get_board_id_from_card(&s, clean_id).await?;
    let card = repo::update_card(&s, clean_id, req).await?;
    fire_board_event(&s, &board_id, "card_updated").await;
    Ok(Json(card))
}

async fn move_card(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<Card>, DbError> {
    let clean_id = id.trim_start_matches("card:");
    let board_id = repo::get_board_id_from_card(&s, clean_id).await?;
    let card = repo::move_card(&s, clean_id, req).await?;
    fire_board_event(&s, &board_id, "card_moved").await;
    Ok(Json(card))
}
async fn toggle_complete(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Card>, DbError> {
    let clean_id = id.trim_start_matches("card:");
    let board_id = repo::get_board_id_from_card(&s, clean_id).await?;
    let card = repo::toggle_card_complete(&s, clean_id).await?;
    fire_board_event(&s, &board_id, "card_updated").await;
    Ok(Json(card))
}

async fn delete_card_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    let clean_id = id.trim_start_matches("card:");
    let board_id = repo::get_board_id_from_card(&s, clean_id).await?;
    repo::delete_card(&s, clean_id).await?;
    fire_board_event(&s, &board_id, "card_deleted").await;
    Ok(Json(()))
}

async fn delete_board_list_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, DbError> {
    let clean_id = id.trim_start_matches("board_list:");
    let board_id = repo::get_board_id_from_list(&s, clean_id).await?;
    repo::delete_board_list(&s, clean_id).await?;
    fire_board_event(&s, &board_id, "list_deleted").await;
    Ok(Json(()))
}

async fn add_comment(
    State(s): State<AppState>,
    Json(req): Json<CreateCardCommentRequest>,
) -> Result<Json<CardComment>, DbError> {
    Ok(Json(repo::add_card_comment(&s, req).await?))
}

#[derive(serde::Deserialize)]
struct CreateChecklistBody {
    title: String,
}

async fn create_checklist(
    State(s): State<AppState>,
    Path(card_id): Path<String>,
    Json(body): Json<CreateChecklistBody>,
) -> Result<Json<CardChecklist>, DbError> {
    Ok(Json(
        repo::create_checklist(&s, card_id.trim_start_matches("card:"), &body.title).await?,
    ))
}
async fn toggle_checklist_item(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ChecklistItem>, DbError> {
    Ok(Json(
        repo::toggle_checklist_item(&s, id.trim_start_matches("checklist_item:")).await?,
    ))
}

// ============================================================================
// Members — أعضاء المشروع
// ============================================================================

async fn list_members(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ProjectMemberWithAccount>>, DbError> {
    Ok(Json(
        repo::get_project_members(&s, id.trim_start_matches("project:")).await?,
    ))
}

async fn add_member_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<ProjectMember>, DbError> {
    let role = req.role.unwrap_or_else(|| "member".to_string());
    let clean_project_id = id.trim_start_matches("project:");
    let clean_member_id = req.member_id.trim_start_matches("account:");
    Ok(Json(
        repo::add_project_member(&s, clean_project_id, clean_member_id, &role).await?,
    ))
}

async fn remove_member_handler(
    State(s): State<AppState>,
    Path((project_id, member_id)): Path<(String, String)>,
) -> Result<Json<()>, DbError> {
    repo::remove_project_member(
        &s,
        project_id.trim_start_matches("project:"),
        member_id.trim_start_matches("account:"),
    )
    .await?;
    Ok(Json(()))
}

/// List all accounts for the member picker
async fn list_all_accounts(
    State(s): State<AppState>,
) -> Result<Json<Vec<crate::models::Account>>, DbError> {
    Ok(Json(s.get_all_accounts().await?))
}

// ============================================================================
// Real-Time Events (SSE)
// ============================================================================

async fn board_events_handler(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let board_id = id.trim_start_matches("board:").to_string();
    let rx = s.board_events.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        match result {
            Ok(ev) => {
                if ev.board_id == board_id {
                    let json = serde_json::to_string(&ev.payload).unwrap_or_default();
                    Some(Ok(Event::default().event(ev.event_type).data(json)))
                } else {
                    None
                }
            }
            Err(_) => None, // receiver lagged
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/{id}", get(get_project).delete(delete_project))
        .route("/projects/{id}/boards", get(get_project_boards))
        .route(
            "/projects/{id}/members",
            get(list_members).post(add_member_handler),
        )
        .route(
            "/projects/{id}/members/{member_id}",
            axum_delete(remove_member_handler),
        )
        .route("/boards", post(create_board))
        .route(
            "/boards/{id}/lists",
            get(get_board_lists_with_cards).post(create_board_list_for_board),
        )
        .route("/boards/{id}/events", get(board_events_handler))
        .route("/board-lists", post(create_board_list))
        .route("/board-lists/{id}", axum_delete(delete_board_list_handler))
        .route(
            "/board-lists/{id}/cards",
            get(get_list_cards).post(create_card_for_list),
        )
        .route("/cards", post(create_card))
        .route(
            "/cards/{id}",
            get(get_card).put(update_card).delete(delete_card_handler),
        )
        .route("/cards/{id}/move", post(move_card))
        .route("/cards/{id}/toggle", post(toggle_complete))
        .route("/cards/{id}/complete", post(toggle_complete))
        .route("/cards/{id}/reopen", post(toggle_complete))
        .route("/cards/{id}/checklists", post(create_checklist))
        .route("/card-comments", post(add_comment))
        .route("/checklist-items/{id}/toggle", put(toggle_checklist_item))
        .route("/accounts", get(list_all_accounts))
}
