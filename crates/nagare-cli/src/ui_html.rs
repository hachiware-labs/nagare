pub(crate) fn list_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(|value| h(value))
        .collect::<Vec<_>>()
        .join("<br>")
}

pub(crate) fn h(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn is_empty_display_value(value: &str) -> bool {
    let normalized = value
        .trim()
        .trim_matches(|ch: char| ch == '-' || ch == ' ' || ch == '　')
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "" | "none" | "no" | "n/a" | "na" | "nil" | "null" | "なし" | "無し" | "ありません"
    )
}
