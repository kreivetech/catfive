use crate::error::ValidationError;
use crate::format_helper::format_template;
use crate::traits::Validator;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;

type Rule = Arc<dyn Fn(&str) -> Option<ValidationError> + Send + Sync>;

/// Fluent validator for string values.
#[derive(Clone, Default)]
pub struct StringValidator {
    rules: Vec<Rule>,
}

impl StringValidator {
    /// Creates a new empty string validator.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Enforces a minimum character length with default error message.
    pub fn min(self, min_len: usize) -> Self {
        self.min_msg(
            min_len,
            "Must be at least {min} characters long, got {count}",
        )
    }

    /// Enforces a minimum character length with a custom message template.
    ///
    /// Available placeholders: `{min}`, `{count}`, `{value}`.
    pub fn min_msg(mut self, min_len: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            let count = val.chars().count();
            if count < min_len {
                let msg = format_template(
                    &tpl,
                    &[
                        ("min", &min_len.to_string()),
                        ("count", &count.to_string()),
                        ("value", val),
                    ],
                );
                Some(ValidationError::new("min_length", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces a maximum character length with default error message.
    pub fn max(self, max_len: usize) -> Self {
        self.max_msg(max_len, "Must not exceed {max} characters, got {count}")
    }

    /// Enforces a maximum character length with a custom message template.
    ///
    /// Available placeholders: `{max}`, `{count}`, `{value}`.
    pub fn max_msg(mut self, max_len: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            let count = val.chars().count();
            if count > max_len {
                let msg = format_template(
                    &tpl,
                    &[
                        ("max", &max_len.to_string()),
                        ("count", &count.to_string()),
                        ("value", val),
                    ],
                );
                Some(ValidationError::new("max_length", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces an exact character length with default error message.
    pub fn length(self, exact_len: usize) -> Self {
        self.length_msg(
            exact_len,
            "Must be exactly {length} characters long, got {count}",
        )
    }

    /// Enforces an exact character length with a custom message template.
    ///
    /// Available placeholders: `{length}`, `{count}`, `{value}`.
    pub fn length_msg(mut self, exact_len: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            let count = val.chars().count();
            if count != exact_len {
                let msg = format_template(
                    &tpl,
                    &[
                        ("length", &exact_len.to_string()),
                        ("count", &count.to_string()),
                        ("value", val),
                    ],
                );
                Some(ValidationError::new("exact_length", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Blocks specific characters from appearing in the string.
    ///
    /// Detailed error logs the exact forbidden character that was encountered.
    pub fn block(self, forbidden_chars: &str) -> Self {
        self.block_msg(forbidden_chars, "Character '{char}' is not allowed")
    }

    /// Blocks specific characters with a custom message template.
    ///
    /// Available placeholders: `{char}`, `{index}`, `{position}`, `{value}`.
    pub fn block_msg(mut self, forbidden_chars: &str, msg_template: &str) -> Self {
        let blocked: HashSet<char> = forbidden_chars.chars().collect();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if blocked.contains(&ch) {
                    let ch_str = ch.to_string();
                    let idx_str = idx.to_string();
                    let pos_str = (idx + 1).to_string();
                    let msg = format_template(
                        &tpl,
                        &[
                            ("char", &ch_str),
                            ("index", &idx_str),
                            ("position", &pos_str),
                            ("value", val),
                        ],
                    );
                    return Some(ValidationError::new("blocked_char", msg));
                }
            }
            None
        }));
        self
    }

    /// Blocks specific characters with a custom closure.
    pub fn block_with<F>(mut self, forbidden_chars: &str, formatter: F) -> Self
    where
        F: Fn(char, usize) -> String + Send + Sync + 'static,
    {
        let blocked: HashSet<char> = forbidden_chars.chars().collect();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if blocked.contains(&ch) {
                    let msg = formatter(ch, idx);
                    return Some(ValidationError::new("blocked_char", msg));
                }
            }
            None
        }));
        self
    }

    /// Requires the string to not be empty (or whitespace-only).
    pub fn not_empty(self) -> Self {
        self.not_empty_msg("Value cannot be empty")
    }

    /// Requires the string to not be empty with a custom message.
    pub fn not_empty_msg(mut self, msg: &str) -> Self {
        let err_msg = msg.to_string();
        self.rules.push(Arc::new(move |val| {
            if val.trim().is_empty() {
                Some(ValidationError::new("not_empty", err_msg.clone()))
            } else {
                None
            }
        }));
        self
    }

    /// Allows only characters present in the provided whitelist string.
    pub fn allow_only(self, allowed_chars: &str) -> Self {
        self.allow_only_msg(allowed_chars, "Character '{char}' is not allowed")
    }

    /// Allows only characters present in the provided whitelist string with custom message template.
    ///
    /// Available placeholders: `{char}`, `{index}`, `{position}`, `{value}`.
    pub fn allow_only_msg(mut self, allowed_chars: &str, msg_template: &str) -> Self {
        let allowed: HashSet<char> = allowed_chars.chars().collect();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if !allowed.contains(&ch) {
                    let ch_str = ch.to_string();
                    let idx_str = idx.to_string();
                    let pos_str = (idx + 1).to_string();
                    let msg = format_template(
                        &tpl,
                        &[
                            ("char", &ch_str),
                            ("index", &idx_str),
                            ("position", &pos_str),
                            ("value", val),
                        ],
                    );
                    return Some(ValidationError::new("unallowed_char", msg));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string is an alphanumeric string.
    pub fn alphanumeric(self) -> Self {
        self.alphanumeric_msg("Character '{char}' is not alphanumeric")
    }

    /// Validates that the string is alphanumeric with a custom message template.
    pub fn alphanumeric_msg(mut self, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if !ch.is_alphanumeric() {
                    let ch_str = ch.to_string();
                    let idx_str = idx.to_string();
                    let msg = format_template(
                        &tpl,
                        &[("char", &ch_str), ("index", &idx_str), ("value", val)],
                    );
                    return Some(ValidationError::new("alphanumeric", msg));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string contains only alphabetic characters.
    pub fn alphabetic(self) -> Self {
        self.alphabetic_msg("Character '{char}' is not alphabetic")
    }

    /// Validates that the string contains only alphabetic characters with a custom message template.
    pub fn alphabetic_msg(mut self, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if !ch.is_alphabetic() {
                    let ch_str = ch.to_string();
                    let idx_str = idx.to_string();
                    let msg = format_template(
                        &tpl,
                        &[("char", &ch_str), ("index", &idx_str), ("value", val)],
                    );
                    return Some(ValidationError::new("alphabetic", msg));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string contains only numeric ASCII digits.
    pub fn numeric(self) -> Self {
        self.numeric_msg("Character '{char}' is not a digit")
    }

    /// Validates that the string contains only numeric ASCII digits with a custom message template.
    pub fn numeric_msg(mut self, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            for (idx, ch) in val.chars().enumerate() {
                if !ch.is_ascii_digit() {
                    let ch_str = ch.to_string();
                    let idx_str = idx.to_string();
                    let msg = format_template(
                        &tpl,
                        &[("char", &ch_str), ("index", &idx_str), ("value", val)],
                    );
                    return Some(ValidationError::new("numeric", msg));
                }
            }
            None
        }));
        self
    }

    /// Validates that the string has a valid email syntax without external dependencies.
    pub fn email(self) -> Self {
        self.email_msg("Invalid email format")
    }

    /// Validates that the string has a valid email syntax with a custom message.
    pub fn email_msg(mut self, msg: &str) -> Self {
        let err_msg = msg.to_string();
        self.rules.push(Arc::new(move |val| {
            let parts: Vec<&str> = val.split('@').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Some(ValidationError::new("email", err_msg.clone()));
            }

            let domain = parts[1];
            if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
                return Some(ValidationError::new("email", err_msg.clone()));
            }

            if val.contains(' ') {
                return Some(ValidationError::new("email", err_msg.clone()));
            }

            None
        }));
        self
    }

    /// Enforces that the string starts with the given prefix.
    pub fn starts_with(self, prefix: &str) -> Self {
        self.starts_with_msg(prefix, "Must start with '{prefix}'")
    }

    /// Enforces that the string starts with the given prefix with a custom message template.
    pub fn starts_with_msg(mut self, prefix: &str, msg_template: &str) -> Self {
        let prefix = prefix.to_string();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if !val.starts_with(&prefix) {
                let msg = format_template(&tpl, &[("prefix", &prefix), ("value", val)]);
                Some(ValidationError::new("starts_with", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the string ends with the given suffix.
    pub fn ends_with(self, suffix: &str) -> Self {
        self.ends_with_msg(suffix, "Must end with '{suffix}'")
    }

    /// Enforces that the string ends with the given suffix with a custom message template.
    pub fn ends_with_msg(mut self, suffix: &str, msg_template: &str) -> Self {
        let suffix = suffix.to_string();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if !val.ends_with(&suffix) {
                let msg = format_template(&tpl, &[("suffix", &suffix), ("value", val)]);
                Some(ValidationError::new("ends_with", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the string contains a given substring.
    pub fn contains(self, substring: &str) -> Self {
        self.contains_msg(substring, "Must contain '{expected}'")
    }

    /// Enforces that the string contains a given substring with a custom message template.
    pub fn contains_msg(mut self, substring: &str, msg_template: &str) -> Self {
        let sub = substring.to_string();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if !val.contains(&sub) {
                let msg = format_template(&tpl, &[("expected", &sub), ("value", val)]);
                Some(ValidationError::new("contains", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the string matches one of the allowed values.
    pub fn in_list(self, allowed: &[&str]) -> Self {
        self.in_list_msg(allowed, "Value '{value}' is not among allowed options")
    }

    /// Enforces that the string matches one of the allowed values with a custom message template.
    pub fn in_list_msg(mut self, allowed: &[&str], msg_template: &str) -> Self {
        let allowed_set: HashSet<String> = allowed.iter().map(|s| s.to_string()).collect();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if !allowed_set.contains(val) {
                let msg = format_template(&tpl, &[("value", val)]);
                Some(ValidationError::new("in_list", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Validates that the string is a valid web URL (starts with http:// or https:// and has a host).
    pub fn url(self) -> Self {
        self.url_msg("Must be a valid URL")
    }

    /// Validates that the string is a valid web URL with a custom message.
    pub fn url_msg(mut self, msg: &str) -> Self {
        let err_msg = msg.to_string();
        self.rules.push(Arc::new(move |val| {
            let is_valid = (val.starts_with("http://") && val.len() > 7)
                || (val.starts_with("https://") && val.len() > 8);
            if !is_valid || val.contains(' ') {
                Some(ValidationError::new("url", err_msg.clone()))
            } else {
                None
            }
        }));
        self
    }

    /// Validates that the string is a standard UUID (8-4-4-4-12 hex characters).
    pub fn uuid(self) -> Self {
        self.uuid_msg("Must be a valid UUID")
    }

    /// Validates that the string is a standard UUID with a custom message.
    pub fn uuid_msg(mut self, msg: &str) -> Self {
        let err_msg = msg.to_string();
        self.rules.push(Arc::new(move |val| {
            let is_valid = if val.len() == 36 {
                val.chars().enumerate().all(|(idx, ch)| {
                    if idx == 8 || idx == 13 || idx == 18 || idx == 23 {
                        ch == '-'
                    } else {
                        ch.is_ascii_hexdigit()
                    }
                })
            } else {
                false
            };

            if !is_valid {
                Some(ValidationError::new("uuid", err_msg.clone()))
            } else {
                None
            }
        }));
        self
    }

    /// Validates that the string is a valid IPv4 or IPv6 address.
    pub fn ip(self) -> Self {
        self.ip_msg("Must be a valid IP address")
    }

    /// Validates that the string is a valid IPv4 or IPv6 address with a custom message.
    pub fn ip_msg(mut self, msg: &str) -> Self {
        let err_msg = msg.to_string();
        self.rules.push(Arc::new(move |val| {
            if val.parse::<IpAddr>().is_err() {
                Some(ValidationError::new("ip", err_msg.clone()))
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
        self.rules.push(Arc::new(move |val| validator(val).err()));
        self
    }

    #[cfg(feature = "regex")]
    /// Validates the string against a regex pattern.
    pub fn regex(self, pattern: &str) -> Self {
        self.regex_msg(pattern, "Value does not match required pattern '{pattern}'")
    }

    #[cfg(feature = "regex")]
    /// Validates the string against a regex pattern with a custom message template.
    ///
    /// Available placeholders: `{pattern}`, `{value}`.
    pub fn regex_msg(mut self, pattern: &str, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        match regex::Regex::new(pattern) {
            Ok(re) => {
                let pat_str = pattern.to_string();
                self.rules.push(Arc::new(move |val| {
                    if !re.is_match(val) {
                        let msg = format_template(&tpl, &[("pattern", &pat_str), ("value", val)]);
                        Some(ValidationError::new("regex", msg))
                    } else {
                        None
                    }
                }));
            }
            Err(e) => {
                let err_msg = format!("Invalid regular expression '{}': {}", pattern, e);
                self.rules.push(Arc::new(move |_| {
                    Some(ValidationError::new(
                        "invalid_regex_pattern",
                        err_msg.clone(),
                    ))
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

impl Validator<&str> for StringValidator {
    fn validate_all(&self, value: &&str) -> Vec<ValidationError> {
        self.validate_all(*value)
    }
}
