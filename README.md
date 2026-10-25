# Catfive 🌪️

**Catfive** is a category-5, fluent, zero-dependency schema and form validation library for Rust.

Inspired by developer-friendly schema libraries like Zod, **Catfive** gives you composable, expressive validation rules without forcing procedural attribute macros (`#[derive(...)]`) onto your structs.

[![Crates.io](https://img.shields.io/crates/v/catfive.svg)](https://crates.io/crates/catfive)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Highlights

- ⚡ **Macro-Free**: Clean, functional, and composable builder API without `#[derive(Validate)]`.
- 📦 **Zero Dependencies by Default**: Core validators operate entirely on the Rust standard library (`std`).
- 🔍 **Detailed & Actionable Errors**: Rules like `.block("...")` report the exact offending character for audit logging.
- 📋 **Form & Struct Schemas**: Validate untyped key-value form submissions or strongly-typed Rust data structures.
- 🎯 **Optional Regex**: Enable the `regex` feature flag whenever you need complex pattern matching.

---

## Installation

Add **Catfive** to your `Cargo.toml`:

```toml
[dependencies]
catfive = "0.1.0"
```

To enable regular expression support:

```toml
[dependencies]
catfive = { version = "0.1.0", features = ["regex"] }
```

---

## Quick Start

### 1. Standalone String Validator & `.block()`

Block specific characters from user input and inspect exactly which forbidden character triggered the validation error:

```rust
use catfive::string;

fn main() {
    let username_validator = string()
        .min(3)
        .max(20)
        .block(";@#$&");

    // Ok:
    assert!(username_validator.validate("kreivetech").is_ok());

    // Err: captures the exact forbidden character
    let err = username_validator.validate("admin;drop").unwrap_err();
    println!("Code: {}", err.code());       // "blocked_char"
    println!("Message: {}", err.message()); // "Character ';' is not allowed"
}
```

---

### 2. Validating Forms (Key-Value Payloads)

Perfect for HTTP requests, query params, and HTML forms:

```rust
use std::collections::HashMap;
use catfive::{form, string};

fn main() {
    let signup_form = form()
        .field("username", string().min(3).block(";@#$&"))
        .field("email", string().email())
        .optional_field("bio", string().max(160));

    let mut form_data = HashMap::new();
    form_data.insert("username", "kreivetech");
    form_data.insert("email", "contact@kreive.com.br");

    match signup_form.validate(&form_data) {
        Ok(()) => println!("Form is valid!"),
        Err(errors) => {
            for (field, field_errors) in errors.iter() {
                for error in field_errors {
                    eprintln!("Field '{}': {}", field, error.message());
                }
            }
        }
    }
}
```

---

### 3. Validating Structs Without Macros

Define schemas for arbitrary structs using typed field selectors:

```rust
use catfive::{schema, string, number};

struct Account {
    username: String,
    age: u8,
}

fn main() {
    let account_schema = schema::<Account>()
        .field("username", |acc| &acc.username, string().min(3).block(";@#$&"))
        .field("age", |acc| &acc.age, number::<u8>().min(18));

    let account = Account {
        username: "kreivetech".to_string(),
        age: 22,
    };

    if let Err(errors) = account_schema.validate(&account) {
        eprintln!("{}", errors);
    }
}
```

---

### 4. Optional Regex Support

```rust
#[cfg(feature = "regex")]
use catfive::string;

let code_validator = string().regex(r"^[A-Z]{3}-\d{4}$");
assert!(code_validator.validate("ABC-1234").is_ok());
```

---

## License

This project is licensed under the [MIT License](LICENSE).
