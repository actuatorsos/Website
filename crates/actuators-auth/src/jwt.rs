use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};

use actuators_core::error::AppError;

// ── Claims ────────────────────────────────────────────────────────────

/// JWT claims for platform-level (super-admin) authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformClaims {
    /// Platform account ID (`sub`ject).
    pub sub: String,
    pub email: String,
    pub full_name: String,
    pub is_super_admin: bool,
    /// Expiration time (seconds since UNIX epoch).
    pub exp: usize,
    /// Issued-at time (seconds since UNIX epoch).
    pub iat: usize,
}

/// JWT claims for tenant-scoped authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantClaims {
    /// Tenant account ID (`sub`ject).
    pub sub: String,
    /// Corresponding platform account ID.
    pub platform_id: String,
    pub tenant_slug: String,
    pub tenant_db: String,
    pub role: String,
    pub services: Vec<String>,
    /// Expiration time (seconds since UNIX epoch).
    pub exp: usize,
    /// Issued-at time (seconds since UNIX epoch).
    pub iat: usize,
}

// ── Token creation ────────────────────────────────────────────────────

/// Encode a `PlatformClaims` struct into a signed HS256 JWT.
pub fn create_platform_token(claims: &PlatformClaims, secret: &str) -> Result<String, AppError> {
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&Header::default(), claims, &key)
        .map_err(|e| AppError::Internal(format!("failed to create platform token: {e}")))
}

/// Encode a `TenantClaims` struct into a signed HS256 JWT.
pub fn create_tenant_token(claims: &TenantClaims, secret: &str) -> Result<String, AppError> {
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&Header::default(), claims, &key)
        .map_err(|e| AppError::Internal(format!("failed to create tenant token: {e}")))
}

// ── Token verification ────────────────────────────────────────────────

/// Decode and validate a platform JWT, returning the embedded claims.
pub fn verify_platform_token(token: &str, secret: &str) -> Result<PlatformClaims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<PlatformClaims>(token, &key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("invalid platform token: {e}")))?;

    Ok(token_data.claims)
}

/// Decode and validate a tenant JWT, returning the embedded claims.
pub fn verify_tenant_token(token: &str, secret: &str) -> Result<TenantClaims, AppError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<TenantClaims>(token, &key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("invalid tenant token: {e}")))?;

    Ok(token_data.claims)
}

// ── Refresh token ─────────────────────────────────────────────────────

/// Generate a cryptographically random 64-character hex string for use as a
/// refresh token.
pub fn generate_refresh_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.r#gen()).collect();
    hex_encode(&bytes)
}

/// Encode bytes as a lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_token_roundtrip() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = PlatformClaims {
            sub: "account:abc123".into(),
            email: "admin@example.com".into(),
            full_name: "Admin User".into(),
            is_super_admin: true,
            iat: now,
            exp: now + 3600,
        };

        let secret = "test-secret-key-at-least-32-chars!!";
        let token = create_platform_token(&claims, secret).unwrap();
        let decoded = verify_platform_token(&token, secret).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
        assert!(decoded.is_super_admin);
    }

    #[test]
    fn tenant_token_roundtrip() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = TenantClaims {
            sub: "account:xyz789".into(),
            platform_id: "account:abc123".into(),
            tenant_slug: "acme-corp".into(),
            tenant_db: "tenant_acme_corp".into(),
            role: "admin".into(),
            services: vec!["hr".into(), "payroll".into()],
            iat: now,
            exp: now + 3600,
        };

        let secret = "test-secret-key-at-least-32-chars!!";
        let token = create_tenant_token(&claims, secret).unwrap();
        let decoded = verify_tenant_token(&token, secret).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.tenant_slug, claims.tenant_slug);
        assert_eq!(decoded.services, claims.services);
    }

    #[test]
    fn refresh_token_length() {
        let token = generate_refresh_token();
        assert_eq!(token.len(), 64);
    }
}
