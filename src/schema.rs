use crate::error::{ValidationError, ValidationErrors};
use crate::traits::Validator;
use std::sync::Arc;

type SchemaRule<T> = Arc<dyn Fn(&T, &mut ValidationErrors) + Send + Sync>;

/// Strongly-typed schema validator for arbitrary structs without macros.
#[derive(Clone)]
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
        self.rules.push(Arc::new(move |obj, errors| {
            let val = extractor(obj);
            let field_errors = validator.validate_all(val);
            errors.add_many(&field_name, field_errors);
        }));
        self
    }

    /// Adds a validation rule bound to an optional field (`Option<V>`).
    ///
    /// If the extractor returns `None`, validation succeeds.
    /// If it returns `Some(&v)`, the validator is executed.
    ///
    /// # Example
    /// ```rust
    /// use catfive::{schema, string};
    ///
    /// struct Profile {
    ///     bio: Option<String>,
    /// }
    ///
    /// let profile_schema = schema::<Profile>()
    ///     .optional_field("bio", |p| p.bio.as_deref(), string().max(160));
    /// ```
    pub fn optional_field<F, V, Val>(
        mut self,
        name: impl Into<String>,
        extractor: F,
        validator: Val,
    ) -> Self
    where
        F: Fn(&T) -> Option<&V> + Send + Sync + 'static,
        V: ?Sized + 'static,
        Val: Validator<V> + Send + Sync + 'static,
    {
        let field_name = name.into();
        self.rules.push(Arc::new(move |obj, errors| {
            if let Some(val) = extractor(obj) {
                let field_errors = validator.validate_all(val);
                errors.add_many(&field_name, field_errors);
            }
        }));
        self
    }

    /// Adds a struct-level cross-field validation rule (refinement).
    ///
    /// # Example
    /// ```rust
    /// use catfive::{schema, string, ValidationError};
    ///
    /// struct Register {
    ///     password: String,
    ///     confirm: String,
    /// }
    ///
    /// let reg_schema = schema::<Register>()
    ///     .field("password", |r| &r.password, string().min(8))
    ///     .refine("confirm", |r| {
    ///         if r.password == r.confirm {
    ///             Ok(())
    ///         } else {
    ///             Err(ValidationError::new("mismatch", "Passwords do not match"))
    ///         }
    ///     });
    /// ```
    pub fn refine<F>(mut self, field: impl Into<String>, check: F) -> Self
    where
        F: Fn(&T) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        let field_name = field.into();
        self.rules.push(Arc::new(move |obj, errors| {
            if let Err(err) = check(obj) {
                errors.add(&field_name, err);
            }
        }));
        self
    }

    /// Composes a nested schema, prefixing all child errors with `"{prefix}."`.
    ///
    /// # Example
    /// ```rust
    /// use catfive::{schema, string};
    ///
    /// struct Address {
    ///     street: String,
    /// }
    ///
    /// struct User {
    ///     name: String,
    ///     address: Address,
    /// }
    ///
    /// let address_schema = schema::<Address>()
    ///     .field("street", |a| &a.street, string().min(5));
    ///
    /// let user_schema = schema::<User>()
    ///     .field("name", |u| &u.name, string().min(2))
    ///     .nested("address", |u| &u.address, address_schema);
    /// ```
    pub fn nested<F, SubT>(
        mut self,
        prefix: impl Into<String>,
        extractor: F,
        sub_schema: Schema<SubT>,
    ) -> Self
    where
        F: Fn(&T) -> &SubT + Send + Sync + 'static,
        SubT: 'static,
    {
        let prefix_str = prefix.into();
        let prefix_with_dot = if prefix_str.ends_with('.') {
            prefix_str
        } else {
            format!("{}.", prefix_str)
        };
        self.rules.push(Arc::new(move |obj, errors| {
            let sub_obj = extractor(obj);
            if let Err(mut sub_errors) = sub_schema.validate(sub_obj) {
                sub_errors.prefix_fields(&prefix_with_dot);
                errors.merge(sub_errors);
            }
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

impl<T: Send + Sync + 'static> Validator<T> for Schema<T> {
    fn validate_all(&self, value: &T) -> Vec<ValidationError> {
        let mut validation_errors = ValidationErrors::new();
        for rule in &self.rules {
            rule(value, &mut validation_errors);
        }
        validation_errors
            .into_inner()
            .into_iter()
            .flat_map(|(field, errs)| {
                errs.into_iter().map(move |err| {
                    ValidationError::custom_code(
                        err.code().to_string(),
                        format!("{}: {}", field, err.message()),
                    )
                })
            })
            .collect()
    }
}
