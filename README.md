# Catfive 🌪️

**Catfive** is a category-5, fluent, zero-dependency schema and form validation library for Rust.

Inspired by developer-friendly schema libraries like Zod, **Catfive** gives you composable, expressive validation rules without forcing procedural attribute macros (`#[derive(...)]`) onto your structs.

[![Crates.io](https://img.shields.io/crates/v/catfive.svg)](https://crates.io/crates/catfive)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Highlights

- ⚡ **Macro-Free**: Clean, functional, and composable builder API without `#[derive(Validate)]`.
- 📦 **Zero Dependencies by Default**: Core validators operate entirely on the Rust standard library (`std`).
- 🔄 **Cloneable Validators**: Every validator implements `Clone` for easy template reuse across your codebase.
- 💬 **Custom & Template Error Messages**: Format errors like `println!` with dynamic placeholders (`{char}`, `{index}`, `{min}`, `{count}`, `{value}`) or closures.
- 📋 **Form & Struct Schemas**: Validate untyped key-value form submissions or strongly-typed Rust structs with nested schemas and cross-field refinements.
- 📚 **Collection Validation**: Validate lists, vectors, and slices using `catfive::list` with uniqueness and length rules.
- 🎯 **Optional Features**:
  - `serde`: Serialize `ValidationErrors` directly to JSON for web APIs (Axum, Actix-web, Rocket).
  - `regex`: Enable regular expression pattern matching.

---

## Installation

Add **Catfive** to your `Cargo.toml`:

```toml
[dependencies]
catfive = "0.1.0"
```

With optional features:

```toml
[dependencies]
catfive = { version = "0.1.0", features = ["serde", "regex"] }
```

---

## Quick Start

### 1. Custom & Template Error Messages (`println!`-style)

You can let Catfive use its precise default error messages (which pinpoint the exact offending character), or provide your own message using template placeholders or closures:

```rust
use catfive::string;

// Using template placeholders:
let username_val = string()
    .min_msg(5, "Username must have at least {min} characters (got {count})")
    .block_msg(";@#$&", "Found forbidden character '{char}' at position {position}");

let err = username_val.validate("usr@").unwrap_err();
println!("{}", err.message()); // "Found forbidden character '@' at position 4"

// Using a custom closure:
let strict_val = string().block_with(";@#", |ch, idx| {
    format!("Prohibited character '{}' at 0-indexed position {}", ch, idx)
});
```

---

### 2. Validating Structs (Optional Fields, Refinements & Nested Schemas)

Define schemas for arbitrary structs using typed field selectors, cross-field validations, and nested schemas:

```rust
use catfive::{schema, string, number, ValidationError};

struct Address {
    city: String,
}

struct User {
    username: String,
    bio: Option<String>,
    password: String,
    confirm_password: String,
    address: Address,
}

fn main() {
    let address_schema = schema::<Address>()
        .field("city", |a| &a.city, string().min(3));

    let user_schema = schema::<User>()
        .field("username", |u| &u.username, string().min(3).block(";@#$&"))
        .optional_field("bio", |u| u.bio.as_deref(), string().max(160))
        .refine("confirm_password", |u| {
            if u.password == u.confirm_password {
                Ok(())
            } else {
                Err(ValidationError::new("password_mismatch", "Passwords do not match"))
            }
        })
        .nested("address", |u| &u.address, address_schema);
}
```

---

### 3. Validating Collections & Arrays (`catfive::list`)

Validate vectors and slices with collection-level constraints and individual item checks:

```rust
use catfive::{list, string, Validator};

let tags_validator = list::<&str, _>(string().min(2).alphanumeric())
    .min_items(1)
    .max_items(5)
    .unique();

let tags = vec!["rust", "web", "api"];
assert!(tags_validator.validate(&tags).is_ok());

let duplicate_tags = vec!["rust", "rust"];
assert!(tags_validator.validate(&duplicate_tags).is_err()); // "unique_items"
```

---

### 4. Validating Forms (Key-Value Payloads)

Perfect for HTTP requests, query params, and HTML forms:

```rust
use std::collections::HashMap;
use catfive::{form, string};

let signup_form = form()
    .field("username", string().min(3).block(";@#$&"))
    .field("email", string().email())
    .optional_field("bio", string().max(160));

let mut form_data = HashMap::new();
form_data.insert("username", "kreivetech");
form_data.insert("email", "contact@kreive.com.br");

assert!(signup_form.validate(&form_data).is_ok());
```

---

### 5. Built-in String Validators

- `.email()` / `.email_msg(...)`
- `.url()` / `.url_msg(...)`
- `.uuid()` / `.uuid_msg(...)`
- `.ip()` / `.ip_msg(...)`
- `.in_list(&["admin", "user"])` / `.in_list_msg(...)`
- `.contains("substr")` / `.contains_msg(...)`
- `.starts_with("prefix")` / `.ends_with("suffix")`
- `.alphanumeric()` / `.alphabetic()` / `.numeric()`
- `.block("chars")` / `.allow_only("chars")`
- `.regex(pattern)` (requires `regex` feature)

---

### 6. JSON Serialization for Web APIs (`serde` feature)

Enable the `serde` feature in `Cargo.toml` to serialize errors directly into JSON responses (e.g. HTTP 422 Unprocessable Entity in Axum or Actix-web):

```rust
#[cfg(feature = "serde")]
fn return_json_error(errors: &catfive::ValidationErrors) -> String {
    serde_json::to_string(errors).unwrap()
}
```

---

## License

This project is licensed under the [MIT License](LICENSE).
