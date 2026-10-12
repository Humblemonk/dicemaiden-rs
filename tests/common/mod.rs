// tests/common/mod.rs - Helpers shared by snapshot_tests.rs and release_diff.rs

/// Put `lead` (such as `"4 + "`) where a number in front of the dice goes:
/// after any flags (`p`, `s`, `nr`, `ul`) and `(label)`, which must come
/// first. Gluing it to the front instead (`4 + (Atk) 1d20`) never parses, so a
/// probe built that way silently skips every flagged or labelled roll.
pub fn with_leading(expression: &str, lead: &str) -> String {
    let mut rest = expression.trim_start();
    while let Some(flag) = ["p ", "s ", "nr ", "ul "]
        .iter()
        .find(|flag| rest.starts_with(**flag))
    {
        rest = rest[flag.len()..].trim_start();
    }
    if rest.starts_with('(')
        && let Some(end) = rest.find(')')
    {
        rest = rest[end + 1..].trim_start();
    }
    let prefix = &expression[..expression.len() - rest.len()];
    format!("{prefix}{lead}{rest}")
}
