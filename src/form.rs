use std::collections::{BTreeMap, HashMap};
use crate::error::{ValidationError, ValidationErrors};
use crate::string::StringValidator;
use crate::traits::Validator;

/// Trait abstracting key-value form data access.
pub trait FormData {
    /// Retrieves the string value for a given form field key.
    fn get_value(&self, key: &str) -> Option<&str>;
}

impl FormData for HashMap<String, String> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).map(|v| v.as_str())
    }
}

impl FormData for HashMap<&str, &str> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).copied()
    }
}

impl FormData for BTreeMap<String, String> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).map(|v| v.as_str())
    }
}

impl FormData for BTreeMap<&str, &str> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).copied()
    }
}

impl FormData for [(&str, &str)] {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
    }
}

impl FormData for Vec<(&str, &str)> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.as_slice().get_value(key)
    }
}

struct FormRule {
    name: String,
    validator: StringValidator,
    required: bool,
}

/// Fluent schema validator for form payloads (key-value data).
#[derive(Default)]
pub struct FormValidator {
    rules: Vec<FormRule>,
}

impl FormValidator {
    /// Creates a new empty form validator.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a required field rule to the form validator.
    pub fn field(mut self, name: impl Into<String>, validator: StringValidator) -> Self {
        self.rules.push(FormRule {
            name: name.into(),
            validator,
            required: true,
        });
        self
    }

    /// Adds an optional field rule to the form validator.
    ///
    /// If the field is not present or is empty, validation succeeds.
    /// If present and non-empty, the validator rules are executed.
    pub fn optional_field(mut self, name: impl Into<String>, validator: StringValidator) -> Self {
        self.rules.push(FormRule {
            name: name.into(),
            validator,
            required: false,
        });
        self
    }

    /// Validates a form data payload, collecting all errors across all fields.
    pub fn validate<F: FormData + ?Sized>(&self, data: &F) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        for rule in &self.rules {
            match data.get_value(&rule.name) {
                Some(val) if !val.is_empty() => {
                    let field_errors = rule.validator.validate_all(val);
                    errors.add_many(&rule.name, field_errors);
                }
                Some(_) | None => {
                    if rule.required {
                        errors.add(
                            &rule.name,
                            ValidationError::new("required", "This field is required"),
                        );
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
