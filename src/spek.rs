use crate::error::ValidationError;
use crate::format_helper::format_template;
use std::collections::HashSet;

/// Parsed rule from a `.spek("[...]")` specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpekKind {
    /// Brazilian CPF (validates format, repeated digits rejection, and modulo 11 checksum).
    Cpf,
    /// Brazilian CNPJ (validates both classic numeric and the new 2026 alphanumeric format).
    Cnpj,
    /// Luhn algorithm (ISO/IEC 7812, credit cards, IMEI).
    Luhn,
    /// Hexadecimal characters `[0-9a-fA-F]`.
    Hex,
    /// Standard Base64 `[A-Za-z0-9+/=]`.
    Base64,
    /// URL-safe Base64 `[A-Za-z0-9-_]`.
    Base64Url,
    /// Bitcoin/IPFS Base58.
    Base58,
    /// All 7-bit ASCII characters (0..=127).
    AsciiAll,
    /// Visible printable ASCII characters (32..=126).
    AsciiPrintable,
    /// Custom ASCII byte range (inclusive).
    AsciiRange(u8, u8),
    /// Single specific ASCII byte code.
    AsciiCode(u8),
    /// Canonical UUID format (8-4-4-4-12 hex).
    Uuid,
    /// ISO Calendar Date `YYYY-MM-DD`.
    Date,
    /// Network MAC address (`XX:XX:XX:XX:XX:XX` or `XX-XX-XX-XX-XX-XX`).
    Mac,
    /// Literal closed set of allowed characters (e.g. `[ABC234]`).
    LiteralCharset(HashSet<char>),
}

/// Parses a specifier string like `"[CPF]"`, `"[ASCII 65..90]"`, or `"[ABC234]"`.
pub fn parse_spek(raw_spec: &str) -> Result<SpekKind, String> {
    let trimmed = raw_spec.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(format!(
            "Spek pattern must be enclosed in brackets '[...]', got '{}'",
            raw_spec
        ));
    }

    let inner = &trimmed[1..trimmed.len() - 1].trim();
    let upper = inner.to_ascii_uppercase();

    match upper.as_str() {
        "CPF" => Ok(SpekKind::Cpf),
        "CNPJ" => Ok(SpekKind::Cnpj),
        "LUHN" => Ok(SpekKind::Luhn),
        "HEX" => Ok(SpekKind::Hex),
        "BASE64" => Ok(SpekKind::Base64),
        "BASE64URL" => Ok(SpekKind::Base64Url),
        "BASE58" => Ok(SpekKind::Base58),
        "ASCII" => Ok(SpekKind::AsciiAll),
        "ASCII PRINTABLE" => Ok(SpekKind::AsciiPrintable),
        "UUID" => Ok(SpekKind::Uuid),
        "DATE" => Ok(SpekKind::Date),
        "MAC" => Ok(SpekKind::Mac),
        _ => {
            // Check for [ASCII ...] variants
            if let Some(stripped) = upper.strip_prefix("ASCII ") {
                let rest = stripped.trim();
                if let Some((start_s, end_s)) = rest.split_once("..=") {
                    let start: u8 = start_s
                        .trim()
                        .parse()
                        .map_err(|_| format!("Invalid ASCII range start in '{}'", raw_spec))?;
                    let end: u8 = end_s
                        .trim()
                        .parse()
                        .map_err(|_| format!("Invalid ASCII range end in '{}'", raw_spec))?;
                    return Ok(SpekKind::AsciiRange(start, end));
                } else if let Some((start_s, end_s)) = rest.split_once("..") {
                    let start: u8 = start_s
                        .trim()
                        .parse()
                        .map_err(|_| format!("Invalid ASCII range start in '{}'", raw_spec))?;
                    let end: u8 = end_s
                        .trim()
                        .parse()
                        .map_err(|_| format!("Invalid ASCII range end in '{}'", raw_spec))?;
                    return Ok(SpekKind::AsciiRange(start, end));
                } else if let Ok(code) = rest.parse::<u8>() {
                    return Ok(SpekKind::AsciiCode(code));
                }
            }

            // Otherwise, treat as a literal closed charset (e.g. `[ABC234]`)
            if !inner.is_empty() {
                let chars: HashSet<char> = inner.chars().collect();
                Ok(SpekKind::LiteralCharset(chars))
            } else {
                Err(format!("Empty spek specification '{}'", raw_spec))
            }
        }
    }
}

/// Validates a string against a parsed `SpekKind`.
pub fn validate_spek_kind(
    kind: &SpekKind,
    val: &str,
    custom_msg: Option<&str>,
) -> Option<ValidationError> {
    match kind {
        SpekKind::Cpf => validate_cpf(val, custom_msg),
        SpekKind::Cnpj => validate_cnpj(val, custom_msg),
        SpekKind::Luhn => validate_luhn(val, custom_msg),
        SpekKind::Hex => validate_hex(val, custom_msg),
        SpekKind::Base64 => validate_base64(val, custom_msg),
        SpekKind::Base64Url => validate_base64url(val, custom_msg),
        SpekKind::Base58 => validate_base58(val, custom_msg),
        SpekKind::AsciiAll => validate_ascii_all(val, custom_msg),
        SpekKind::AsciiPrintable => validate_ascii_printable(val, custom_msg),
        SpekKind::AsciiRange(min, max) => validate_ascii_range(val, *min, *max, custom_msg),
        SpekKind::AsciiCode(code) => validate_ascii_code(val, *code, custom_msg),
        SpekKind::Uuid => validate_uuid(val, custom_msg),
        SpekKind::Date => validate_date(val, custom_msg),
        SpekKind::Mac => validate_mac(val, custom_msg),
        SpekKind::LiteralCharset(set) => validate_literal_charset(val, set, custom_msg),
    }
}

// ============================================================================
// CPF Validator
// ============================================================================
fn validate_cpf(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    // Accepts clean digits (11) or formatted XXX.XXX.XXX-XX (14)
    let cleaned: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.len() != 11 {
        return Some(make_err(
            "invalid_cpf",
            "Must be a valid 11-digit CPF",
            custom_msg,
            &[("value", val)],
        ));
    }

    // If formatted, verify separator structure
    if val.len() == 14 {
        let chars: Vec<char> = val.chars().collect();
        if chars[3] != '.' || chars[7] != '.' || chars[11] != '-' {
            return Some(make_err(
                "invalid_cpf",
                "CPF format must be XXX.XXX.XXX-XX",
                custom_msg,
                &[("value", val)],
            ));
        }
    } else if val.len() != 11 {
        return Some(make_err(
            "invalid_cpf",
            "Must be a valid 11-digit CPF",
            custom_msg,
            &[("value", val)],
        ));
    }

    // Check for all-identical digits (e.g. 00000000000, 11111111111)
    let first = cleaned.chars().next().unwrap();
    if cleaned.chars().all(|c| c == first) {
        return Some(make_err(
            "invalid_cpf",
            "CPF with all identical digits is invalid",
            custom_msg,
            &[("value", val)],
        ));
    }

    let digits: Vec<u32> = cleaned.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // DV 1
    let sum1: u32 = digits[0..9]
        .iter()
        .enumerate()
        .map(|(idx, &d)| d * (10 - idx as u32))
        .sum();
    let mod1 = sum1 % 11;
    let dv1 = if mod1 < 2 { 0 } else { 11 - mod1 };
    if digits[9] != dv1 {
        return Some(make_err(
            "invalid_cpf",
            "CPF verification digits do not match",
            custom_msg,
            &[("value", val)],
        ));
    }

    // DV 2
    let sum2: u32 = digits[0..10]
        .iter()
        .enumerate()
        .map(|(idx, &d)| d * (11 - idx as u32))
        .sum();
    let mod2 = sum2 % 11;
    let dv2 = if mod2 < 2 { 0 } else { 11 - mod2 };
    if digits[10] != dv2 {
        return Some(make_err(
            "invalid_cpf",
            "CPF verification digits do not match",
            custom_msg,
            &[("value", val)],
        ));
    }

    None
}

// ============================================================================
// CNPJ Validator (Classic + New 2026 Alphanumeric Standard IN RFB 2229/2024)
// ============================================================================
fn validate_cnpj(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    // Formatted: XX.XXX.XXX/XXXX-DD (18 chars) or clean (14 chars)
    let is_formatted = val.len() == 18;
    if is_formatted {
        let b = val.as_bytes();
        if b[2] != b'.' || b[6] != b'.' || b[10] != b'/' || b[15] != b'-' {
            return Some(make_err(
                "invalid_cnpj",
                "CNPJ format must be XX.XXX.XXX/XXXX-DD",
                custom_msg,
                &[("value", val)],
            ));
        }
    } else if val.len() != 14 {
        return Some(make_err(
            "invalid_cnpj",
            "Must be a valid 14-character CNPJ",
            custom_msg,
            &[("value", val)],
        ));
    }

    // Strip formatting
    let clean_str: String = val
        .chars()
        .filter(|&c| c != '.' && c != '/' && c != '-')
        .collect();

    if clean_str.len() != 14 {
        return Some(make_err(
            "invalid_cnpj",
            "Must be a valid 14-character CNPJ",
            custom_msg,
            &[("value", val)],
        ));
    }

    let upper_chars: Vec<char> = clean_str.to_ascii_uppercase().chars().collect();

    // Check for all-identical characters
    let first = upper_chars[0];
    if upper_chars.iter().all(|&c| c == first) {
        return Some(make_err(
            "invalid_cnpj",
            "CNPJ with all identical characters is invalid",
            custom_msg,
            &[("value", val)],
        ));
    }

    // First 12 positions: [A-Z0-9]
    for &ch in &upper_chars[0..12] {
        if !ch.is_ascii_alphanumeric() {
            return Some(make_err(
                "invalid_cnpj",
                "CNPJ root and order must be alphanumeric",
                custom_msg,
                &[("value", val)],
            ));
        }
    }

    // Last 2 positions (DVs): MUST be digits [0-9]
    if !upper_chars[12].is_ascii_digit() || !upper_chars[13].is_ascii_digit() {
        return Some(make_err(
            "invalid_cnpj",
            "CNPJ check digits must be numeric",
            custom_msg,
            &[("value", val)],
        ));
    }

    // Receita Federal Alphanumeric Modulo 11 Algorithm:
    // Character value = ASCII code - 48 (applies to both '0'..'9' and 'A'..'Z')
    let char_val = |c: char| -> u32 { (c as u32) - 48 };

    // Weights for DV1: [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
    let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = upper_chars[0..12]
        .iter()
        .zip(weights1.iter())
        .map(|(&c, &w)| char_val(c) * w)
        .sum();
    let mod1 = sum1 % 11;
    let dv1 = if mod1 < 2 { 0 } else { 11 - mod1 };
    if upper_chars[12].to_digit(10).unwrap() != dv1 {
        return Some(make_err(
            "invalid_cnpj",
            "CNPJ verification digits do not match",
            custom_msg,
            &[("value", val)],
        ));
    }

    // Weights for DV2: [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
    let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = upper_chars[0..13]
        .iter()
        .zip(weights2.iter())
        .map(|(&c, &w)| char_val(c) * w)
        .sum();
    let mod2 = sum2 % 11;
    let dv2 = if mod2 < 2 { 0 } else { 11 - mod2 };
    if upper_chars[13].to_digit(10).unwrap() != dv2 {
        return Some(make_err(
            "invalid_cnpj",
            "CNPJ verification digits do not match",
            custom_msg,
            &[("value", val)],
        ));
    }

    None
}

// ============================================================================
// Luhn Algorithm (ISO/IEC 7812)
// ============================================================================
fn validate_luhn(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    let digits: Vec<u32> = val
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.is_empty() {
        return Some(make_err(
            "invalid_luhn",
            "Value cannot be empty",
            custom_msg,
            &[("value", val)],
        ));
    }

    let mut sum = 0;
    let mut double = false;
    for &d in digits.iter().rev() {
        if double {
            let doubled = d * 2;
            sum += if doubled > 9 { doubled - 9 } else { doubled };
        } else {
            sum += d;
        }
        double = !double;
    }

    if sum % 10 != 0 {
        Some(make_err(
            "invalid_luhn",
            "Value does not satisfy Luhn check digit",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

// ============================================================================
// Encodings & Charsets
// ============================================================================
fn validate_hex(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    if val.is_empty() || !val.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(make_err(
            "invalid_hex",
            "Value must contain only hexadecimal characters",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_base64(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    let is_valid = !val.is_empty()
        && val
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');
    if !is_valid {
        Some(make_err(
            "invalid_base64",
            "Value must be valid Base64",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_base64url(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    let is_valid = !val.is_empty()
        && val
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !is_valid {
        Some(make_err(
            "invalid_base64url",
            "Value must be valid Base64URL",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_base58(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    const B58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let is_valid = !val.is_empty() && val.bytes().all(|b| B58.contains(&b));
    if !is_valid {
        Some(make_err(
            "invalid_base58",
            "Value must be valid Base58",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_ascii_all(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    for ch in val.chars() {
        if !ch.is_ascii() {
            return Some(make_err(
                "invalid_ascii",
                "Character is not standard ASCII",
                custom_msg,
                &[("char", &ch.to_string()), ("value", val)],
            ));
        }
    }
    None
}

fn validate_ascii_printable(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    for ch in val.chars() {
        let b = ch as u32;
        if !(32..=126).contains(&b) {
            return Some(make_err(
                "invalid_ascii",
                "Character is not printable ASCII (32..=126)",
                custom_msg,
                &[("char", &ch.to_string()), ("value", val)],
            ));
        }
    }
    None
}

fn validate_ascii_range(
    val: &str,
    min: u8,
    max: u8,
    custom_msg: Option<&str>,
) -> Option<ValidationError> {
    for (idx, ch) in val.chars().enumerate() {
        let code = ch as u32;
        if code < min as u32 || code > max as u32 {
            let msg = make_err(
                "invalid_ascii_range",
                format!(
                    "Character '{}' with ASCII {} is outside range {}..={}",
                    ch, code, min, max
                ),
                custom_msg,
                &[
                    ("char", &ch.to_string()),
                    ("code", &code.to_string()),
                    ("index", &idx.to_string()),
                    ("min", &min.to_string()),
                    ("max", &max.to_string()),
                    ("value", val),
                ],
            );
            return Some(msg);
        }
    }
    None
}

fn validate_ascii_code(
    val: &str,
    target_code: u8,
    custom_msg: Option<&str>,
) -> Option<ValidationError> {
    for ch in val.chars() {
        if (ch as u32) != (target_code as u32) {
            return Some(make_err(
                "invalid_ascii_code",
                format!("Must only contain character with ASCII {}", target_code),
                custom_msg,
                &[
                    ("char", &ch.to_string()),
                    ("code", &target_code.to_string()),
                    ("value", val),
                ],
            ));
        }
    }
    None
}

fn validate_uuid(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    let is_valid = if val.len() == 36 {
        val.chars().enumerate().all(|(idx, ch)| {
            if idx == 8 || idx == 13 || idx == 18 || idx == 23 {
                ch == '-'
            } else {
                ch.is_ascii_hexdigit()
            }
        })
    } else {
        false
    };

    if !is_valid {
        Some(make_err(
            "invalid_uuid",
            "Must be a valid UUID",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_date(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    // YYYY-MM-DD
    if val.len() != 10 {
        return Some(make_err(
            "invalid_date",
            "Date must follow YYYY-MM-DD format",
            custom_msg,
            &[("value", val)],
        ));
    }
    let parts: Vec<&str> = val.split('-').collect();
    if parts.len() != 3 {
        return Some(make_err(
            "invalid_date",
            "Date must follow YYYY-MM-DD format",
            custom_msg,
            &[("value", val)],
        ));
    }

    let year = parts[0].parse::<u32>();
    let month = parts[1].parse::<u32>();
    let day = parts[2].parse::<u32>();

    match (year, month, day) {
        (Ok(y), Ok(m), Ok(d))
            if (1000..=9999).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) =>
        {
            None
        }
        _ => Some(make_err(
            "invalid_date",
            "Invalid date calendar values",
            custom_msg,
            &[("value", val)],
        )),
    }
}

fn validate_mac(val: &str, custom_msg: Option<&str>) -> Option<ValidationError> {
    let is_valid = if val.len() == 17 {
        let sep = val.chars().nth(2).unwrap();
        (sep == ':' || sep == '-')
            && val.split(sep).count() == 6
            && val
                .split(sep)
                .all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit()))
    } else {
        false
    };

    if !is_valid {
        Some(make_err(
            "invalid_mac",
            "Must be a valid MAC address",
            custom_msg,
            &[("value", val)],
        ))
    } else {
        None
    }
}

fn validate_literal_charset(
    val: &str,
    allowed: &HashSet<char>,
    custom_msg: Option<&str>,
) -> Option<ValidationError> {
    for (idx, ch) in val.chars().enumerate() {
        if !allowed.contains(&ch) {
            return Some(make_err(
                "invalid_spek_charset",
                format!("Character '{}' is not within the specified pattern", ch),
                custom_msg,
                &[
                    ("char", &ch.to_string()),
                    ("index", &idx.to_string()),
                    ("value", val),
                ],
            ));
        }
    }
    None
}

fn make_err(
    code: &'static str,
    default_msg: impl Into<String>,
    custom_msg: Option<&str>,
    vars: &[(&str, &str)],
) -> ValidationError {
    let msg = match custom_msg {
        Some(tpl) => format_template(tpl, vars),
        None => default_msg.into(),
    };
    ValidationError::new(code, msg)
}
