//! Projects Advanced Repository — عمليات إدارة المشاريع (Trello Style)

use super::models::*;
use crate::db::{AppState, DbError};

/// Helper: convert "YYYY-MM-DD" to "YYYY-MM-DDT00:00:00Z" for SurrealDB datetime
fn to_iso_datetime(val: Option<String>) -> Option<String> {
    val.and_then(|d| {
        let d = d.trim().to_string();
        if d.is_empty() {
            return None;
        }
        if d.contains('T') {
            Some(d)
        } else {
            Some(format!("{}T00:00:00Z", d))
        }
    })
}

pub async fn create_project(
    state: &AppState,
    req: CreateProjectRequest,
    creator_id: &str,
) -> Result<Project, DbError> {
    let title = req.title;
    let description = req.description;
    let budget = req.budget;
    let start_date = to_iso_datetime(req.start_date);
    let end_date = to_iso_datetime(req.end_date);
    let priority = req.priority.unwrap_or_else(|| "medium".to_string());
    let visibility = req.visibility.unwrap_or_else(|| "team".to_string());

    // Build dynamic query parts for optional datetime fields
    let start_expr = if start_date.is_some() {
        "type::datetime($start)"
    } else {
        "NONE"
    };
    let end_expr = if end_date.is_some() {
        "type::datetime($end)"
    } else {
        "NONE"
    };
    let query = format!(
        "CREATE project SET \
         title = $title, description = $desc, \
         budget = $budget, start_date = {start_expr}, end_date = {end_expr}, \
         priority = $priority, visibility = $vis, \
         status = 'active', spent = 0, progress_percent = 0"
    );

    let p: Option<Project> = state
        .db
        .query(&query)
        .bind(("title", title))
        .bind(("desc", description))
        .bind(("budget", budget))
        .bind(("start", start_date.unwrap_or_default()))
        .bind(("end", end_date.unwrap_or_default()))
        .bind(("priority", priority))
        .bind(("vis", visibility))
        .await?
        .take(0)?;
    let project = p.ok_or(DbError::NotFound)?;

    // Auto-add creator as owner
    let proj_id = project
        .id
        .as_ref()
        .map(|t| t.id.to_raw())
        .unwrap_or_default();
    let _ = add_project_member(state, &proj_id, creator_id, "owner").await;

    Ok(project)
}

pub async fn get_all_projects(
    state: &AppState,
    user_id: &str,
    user_role: &str,
) -> Result<Vec<Project>, DbError> {
    if user_role == "admin" {
        // Admin sees all active projects
        let projs: Vec<Project> = state.db
            .query("SELECT * FROM project WHERE is_archived = false OR is_archived = NONE ORDER BY created_at DESC")
            .await?.take(0)?;
        Ok(projs)
    } else {
        // Others see only projects they are members of
        let projs: Vec<Project> = state
            .db
            .query(
                "SELECT * FROM project WHERE \
                 (is_archived = false OR is_archived = NONE) AND \
                 id IN (SELECT VALUE project FROM project_member WHERE \
                   member = type::thing('account', $uid) AND \
                   (is_archived = false OR is_archived = NONE)) \
                 ORDER BY created_at DESC",
            )
            .bind(("uid", user_id.to_string()))
            .await?
            .take(0)?;
        Ok(projs)
    }
}

pub async fn get_project(state: &AppState, id: &str) -> Result<Project, DbError> {
    let id = id.to_string();
    let p: Option<Project> = state.db.select(("project", id)).await?;
    p.ok_or(DbError::NotFound)
}

pub async fn delete_project(state: &AppState, id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "project", id).await?;
    Ok(())
}

pub async fn create_board(state: &AppState, req: CreateBoardRequest) -> Result<Board, DbError> {
    let project_id = req.project_id;
    let title = req.title;
    let background = req.background;

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM board WHERE project = type::thing('project', $id) GROUP ALL")
        .bind(("id", project_id.clone()))
        .await?
        .take(0)
        .unwrap_or_default();
    let pos = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let b: Option<Board> = state
        .db
        .query(
            "CREATE board SET \
             project = type::thing('project', $project_id), \
             title = $title, background = $bg, \
             position = $pos, is_default = false",
        )
        .bind(("project_id", project_id))
        .bind(("title", title))
        .bind(("bg", background))
        .bind(("pos", pos))
        .await?
        .take(0)?;
    b.ok_or(DbError::NotFound)
}

pub async fn create_default_board(state: &AppState, project_id: &str) -> Result<Board, DbError> {
    let project_id = project_id.to_string();
    let board = create_board(
        state,
        CreateBoardRequest {
            project_id: project_id.clone(),
            title: "لوحة المشروع الرئيسية".to_string(),
            background: Some("#1e40af".to_string()),
        },
    )
    .await?;

    let board_id = board.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();

    let default_lists = vec![
        ("📥 Backlog".to_string(), "#6b7280".to_string()),
        ("🧭 To Do".to_string(), "#3b82f6".to_string()),
        ("⚙️ In Progress".to_string(), "#f59e0b".to_string()),
        ("🔍 Review".to_string(), "#8b5cf6".to_string()),
        ("✅ Done".to_string(), "#10b981".to_string()),
        ("📚 Resources".to_string(), "#06b6d4".to_string()),
    ];

    for (i, (list_title, list_color)) in default_lists.into_iter().enumerate() {
        let bid = board_id.clone();
        let _ = state
            .db
            .query(
                "CREATE board_list SET \
                 board = type::thing('board', $board_id), \
                 title = $title, position = $pos, color = $color, \
                 is_closed = false",
            )
            .bind(("board_id", bid))
            .bind(("title", list_title))
            .bind(("pos", i as i64))
            .bind(("color", list_color))
            .await;
    }

    Ok(board)
}

pub async fn get_project_boards(state: &AppState, project_id: &str) -> Result<Vec<Board>, DbError> {
    let id = project_id.to_string();
    let boards: Vec<Board> = state.db
        .query("SELECT * FROM board WHERE project = type::thing('project', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY position ASC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(boards)
}

pub async fn create_board_list(
    state: &AppState,
    req: CreateBoardListRequest,
) -> Result<BoardList, DbError> {
    let board_id = req.board_id;
    let title = req.title;
    let color = req.color;

    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM board_list WHERE board = type::thing('board', $id) GROUP ALL")
        .bind(("id", board_id.clone()))
        .await?
        .take(0)
        .unwrap_or_default();
    let pos = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let bl: Option<BoardList> = state
        .db
        .query(
            "CREATE board_list SET \
             board = type::thing('board', $board_id), \
             title = $title, position = $pos, color = $color, is_closed = false",
        )
        .bind(("board_id", board_id))
        .bind(("title", title))
        .bind(("pos", pos))
        .bind(("color", color))
        .await?
        .take(0)?;
    bl.ok_or(DbError::NotFound)
}

pub async fn get_board_lists(state: &AppState, board_id: &str) -> Result<Vec<BoardList>, DbError> {
    let id = board_id.to_string();
    let lists: Vec<BoardList> = state.db
        .query("SELECT * FROM board_list WHERE board = type::thing('board', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY position ASC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(lists)
}

pub async fn create_card(state: &AppState, req: CreateCardRequest) -> Result<Card, DbError> {
    let list_id = req.board_list_id;
    let title = req.title;
    let description = req.description;
    let priority = req.priority.unwrap_or_else(|| "medium".to_string());
    // Convert YYYY-MM-DD to ISO 8601 datetime string for SurrealDB datetime type
    let due_date: Option<String> = req.due_date.and_then(|d| {
        let d = d.trim().to_string();
        if d.is_empty() {
            return None;
        }
        // If already contains 'T', assume it's a valid ISO datetime
        if d.contains('T') {
            Some(d)
        } else {
            Some(format!("{}T00:00:00Z", d))
        }
    });
    let estimated_hours = req.estimated_hours;

    let count: Vec<serde_json::Value> = state
        .db
        .query(
            "SELECT count() FROM card WHERE board_list = type::thing('board_list', $id) GROUP ALL",
        )
        .bind(("id", list_id.clone()))
        .await?
        .take(0)
        .unwrap_or_default();
    let pos = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Use IF/ELSE in SurrealQL to conditionally cast due_date
    let query = if due_date.is_some() {
        "CREATE card SET \
         board_list = type::thing('board_list', $list_id), \
         title = $title, description = $desc, \
         position = $pos, priority = $priority, \
         due_date = type::datetime($due_date), estimated_hours = $est_hours, \
         is_complete = false"
    } else {
        "CREATE card SET \
         board_list = type::thing('board_list', $list_id), \
         title = $title, description = $desc, \
         position = $pos, priority = $priority, \
         due_date = NONE, estimated_hours = $est_hours, \
         is_complete = false"
    };

    let card: Option<Card> = state
        .db
        .query(query)
        .bind(("list_id", list_id))
        .bind(("title", title))
        .bind(("desc", description))
        .bind(("pos", pos))
        .bind(("priority", priority))
        .bind(("due_date", due_date.unwrap_or_default()))
        .bind(("est_hours", estimated_hours))
        .await?
        .take(0)?;
    card.ok_or(DbError::NotFound)
}

pub async fn get_list_cards(state: &AppState, list_id: &str) -> Result<Vec<Card>, DbError> {
    let id = list_id.to_string();
    let cards: Vec<Card> = state.db
        .query("SELECT * FROM card WHERE board_list = type::thing('board_list', $id) AND (is_archived = false OR is_archived = NONE) ORDER BY position ASC")
        .bind(("id", id))
        .await?.take(0)?;
    Ok(cards)
}

pub async fn move_card(
    state: &AppState,
    card_id: &str,
    req: MoveCardRequest,
) -> Result<Card, DbError> {
    let id = card_id.to_string();
    let target_list = req.target_list_id;
    let position = req.position;
    // Use SurrealQL with type::thing() to set a proper record link
    let card: Option<Card> = state
        .db
        .query(
            "UPDATE type::thing('card', $id) SET \
             board_list = type::thing('board_list', $target_list), \
             position = $pos",
        )
        .bind(("id", id))
        .bind(("target_list", target_list))
        .bind(("pos", position))
        .await?
        .take(0)?;
    card.ok_or(DbError::NotFound)
}

pub async fn delete_card(state: &AppState, card_id: &str) -> Result<(), DbError> {
    crate::db::soft_delete(&state.db, "card", card_id).await?;
    Ok(())
}

pub async fn delete_board_list(state: &AppState, list_id: &str) -> Result<(), DbError> {
    // Soft-delete all cards in this list first
    let id = list_id.to_string();
    let _: Vec<serde_json::Value> = state
        .db
        .query(
            "UPDATE card SET is_archived = true WHERE board_list = type::thing('board_list', $id)",
        )
        .bind(("id", id))
        .await?
        .take(0)
        .unwrap_or_default();
    crate::db::soft_delete(&state.db, "board_list", list_id).await?;
    Ok(())
}

pub async fn toggle_card_complete(state: &AppState, id: &str) -> Result<Card, DbError> {
    let id = id.to_string();
    let current: Option<Card> = state.db.select(("card", id.clone())).await?;
    let current = current.ok_or(DbError::NotFound)?;
    let new_state = !current.is_complete.unwrap_or(false);

    let card: Option<Card> = state
        .db
        .update(("card", id))
        .merge(serde_json::json!({ "is_complete": new_state }))
        .await?;
    card.ok_or(DbError::NotFound)
}

pub async fn add_card_comment(
    state: &AppState,
    req: CreateCardCommentRequest,
) -> Result<CardComment, DbError> {
    let card_id = req.card_id;
    let author_id = req.author_id;
    let content = req.content;

    let comment: Option<CardComment> = state
        .db
        .query(
            "CREATE card_comment SET \
             card = type::thing('card', $card_id), \
             author = type::thing('account', $author_id), \
             content = $content",
        )
        .bind(("card_id", card_id))
        .bind(("author_id", author_id))
        .bind(("content", content))
        .await?
        .take(0)?;
    comment.ok_or(DbError::NotFound)
}

pub async fn create_checklist(
    state: &AppState,
    card_id: &str,
    title: &str,
) -> Result<CardChecklist, DbError> {
    let card_id = card_id.to_string();
    let title = title.to_string();
    let count: Vec<serde_json::Value> = state
        .db
        .query("SELECT count() FROM card_checklist WHERE card = type::thing('card', $id) GROUP ALL")
        .bind(("id", card_id.clone()))
        .await?
        .take(0)
        .unwrap_or_default();
    let pos = count
        .first()
        .and_then(|v| v.get("count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let cl: Option<CardChecklist> = state.db
        .query("CREATE card_checklist SET card = type::thing('card', $card_id), title = $title, position = $pos")
        .bind(("card_id", card_id))
        .bind(("title", title))
        .bind(("pos", pos))
        .await?.take(0)?;
    cl.ok_or(DbError::NotFound)
}

pub async fn toggle_checklist_item(
    state: &AppState,
    item_id: &str,
) -> Result<ChecklistItem, DbError> {
    let id = item_id.to_string();
    let current: Option<ChecklistItem> = state.db.select(("checklist_item", id.clone())).await?;
    let current = current.ok_or(DbError::NotFound)?;
    let new_state = !current.is_checked.unwrap_or(false);

    let item: Option<ChecklistItem> = state
        .db
        .update(("checklist_item", id))
        .merge(serde_json::json!({ "is_checked": new_state }))
        .await?;
    item.ok_or(DbError::NotFound)
}

/// Get a single card by ID
pub async fn get_card(state: &AppState, card_id: &str) -> Result<Card, DbError> {
    let id = card_id.to_string();
    let card: Option<Card> = state.db.select(("card", id)).await?;
    card.ok_or(DbError::NotFound)
}

/// Update a card's fields
pub async fn update_card(
    state: &AppState,
    card_id: &str,
    req: UpdateCardRequest,
) -> Result<Card, DbError> {
    let id = card_id.to_string();

    // Build dynamic SET clauses for SurrealQL
    let mut set_parts: Vec<String> = Vec::new();
    let mut title_val = String::new();
    let mut desc_val = String::new();
    let mut priority_val = String::new();
    let mut due_val = String::new();
    let mut est_hours_val: Option<f64> = None;

    if let Some(title) = req.title {
        title_val = title;
        set_parts.push("title = $title".to_string());
    }
    if let Some(desc) = req.description {
        desc_val = desc;
        set_parts.push("description = $desc".to_string());
    }
    if let Some(priority) = req.priority {
        priority_val = priority;
        set_parts.push("priority = $priority".to_string());
    }
    if let Some(due) = req.due_date {
        if due.is_empty() {
            set_parts.push("due_date = NONE".to_string());
        } else {
            due_val = to_iso_datetime(Some(due)).unwrap_or_default();
            set_parts.push("due_date = type::datetime($due_date)".to_string());
        }
    }
    if let Some(hours) = req.estimated_hours {
        est_hours_val = Some(hours);
        set_parts.push("estimated_hours = $est_hours".to_string());
    }

    if set_parts.is_empty() {
        return get_card(state, card_id).await;
    }

    let query = format!(
        "UPDATE type::thing('card', $id) SET {}",
        set_parts.join(", ")
    );

    let card: Option<Card> = state
        .db
        .query(&query)
        .bind(("id", id))
        .bind(("title", title_val))
        .bind(("desc", desc_val))
        .bind(("priority", priority_val))
        .bind(("due_date", due_val))
        .bind(("est_hours", est_hours_val))
        .await?
        .take(0)?;
    card.ok_or(DbError::NotFound)
}

/// Get board lists with their cards embedded
pub async fn get_board_lists_with_cards(
    state: &AppState,
    board_id: &str,
) -> Result<Vec<BoardListWithCards>, DbError> {
    let lists = get_board_lists(state, board_id).await?;
    let mut result = Vec::with_capacity(lists.len());
    for list in lists {
        let list_id = list.id.as_ref().map(|t| t.id.to_raw()).unwrap_or_default();
        let cards = get_list_cards(state, &list_id).await.unwrap_or_default();
        result.push(BoardListWithCards {
            id: list.id,
            board: list.board,
            title: list.title,
            position: list.position,
            color: list.color,
            is_closed: list.is_closed,
            cards,
        });
    }
    Ok(result)
}

// ============================================================================
// Project Members — إدارة أعضاء المشروع
// ============================================================================

pub async fn add_project_member(
    state: &AppState,
    project_id: &str,
    member_id: &str,
    role: &str,
) -> Result<ProjectMember, DbError> {
    let pm: Option<ProjectMember> = state
        .db
        .query(
            "CREATE project_member SET \
             project = type::thing('project', $pid), \
             member = type::thing('account', $mid), \
             role = $role",
        )
        .bind(("pid", project_id.to_string()))
        .bind(("mid", member_id.to_string()))
        .bind(("role", role.to_string()))
        .await?
        .take(0)?;
    pm.ok_or(DbError::NotFound)
}

pub async fn remove_project_member(
    state: &AppState,
    project_id: &str,
    member_id: &str,
) -> Result<(), DbError> {
    state
        .db
        .query(
            "DELETE project_member WHERE \
             project = type::thing('project', $pid) AND \
             member = type::thing('account', $mid)",
        )
        .bind(("pid", project_id.to_string()))
        .bind(("mid", member_id.to_string()))
        .await?;
    Ok(())
}

pub async fn get_project_members(
    state: &AppState,
    project_id: &str,
) -> Result<Vec<ProjectMemberWithAccount>, DbError> {
    let members: Vec<ProjectMemberWithAccount> = state.db
        .query(
            "SELECT *, \
             (SELECT VALUE email FROM account WHERE id = $parent.member LIMIT 1)[0] AS member_email, \
             (SELECT VALUE role FROM account WHERE id = $parent.member LIMIT 1)[0] AS member_role \
             FROM project_member \
             WHERE project = type::thing('project', $pid) \
             AND (is_archived = false OR is_archived = NONE) \
             ORDER BY joined_at ASC"
        )
        .bind(("pid", project_id.to_string()))
        .await?.take(0)?;
    Ok(members)
}

pub async fn get_board_id_from_list(state: &AppState, list_id: &str) -> Result<String, DbError> {
    let board: Option<surrealdb::sql::Thing> = state
        .db
        .query("SELECT VALUE board FROM type::thing('board_list', $id)")
        .bind(("id", list_id.to_string()))
        .await?
        .take(0)?;
    Ok(board.map(|t| t.id.to_raw()).unwrap_or_default())
}

pub async fn get_board_id_from_card(state: &AppState, card_id: &str) -> Result<String, DbError> {
    let board: Option<surrealdb::sql::Thing> = state
        .db
        .query("SELECT VALUE board_list.board FROM type::thing('card', $id)")
        .bind(("id", card_id.to_string()))
        .await?
        .take(0)?;
    Ok(board.map(|t| t.id.to_raw()).unwrap_or_default())
}
