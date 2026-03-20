// Copyright (c) 2025 Dr.Machine. All Rights Reserved.
// AI-assisted. PROPRIETARY AND CONFIDENTIAL.

//! Account Model
//!
//! نموذج بيانات الحسابات — الهوية الموحدة (RBAC)
//! يطابق جدول `account` في `hr-schema.surql`

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use validator::Validate;

/// Account role — matches schema ASSERT constraint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountRole {
    /// مدير النظام
    Admin,
    /// مدير قسم
    Manager,
    /// موظف
    Employee,
    /// متدرب
    Intern,
    /// متقدم لوظيفة
    Applicant,
}

impl std::fmt::Display for AccountRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountRole::Admin => write!(f, "admin"),
            AccountRole::Manager => write!(f, "manager"),
            AccountRole::Employee => write!(f, "employee"),
            AccountRole::Intern => write!(f, "intern"),
            AccountRole::Applicant => write!(f, "applicant"),
        }
    }
}

impl AccountRole {
    /// Parse role from string, defaulting to `applicant`
    pub fn from_str_or_default(s: &str) -> Self {
        match s {
            "admin" => AccountRole::Admin,
            "manager" => AccountRole::Manager,
            "employee" => AccountRole::Employee,
            "intern" => AccountRole::Intern,
            _ => AccountRole::Applicant,
        }
    }

    /// Check if this role is at least the given level
    /// admin > manager > employee > intern > applicant
    pub fn has_at_least(&self, required: &AccountRole) -> bool {
        self.level() >= required.level()
    }

    /// Numeric level for role hierarchy comparison.
    /// admin=4, manager=3, employee=2, intern=1, applicant=0
    pub fn level(&self) -> u8 {
        match self {
            AccountRole::Admin => 4,
            AccountRole::Manager => 3,
            AccountRole::Employee => 2,
            AccountRole::Intern => 1,
            AccountRole::Applicant => 0,
        }
    }
}

/// Account record — maps to SurrealDB `account` table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// SurrealDB record ID (e.g., account:xyz)
    pub id: Option<Thing>,
    /// Email address (unique, validated)
    pub email: String,
    /// Argon2 password hash — never expose via API
    #[serde(skip_serializing, default)]
    pub password_hash: String,
    /// RBAC role
    pub role: AccountRole,
    /// Whether account is active
    pub is_active: bool,
    /// Whether email is verified
    #[serde(default)]
    pub email_verified: bool,
    /// User's full name
    #[serde(default)]
    pub full_name: Option<String>,
    /// Phone number
    #[serde(default)]
    pub phone: Option<String>,
    /// User type: platform_admin, startup_owner, team_leader, member, trainee, volunteer
    #[serde(default = "default_user_type")]
    pub user_type: String,
    /// Organization this user belongs to (if any)
    #[serde(default)]
    pub organization: Option<serde_json::Value>,
    /// Auth methods: [{type: "rfid"|"email"|"device", value: "..."}]
    #[serde(default)]
    pub auth_methods: Vec<serde_json::Value>,
    /// Account creation timestamp
    pub created_at: Option<String>,
    /// Last login timestamp
    pub last_login: Option<String>,
    /// Profile picture URL/Base64
    pub avatar: Option<String>,
}

fn default_user_type() -> String {
    "member".to_string()
}

impl Account {
    /// Extract record ID as string (e.g., "xyz" from account:xyz)
    pub fn id_string(&self) -> String {
        self.id
            .as_ref()
            .map(|thing| thing.id.to_raw())
            .unwrap_or_default()
    }
}

/// Login request payload
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// Email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    /// Plain-text password (will be verified against hash)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Signup request payload
#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default = "default_user_type")]
    pub user_type: String,
}

/// Successful auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// JWT access token
    pub token: String,
    /// Account info (safe subset)
    pub user: AuthUser,
}

/// Safe user info returned in API responses (no password_hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthUser {
    /// Record ID string
    pub id: String,
    /// Email
    pub email: String,
    /// Role
    pub role: AccountRole,
    /// Active status
    pub is_active: bool,
    /// Whether email is verified
    pub email_verified: bool,
    /// Profile picture URL/Base64
    pub avatar: Option<String>,
    pub full_name: Option<String>,
    pub user_type: String,
}

impl From<&Account> for AuthUser {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id_string(),
            email: account.email.clone(),
            role: account.role.clone(),
            is_active: account.is_active,
            email_verified: account.email_verified,
            avatar: account.avatar.clone(),
            full_name: account.full_name.clone(),
            user_type: account.user_type.clone(),
        }
    }
}

/// JWT claims payload — what's encoded inside the token
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject = account record ID
    pub sub: String,
    /// Email
    pub email: String,
    /// Role
    pub role: String,
    /// Issued at (unix timestamp)
    pub iat: usize,
    /// Expiration (unix timestamp)
    pub exp: usize,
}

/// Current authenticated user — extracted from JWT by middleware
/// Used as an Axum extractor: `CurrentUser` in handler params
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// Account record ID (e.g., "account:xyz")
    pub id: String,
    /// Email
    pub email: String,
    /// Role
    pub role: AccountRole,
}

// ============================================================================
// Password Hashing (Argon2)
// ============================================================================

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Hash a plain-text password using Argon2id
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a plain-text password against an Argon2 hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
