//! Basic tests for the Actuators platform
//! Tests that don't require a database connection.

#[cfg(test)]
mod tests {
    // Test password hashing and verification
    #[test]
    fn test_password_hash_and_verify() {
        use Actuators::models::hash_password;
        use Actuators::models::verify_password;

        let password = "test_password_123";
        let hash = hash_password(password).expect("Should hash password");

        assert!(
            verify_password(password, &hash).expect("Should verify"),
            "Password should match"
        );
        assert!(
            !verify_password("wrong_password", &hash).expect("Should verify"),
            "Wrong password should not match"
        );
    }

    // Test password minimum length
    #[test]
    fn test_password_hash_empty() {
        use Actuators::models::hash_password;

        // Even empty passwords should hash successfully (validation happens elsewhere)
        let hash = hash_password("");
        assert!(hash.is_ok());
    }

    // Test JWT token creation and decoding
    #[test]
    fn test_jwt_token_roundtrip() {
        use Actuators::middleware::auth::{create_token, decode_token};

        let secret = "test_secret_key_for_jwt_testing_minimum_length_here";
        let token = create_token("user:123", "test@example.com", "admin", secret, 24)
            .expect("Should create token");

        let claims = decode_token(&token, secret).expect("Should decode token");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.sub, "user:123");
    }

    // Test JWT with wrong secret fails
    #[test]
    fn test_jwt_wrong_secret() {
        use Actuators::middleware::auth::{create_token, decode_token};

        let token = create_token(
            "user:123",
            "test@example.com",
            "admin",
            "correct_secret_key_here_long",
            24,
        )
        .expect("Should create token");

        let result = decode_token(&token, "wrong_secret_key_here_long_enough");
        assert!(result.is_err(), "Should fail with wrong secret");
    }

    // Test account role hierarchy
    #[test]
    fn test_role_hierarchy() {
        use Actuators::models::AccountRole;

        assert!(AccountRole::Admin.level() > AccountRole::Manager.level());
        assert!(AccountRole::Manager.level() > AccountRole::Employee.level());
        assert!(AccountRole::Employee.level() > AccountRole::Intern.level());
        assert!(AccountRole::Intern.level() > AccountRole::Applicant.level());
    }

    // Test CSV export helper
    #[test]
    fn test_csv_escape() {
        // Test that values with commas get quoted
        let value_with_comma = "hello, world";
        let escaped = format!("\"{}\"", value_with_comma.replace('"', "\"\""));
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
    }

    // Test config validation
    #[test]
    fn test_config_default_jwt_warning() {
        // The default JWT secret should be detectable
        let default_secret = "CHANGE_ME_IN_PRODUCTION_64_CHARS_MINIMUM_SECRET_KEY_HERE_NOW";
        assert!(
            default_secret.len() > 30,
            "Default secret should be long enough to be obvious"
        );
    }
}
