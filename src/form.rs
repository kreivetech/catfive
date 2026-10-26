use crate::error::{ValidationError, ValidationErrors};
use crate::string::StringValidator;
use crate::traits::Validator;
use std::collections::{BTreeMap, HashMap};

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

impl FormData for HashMap<String, Vec<String>> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.first()).map(|s| s.as_str())
    }
}

impl FormData for HashMap<&str, Vec<&str>> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.first()).copied()
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
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

impl FormData for Vec<(&str, &str)> {
    fn get_value(&self, key: &str) -> Option<&str> {
        self.as_slice().get_value(key)
    }
}

#[derive(Clone)]
struct FormRule {
    name: String,
    validator: StringValidator,
    required: bool,
    required_message: String,
}

/// Fluent schema validator for form payloads (key-value data).
#[derive(Clone, Default)]
pub struct FormValidator {
    rules: Vec<FormRule>,
}

impl FormValidator {
    /// Creates a new empty form validator.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a required field rule with default error message ("This field is required").
    pub fn field(self, name: impl Into<String>, validator: StringValidator) -> Self {
        self.field_msg(name, validator, "This field is required")
    }

    /// Adds a required field rule with a custom error message.
    pub fn field_msg(
        mut self,
        name: impl Into<String>,
        validator: StringValidator,
        required_msg: impl Into<String>,
    ) -> Self {
        self.rules.push(FormRule {
            name: name.into(),
            validator,
            required: true,
            required_message: required_msg.into(),
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
            required_message: String::new(),
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
                            ValidationError::new("required", rule.required_message.clone()),
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
