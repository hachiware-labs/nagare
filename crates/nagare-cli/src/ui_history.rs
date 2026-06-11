use crate::ui_agent::agent_label;
use crate::ui_html::{h, is_empty_display_value};

pub(crate) fn render_run_history_panel(
    snapshot: &nagare_core::WorkItemSnapshot,
    running: Option<&str>,
    profiles: &[nagare_core::AgentProfile],
    dispatch_step_details: Option<&str>,
) -> String {
    let mut events = snapshot
        .history_steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let step_details = if step.kind == "dispatch" {
                dispatch_step_details
            } else {
                None
            };
            render_history_step(step, index + 1, profiles, step_details)
        })
        .collect::<Vec<_>>();

    if let Some(running) = running {
        events.push(format!(
            r#"<article class="history-event running">
  <div class="event-head"><span class="badge blue">処理中</span><b>{}</b></div>
  <p>処理中です。ワークフローを続行できる間、このページは自動更新されます。</p>
</article>"#,
            h(running)
        ));
    }

    if events.is_empty() {
        return r#"<section class="panel workflow-panel step-detail-panel"><div class="panel-head"><div><h2>1ステップずつの内容</h2><p class="muted">各ステップの担当、実行内容、結果をここに表示します。</p></div><span class="badge gray">empty</span></div><p class="muted">まだステップは記録されていません。</p></section>"#.to_string();
    }

    format!(
        r#"<section class="panel workflow-panel step-detail-panel">
  <div class="panel-head">
    <div>
      <h2>1ステップずつの内容</h2>
      <p class="muted">各ステップで、どのエージェントが何をして、どう進んだかを表示しています。</p>
    </div>
    <span class="badge gray">{} steps</span>
  </div>
  <div class="history-list">{}</div>
</section>"#,
        events.len(),
        events.join("\n")
    )
}

fn render_history_step(
    step: &nagare_core::WorkItemHistoryStep,
    sequence: usize,
    profiles: &[nagare_core::AgentProfile],
    step_details: Option<&str>,
) -> String {
    let facts = render_step_facts(step, profiles);
    let embedded_details = step_details.unwrap_or("");
    let actor = step
        .actor
        .as_deref()
        .map(|actor| agent_label(profiles, actor))
        .unwrap_or_else(|| "-".to_string());
    let timing = match (step.started_at.as_deref(), step.ended_at.as_deref()) {
        (Some(started), Some(ended)) if started != ended => format!("{started} -> {ended}"),
        (_, Some(ended)) => ended.to_string(),
        (Some(started), None) => started.to_string(),
        _ => "-".to_string(),
    };
    let visible_timing = if is_machine_timing(&timing) {
        String::new()
    } else {
        timing.clone()
    };
    let summary = history_step_summary(step, profiles);
    let result_label = history_step_result_label(step);
    let timing_detail = if visible_timing.is_empty() {
        String::new()
    } else {
        format!("<dt>時間</dt><dd>{}</dd>", h(&visible_timing))
    };

    format!(
        r#"<article class="history-event" data-event-type="{}">
  <div class="event-head">
    <span class="history-step">Step {}</span>
    <span class="badge {}">{}</span>
    <div class="history-title"><b>{}</b><span class="muted">{}</span></div>
    <span class="muted history-time">{}</span>
  </div>
  <div class="step-result"><span>{}</span><p>{}</p></div>
  {}
  <details class="history-details" data-history-key="{}"><summary>詳細</summary>{}<div class="detail-section"><h3>ステップ情報</h3><dl><dt>種別</dt><dd>{}</dd><dt>担当</dt><dd>{}</dd>{}<dt>次にやること</dt><dd>{}</dd></dl></div></details>
</article>"#,
        h(&step.kind),
        sequence,
        history_step_status_class(step),
        h(&history_step_status_label(&step.state)),
        h(&step.title),
        h(&actor),
        h(&visible_timing),
        h(result_label),
        h(&summary),
        facts,
        h(&step.id),
        embedded_details,
        h(history_step_kind_label(step)),
        h(&actor),
        timing_detail,
        h(next_action_label(
            step.next_action.as_deref().unwrap_or("-")
        )),
    )
}

fn history_step_result_label(step: &nagare_core::WorkItemHistoryStep) -> &'static str {
    match step.kind.as_str() {
        "dispatch" => "選定理由",
        "work" => "作業結果",
        "synthesis" => "統合結果",
        "review" => "レビュー結果",
        "recovery" => "復旧内容",
        "answer" | "human_feedback" => "入力内容",
        _ => "結果",
    }
}

fn history_step_kind_label(step: &nagare_core::WorkItemHistoryStep) -> &'static str {
    match step.kind.as_str() {
        "request" => "依頼作成",
        "dispatch" => "Agent選定",
        "work" => "作業",
        "synthesis" => "統合サマリー",
        "review" => "レビュー",
        "approval" => "承認",
        "recovery" => "復旧",
        "answer" | "human_feedback" => "ユーザー入力",
        _ => "その他",
    }
}

fn history_step_summary(
    step: &nagare_core::WorkItemHistoryStep,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    if step.kind == "dispatch" {
        if let Some(target) = step.facts.iter().find(|fact| fact.label == "Target") {
            return format!(
                "{} を作業エージェントに選定",
                agent_label(profiles, &target.value)
            );
        }
    }
    step.summary.clone()
}

fn render_step_facts(
    step: &nagare_core::WorkItemHistoryStep,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    if step.facts.is_empty() {
        return String::new();
    }

    let rows = step
        .facts
        .iter()
        .filter(|fact| should_show_fact(fact))
        .take(6)
        .map(|fact| {
            format!(
                r#"<div><span>{}</span><b>{}</b></div>"#,
                h(fact_label(&fact.label)),
                h(&history_fact_value(fact, profiles))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    if rows.is_empty() {
        return String::new();
    }
    format!(r#"<div class="history-facts">{rows}</div>"#)
}

fn should_show_fact(fact: &nagare_core::WorkItemHistoryFact) -> bool {
    let value = fact.value.trim();
    if is_empty_display_value(value) {
        return false;
    }
    if value.contains("run_")
        || value.contains("exec_")
        || value.contains("out_")
        || value.contains("review_")
    {
        return false;
    }
    if fact.label == "Criteria" && value == "0/0" {
        return false;
    }
    !matches!(
        fact.label.as_str(),
        "Workflow Decision"
            | "Reason"
            | "Decision source"
            | "Confidence"
            | "Decision warnings"
            | "Warnings"
            | "Missing info"
            | "Expected artifacts"
            | "Process"
            | "Process status"
            | "Exit"
            | "Process exit"
            | "Evidence"
            | "output record"
    )
}

fn fact_label(label: &str) -> &str {
    match label {
        "Title" => "タイトル",
        "Acceptance" => "完了条件",
        "Target" => "選定先",
        "Dispatch agent" => "Dispatcher",
        "Agent" => "エージェント",
        "Process" | "Process status" => "実行状態",
        "Exit" | "Process exit" => "終了コード",
        "Artifacts" => "成果物",
        "Evidence" => "根拠",
        "Status" => "状態",
        "Verdict" => "判定",
        "Criteria" => "条件",
        "Findings" => "指摘",
        "Requested changes" => "修正依頼",
        "Parse status" => "出力解析",
        "Decision" => "判定",
        "Rationale" => "理由",
        "evidence detail" => "実行メモ",
        "completed" => "完了内容",
        "next_notes" => "次のメモ",
        "questions" => "質問",
        other => other,
    }
}

fn history_fact_value(
    fact: &nagare_core::WorkItemHistoryFact,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    match fact.label.as_str() {
        "Agent" | "Target" | "Dispatch agent" | "From" | "To" => agent_label(profiles, &fact.value),
        _ => fact.value.clone(),
    }
}

fn is_machine_timing(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed == "-" || trimmed.is_empty() {
        return true;
    }
    trimmed.split("->").all(|part| {
        let part = part.trim();
        part.len() >= 9 && part.chars().all(|ch| ch.is_ascii_digit())
    })
}

fn history_step_status_label(state: &str) -> String {
    match state {
        "recorded" => "記録済み",
        "succeeded" | "passed" | "pass" | "accepted" | "answered" | "approve" | "done" => "完了",
        "draft" | "ready" => "準備済み",
        "needs_input" => "入力待ち",
        "needs_recovery" | "contract_invalid" | "failed" | "request_changes" | "blocked"
        | "unparsed" => "要対応",
        "running" | "in_progress" => "処理中",
        _ => state,
    }
    .to_string()
}

fn next_action_label(action: &str) -> &str {
    match action {
        "approve" => "承認",
        "dispatch" => "Agent選定",
        "recover" => "復旧",
        "review" => "レビュー",
        "run_agent" => "作業",
        "synthesize" | "run_synthesis" => "統合サマリー",
        "needs_input" => "入力待ち",
        "stop" | "none" | "-" => "-",
        other => other,
    }
}

fn history_step_status_class(step: &nagare_core::WorkItemHistoryStep) -> &'static str {
    match step.state.as_str() {
        "succeeded" | "passed" | "pass" | "accepted" | "answered" | "approve" | "done" => "green",
        "needs_input" | "draft" | "ready" => "amber",
        "contract_invalid" | "failed" | "request_changes" | "blocked" | "unparsed" => "red",
        "running" | "in_progress" => "blue",
        _ if step.kind == "work" || step.kind == "dispatch" => "blue",
        _ => "gray",
    }
}
