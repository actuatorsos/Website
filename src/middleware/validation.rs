//! Input Validation
//!
//! Common validation functions for user input.

use std::borrow::Cow;

/// Validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field name.
    pub field: String,
    /// Error message.
    pub message: String,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validation result type.
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validate that a string is not empty.
pub fn validate_required(field: &str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new(field, "هذا الحقل مطلوب"))
    } else {
        Ok(())
    }
}

/// Validate string length.
pub fn validate_length(
    field: &str,
    value: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), ValidationError> {
    let len = value.chars().count();

    if let Some(min_len) = min {
        if len < min_len {
            return Err(ValidationError::new(
                field,
                format!("يجب أن يكون على الأقل {} أحرف", min_len),
            ));
        }
    }

    if let Some(max_len) = max {
        if len > max_len {
            return Err(ValidationError::new(
                field,
                format!("يجب أن لا يتجاوز {} أحرف", max_len),
            ));
        }
    }

    Ok(())
}

/// Validate email format.
pub fn validate_email(field: &str, value: &str) -> Result<(), ValidationError> {
    // Simple email regex check
    if !value.contains('@') || !value.contains('.') {
        return Err(ValidationError::new(field, "بريد إلكتروني غير صالح"));
    }

    let parts: Vec<&str> = value.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::new(field, "بريد إلكتروني غير صالح"));
    }

    Ok(())
}

/// Validate phone number (Saudi format).
pub fn validate_phone_sa(field: &str, value: &str) -> Result<(), ValidationError> {
    let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

    // Saudi mobile: 05XXXXXXXX (10 digits) or +9665XXXXXXXX (12 digits)
    if cleaned.len() == 10 && cleaned.starts_with("05") {
        return Ok(());
    }

    if cleaned.len() == 12 && cleaned.starts_with("9665") {
        return Ok(());
    }

    Err(ValidationError::new(field, "رقم هاتف غير صالح"))
}

/// Validate positive number.
pub fn validate_positive<T: PartialOrd + Default>(
    field: &str,
    value: T,
) -> Result<(), ValidationError> {
    if value <= T::default() {
        return Err(ValidationError::new(field, "يجب أن يكون رقماً موجباً"));
    }
    Ok(())
}

/// Validate value is within range.
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    field: &str,
    value: T,
    min: T,
    max: T,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        return Err(ValidationError::new(
            field,
            format!("يجب أن يكون بين {} و {}", min, max),
        ));
    }
    Ok(())
}

/// Sanitize string input (remove dangerous characters).
pub fn sanitize_string(input: &str) -> Cow<'_, str> {
    // Remove null bytes and control characters
    if input
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
    {
        Cow::Owned(
            input
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
                .collect(),
        )
    } else {
        Cow::Borrowed(input)
    }
}

/// Escape HTML special characters.
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Validate serial number format.
pub fn validate_serial_number(field: &str, value: &str) -> Result<(), ValidationError> {
    // Allow alphanumeric, hyphens, and underscores
    if value
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            "الرقم التسلسلي يجب أن يحتوي على أحرف وأرقام فقط",
        ))
    }
}

/// Validator builder for chaining validations.
pub struct Validator {
    errors: Vec<ValidationError>,
}

impl Validator {
    /// Create a new validator.
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    /// Add a validation check.
    pub fn check(mut self, result: Result<(), ValidationError>) -> Self {
        if let Err(e) = result {
            self.errors.push(e);
        }
        self
    }

    /// Finish validation and return result.
    pub fn finish(self) -> ValidationResult {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
