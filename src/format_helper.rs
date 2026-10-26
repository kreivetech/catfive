/// Replaces `{key}` placeholders in a template string with given value replacements.
pub(crate) fn format_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, val) in replacements {
        let pattern = format!("{{{}}}", key);
        result = result.replace(&pattern, val);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_template() {
        let tpl = "Character '{char}' at index {index} is forbidden (max: {max})";
        let rendered = format_template(tpl, &[("char", ";"), ("index", "3"), ("max", "10")]);
        assert_eq!(rendered, "Character ';' at index 3 is forbidden (max: 10)");
    }
}
