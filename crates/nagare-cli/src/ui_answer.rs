use nagare_core::{AgentRunPurpose, WorkItemStatus};

use crate::ui_agent::{agent_label, agent_meta};
use crate::ui_html::{h, list_or_dash};

pub(crate) struct AnswerView {
    label: &'static str,
    class_name: &'static str,
    body: String,
    validation: Vec<String>,
    trace: Vec<String>,
    needs_attention: bool,
}

pub(crate) fn answer_view(
    snapshot: &nagare_core::WorkItemSnapshot,
    profiles: &[nagare_core::AgentProfile],
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
            validation: vec!["出力契約: 未記録".to_string()],
            trace: Vec::new(),
            needs_attention: false,
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
    let mut validation = Vec::new();
    validation.push(if contract_invalid {
        "出力契約: 不正".to_string()
    } else if contract_warnings {
        format!("出力契約: 警告あり ({})", output.warnings.join(", "))
    } else {
        "出力契約: 解析済み".to_string()
    });
    validation.push(match latest_review {
        Some(review) => format!("レビュー: {}", review.verdict),
        None => "レビュー: 未実施".to_string(),
    });
    validation.push(if snapshot.item.status == WorkItemStatus::Done {
        "承認: 承認済み".to_string()
    } else if snapshot.approval_gate.ready {
        "承認: 承認待ち".to_string()
    } else {
        "承認: 未到達".to_string()
    });
    let mut trace = vec![
        format!(
            "作業エージェント: {}",
            agent_label(profiles, &output.agent_profile_id)
        ),
        format!(
            "ツール/モデル: {}",
            agent_meta(profiles, &output.agent_profile_id)
        ),
        format!("実行: {}", output.agent_run_id),
        format!("実行記録: {}", output.execution_record_id),
    ];
    if let Some(review) = latest_review {
        trace.push(format!(
            "最新レビュー: {} ({})",
            review.agent_run_id, review.verdict
        ));
    }
    AnswerView {
        label,
        class_name,
        body,
        validation,
        trace,
        needs_attention: contract_invalid || contract_warnings || review_invalid,
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

pub(crate) fn render_answer_panel(answer: &AnswerView) -> String {
    let attention = if answer.needs_attention {
        format!(
            r#"<div class="detail-section answer-attention"><h3>確認が必要</h3><p>{}</p></div>"#,
            list_or_dash(&answer.validation)
        )
    } else {
        String::new()
    };
    format!(
        r#"<section class="panel answer-panel">
  <div class="panel-head"><h2>作業結果</h2><span class="badge {}">{}</span></div>
  <p class="answer-body">{}</p>
  {}
  <details class="detail-disclosure answer-details">
    <summary><span>検証と実行情報</span><small>通常は確認不要です</small></summary>
    <div class="detail-section"><h3>検証</h3><p>{}</p></div>
    <div class="detail-section"><h3>実行情報</h3><p>{}</p></div>
  </details>
</section>"#,
        answer.class_name,
        h(answer.label),
        h(&answer.body),
        attention,
        list_or_dash(&answer.validation),
        list_or_dash(&answer.trace)
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
