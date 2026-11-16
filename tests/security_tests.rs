use catfive::prelude::*;

#[test]
fn test_security_hidden_characters_detection() {
    let validator = string().no_hidden_chars();

    // Normal text with spaces and line breaks is OK
    assert!(
        validator
            .validate("Hello world\nSecond line\tTabbed")
            .is_ok()
    );

    // Null byte injection (common in path traversal / C-binding bypass)
    let null_byte = "admin\0.pdf";
    let err = validator.validate(null_byte).unwrap_err();
    assert_eq!(err.code(), "hidden_character");

    // Zero-width space (U+200B) spoofing
    let zwsp_spoof = "admin​user"; // contains zero-width space
    assert!(validator.validate(zwsp_spoof).is_err());

    // Byte Order Mark (BOM U+FEFF) hidden in string
    let bom_str = "\u{FEFF}hidden";
    assert!(validator.validate(bom_str).is_err());

    // Trojan Source / Bidirectional override (RLO U+202E)
    let bidi_override = "user\u{202E}txt.exe";
    let err_bidi = validator.validate(bidi_override).unwrap_err();
    assert_eq!(err_bidi.code(), "hidden_character");

    // Custom message
    let custom_val = string().no_hidden_chars_msg("Disallowed hidden char at pos {position}");
    let err_custom = custom_val.validate("test\0").unwrap_err();
    assert_eq!(err_custom.message(), "Disallowed hidden char at pos 5");
}

#[test]
fn test_security_xss_detection() {
    let validator = string().no_xss();

    // Normal safe strings
    assert!(
        validator
            .validate("Regular user comment about <math> 3 < 5")
            .is_ok()
    );
    assert!(
        validator
            .validate("Here is a link: https://example.com")
            .is_ok()
    );

    // Classic <script> injection
    assert!(validator.validate("<script>alert('xss')</script>").is_err());
    assert!(
        validator
            .validate("<SCRIPT SRC='http://evil.com/xss.js'></SCRIPT>")
            .is_err()
    );

    // Inline event handlers
    assert!(validator.validate("<img src=x onerror=alert(1)>").is_err());
    assert!(
        validator
            .validate("<svg onload=alert(document.cookie)>")
            .is_err()
    );
    assert!(validator.validate("<body onload = alert(1)>").is_err());
    assert!(
        validator
            .validate("<a href='#' onclick='malicious()'>Click</a>")
            .is_err()
    );

    // JavaScript pseudo-protocols
    assert!(validator.validate("javascript:alert(1)").is_err());
    assert!(validator.validate("javascript :alert(1)").is_err());
    assert!(
        validator
            .validate("data:text/html;base64,PHNjcmlwdD4=")
            .is_err()
    );

    // Malicious HTML containers (iframe, object, embed)
    assert!(
        validator
            .validate("<iframe src='http://attacker.com'></iframe>")
            .is_err()
    );
    assert!(
        validator
            .validate("<object data='malicious.swf'></object>")
            .is_err()
    );

    // Custom message
    let custom = string().no_xss_msg("XSS violation: {pattern}");
    let err = custom.validate("<script>foo</script>").unwrap_err();
    assert_eq!(err.message(), "XSS violation: script_tag");
}

#[test]
fn test_security_sql_injection_detection() {
    let validator = string().no_sql_injection();

    // Legitimate natural language phrases that shouldn't trigger false positives
    assert!(validator.validate("This is or isn't a problem").is_ok());
    assert!(validator.validate("Select your preferred option").is_ok());
    assert!(validator.validate("Order placed successfully").is_ok());

    // Classic SQL comments
    assert!(validator.validate("admin' --").is_err());
    assert!(validator.validate("admin' /* comment */").is_err());

    // Classic tautologies
    assert!(validator.validate("' OR '1'='1").is_err());
    assert!(validator.validate("' or 1=1").is_err());
    assert!(validator.validate("\" or \"\"=\"").is_err());

    // Destructive stacked commands
    assert!(validator.validate("foo'; DROP TABLE users;").is_err());
    assert!(validator.validate("1; DELETE FROM accounts").is_err());
    assert!(validator.validate("test'; TRUNCATE TABLE logs;").is_err());

    // UNION SELECT attacks
    assert!(
        validator
            .validate("1 UNION SELECT username, password FROM users")
            .is_err()
    );
    assert!(
        validator
            .validate("1' UNION ALL SELECT null, null--")
            .is_err()
    );

    // Time-based blind SQLi functions
    assert!(validator.validate("1 AND SLEEP(5)").is_err());
    assert!(validator.validate("1; WAITFOR DELAY '0:0:5'").is_err());
    assert!(validator.validate("1 AND pg_sleep(5)").is_err());

    // Custom message
    let custom = string().no_sql_injection_msg("SQLi signature detected: {pattern}");
    let err = custom.validate("' OR 1=1 --").unwrap_err();
    assert!(err.message().contains("SQLi signature detected"));
}

#[derive(Debug)]
struct UserRegistrationInput {
    username: String,
    bio: String,
    document: String,
    search_query: String,
}

#[test]
fn test_real_world_production_threat_scenario() {
    // A comprehensive schema simulating an actual web application registration DTO
    let registration_schema = schema::<UserRegistrationInput>()
        .field(
            "username",
            |u| &u.username,
            string()
                .min(3)
                .max(30)
                .no_hidden_chars()
                .no_xss()
                .block(";`|&$"),
        )
        .field(
            "bio",
            |u| &u.bio,
            string().max(250).no_hidden_chars().no_xss(),
        )
        .field("document", |u| &u.document, string().spek("[CPF]"))
        .field(
            "search_query",
            |u| &u.search_query,
            string().no_sql_injection().no_hidden_chars(),
        );

    // 1. Valid production input
    let clean_user = UserRegistrationInput {
        username: "kreivetech".to_string(),
        bio: "Backend developer writing safe Rust systems.".to_string(),
        document: "529.982.247-25".to_string(),
        search_query: "actix web microservices".to_string(),
    };
    assert!(registration_schema.validate(&clean_user).is_ok());

    // 2. Attack payload simulating multiple concurrent threat vectors:
    // - Trojan Source bidi override in username
    // - Stored XSS in bio
    // - Fake CPF
    // - SQL Injection tautology in search query
    let malicious_user = UserRegistrationInput {
        username: "admin\u{202E}exe".to_string(),
        bio: "<img src=x onerror=alert('hacked')>".to_string(),
        document: "000.000.000-00".to_string(),
        search_query: "' UNION SELECT username, password FROM users --".to_string(),
    };

    let errors = registration_schema.validate(&malicious_user).unwrap_err();
    assert_eq!(errors.len(), 4);
    assert!(errors.has_field("username"));
    assert!(errors.has_field("bio"));
    assert!(errors.has_field("document"));
    assert!(errors.has_field("search_query"));

    assert_eq!(
        errors.get("username").unwrap()[0].code(),
        "hidden_character"
    );
    assert_eq!(errors.get("bio").unwrap()[0].code(), "xss_detected");
    assert_eq!(errors.get("document").unwrap()[0].code(), "invalid_cpf");
    assert_eq!(
        errors.get("search_query").unwrap()[0].code(),
        "sql_injection_detected"
    );
}
