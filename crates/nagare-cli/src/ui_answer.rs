use nagare_core::{AgentRunPurpose, WorkItemStatus};

use crate::ui_html::h;

pub(crate) struct AnswerView {
    label: &'static str,
    class_name: &'static str,
    body: String,
}

pub(crate) fn answer_view(
    snapshot: &nagare_core::WorkItemSnapshot,
    _profiles: &[nagare_core::AgentProfile],
) -> AnswerView {
    let latest_work_output = snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.purpose == AgentRunPurpose::Work);
    let Some(output) = latest_work_output else {
        return AnswerView {
            label: "回答なし",
            class_name: "gray",
            body: "作業エージェントの回答はまだ記録されていません。".to_string(),
        };
    };
    let latest_review = snapshot.review_results.iter().rev().next();
    let contract_invalid = output.parse_status == nagare_core::AgentOutputParseStatus::Unparsed;
    let contract_warnings = !output.warnings.is_empty();
    let review_invalid =
        latest_review.is_some_and(|review| review.verdict == nagare_core::ReviewVerdict::Unknown);
    let (label, class_name) =
        if snapshot.item.status == WorkItemStatus::Done || snapshot.approval_gate.ready {
            ("最終結果", "green")
        } else if contract_invalid || review_invalid {
            ("確認が必要", "red")
        } else if contract_warnings {
            ("下書き", "amber")
        } else {
            ("現在の結果", "blue")
        };
    let body = answer_body_from_output(output);
    AnswerView {
        label,
        class_name,
        body,
    }
}

fn answer_body_from_output(output: &nagare_core::AgentOutputRecord) -> String {
    let values = output
        .fields
        .get("summary")
        .filter(|values| !values.is_empty())
        .or_else(|| output.fields.get("completed"))
        .cloned()
        .unwrap_or_default();
    let cleaned = values
        .iter()
        .flat_map(|value| value.lines())
        .filter_map(clean_answer_line)
        .collect::<Vec<_>>();
    if cleaned.is_empty() {
        return "最新の作業出力から回答本文を抽出できませんでした。".to_string();
    }
    cleaned.join("\n")
}

fn clean_answer_line(line: &str) -> Option<String> {
    let trimmed = line.trim().trim_start_matches("- ").trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    for skip in [
        "status:",
        "completed:",
        "artifacts:",
        "evidence:",
        "questions:",
        "next_notes:",
        "next_action:",
    ] {
        if lower.starts_with(skip) {
            return None;
        }
    }
    if lower.starts_with("summary:") {
        return Some(trimmed["summary:".len()..].trim().to_string());
    }
    Some(trimmed.to_string())
}

pub(crate) fn render_answer_preview(answer: Option<&AnswerView>) -> String {
    let Some(answer) = answer else {
        return "<span class=\"muted\">回答なし</span>".to_string();
    };
    format!(
        r#"<div class="answer-preview"><span class="badge {}">{}</span><div>{}</div></div>"#,
        answer.class_name,
        h(answer.label),
        h(&truncate_text(&answer.body, 140))
    )
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= max_chars {
        return compact;
    }
    let mut truncated = compact.chars().take(max_chars).collect::<String>();
    truncated.push('…');
    truncated
}
