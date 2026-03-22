use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use actuators_core::error::AppError;

/// Hash a plaintext password using Argon2id with a random salt.
///
/// Returns the PHC-formatted hash string.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))?;

    Ok(hash.to_string())
}

/// Verify a plaintext password against a PHC-formatted Argon2id hash.
///
/// Returns `Ok(true)` if the password matches, `Ok(false)` otherwise.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("invalid password hash format: {e}")))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Internal(format!(
            "password verification failed: {e}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let password = "correct-horse-battery-staple";
        let hash = hash_password(password).expect("hashing should succeed");

        assert!(verify_password(password, &hash).expect("verify should succeed"));
        assert!(!verify_password("wrong-password", &hash).expect("verify should succeed"));
    }
}
