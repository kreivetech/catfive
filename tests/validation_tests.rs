use catfive::prelude::*;
use std::collections::HashMap;

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

#[test]
fn test_custom_template_messages() {
    let val = string()
        .min_msg(5, "Minimum {min} characters required, got {count}")
        .block_msg(
            ";@#",
            "Offending character '{char}' detected at position {position}",
        );

    let err1 = val.validate("abc").unwrap_err();
    assert_eq!(err1.message(), "Minimum 5 characters required, got 3");

    let err2 = val.validate("hello@world").unwrap_err();
    assert_eq!(
        err2.message(),
        "Offending character '@' detected at position 6"
    );

    let num_val = number::<u32>().range_msg(10, 50, "Value {value} out of bounds [{min}..{max}]");
    let err3 = num_val.validate(&5).unwrap_err();
    assert_eq!(err3.message(), "Value 5 out of bounds [10..50]");
}

#[test]
fn test_block_with_closure() {
    let val = string().block_with(";@#", |ch, idx| {
        format!(
            "Strict block: character '{}' found at 0-indexed pos {}",
            ch, idx
        )
    });

    let err = val.validate("foo;bar").unwrap_err();
    assert_eq!(
        err.message(),
        "Strict block: character ';' found at 0-indexed pos 3"
    );
}

#[test]
fn test_validators_are_cloneable() {
    let base_string = string().min(3).max(20);
    let cloned_string = base_string.clone().block("!");

    assert!(base_string.validate("hi!").is_ok());
    assert!(cloned_string.validate("hi!").is_err());

    let base_num = number::<i32>().min(0);
    let cloned_num = base_num.clone().max(10);
    assert!(base_num.validate(&20).is_ok());
    assert!(cloned_num.validate(&20).is_err());
}

struct Account {
    name: String,
    bio: Option<String>,
    password: String,
    confirm_password: String,
}

#[test]
fn test_schema_optional_field_and_refine() {
    let account_schema = schema::<Account>()
        .field("name", |a| &a.name, string().min(2))
        .optional_field("bio", |a| a.bio.as_deref(), string().max(20))
        .refine("confirm_password", |a| {
            if a.password == a.confirm_password {
                Ok(())
            } else {
                Err(ValidationError::new("mismatch", "Passwords must match"))
            }
        });

    let valid = Account {
        name: "Alice".to_string(),
        bio: None,
        password: "secret".to_string(),
        confirm_password: "secret".to_string(),
    };
    assert!(account_schema.validate(&valid).is_ok());

    let invalid_mismatch = Account {
        name: "Alice".to_string(),
        bio: Some("Too long bio that exceeds maximum allowed length".to_string()),
        password: "secret".to_string(),
        confirm_password: "other".to_string(),
    };
    let errors = account_schema.validate(&invalid_mismatch).unwrap_err();
    assert!(errors.has_field("bio"));
    assert!(errors.has_field("confirm_password"));
    assert_eq!(
        errors.get("confirm_password").unwrap()[0].code(),
        "mismatch"
    );
}

struct Address {
    city: String,
}

struct Company {
    name: String,
    address: Address,
}

#[test]
fn test_schema_nested() {
    let address_schema = schema::<Address>().field("city", |a| &a.city, string().min(3));

    let company_schema = schema::<Company>()
        .field("name", |c| &c.name, string().min(2))
        .nested("address", |c| &c.address, address_schema);

    let company = Company {
        name: "K".to_string(),
        address: Address {
            city: "NY".to_string(),
        },
    };

    let errors = company_schema.validate(&company).unwrap_err();
    assert!(errors.has_field("name"));
    assert!(errors.has_field("address.city"));
}

#[test]
fn test_list_validation() {
    let tags_val = list::<&str, _>(string().min(2).alphanumeric())
        .min_items(1)
        .max_items(3)
        .unique();

    let valid_tags = vec!["rust", "web", "api"];
    assert!(tags_val.validate(&valid_tags).is_ok());

    // Duplicate item
    let dup_tags = vec!["rust", "rust"];
    let err = tags_val.validate(&dup_tags).unwrap_err();
    assert_eq!(err.code(), "unique_items");

    // Item validation failure
    let invalid_item = vec!["a", "valid"];
    let errs = tags_val.validate_all(&invalid_item);
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message().contains("[index 0]"));
}

#[test]
fn test_additional_string_validators() {
    let url_val = string().url();
    assert!(url_val.validate("https://google.com").is_ok());
    assert!(url_val.validate("ftp://invalid").is_err());

    let uuid_val = string().uuid();
    assert!(
        uuid_val
            .validate("123e4567-e89b-12d3-a456-426614174000")
            .is_ok()
    );
    assert!(uuid_val.validate("not-a-uuid").is_err());

    let ip_val = string().ip();
    assert!(ip_val.validate("192.168.1.1").is_ok());
    assert!(ip_val.validate("::1").is_ok());
    assert!(ip_val.validate("999.999.999.999").is_err());

    let role_val = string().in_list(&["admin", "user", "guest"]);
    assert!(role_val.validate("admin").is_ok());
    assert!(role_val.validate("superuser").is_err());

    let sub_val = string().contains("abc");
    assert!(sub_val.validate("xyzabcdef").is_ok());
    assert!(sub_val.validate("xyz").is_err());
}

#[test]
fn test_form_with_vector_values() {
    let user_form = form()
        .field("username", string().min(3))
        .field("email", string().email());

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    map.insert("username".to_string(), vec!["kreivetech".to_string()]);
    map.insert(
        "email".to_string(),
        vec!["contact@kreive.com.br".to_string()],
    );

    assert!(user_form.validate(&map).is_ok());

    let mut str_map: HashMap<&str, Vec<&str>> = HashMap::new();
    str_map.insert("username", vec!["kreivetech"]);
    str_map.insert("email", vec!["contact@kreive.com.br"]);
    assert!(user_form.validate(&str_map).is_ok());
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_serialization() {
    let mut errors = ValidationErrors::new();
    errors.add(
        "email",
        ValidationError::new("invalid_format", "Invalid email address"),
    );
    errors.add(
        "email",
        ValidationError::new("blocked_char", "Character ';' not allowed"),
    );

    let json = serde_json::to_string(&errors).unwrap();
    assert!(json.contains("email"));
    assert!(json.contains("invalid_format"));
    assert!(json.contains("Invalid email address"));

    // Deserialization round-trip
    let deserialized: ValidationErrors = serde_json::from_str(&json).unwrap();
    assert_eq!(errors, deserialized);
}
