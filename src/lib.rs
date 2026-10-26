//! # Catfive
//!
//! **Catfive** is a category-5, fluent, zero-dependency schema and form validation library for Rust.
//! Inspired by developer-friendly validation libraries, it lets you construct composable validation
//! rules without procedural attribute macros (`#[derive(...)]`).
//!
//! ## Key Features
//! - **Macro-free**: Clean, functional, and composable builder API.
//! - **Zero dependencies**: Core validation operates entirely on `std`.
//! - **Cloneable Validators**: All validators implement `Clone` for easy template reuse.
//! - **Detailed errors & Custom Messages**: Rules support `println!`-style placeholders (e.g. `{char}`, `{min}`, `{count}`) or closures.
//! - **Form and Schema support**: Validate untyped form payloads as well as strongly-typed Rust structs with nested schemas and refinements.
//! - **Collections & Lists**: Validate arrays and vectors with `list(...)`.
//! - **Optional Serde**: Enable the `serde` Cargo feature to serialize validation errors to JSON for REST APIs.
//! - **Optional Regex**: Enable the `regex` Cargo feature for arbitrary pattern matching.

pub mod error;
pub mod form;
pub(crate) mod format_helper;
pub mod list;
pub mod number;
pub mod schema;
pub mod string;
pub mod traits;

// Re-exports of primary types
pub use error::{FieldError, ValidationError, ValidationErrors};
pub use form::{FormData, FormValidator};
pub use list::ListValidator;
pub use number::NumberValidator;
pub use schema::Schema;
pub use string::StringValidator;
pub use traits::Validator;

/// Creates a new fluent string validator.
///
/// # Example
/// ```rust
/// use catfive::string;
///
/// let name_val = string().min(3).max(20).block(";@#$&");
/// ```
pub fn string() -> StringValidator {
    StringValidator::new()
}

/// Creates a new fluent numeric validator for any compatible number type.
///
/// # Example
/// ```rust
/// use catfive::number;
///
/// let age_val = number::<u8>().min(18).max(120);
/// ```
pub fn number<T>() -> NumberValidator<T>
where
    T: PartialOrd + Copy + std::fmt::Display + Send + Sync + 'static,
{
    NumberValidator::new()
}

/// Creates a new fluent list/array validator for collections of items.
///
/// # Example
/// ```rust
/// use catfive::{list, string, Validator};
///
/// let tags_val = list::<&str, _>(string().min(2)).min_items(1).max_items(5);
/// let tags = vec!["rust", "actix"];
/// assert!(tags_val.validate(&tags).is_ok());
/// ```
pub fn list<T, Val>(item_validator: Val) -> ListValidator<T, Val>
where
    T: Send + Sync + 'static,
    Val: Validator<T> + Send + Sync + 'static,
{
    ListValidator::new(item_validator)
}

/// Creates a new form validator for key-value payloads.
///
/// # Example
/// ```rust
/// use catfive::{form, string};
///
/// let contact_form = form()
///     .field("username", string().min(3).block(";@#$&"))
///     .field("email", string().email());
/// ```
pub fn form() -> FormValidator {
    FormValidator::new()
}

/// Creates a new strongly-typed schema validator for struct `T`.
///
/// # Example
/// ```rust
/// use catfive::{schema, string};
///
/// struct User {
///     username: String,
/// }
///
/// let user_schema = schema::<User>()
///     .field("username", |u| &u.username, string().min(3).block(";@#$&"));
/// ```
pub fn schema<T>() -> Schema<T> {
    Schema::new()
}

/// Convenient prelude module to import all essential traits and constructors.
pub mod prelude {
    pub use crate::error::{FieldError, ValidationError, ValidationErrors};
    pub use crate::form::{FormData, FormValidator};
    pub use crate::list::ListValidator;
    pub use crate::number::NumberValidator;
    pub use crate::schema::Schema;
    pub use crate::string::StringValidator;
    pub use crate::traits::Validator;
    pub use crate::{form, list, number, schema, string};
}
