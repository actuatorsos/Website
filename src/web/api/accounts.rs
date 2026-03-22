// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Account Management API
//!
//! نقاط النهاية لإدارة الحسابات (admin فقط)

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch},
};
use serde::Deserialize;

use crate::db::{self, AppState};
use crate::models::{AuthUser, CurrentUser, SignupRequest, hash_password, verify_password};

/// Configure account management routes
/// Admin-only: list, create, role, status
/// Profile routes (update_profile, delete_own_account) are user-level
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_accounts))
        .route("/", axum::routing::post(create_account))
        .route("/role", patch(update_role))
        .route("/status", patch(update_status))
        .route("/profile", patch(update_profile))
        .route("/profile", delete(delete_own_account))
}

/// GET /api/accounts — list accounts filtered by caller's organization
async fn list_accounts(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> impl IntoResponse {
    // Filter by organization — admin only sees their own org's accounts
    let result = if let Some(ref org) = user.organization_id {
        state.db
            .query("SELECT * FROM account WHERE organization = type::record($org) ORDER BY created_at DESC")
            .bind(("org", org.clone()))
            .await
    } else {
        state.db
            .query("SELECT * FROM account ORDER BY created_at DESC")
            .await
    };

    match result {
        Ok(mut r) => {
            let accounts: Vec<crate::models::Account> = r.take(0).unwrap_or_default();
            let users: Vec<AuthUser> = accounts.iter().map(AuthUser::from).collect();
            (StatusCode::OK, Json(serde_json::json!({ "accounts": users }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct UpdateRoleRequest {
    id: String,
    role: String,
}

/// PATCH /api/accounts/role — update account role (same org only)
async fn update_role(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    // Validate role
    let valid_roles = ["admin", "manager", "employee", "intern", "applicant"];
    if !valid_roles.contains(&req.role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid role" })),
        )
            .into_response();
    }

    // Verify target account belongs to same organization
    if let Some(ref org) = user.organization_id {
        if let Ok(target) = state.get_account_by_id(&req.id).await {
            let target_org = target.organization.as_ref().map(|o| format!("{}:{}", o.tb, o.id));
            if target_org.as_deref() != Some(org) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Cannot modify accounts from another organization" })),
                ).into_response();
            }
        }
    }

    match state.update_account_role(&req.id, &req.role).await {
        Ok(()) => {
            let _ = db::audit_log(
                &state.db,
                Some(&user.email),
                "update",
                "account",
                Some(&req.id),
                None,
                Some(serde_json::json!({ "role": req.role })),
            )
            .await;
            (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct UpdateStatusRequest {
    id: String,
    is_active: bool,
}

/// PATCH /api/accounts/status — enable/disable account (same org only)
async fn update_status(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    // Verify target account belongs to same organization
    if let Some(ref org) = user.organization_id {
        if let Ok(target) = state.get_account_by_id(&req.id).await {
            let target_org = target.organization.as_ref().map(|o| format!("{}:{}", o.tb, o.id));
            if target_org.as_deref() != Some(org) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "error": "Cannot modify accounts from another organization" })),
                ).into_response();
            }
        }
    }

    match state.set_account_active(&req.id, req.is_active).await {
        Ok(()) => {
            let _ = db::audit_log(
                &state.db,
                Some(&user.email),
                "update",
                "account",
                Some(&req.id),
                None,
                Some(serde_json::json!({ "is_active": req.is_active })),
            )
            .await;
            (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountAdminRequest {
    pub email: String,
    pub password: String,
    pub role: String,
}

/// POST /api/accounts - create a new account by admin
async fn create_account(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateAccountAdminRequest>,
) -> impl IntoResponse {
    let valid_roles = ["admin", "manager", "employee", "intern", "applicant"];
    if !valid_roles.contains(&req.role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid role" })),
        )
            .into_response();
    }

    // Validate password length (must match LoginRequest minimum of 8)
    if req.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Password must be at least 8 characters" })),
        )
            .into_response();
    }

    // Check if email already exists
    if state.get_account_by_email(&req.email).await.is_ok() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Email already registered"})),
        )
            .into_response();
    }

    let password = req.password.clone();
    let hashed = match tokio::task::spawn_blocking(move || hash_password(&password)).await {
        Ok(Ok(h)) => h,
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Hashing failed"})),
            )
                .into_response();
        }
    };

    let signup_req = SignupRequest {
        email: req.email,
        password: req.password,
        full_name: None,
        phone: None,
        user_type: "member".to_string(),
    };

    // Create account (initially applicant)
    let account = match state.create_account(&signup_req, &hashed).await {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Update to requested role
    let _ = state
        .update_account_role(&account.id_string(), &req.role)
        .await;

    // Assign to creator's organization
    if let Some(ref org) = user.organization_id {
        let _ = state.db
            .query("UPDATE type::thing('account', $id) SET organization = type::record($org)")
            .bind(("id", account.id_string()))
            .bind(("org", org.clone()))
            .await;
    }

    let _ = db::audit_log(
        &state.db,
        Some(&user.email),
        "create",
        "account",
        Some(&account.id_string()),
        None,
        Some(serde_json::json!({ "email": signup_req.email, "role": req.role })),
    )
    .await;

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "success": true, "message": "Account created" })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub avatar: Option<String>,
    pub current_password: Option<String>,
    pub new_password: Option<String>,
    pub full_name: Option<String>,
    pub phone: Option<String>,
}

/// PATCH /api/accounts/profile - update own profile avatar & password
async fn update_profile(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let account = match state.get_account_by_id(&user.id).await {
        Ok(a) => a,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Account not found"})),
            )
                .into_response();
        }
    };

    // Handle password change
    if let (Some(cur_pass), Some(new_pass)) = (&req.current_password, &req.new_password) {
        if new_pass.len() < 8 {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Password must be at least 8 characters"})),
            )
                .into_response();
        }

        let is_valid = match tokio::task::spawn_blocking({
            let h = account.password_hash.clone();
            let c = cur_pass.clone();
            move || verify_password(&c, &h)
        })
        .await
        {
            Ok(Ok(valid)) => valid,
            _ => false,
        };

        if !is_valid {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid current password"})),
            )
                .into_response();
        }

        let hashed = match tokio::task::spawn_blocking({
            let n = new_pass.clone();
            move || hash_password(&n)
        })
        .await
        {
            Ok(Ok(h)) => h,
            _ => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Hashing failed"})),
                )
                    .into_response();
            }
        };

        if let Err(e) = state.update_account_password(&user.id, &hashed).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    // Handle avatar change
    if let Some(avatar) = req.avatar {
        if let Err(e) = state.update_account_avatar(&user.id, &avatar).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    // Handle full_name / phone update
    if req.full_name.is_some() || req.phone.is_some() {
        if let Err(e) = state
            .update_account_info(&user.id, req.full_name.as_deref(), req.phone.as_deref())
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response()
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

/// DELETE /api/accounts/profile - delete own account
async fn delete_own_account(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<DeleteAccountRequest>,
) -> impl IntoResponse {
    let account = match state.get_account_by_id(&user.id).await {
        Ok(a) => a,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Account not found"})),
            )
                .into_response();
        }
    };

    // Verify password before deletion
    let is_valid = match tokio::task::spawn_blocking({
        let h = account.password_hash.clone();
        let p = req.password.clone();
        move || verify_password(&p, &h)
    })
    .await
    {
        Ok(Ok(valid)) => valid,
        _ => false,
    };

    if !is_valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid password"})),
        )
            .into_response();
    }

    // Don't allow platform admins to delete themselves
    if account.role == crate::models::AccountRole::Admin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin accounts cannot be self-deleted"})),
        )
            .into_response();
    }

    if let Err(e) = state.delete_account(&user.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response();
    }

    let _ = db::audit_log(
        &state.db,
        Some(&user.email),
        "delete",
        "account",
        Some(&user.id),
        None,
        None,
    )
    .await;

    (StatusCode::OK, Json(serde_json::json!({ "deleted": true }))).into_response()
}
