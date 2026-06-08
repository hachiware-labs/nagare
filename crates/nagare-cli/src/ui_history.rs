use crate::ui_agent::agent_label;
use crate::ui_html::h;

pub(crate) fn render_run_history_panel(
    snapshot: &nagare_core::WorkItemSnapshot,
    running: Option<&str>,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    let mut events = snapshot
        .history_steps
        .iter()
        .enumerate()
        .map(|(index, step)| render_history_step(step, index + 1, profiles))
        .collect::<Vec<_>>();

    if let Some(running) = running {
        events.push(format!(
            r#"<article class="history-event running">
  <div class="event-head"><span class="badge blue">running</span><b>{}</b></div>
  <p>Processing is in progress. This page will refresh while the workflow can continue automatically.</p>
</article>"#,
            h(running)
        ));
    }

    if events.is_empty() {
        return r#"<section class="panel workflow-panel"><div class="panel-head"><div><h2>詳細ログ</h2><p class="muted">Dispatcherや各エージェントの実行記録です。</p></div><span class="badge gray">empty</span></div><p class="muted">No workflow event has been recorded yet.</p></section>"#.to_string();
    }

    format!(
        r#"<section class="panel workflow-panel">
  <div class="panel-head">
    <div>
      <h2>詳細ログ</h2>
      <p class="muted">Dispatcherや各エージェントの実行記録です。全体の流れは上の進行フローを見てください。</p>
    </div>
    <span class="badge gray">{} events</span>
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
) -> String {
    let facts = render_step_facts(step, profiles);
    let links = render_step_links(step);
    let actor = step
        .actor
        .as_deref()
        .map(|actor| agent_label(profiles, actor))
        .unwrap_or_else(|| "-".to_string());
    let source_ids = if step.source_record_ids.is_empty() {
        "-".to_string()
    } else {
        step.source_record_ids
            .iter()
            .map(|id| h(id))
            .collect::<Vec<_>>()
            .join("<br>")
    };
    let timing = match (step.started_at.as_deref(), step.ended_at.as_deref()) {
        (Some(started), Some(ended)) if started != ended => format!("{started} -> {ended}"),
        (_, Some(ended)) => ended.to_string(),
        (Some(started), None) => started.to_string(),
        _ => "-".to_string(),
    };
    let summary = history_step_summary(step, profiles);

    format!(
        r#"<article class="history-event" data-event-type="{}">
  <div class="event-head">
    <span class="history-step">Step {}</span>
    <span class="badge {}">{}</span>
    <div class="history-title"><b>{}</b><span class="muted">{}</span></div>
    <span class="muted history-time">{}</span>
  </div>
  <p class="event-summary">{}</p>
  {}
  <details class="history-details" data-history-key="{}"><summary>Details</summary><div class="detail-section"><h3>Source records</h3><dl><dt>Kind</dt><dd>{}</dd><dt>Actor</dt><dd>{}</dd><dt>Timing</dt><dd>{}</dd><dt>Next</dt><dd>{}</dd><dt>Sources</dt><dd>{}</dd></dl></div>{}</details>
</article>"#,
        h(&step.kind),
        sequence,
        history_step_status_class(step),
        h(&history_step_status_label(&step.state)),
        h(&step.title),
        h(&actor),
        h(&timing),
        h(&summary),
        facts,
        h(&step.id),
        h(&step.kind),
        h(&actor),
        h(&timing),
        h(step.next_action.as_deref().unwrap_or("-")),
        source_ids,
        links
    )
}

fn history_step_summary(
    step: &nagare_core::WorkItemHistoryStep,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    if step.kind == "dispatch" {
        if let Some(target) = step.facts.iter().find(|fact| fact.label == "Target") {
            let suffix = step
                .summary
                .split_once('。')
                .map(|(_, suffix)| suffix.trim())
                .filter(|suffix| !suffix.is_empty())
                .map(|suffix| format!("。{suffix}"))
                .unwrap_or_default();
            return format!(
                "{} を作業エージェントに選定{}",
                agent_label(profiles, &target.value),
                suffix
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
        .take(12)
        .map(|fact| {
            format!(
                r#"<div><span>{}</span><b>{}</b></div>"#,
                h(&fact.label),
                h(&history_fact_value(fact, profiles))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="history-facts">{rows}</div>"#)
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

fn render_step_links(step: &nagare_core::WorkItemHistoryStep) -> String {
    if step.links.is_empty() {
        return String::new();
    }

    let rows = step
        .links
        .iter()
        .map(|link| {
            format!(
                "<dt>{}</dt><dd>{} ({})</dd>",
                h(&link.label),
                h(&link.record_id),
                h(&link.record_type)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="detail-section"><h3>Links</h3><dl>{rows}</dl></div>"#)
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
