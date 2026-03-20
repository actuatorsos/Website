// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Auth API Routes
//!
//! نقاط النهاية للتوثيق: تسجيل الدخول، إنشاء حساب، بيانات المستخدم، تسجيل الخروج

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::db::AppState;
use crate::domains::email::models::SendEmailRequest;
use crate::domains::email::service::send_email;
use crate::middleware::auth::create_token;
use crate::models::{
    AuthResponse, AuthUser, CurrentUser, LoginRequest, SignupRequest, hash_password,
    verify_password,
};

/// Request for password reset
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Request to complete password reset
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// Request to verify email
#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

/// Generate a cryptographically secure random token
fn generate_secure_token() -> String {
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Public auth routes (signup, login, logout — no auth needed)
pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/verify-email", post(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/org-request", post(org_registration_request))
}

/// Protected auth routes (needs JWT — /me)
pub fn protected_routes() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

// ============================================================================
// POST /api/auth/signup — إنشاء حساب جديد
// ============================================================================

async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    // Check if email already exists
    if state.get_account_by_email(&req.email).await.is_ok() {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Email already registered"})),
        )
            .into_response();
    }

    // Hash password (CPU-intensive — offload to blocking thread)
    let password = req.password.clone();
    let hashed = match tokio::task::spawn_blocking(move || hash_password(&password)).await {
        Ok(Ok(h)) => h,
        Ok(Err(e)) => {
            tracing::error!("Password hashing failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Account creation failed"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Blocking task panicked: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Account creation failed"})),
            )
                .into_response();
        }
    };

    // Create account in database
    let account = match state.create_account(&req, &hashed).await {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Failed to create account: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Account creation failed"})),
            )
                .into_response();
        }
    };

    // Send verification email
    let verification_token = generate_secure_token();
    if let Err(e) = state.create_auth_token(
        &account.id_string(),
        &verification_token,
        "email_verification",
        1440, // 24 hours
    ).await {
        tracing::warn!("Failed to create verification token: {}", e);
    } else {
        // Send verification email (fire and forget)
        let state_clone = state.clone();
        let email_clone = req.email.clone();
        let token_clone = verification_token.clone();
        tokio::spawn(async move {
            let verify_url = format!("/verify-email?token={}", token_clone);
            let body = format!(
                "<div style='font-family:monospace;background:#0B1A2A;color:#fff;padding:40px;text-align:center;'>\
                <h2 style='color:#00B7C2;'>Actuators — تأكيد البريد الإلكتروني</h2>\
                <p>مرحباً، يرجى تأكيد بريدك الإلكتروني بالضغط على الزر أدناه:</p>\
                <a href='{}' style='display:inline-block;margin:20px 0;padding:12px 32px;background:#FF6A2A;color:#0B1A2A;font-weight:bold;text-decoration:none;'>تأكيد البريد الإلكتروني</a>\
                <p style='color:rgba(255,255,255,0.5);font-size:12px;'>أو انسخ الرابط: {}</p>\
                </div>",
                verify_url, verify_url
            );
            let email_req = SendEmailRequest {
                recipients: vec![email_clone],
                subject: "Actuators — تأكيد البريد الإلكتروني".to_string(),
                body,
                template_id: None,
                variables: None,
            };
            if let Err(e) = send_email(&state_clone, &email_req).await {
                tracing::warn!("Failed to send verification email: {}", e);
            }
        });
    }

    // Generate JWT token
    let token = match create_token(
        &account.id_string(),
        &account.email,
        &account.role.to_string(),
        &state.jwt_secret,
        state.jwt_expiry_hours,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token creation failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token generation failed"})),
            )
                .into_response();
        }
    };

    let response = AuthResponse {
        token,
        user: AuthUser::from(&account),
    };

    (StatusCode::CREATED, Json(json!(response))).into_response()
}

// ============================================================================
// POST /api/auth/login — تسجيل الدخول
// ============================================================================

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation failed",
                "details": errors.to_string()
            })),
        )
            .into_response();
    }

    // Find account by email
    let account = match state.get_account_by_email(&req.email).await {
        Ok(a) => a,
        Err(_) => {
            // Don't reveal whether email exists — generic error
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid email or password"})),
            )
                .into_response();
        }
    };

    // Check if account is active
    if !account.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Account is deactivated"})),
        )
            .into_response();
    }

    // Verify password (CPU-intensive — offload to blocking thread)
    let password = req.password.clone();
    let stored_hash = account.password_hash.clone();
    let is_valid =
        match tokio::task::spawn_blocking(move || verify_password(&password, &stored_hash)).await {
            Ok(Ok(valid)) => valid,
            Ok(Err(e)) => {
                tracing::error!("Password verification error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Authentication failed"})),
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!("Blocking task panicked: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Authentication failed"})),
                )
                    .into_response();
            }
        };

    if !is_valid {
        // Record failed login attempt
        let ip = headers
            .get("x-real-ip")
            .or_else(|| headers.get("x-forwarded-for"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();
        let email_clone = req.email.clone();
        let db = state.db.clone();
        tokio::spawn(async move {
            let _ = db
                .query(
                    "CREATE login_history SET \
                        email = $email, \
                        ip = $ip, \
                        success = false, \
                        created_at = time::now()",
                )
                .bind(("email", email_clone))
                .bind(("ip", ip))
                .await;
        });

        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid email or password"})),
        )
            .into_response();
    }

    // Update last_login
    if let Err(e) = state.update_last_login(&account.id_string()).await {
        tracing::warn!("Failed to update last_login: {}", e);
    }

    // Record login history
    let ip = headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_string();

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let account_id = account.id_string();
    let db = state.db.clone();
    tokio::spawn(async move {
        let _ = db
            .query(
                "CREATE login_history SET \
                    account = type::thing('account', $account_id), \
                    ip = $ip, \
                    user_agent = $user_agent, \
                    success = true, \
                    created_at = time::now()",
            )
            .bind(("account_id", account_id))
            .bind(("ip", ip))
            .bind(("user_agent", user_agent))
            .await;
    });

    // Generate JWT token
    let token = match create_token(
        &account.id_string(),
        &account.email,
        &account.role.to_string(),
        &state.jwt_secret,
        state.jwt_expiry_hours,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Token creation failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token generation failed"})),
            )
                .into_response();
        }
    };

    let response = AuthResponse {
        token,
        user: AuthUser::from(&account),
    };

    (StatusCode::OK, Json(json!(response))).into_response()
}

// ============================================================================
// GET /api/auth/me — بيانات المستخدم الحالي (يتطلب JWT)
// ============================================================================

async fn me(State(state): State<AppState>, request: axum::extract::Request) -> impl IntoResponse {
    let current_user = match request.extensions().get::<CurrentUser>() {
        Some(u) => u.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated"})),
            )
                .into_response();
        }
    };

    // Fetch full account data from DB
    match state.get_account_by_id(&current_user.id).await {
        Ok(account) => {
            let user = AuthUser::from(&account);
            (StatusCode::OK, Json(json!(user))).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Account not found"})),
        )
            .into_response(),
    }
}

// ============================================================================
// POST /api/auth/logout — تسجيل الخروج (مسح الكوكي)
// ============================================================================

async fn logout() -> impl IntoResponse {
    // Clear auth_token cookie
    let cookie = "auth_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict";

    (
        StatusCode::OK,
        [(axum::http::header::SET_COOKIE, cookie)],
        Json(json!({"message": "Logged out successfully"})),
    )
        .into_response()
}

// ============================================================================
// POST /api/auth/verify-email — تأكيد البريد الإلكتروني
// ============================================================================

async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    match state.validate_auth_token(&req.token, "email_verification").await {
        Ok(account_id) => {
            if let Err(e) = state.set_email_verified(&account_id, true).await {
                tracing::error!("Failed to set email verified: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Verification failed"}))).into_response();
            }
            (StatusCode::OK, Json(json!({"message": "تم تأكيد البريد الإلكتروني بنجاح"}))).into_response()
        }
        Err(_) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": "رابط التأكيد غير صالح أو منتهي الصلاحية"}))).into_response()
        }
    }
}

// ============================================================================
// POST /api/auth/resend-verification — إعادة إرسال رابط التأكيد
// ============================================================================

async fn resend_verification(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    // Always return success to prevent email enumeration
    let response = json!({"message": "إذا كان البريد مسجلاً، سيتم إرسال رابط التأكيد"});

    if let Ok(account) = state.get_account_by_email(&req.email).await {
        if !account.email_verified {
            let token = generate_secure_token();
            if let Ok(()) = state.create_auth_token(
                &account.id_string(), &token, "email_verification", 1440,
            ).await {
                let state_clone = state.clone();
                let email_clone = req.email.clone();
                tokio::spawn(async move {
                    let verify_url = format!("/verify-email?token={}", token);
                    let body = format!(
                        "<div style='font-family:monospace;background:#0B1A2A;color:#fff;padding:40px;text-align:center;'>\
                        <h2 style='color:#00B7C2;'>Actuators — تأكيد البريد الإلكتروني</h2>\
                        <p>يرجى تأكيد بريدك الإلكتروني:</p>\
                        <a href='{}' style='display:inline-block;margin:20px 0;padding:12px 32px;background:#FF6A2A;color:#0B1A2A;font-weight:bold;text-decoration:none;'>تأكيد البريد</a>\
                        <p style='color:rgba(255,255,255,0.5);font-size:12px;'>{}</p></div>",
                        verify_url, verify_url
                    );
                    let _ = send_email(&state_clone, &SendEmailRequest {
                        recipients: vec![email_clone],
                        subject: "Actuators — تأكيد البريد الإلكتروني".to_string(),
                        body,
                        template_id: None,
                        variables: None,
                    }).await;
                });
            }
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// POST /api/auth/forgot-password — طلب إعادة تعيين كلمة المرور
// ============================================================================

async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> impl IntoResponse {
    // Always return success to prevent email enumeration
    let response = json!({"message": "إذا كان البريد مسجلاً، سيتم إرسال رابط إعادة التعيين"});

    if let Ok(account) = state.get_account_by_email(&req.email).await {
        let token = generate_secure_token();
        if let Ok(()) = state.create_auth_token(
            &account.id_string(), &token, "password_reset", 30, // 30 minutes
        ).await {
            let state_clone = state.clone();
            let email_clone = req.email.clone();
            tokio::spawn(async move {
                let reset_url = format!("/reset-password?token={}", token);
                let body = format!(
                    "<div style='font-family:monospace;background:#0B1A2A;color:#fff;padding:40px;text-align:center;'>\
                    <h2 style='color:#00B7C2;'>Actuators — إعادة تعيين كلمة المرور</h2>\
                    <p>لقد طلبت إعادة تعيين كلمة المرور. اضغط على الزر أدناه:</p>\
                    <a href='{}' style='display:inline-block;margin:20px 0;padding:12px 32px;background:#FF6A2A;color:#0B1A2A;font-weight:bold;text-decoration:none;'>إعادة تعيين كلمة المرور</a>\
                    <p style='color:rgba(255,255,255,0.5);font-size:12px;'>هذا الرابط صالح لمدة 30 دقيقة فقط</p>\
                    <p style='color:rgba(255,255,255,0.5);font-size:12px;'>إذا لم تطلب ذلك، تجاهل هذه الرسالة</p></div>",
                    reset_url
                );
                let _ = send_email(&state_clone, &SendEmailRequest {
                    recipients: vec![email_clone],
                    subject: "Actuators — إعادة تعيين كلمة المرور".to_string(),
                    body,
                    template_id: None,
                    variables: None,
                }).await;
            });
        }
    }

    (StatusCode::OK, Json(response)).into_response()
}

// ============================================================================
// POST /api/auth/reset-password — تنفيذ إعادة تعيين كلمة المرور
// ============================================================================

// ============================================================================
// POST /api/auth/org-request — طلب تسجيل منظمة
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OrgRegistrationRequest {
    pub name: String,
    pub org_type: String,
    pub contact_email: String,
    #[serde(default)]
    pub contact_phone: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub team_size: i32,
}

async fn org_registration_request(
    State(state): State<AppState>,
    Json(req): Json<OrgRegistrationRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() || req.contact_email.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "الاسم والبريد مطلوبان"}))).into_response();
    }

    let result = state.db
        .query(
            "CREATE org_request SET \
                name = $name, \
                org_type = $org_type, \
                contact_email = $email, \
                contact_phone = $phone, \
                city = $city, \
                website = $website, \
                description = $desc, \
                team_size = $size, \
                status = 'pending', \
                created_at = time::now()"
        )
        .bind(("name", req.name))
        .bind(("org_type", req.org_type))
        .bind(("email", req.contact_email))
        .bind(("phone", req.contact_phone))
        .bind(("city", req.city))
        .bind(("website", req.website))
        .bind(("desc", req.description))
        .bind(("size", req.team_size))
        .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(json!({"message": "تم إرسال طلب التسجيل بنجاح. سيتم مراجعته من قبل فريق Actuators"}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to create org request: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "فشل في إرسال الطلب"}))).into_response()
        }
    }
}

async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    if req.new_password.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "كلمة المرور يجب أن تكون 8 أحرف على الأقل"}))).into_response();
    }

    let account_id = match state.validate_auth_token(&req.token, "password_reset").await {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "رابط إعادة التعيين غير صالح أو منتهي الصلاحية"}))).into_response();
        }
    };

    let password = req.new_password.clone();
    let hashed = match tokio::task::spawn_blocking(move || hash_password(&password)).await {
        Ok(Ok(h)) => h,
        _ => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "فشل في تحديث كلمة المرور"}))).into_response();
        }
    };

    if let Err(e) = state.update_account_password(&account_id, &hashed).await {
        tracing::error!("Failed to update password: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "فشل في تحديث كلمة المرور"}))).into_response();
    }

    (StatusCode::OK, Json(json!({"message": "تم تغيير كلمة المرور بنجاح. يمكنك تسجيل الدخول الآن"}))).into_response()
}
