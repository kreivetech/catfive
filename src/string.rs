use crate::error::ValidationError;
use crate::traits::Validator;

type Rule = Box<dyn Fn(&str) -> Option<ValidationError> + Send + Sync>;

/// Fluent validator for string values.
#[derive(Default)]
pub struct StringValidator {
    rules: Vec<Rule>,
}

impl StringValidator {
    /// Creates a new empty string validator.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Enforces a minimum character length.
    pub fn min(mut self, min_len: usize) -> Self {
        self.rules.push(Box::new(move |val| {
            let count = val.chars().count();
            if count < min_len {
                Some(ValidationError::new(
                    "min_length",
                    format!("Must be at least {} characters long, got {}", min_len, count),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces a maximum character length.
    pub fn max(mut self, max_len: usize) -> Self {
        self.rules.push(Box::new(move |val| {
            let count = val.chars().count();
            if count > max_len {
                Some(ValidationError::new(
                    "max_length",
                    format!("Must not exceed {} characters, got {}", max_len, count),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces an exact character length.
    pub fn length(mut self, exact_len: usize) -> Self {
        self.rules.push(Box::new(move |val| {
            let count = val.chars().count();
            if count != exact_len {
                Some(ValidationError::new(
                    "exact_length",
                    format!("Must be exactly {} characters long, got {}", exact_len, count),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Blocks specific characters from appearing in the string.
    ///
    /// Detailed error logs the exact forbidden character that was encountered.
    pub fn block(mut self, forbidden_chars: &str) -> Self {
        let blocked: Vec<char> = forbidden_chars.chars().collect();
        self.rules.push(Box::new(move |val| {
            for ch in val.chars() {
                if blocked.contains(&ch) {
                    return Some(ValidationError::new(
                        "blocked_char",
                        format!("Character '{}' is not allowed", ch),
                    ));
                }
            }
            None
        }));
        self
    }

    /// Requires the string to not be empty (or whitespace-only).
    pub fn not_empty(mut self) -> Self {
        self.rules.push(Box::new(|val| {
            if val.trim().is_empty() {
                Some(ValidationError::new("not_empty", "Value cannot be empty"))
            } else {
                None
            }
        }));
        self
    }

    /// Allows only characters present in the provided whitelist string.
    pub fn allow_only(mut self, allowed_chars: &str) -> Self {
        let allowed: Vec<char> = allowed_chars.chars().collect();
        self.rules.push(Box::new(move |val| {
            for ch in val.chars() {
                if !allowed.contains(&ch) {
                    return Some(ValidationError::new(
                        "unallowed_char",
                        format!("Character '{}' is not allowed", ch),
                    ));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string is an alphanumeric string.
    pub fn alphanumeric(mut self) -> Self {
        self.rules.push(Box::new(|val| {
            for ch in val.chars() {
                if !ch.is_alphanumeric() {
                    return Some(ValidationError::new(
                        "alphanumeric",
                        format!("Character '{}' is not alphanumeric", ch),
                    ));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string contains only alphabetic characters.
    pub fn alphabetic(mut self) -> Self {
        self.rules.push(Box::new(|val| {
            for ch in val.chars() {
                if !ch.is_alphabetic() {
                    return Some(ValidationError::new(
                        "alphabetic",
                        format!("Character '{}' is not alphabetic", ch),
                    ));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string contains only numeric ASCII digits.
    pub fn numeric(mut self) -> Self {
        self.rules.push(Box::new(|val| {
            for ch in val.chars() {
                if !ch.is_ascii_digit() {
                    return Some(ValidationError::new(
                        "numeric",
                        format!("Character '{}' is not a digit", ch),
                    ));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string has a valid email syntax without external dependencies.
    pub fn email(mut self) -> Self {
        self.rules.push(Box::new(|val| {
            let parts: Vec<&str> = val.split('@').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Some(ValidationError::new("email", "Invalid email format"));
            }

            let domain = parts[1];
            if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
                return Some(ValidationError::new("email", "Invalid email domain"));
            }

            if val.contains(' ') {
                return Some(ValidationError::new("email", "Email cannot contain spaces"));
            }

            None
        }));
        self
    }

    /// Enforces that the string starts with the given prefix.
    pub fn starts_with(mut self, prefix: &str) -> Self {
        let prefix = prefix.to_string();
        self.rules.push(Box::new(move |val| {
            if !val.starts_with(&prefix) {
                Some(ValidationError::new(
                    "starts_with",
                    format!("Must start with '{}'", prefix),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the string ends with the given suffix.
    pub fn ends_with(mut self, suffix: &str) -> Self {
        let suffix = suffix.to_string();
        self.rules.push(Box::new(move |val| {
            if !val.ends_with(&suffix) {
                Some(ValidationError::new(
                    "ends_with",
                    format!("Must end with '{}'", suffix),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Adds a custom user-defined validation closure.
    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.rules.push(Box::new(move |val| validator(val).err()));
        self
    }

    #[cfg(feature = "regex")]
    /// Validates the string against a compiled regex or regex pattern.
    pub fn regex(mut self, pattern: &str) -> Self {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                let pat_str = pattern.to_string();
                self.rules.push(Box::new(move |val| {
                    if !re.is_match(val) {
                        Some(ValidationError::new(
                            "regex",
                            format!("Value does not match required pattern '{}'", pat_str),
                        ))
                    } else {
                        None
                    }
                }));
            }
            Err(e) => {
                let err_msg = format!("Invalid regular expression '{}': {}", pattern, e);
                self.rules.push(Box::new(move |_| {
                    Some(ValidationError::new("invalid_regex_pattern", err_msg.clone()))
                }));
            }
        }
        self
    }
}

impl Validator<str> for StringValidator {
    fn validate_all(&self, value: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            if let Some(err) = rule(value) {
                errors.push(err);
            }
        }
        errors
    }
}

impl Validator<String> for StringValidator {
    fn validate_all(&self, value: &String) -> Vec<ValidationError> {
        self.validate_all(value.as_str())
    }
}
