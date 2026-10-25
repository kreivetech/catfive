use crate::error::ValidationErrors;
use crate::traits::Validator;

type SchemaRule<T> = Box<dyn Fn(&T, &mut ValidationErrors) + Send + Sync>;

/// Strongly-typed schema validator for arbitrary structs without macros.
pub struct Schema<T> {
    rules: Vec<SchemaRule<T>>,
}

impl<T> Default for Schema<T> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<T> Schema<T> {
    /// Creates a new schema validator for type `T`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a validation rule bound to a specific field selector on `T`.
    ///
    /// # Example
    /// ```rust
    /// use catfive::{schema, string};
    ///
    /// struct Account {
    ///     username: String,
    /// }
    ///
    /// let account_schema = schema::<Account>()
    ///     .field("username", |a| &a.username, string().min(3).block(";@#$&"));
    /// ```
    pub fn field<F, V, Val>(mut self, name: impl Into<String>, extractor: F, validator: Val) -> Self
    where
        F: Fn(&T) -> &V + Send + Sync + 'static,
        V: ?Sized + 'static,
        Val: Validator<V> + Send + Sync + 'static,
    {
        let field_name = name.into();
        self.rules.push(Box::new(move |obj, errors| {
            let val = extractor(obj);
            let field_errors = validator.validate_all(val);
            errors.add_many(&field_name, field_errors);
        }));
        self
    }

    /// Validates an instance of `T`, accumulating all errors into `ValidationErrors`.
    pub fn validate(&self, target: &T) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        for rule in &self.rules {
            rule(target, &mut errors);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
