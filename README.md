# Catfive 🌪️

**Catfive** is a fluent, zero-dependency schema and form validation library for Rust.

Designed with composability and developer experience in mind, **Catfive** provides an expressive builder API for runtime data validation without requiring procedural attribute macros (`#[derive(...)]`) on your domain models.

[![Crates.io](https://img.shields.io/crates/v/catfive.svg)](https://crates.io/crates/catfive)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Features

- **Macro-Free**: Decoupled, functional validation logic without polluting domain structs with `#[derive(Validate)]`.
- **Zero Dependencies by Default**: Core validators rely strictly on the Rust standard library (`std`).
- **Universal Pattern Engine (`spek`)**: Declarative pattern matching for standards and verification algorithms:
  - `[CPF]`: Brazilian CPF validation (Modulo 11 check digits, rejects repeated sequences).
  - `[CNPJ]`: Brazilian CNPJ validation (natively supports classic numeric and the 2026 alphanumeric standard IN RFB 2229/2024).
  - `[LUHN]`: ISO/IEC 7812 check digit algorithm (payment cards, IMEI).
  - `[ASCII 65..90]`, `[ASCII]`, `[ASCII PRINTABLE]`, and custom literal character sets (e.g. `[ABC234]`).
- **Application Security & Threat Mitigation**: Built-in protection against malicious inputs:
  - `.no_hidden_chars()`: Detects invisible control codes, zero-width joiners, and Trojan Source bidirectional overrides (`U+202E`, etc.).
  - `.no_xss()`: Identifies Cross-Site Scripting vectors (`<script>`, inline handlers `onerror=`, `javascript:` URIs).
  - `.no_sql_injection()`: Catches SQL injection signatures (tautologies `OR 1=1`, stacked commands `; DROP`, SQL comments `--`, `UNION SELECT`).
- **Composable Error Formatting**: Granular error messages with dynamic runtime placeholders (`{char}`, `{index}`, `{min}`, `{count}`, `{value}`) or custom closures.
- **Form & Struct Schemas**: Validate untyped key-value maps, nested structs, optional fields, and cross-field rules (`.refine()`).
- **Collection Validation**: Validate arrays, slices, and vectors with uniqueness and cardinality constraints via `catfive::list`.
- **Reusability**: All validators implement `Clone` for shared validation rules and templates.
- **Optional Integrations**:
  - `serde`: Direct serialization of `ValidationErrors` for web frameworks (Axum, Actix-web, Rocket).
  - `regex`: Regular expression matching via the `regex` crate.

---

## Installation

Add **Catfive** to your `Cargo.toml`:

```toml
[dependencies]
catfive = "0.2.0"
```

With optional features:

```toml
[dependencies]
catfive = { version = "0.2.0", features = ["serde", "regex"] }
```

---

## Usage

### 1. String Validation & Character Filtering

Enforce constraints, block specific characters, and inspect detailed failure reasons:

```rust
use catfive::string;

let username_validator = string()
    .min(3)
    .max(20)
    .block(";@#$&");

// Valid input:
assert!(username_validator.validate("kreivetech").is_ok());

// Validation failure:
let err = username_validator.validate("admin;drop").unwrap_err();
assert_eq!(err.code(), "blocked_char");
assert_eq!(err.message(), "Character ';' is not allowed");
```

### 2. Error Message Templating

Customize error messages using dynamic contextual placeholders or formatting closures:

```rust
use catfive::string;

// Using placeholder templates:
let username = string()
    .min_msg(5, "Username must contain at least {min} characters (received {count})")
    .block_msg(";@#$&", "Forbidden character '{char}' detected at position {position}");

let err = username.validate("usr@").unwrap_err();
assert_eq!(err.message(), "Forbidden character '@' detected at position 4");

// Using a custom closure:
let strict_rule = string().block_with(";@#", |ch, idx| {
    format!("Prohibited character '{}' found at index {}", ch, idx)
});
```

### 3. Specification Engine (`.spek()`)

Apply official verification algorithms, standards, and character specifications:

```rust
use catfive::string;

// Brazilian CNPJ (supports both classic numeric and the 2026 alphanumeric standard):
let cnpj = string().spek("[CNPJ]").block(";@#");
assert!(cnpj.validate("12.345.678/0001-95").is_ok()); // Numeric
assert!(cnpj.validate("12.ABC.345/01DE-35").is_ok()); // Alphanumeric (IN RFB 2229/2024)

// Brazilian CPF:
let cpf = string().spek("[CPF]");
assert!(cpf.validate("529.982.247-25").is_ok());

// Luhn check digit algorithm (payment cards, IMEI):
let card = string().spek("[LUHN]");
assert!(card.validate("79927398713").is_ok());

// ASCII range constraint:
let uppercase = string().spek("[ASCII 65..90]"); // 'A'..='Z'

// Literal character whitelist:
let custom_charset = string().spek("[ABC234]").block("!");
```

### 4. Struct Validation Without Macros

Define schemas for strongly-typed data structures, including optional fields, cross-field validations, and nested schemas:

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
                Err(ValidationError::new("password_mismatch", "Passwords must match"))
            }
        })
        .nested("address", |u| &u.address, address_schema);
}
```

### 5. Collection & List Validation (`catfive::list`)

Validate slices and vectors with collection-level constraints and per-item checks:

```rust
use catfive::{list, string, Validator};

let tags_validator = list::<&str, _>(string().min(2).alphanumeric())
    .min_items(1)
    .max_items(5)
    .unique();

let tags = vec!["rust", "web", "api"];
assert!(tags_validator.validate(&tags).is_ok());

let duplicate_tags = vec!["rust", "rust"];
assert!(tags_validator.validate(&duplicate_tags).is_err());
```

### 6. Untyped Form Payloads (`catfive::form`)

Validate key-value maps (such as HTTP query strings and URL-encoded form submissions):

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

### 7. Serialization for Web Handlers (`serde` feature)

When the `serde` feature is enabled, `ValidationErrors` can be serialized directly to JSON in HTTP handlers (e.g. Axum, Actix-web, Rocket):

```rust
#[cfg(feature = "serde")]
fn serialize_errors(errors: &catfive::ValidationErrors) -> String {
    serde_json::to_string(errors).unwrap()
}
```

---

## License

This project is licensed under the [MIT License](LICENSE).
