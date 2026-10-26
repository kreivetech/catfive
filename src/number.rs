use crate::error::ValidationError;
use crate::format_helper::format_template;
use crate::traits::Validator;
use std::fmt::Display;
use std::sync::Arc;

type NumberRule<T> = Arc<dyn Fn(&T) -> Option<ValidationError> + Send + Sync>;

/// Fluent validator for numeric values.
#[derive(Clone)]
pub struct NumberValidator<T> {
    rules: Vec<NumberRule<T>>,
}

impl<T> Default for NumberValidator<T> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<T> NumberValidator<T>
where
    T: PartialOrd + Copy + Display + Send + Sync + 'static,
{
    /// Creates a new empty numeric validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enforces a minimum numeric value.
    pub fn min(self, minimum: T) -> Self {
        self.min_msg(minimum, "Must be at least {min}, got {value}")
    }

    /// Enforces a minimum numeric value with a custom message template.
    ///
    /// Available placeholders: `{min}`, `{value}`.
    pub fn min_msg(mut self, minimum: T, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if *val < minimum {
                let msg = format_template(
                    &tpl,
                    &[("min", &minimum.to_string()), ("value", &val.to_string())],
                );
                Some(ValidationError::new("min_value", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces a maximum numeric value.
    pub fn max(self, maximum: T) -> Self {
        self.max_msg(maximum, "Must not exceed {max}, got {value}")
    }

    /// Enforces a maximum numeric value with a custom message template.
    ///
    /// Available placeholders: `{max}`, `{value}`.
    pub fn max_msg(mut self, maximum: T, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if *val > maximum {
                let msg = format_template(
                    &tpl,
                    &[("max", &maximum.to_string()), ("value", &val.to_string())],
                );
                Some(ValidationError::new("max_value", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the value is within a given range (inclusive).
    pub fn range(self, min_val: T, max_val: T) -> Self {
        self.range_msg(
            min_val,
            max_val,
            "Must be between {min} and {max}, got {value}",
        )
    }

    /// Enforces that the value is within a given range with a custom message template.
    ///
    /// Available placeholders: `{min}`, `{max}`, `{value}`.
    pub fn range_msg(mut self, min_val: T, max_val: T, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if *val < min_val || *val > max_val {
                let msg = format_template(
                    &tpl,
                    &[
                        ("min", &min_val.to_string()),
                        ("max", &max_val.to_string()),
                        ("value", &val.to_string()),
                    ],
                );
                Some(ValidationError::new("range", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Adds a custom user-defined validation closure for numeric types.
    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&T) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.rules.push(Arc::new(move |val| validator(val).err()));
        self
    }
}

impl<T> NumberValidator<T>
where
    T: PartialOrd + Default + Copy + Display + Send + Sync + 'static,
{
    /// Enforces that the numeric value is strictly positive (> 0).
    pub fn positive(self) -> Self {
        self.positive_msg("Must be greater than 0, got {value}")
    }

    /// Enforces that the numeric value is strictly positive with a custom message template.
    ///
    /// Available placeholders: `{value}`.
    pub fn positive_msg(mut self, msg_template: &str) -> Self {
        let zero = T::default();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if *val <= zero {
                let msg = format_template(&tpl, &[("value", &val.to_string())]);
                Some(ValidationError::new("positive", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the numeric value is strictly negative (< 0).
    pub fn negative(self) -> Self {
        self.negative_msg("Must be less than 0, got {value}")
    }

    /// Enforces that the numeric value is strictly negative with a custom message template.
    ///
    /// Available placeholders: `{value}`.
    pub fn negative_msg(mut self, msg_template: &str) -> Self {
        let zero = T::default();
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |val| {
            if *val >= zero {
                let msg = format_template(&tpl, &[("value", &val.to_string())]);
                Some(ValidationError::new("negative", msg))
            } else {
                None
            }
        }));
        self
    }
}

impl<T: Send + Sync> Validator<T> for NumberValidator<T> {
    fn validate_all(&self, value: &T) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            if let Some(err) = rule(value) {
                errors.push(err);
            }
        }
        errors
    }
}
