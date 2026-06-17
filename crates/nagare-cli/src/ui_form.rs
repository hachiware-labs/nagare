use std::collections::HashMap;

pub(crate) fn parse_form_urlencoded(body: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    for (key, value) in body.split('&').filter_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        Some((url_decode(key)?, url_decode(value)?))
    }) {
        fields
            .entry(key)
            .and_modify(|existing: &mut String| {
                if !existing.is_empty() {
                    existing.push('\n');
                }
                existing.push_str(&value);
            })
            .or_insert(value);
    }
    fields
}

pub(crate) fn url_decode(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut chars = value.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let high = chars.next()?;
                let low = chars.next()?;
                bytes.push((hex(high)? << 4) | hex(low)?);
            }
            other => bytes.push(other),
        }
    }
    String::from_utf8(bytes).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn split_lines(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .lines()
        .flat_map(|line| line.split(','))
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub(crate) fn split_list(value: Option<&str>) -> Vec<String> {
    split_lines(value)
}

pub(crate) fn derive_work_item_title(title: Option<&str>, description: &str) -> String {
    let title = title.unwrap_or("").trim();
    if !title.is_empty() {
        return title.to_string();
    }
    let summary = description
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    let summary = summary.split_whitespace().collect::<Vec<_>>().join(" ");
    if summary.chars().count() <= 40 {
        return summary;
    }
    let mut compact = summary.chars().take(40).collect::<String>();
    compact.push_str("...");
    compact
}

pub(crate) fn agent_description_from_fields(fields: &HashMap<String, String>) -> String {
    fields
        .get("description")
        .map(String::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

pub(crate) fn json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
