//! # Catfive
//!
//! **Catfive** is a category-5, fluent, zero-dependency schema and form validation library for Rust.
//! Inspired by developer-friendly validation libraries, it lets you construct composable validation
//! rules without procedural attribute macros (`#[derive(...)]`).
//!
//! ## Key Features
//! - **Macro-free**: Clean, functional, and composable builder API.
//! - **Zero dependencies**: Core validation operates entirely on `std`.
//! - **Detailed errors**: Rules like `.block("...")` capture the exact offending character for logs and auditing.
//! - **Form and Schema support**: Validate untyped form payloads as well as strongly-typed Rust structs.
//! - **Optional Regex**: Enable the `regex` Cargo feature for arbitrary pattern matching.

pub mod error;
pub mod form;
pub mod number;
pub mod schema;
pub mod string;
pub mod traits;

// Re-exports of primary types
pub use error::{FieldError, ValidationError, ValidationErrors};
pub use form::{FormData, FormValidator};
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
    pub use crate::number::NumberValidator;
    pub use crate::schema::Schema;
    pub use crate::string::StringValidator;
    pub use crate::traits::Validator;
    pub use crate::{form, number, schema, string};
}
