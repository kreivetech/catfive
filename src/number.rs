use std::fmt::Display;
use crate::error::ValidationError;
use crate::traits::Validator;

type NumberRule<T> = Box<dyn Fn(&T) -> Option<ValidationError> + Send + Sync>;

/// Fluent validator for numeric values.
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
    pub fn min(mut self, minimum: T) -> Self {
        self.rules.push(Box::new(move |val| {
            if *val < minimum {
                Some(ValidationError::new(
                    "min_value",
                    format!("Must be at least {}, got {}", minimum, val),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces a maximum numeric value.
    pub fn max(mut self, maximum: T) -> Self {
        self.rules.push(Box::new(move |val| {
            if *val > maximum {
                Some(ValidationError::new(
                    "max_value",
                    format!("Must not exceed {}, got {}", maximum, val),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the value is within a given range (inclusive).
    pub fn range(mut self, min_val: T, max_val: T) -> Self {
        self.rules.push(Box::new(move |val| {
            if *val < min_val || *val > max_val {
                Some(ValidationError::new(
                    "range",
                    format!("Must be between {} and {}, got {}", min_val, max_val, val),
                ))
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
        self.rules.push(Box::new(move |val| validator(val).err()));
        self
    }
}

impl<T> NumberValidator<T>
where
    T: PartialOrd + Default + Copy + Display + Send + Sync + 'static,
{
    /// Enforces that the numeric value is strictly positive (> 0).
    pub fn positive(mut self) -> Self {
        let zero = T::default();
        self.rules.push(Box::new(move |val| {
            if *val <= zero {
                Some(ValidationError::new(
                    "positive",
                    format!("Must be greater than 0, got {}", val),
                ))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces that the numeric value is strictly negative (< 0).
    pub fn negative(mut self) -> Self {
        let zero = T::default();
        self.rules.push(Box::new(move |val| {
            if *val >= zero {
                Some(ValidationError::new(
                    "negative",
                    format!("Must be less than 0, got {}", val),
                ))
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
