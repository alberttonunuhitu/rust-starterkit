use std::collections::HashMap;

use crate::common::error::ApiError;

pub const EMAIL_MAX_LENGTH: usize = 255;
pub const PASSWORD_MIN_LENGTH: usize = 8;
pub const PASSWORD_MAX_LENGTH: usize = 128;

/// Collects field errors and turns them into [`ApiError::ValidationError`].
#[derive(Debug, Default)]
pub struct Validator {
    fields: HashMap<String, Vec<String>>,
}

impl Validator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.fields
            .entry(field.to_string())
            .or_default()
            .push(message.into());
    }

    pub fn email(&mut self, field: &str, value: &str) {
        let value = value.trim();

        if value.is_empty() {
            self.add(field, "Email is required");
        } else if value.chars().count() > EMAIL_MAX_LENGTH {
            self.add(
                field,
                format!("Email must be at most {EMAIL_MAX_LENGTH} characters"),
            );
        } else if !is_valid_email(value) {
            self.add(field, "Email format is invalid");
        }
    }

    pub fn password(&mut self, field: &str, value: &str) {
        let length = value.chars().count();

        if length < PASSWORD_MIN_LENGTH {
            self.add(
                field,
                format!("Password must be at least {PASSWORD_MIN_LENGTH} characters"),
            );
        } else if length > PASSWORD_MAX_LENGTH {
            self.add(
                field,
                format!("Password must be at most {PASSWORD_MAX_LENGTH} characters"),
            );
        }
    }

    pub fn finish(self) -> Result<(), ApiError> {
        if self.fields.is_empty() {
            Ok(())
        } else {
            Err(ApiError::ValidationError {
                fields: self.fields,
            })
        }
    }
}

/// Pragmatic email check: `local@domain.tld`, no whitespace, a single `@`,
/// and a domain made of non-empty labels.
fn is_valid_email(email: &str) -> bool {
    if email.chars().any(char::is_whitespace) {
        return false;
    }

    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };

    !local.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && domain.split('.').all(|label| !label.is_empty())
}

/// Escape `LIKE` wildcards so user input is matched literally (escape char `\`).
pub fn escape_like(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for c in value.chars() {
        if matches!(c, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(c);
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_format() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("first.last+tag@sub.example.co.id"));

        for invalid in [
            "",
            "user",
            "@example.com",
            "user@",
            "user@example",
            "user@@example.com",
            "user@example..com",
            "user@.example.com",
            "us er@example.com",
        ] {
            assert!(!is_valid_email(invalid), "{invalid} should be invalid");
        }
    }

    #[test]
    fn validator_collects_errors() {
        let mut validator = Validator::new();
        validator.email("email", "not-an-email");
        validator.password("password", "short");

        let Err(ApiError::ValidationError { fields }) = validator.finish() else {
            panic!("expected validation error");
        };
        assert!(fields.contains_key("email"));
        assert!(fields.contains_key("password"));

        let mut validator = Validator::new();
        validator.email("email", "user@example.com");
        validator.password("password", "long-enough");
        assert!(validator.finish().is_ok());
    }

    #[test]
    fn like_escaping() {
        assert_eq!(escape_like("a%b_c\\d"), "a\\%b\\_c\\\\d");
        assert_eq!(escape_like("plain"), "plain");
    }
}
