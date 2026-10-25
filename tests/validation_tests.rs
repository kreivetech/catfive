use std::collections::HashMap;
use catfive::prelude::*;

#[test]
fn test_string_block_catches_forbidden_characters() {
    let name_validator = string().min(3).block(";@#$&");

    // Valid string
    assert!(name_validator.validate("John Doe").is_ok());

    // Invalid string with ';'
    let err = name_validator.validate("John;Doe").unwrap_err();
    assert_eq!(err.code(), "blocked_char");
    assert_eq!(err.message(), "Character ';' is not allowed");

    // Invalid string with '@'
    let err = name_validator.validate("admin@root").unwrap_err();
    assert_eq!(err.code(), "blocked_char");
    assert_eq!(err.message(), "Character '@' is not allowed");
}

#[test]
fn test_string_min_max_and_email() {
    let email_val = string().min(5).email();

    assert!(email_val.validate("user@example.com").is_ok());
    assert!(email_val.validate("invalid-email").is_err());
    assert!(email_val.validate("user@").is_err());
    assert!(email_val.validate("@domain.com").is_err());
}

#[test]
fn test_number_validation() {
    let age_validator = number::<i32>().min(18).max(100);

    assert!(age_validator.validate(&25).is_ok());
    assert!(age_validator.validate(&17).is_err());
    assert!(age_validator.validate(&101).is_err());
}

#[test]
fn test_form_validation() {
    let signup_form = form()
        .field("username", string().min(3).block(";@#$&"))
        .field("email", string().email())
        .optional_field("bio", string().max(100));

    let mut data = HashMap::new();
    data.insert("username", "alice");
    data.insert("email", "alice@example.com");

    assert!(signup_form.validate(&data).is_ok());

    // Test with forbidden character in username
    data.insert("username", "alice;hacker");
    let errs = signup_form.validate(&data).unwrap_err();
    assert!(errs.get("username").is_some());
    assert_eq!(errs.get("username").unwrap()[0].code(), "blocked_char");

    // Test with missing required field
    data.remove("username");
    let errs = signup_form.validate(&data).unwrap_err();
    assert_eq!(errs.get("username").unwrap()[0].code(), "required");
}

struct UserProfile {
    username: String,
    age: u8,
}

#[test]
fn test_struct_schema_without_macros() {
    let profile_schema = schema::<UserProfile>()
        .field("username", |p| &p.username, string().min(3).block(";@#$&"))
        .field("age", |p| &p.age, number::<u8>().min(18));

    let valid_user = UserProfile {
        username: "kreivetech".to_string(),
        age: 25,
    };
    assert!(profile_schema.validate(&valid_user).is_ok());

    let invalid_user = UserProfile {
        username: "h@ck;".to_string(),
        age: 15,
    };
    let errs = profile_schema.validate(&invalid_user).unwrap_err();
    assert_eq!(errs.len(), 2);
    assert!(errs.get("username").is_some());
    assert!(errs.get("age").is_some());
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_feature() {
    let code_validator = string().regex(r"^[A-Z]{3}-\d{4}$");

    assert!(code_validator.validate("ABC-1234").is_ok());
    assert!(code_validator.validate("abc-1234").is_err());
    assert!(code_validator.validate("ABCD-123").is_err());
}
