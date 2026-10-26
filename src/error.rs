use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

/// Represents a single validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidationError {
    code: Cow<'static, str>,
    message: String,
}

impl ValidationError {
    /// Creates a new validation error with a machine-readable code and a human-readable message.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code: Cow::Borrowed(code),
            message: message.into(),
        }
    }

    /// Creates a new validation error with a dynamic or static code.
    pub fn custom_code(code: impl Into<Cow<'static, str>>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Error code identifying the validation failure type (e.g. `"blocked_char"`, `"min_length"`).
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Detailed message describing the failure.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Represents a validation failure bound to a specific field name.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldError {
    field: String,
    errors: Vec<ValidationError>,
}

impl FieldError {
    /// Creates a new field error containing one or more validation errors for the field.
    pub fn new(field: impl Into<String>, errors: Vec<ValidationError>) -> Self {
        Self {
            field: field.into(),
            errors,
        }
    }

    /// Name of the invalid field.
    pub fn field(&self) -> &str {
        &self.field
    }

    /// List of errors associated with this field.
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "field '{}' has {} error(s)",
            self.field,
            self.errors.len()
        )
    }
}

impl std::error::Error for FieldError {}

/// Collection of validation errors indexed by field name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidationErrors {
    fields: HashMap<String, Vec<ValidationError>>,
}

impl ValidationErrors {
    /// Creates an empty collection of validation errors.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Adds a validation error for a given field name.
    pub fn add(&mut self, field: impl Into<String>, error: ValidationError) {
        self.fields.entry(field.into()).or_default().push(error);
    }

    /// Adds multiple validation errors for a given field name.
    pub fn add_many(&mut self, field: impl Into<String>, errors: Vec<ValidationError>) {
        if !errors.is_empty() {
            self.fields.entry(field.into()).or_default().extend(errors);
        }
    }

    /// Merges another set of `ValidationErrors` into this one.
    pub fn merge(&mut self, other: ValidationErrors) {
        for (field, errors) in other.fields {
            self.add_many(field, errors);
        }
    }

    /// Prefixes all field keys with the given prefix (e.g. `"address."` for nested structures).
    pub fn prefix_fields(&mut self, prefix: &str) {
        let old_fields = std::mem::take(&mut self.fields);
        for (field, errors) in old_fields {
            self.fields.insert(format!("{}{}", prefix, field), errors);
        }
    }

    /// Returns `true` if there are no validation errors.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the total count of invalid fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns `true` if the specified field has any validation errors.
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    /// Returns errors for a specific field, if any.
    pub fn get(&self, field: &str) -> Option<&[ValidationError]> {
        self.fields.get(field).map(|v| v.as_slice())
    }

    /// Returns an iterator over all `(field_name, errors)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[ValidationError])> {
        self.fields.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    /// Returns a flattened list of all errors as `(field_name, &ValidationError)` tuples.
    pub fn all_errors(&self) -> Vec<(&str, &ValidationError)> {
        let mut all = Vec::new();
        for (field, errs) in &self.fields {
            for err in errs {
                all.push((field.as_str(), err));
            }
        }
        all
    }

    /// Consumes the container and returns the inner `HashMap`.
    pub fn into_inner(self) -> HashMap<String, Vec<ValidationError>> {
        self.fields
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Validation failed with {} invalid field(s):",
            self.fields.len()
        )?;
        for (field, errors) in &self.fields {
            for err in errors {
                writeln!(f, "  - [{}]: {}", field, err)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}
