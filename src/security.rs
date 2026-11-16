use crate::error::ValidationError;

/// Scans a string for invisible unicode characters, bidirectional control overrides,
/// zero-width characters, and non-printable control codes.
///
/// Returns the first offending character and its zero-indexed position if detected.
pub fn find_hidden_character(val: &str) -> Option<(char, usize)> {
    for (idx, ch) in val.chars().enumerate() {
        let code = ch as u32;

        // Null byte
        if code == 0 {
            return Some((ch, idx));
        }

        // C0 Controls (except standard whitespace \t=9, \n=10, \r=13)
        if code < 32 && code != 9 && code != 10 && code != 13 {
            return Some((ch, idx));
        }

        // C1 Controls & Delete
        if (127..=159).contains(&code) {
            return Some((ch, idx));
        }

        // Zero-width spaces & joiners
        // U+200B (Zero Width Space), U+200C (ZWNJ), U+200D (ZWJ),
        // U+200E (LRM), U+200F (RLM), U+FEFF (BOM / Zero Width No-Break Space), U+2060 (Word Joiner)
        if (0x200B..=0x200F).contains(&code) || code == 0xFEFF || code == 0x2060 {
            return Some((ch, idx));
        }

        // Bidirectional override characters (used in Trojan Source attacks)
        // U+202A to U+202E, U+2066 to U+2069
        if (0x202A..=0x202E).contains(&code) || (0x2066..=0x2069).contains(&code) {
            return Some((ch, idx));
        }
    }
    None
}

/// Detects dangerous Cross-Site Scripting (XSS) patterns:
/// - `<script...>` tags
/// - `javascript:...` URI schemes
/// - Inline event handlers (`onerror=`, `onload=`, `onclick=`, etc.)
/// - `<iframe>`, `<object>`, `<embed>`, `<svg onload...>`
pub fn find_xss_pattern(val: &str) -> Option<&'static str> {
    let lower = val.to_ascii_lowercase();

    // 1. Script tags
    if lower.contains("<script") || lower.contains("</script") {
        return Some("script_tag");
    }

    // 2. Dangerous HTML tags
    if lower.contains("<iframe")
        || lower.contains("<object")
        || lower.contains("<embed")
        || lower.contains("<applet")
        || lower.contains("<meta")
        || lower.contains("<base")
    {
        return Some("dangerous_tag");
    }

    // 3. JavaScript pseudo-protocol
    let stripped_lower: String = lower
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\0')
        .collect();
    if stripped_lower.contains("javascript:") || stripped_lower.contains("vbscript:") {
        return Some("javascript_uri");
    }

    if stripped_lower.contains("data:text/html") {
        return Some("data_html_uri");
    }

    // 4. Inline HTML Event Handlers: onerror=, onload=, onclick=, onmouseover=, etc.
    let event_handlers = [
        "onerror=",
        "onload=",
        "onclick=",
        "onmouseover=",
        "onfocus=",
        "onblur=",
        "onkeydown=",
        "onkeyup=",
        "onsubmit=",
        "onchange=",
    ];
    for &handler in &event_handlers {
        if lower.contains(handler) {
            return Some("inline_event_handler");
        }
    }

    // Handlers with whitespace before '=' (e.g. `onload =`)
    let handler_prefixes = ["onerror", "onload", "onclick", "onmouseover", "onfocus"];
    for &prefix in &handler_prefixes {
        if let Some(pos) = lower.find(prefix) {
            let after = &lower[pos + prefix.len()..];
            let trimmed_after = after.trim_start();
            if trimmed_after.starts_with('=') {
                return Some("inline_event_handler");
            }
        }
    }

    None
}

/// Detects obvious SQL Injection patterns and signatures:
/// - SQL line comments (`--`, `/* ... */`)
/// - Classic tautologies (`' OR '1'='1`, `' OR 1=1`, `" OR ""="`, `OR true`)
/// - Stacked queries with dangerous keywords (`; DROP TABLE`, `; DELETE FROM`, `; TRUNCATE`)
/// - UNION based injections (`UNION SELECT`, `UNION ALL SELECT`)
/// - Time-based / sleep functions (`SLEEP(`, `WAITFOR DELAY`, `BENCHMARK(`)
pub fn find_sql_injection_pattern(val: &str) -> Option<&'static str> {
    let lower = val.to_ascii_lowercase();

    // 1. SQL Comments
    if lower.contains("--") || lower.contains("/*") || lower.contains("*/") {
        return Some("sql_comment");
    }

    // 2. UNION SELECT injection
    if lower.contains("union select") || lower.contains("union all select") {
        return Some("sql_union_select");
    }

    // 3. Stacked destructive commands
    let stacked_keywords = [
        "; drop table",
        "; drop database",
        "; delete from",
        "; truncate table",
        "; truncate ",
        "; insert into",
        "; update ",
    ];
    for &keyword in &stacked_keywords {
        if lower.contains(keyword) {
            return Some("sql_stacked_query");
        }
    }

    // 4. Time-based blind SQLi functions
    if lower.contains("sleep(")
        || lower.contains("waitfor delay")
        || lower.contains("benchmark(")
        || lower.contains("pg_sleep(")
    {
        return Some("sql_time_based_injection");
    }

    // 5. Classic tautologies (OR 1=1, OR '1'='1', OR ""="", OR true)
    let compact_no_spaces: String = lower.chars().filter(|c| !c.is_whitespace()).collect();
    if compact_no_spaces.contains("or\"\"=\"\"")
        || compact_no_spaces.contains("or''=''")
        || compact_no_spaces.contains("or1=1")
        || compact_no_spaces.contains("or'1'='1'")
        || compact_no_spaces.contains("or\"1\"=\"1\"")
        || compact_no_spaces.contains("or'a'='a'")
        || compact_no_spaces.contains("ortrue")
    {
        return Some("sql_tautology");
    }

    let compact_stripped_quotes: String = lower
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\'' && *c != '"')
        .collect();
    if compact_stripped_quotes.contains("or1=1")
        || compact_stripped_quotes.contains("ora=a")
        || compact_stripped_quotes.contains("orx=x")
        || compact_stripped_quotes.contains("or=")
        || compact_stripped_quotes.contains("ortrue")
    {
        return Some("sql_tautology");
    }

    None
}

/// Runs automatic parsing sanitization checks across raw string values.
pub fn inspect_string_security(val: &str) -> Option<ValidationError> {
    if let Some((_ch, idx)) = find_hidden_character(val) {
        return Some(ValidationError::new(
            "hidden_character",
            format!("String contains invisible or non-printable character at index {}", idx),
        ));
    }

    if let Some(pattern) = find_xss_pattern(val) {
        return Some(ValidationError::new(
            "xss_detected",
            format!("Potential Cross-Site Scripting (XSS) payload detected ({})", pattern),
        ));
    }

    if let Some(pattern) = find_sql_injection_pattern(val) {
        return Some(ValidationError::new(
            "sql_injection_detected",
            format!("Potential SQL injection signature detected ({})", pattern),
        ));
    }

    None
}
