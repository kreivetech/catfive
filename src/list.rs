use crate::error::ValidationError;
use crate::format_helper::format_template;
use crate::traits::Validator;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

type ListRule<T> = Arc<dyn Fn(&[T]) -> Option<ValidationError> + Send + Sync>;

/// Fluent validator for collections, vectors, and slices.
#[derive(Clone)]
pub struct ListValidator<T, Val> {
    item_validator: Val,
    rules: Vec<ListRule<T>>,
    _phantom: PhantomData<T>,
}

impl<T, Val> ListValidator<T, Val>
where
    T: Send + Sync + 'static,
    Val: Validator<T> + Send + Sync + 'static,
{
    /// Creates a new list validator with an item validator.
    pub fn new(item_validator: Val) -> Self {
        Self {
            item_validator,
            rules: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Enforces a minimum number of elements.
    pub fn min_items(self, min_count: usize) -> Self {
        self.min_items_msg(
            min_count,
            "Must contain at least {min} item(s), got {count}",
        )
    }

    /// Enforces a minimum number of elements with a custom message template.
    ///
    /// Available placeholders: `{min}`, `{count}`.
    pub fn min_items_msg(mut self, min_count: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |items| {
            if items.len() < min_count {
                let msg = format_template(
                    &tpl,
                    &[
                        ("min", &min_count.to_string()),
                        ("count", &items.len().to_string()),
                    ],
                );
                Some(ValidationError::new("min_items", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces a maximum number of elements.
    pub fn max_items(self, max_count: usize) -> Self {
        self.max_items_msg(max_count, "Must not exceed {max} item(s), got {count}")
    }

    /// Enforces a maximum number of elements with a custom message template.
    ///
    /// Available placeholders: `{max}`, `{count}`.
    pub fn max_items_msg(mut self, max_count: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |items| {
            if items.len() > max_count {
                let msg = format_template(
                    &tpl,
                    &[
                        ("max", &max_count.to_string()),
                        ("count", &items.len().to_string()),
                    ],
                );
                Some(ValidationError::new("max_items", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Enforces an exact number of elements.
    pub fn exact_items(self, exact_count: usize) -> Self {
        self.exact_items_msg(
            exact_count,
            "Must contain exactly {length} item(s), got {count}",
        )
    }

    /// Enforces an exact number of elements with a custom message template.
    ///
    /// Available placeholders: `{length}`, `{count}`.
    pub fn exact_items_msg(mut self, exact_count: usize, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |items| {
            if items.len() != exact_count {
                let msg = format_template(
                    &tpl,
                    &[
                        ("length", &exact_count.to_string()),
                        ("count", &items.len().to_string()),
                    ],
                );
                Some(ValidationError::new("exact_items", msg))
            } else {
                None
            }
        }));
        self
    }

    /// Adds a custom user-defined rule for the whole list.
    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&[T]) -> Result<(), ValidationError> + Send + Sync + 'static,
    {
        self.rules
            .push(Arc::new(move |items| validator(items).err()));
        self
    }
}

impl<T, Val> ListValidator<T, Val>
where
    T: Eq + Hash + Display + Send + Sync + 'static,
    Val: Validator<T> + Send + Sync + 'static,
{
    /// Enforces that all elements in the collection are unique.
    pub fn unique(self) -> Self {
        self.unique_msg("Duplicate item '{value}' at index {index}")
    }

    /// Enforces that all elements in the collection are unique with a custom message template.
    ///
    /// Available placeholders: `{value}`, `{index}`.
    pub fn unique_msg(mut self, msg_template: &str) -> Self {
        let tpl = msg_template.to_string();
        self.rules.push(Arc::new(move |items| {
            let mut seen = HashSet::new();
            for (idx, item) in items.iter().enumerate() {
                if !seen.insert(item) {
                    let msg = format_template(
                        &tpl,
                        &[("value", &item.to_string()), ("index", &idx.to_string())],
                    );
                    return Some(ValidationError::new("unique_items", msg));
                }
            }
            None
        }));
        self
    }
}

impl<T, Val> Validator<[T]> for ListValidator<T, Val>
where
    T: Send + Sync + 'static,
    Val: Validator<T> + Send + Sync + 'static,
{
    fn validate_all(&self, value: &[T]) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // 1. List-level rules
        for rule in &self.rules {
            if let Some(err) = rule(value) {
                errors.push(err);
            }
        }

        // 2. Item-level validation
        for (idx, item) in value.iter().enumerate() {
            let item_errors = self.item_validator.validate_all(item);
            for err in item_errors {
                errors.push(ValidationError::custom_code(
                    err.code().to_string(),
                    format!("[index {}]: {}", idx, err.message()),
                ));
            }
        }

        errors
    }
}

impl<T, Val> Validator<Vec<T>> for ListValidator<T, Val>
where
    T: Send + Sync + 'static,
    Val: Validator<T> + Send + Sync + 'static,
{
    fn validate_all(&self, value: &Vec<T>) -> Vec<ValidationError> {
        self.validate_all(value.as_slice())
    }
}
