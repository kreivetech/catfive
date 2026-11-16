use catfive::prelude::*;

#[test]
fn test_default_parsing_catches_hidden_characters() {
    // Normal string validator without ANY special method declared
    let validator = string().min(3);

    // Normal text is OK
    assert!(validator.validate("Hello world").is_ok());

    // Automatically rejects Null byte
    let err = validator.validate("admin\0.pdf").unwrap_err();
    assert_eq!(err.code(), "hidden_character");

    // Automatically rejects Zero-width spaces
    let zwsp_spoof = "admin​user";
    let err_zwsp = validator.validate(zwsp_spoof).unwrap_err();
    assert_eq!(err_zwsp.code(), "hidden_character");

    // Automatically rejects BOM in body
    let bom_str = "\u{FEFF}hidden";
    assert_eq!(validator.validate(bom_str).unwrap_err().code(), "hidden_character");

    // Automatically rejects Trojan Source / Bidirectional override
    let bidi_override = "user\u{202E}txt.exe";
    let err_bidi = validator.validate(bidi_override).unwrap_err();
    assert_eq!(err_bidi.code(), "hidden_character");
}

#[test]
fn test_default_parsing_catches_xss_vectors() {
    // Plain string validator without declaring any xss method
    let validator = string();

    // Normal strings are fine
    assert!(validator.validate("Regular safe comment with 3 < 5").is_ok());

    // Catches script tags
    let err1 = validator.validate("<script>alert('xss')</script>").unwrap_err();
    assert_eq!(err1.code(), "xss_detected");

    // Catches inline handlers
    let err2 = validator.validate("<img src=x onerror=alert(1)>").unwrap_err();
    assert_eq!(err2.code(), "xss_detected");

    // Catches javascript URI schemes
    let err3 = validator.validate("javascript:alert(1)").unwrap_err();
    assert_eq!(err3.code(), "xss_detected");
}

#[test]
fn test_default_parsing_catches_sql_injection() {
    // Plain string validator without declaring any sql injection method
    let validator = string();

    // Legitimate natural phrases are OK
    assert!(validator.validate("This is or isn't a problem").is_ok());
    assert!(validator.validate("Select your option").is_ok());

    // Catches SQL comments
    let err1 = validator.validate("admin' --").unwrap_err();
    assert_eq!(err1.code(), "sql_injection_detected");

    // Catches tautologies
    let err2 = validator.validate("' OR '1'='1").unwrap_err();
    assert_eq!(err2.code(), "sql_injection_detected");

    // Catches stacked destructive statements
    let err3 = validator.validate("foo'; DROP TABLE users;").unwrap_err();
    assert_eq!(err3.code(), "sql_injection_detected");

    // Catches UNION SELECT
    let err4 = validator.validate("1 UNION SELECT username, password FROM users").unwrap_err();
    assert_eq!(err4.code(), "sql_injection_detected");
}

#[derive(Debug)]
struct UserProfileInput {
    username: String,
    bio: String,
    search_query: String,
}

#[test]
fn test_real_world_production_threat_scenario_with_clean_schema() {
    // Standard schema: clean, simple, without any explicit security methods!
    let registration_schema = schema::<UserProfileInput>()
        .field("username", |u| &u.username, string().min(3).max(30))
        .field("bio", |u| &u.bio, string().max(250))
        .field("search_query", |u| &u.search_query, string());

    // 1. Clean valid payload
    let clean_user = UserProfileInput {
        username: "kreivetech".to_string(),
        bio: "Backend developer writing safe Rust systems.".to_string(),
        search_query: "actix web microservices".to_string(),
    };
    assert!(registration_schema.validate(&clean_user).is_ok());

    // 2. Attack payload simulating multi-vector threats:
    // - Trojan Source in username
    // - Stored XSS in bio
    // - SQL Injection in search_query
    let malicious_user = UserProfileInput {
        username: "admin\u{202E}exe".to_string(),
        bio: "<img src=x onerror=alert('hacked')>".to_string(),
        search_query: "' UNION SELECT username, password FROM users --".to_string(),
    };

    let errors = registration_schema.validate(&malicious_user).unwrap_err();
    assert_eq!(errors.len(), 3);
    assert!(errors.has_field("username"));
    assert!(errors.has_field("bio"));
    assert!(errors.has_field("search_query"));

    assert_eq!(errors.get("username").unwrap()[0].code(), "hidden_character");
    assert_eq!(errors.get("bio").unwrap()[0].code(), "xss_detected");
    assert_eq!(errors.get("search_query").unwrap()[0].code(), "sql_injection_detected");
}
