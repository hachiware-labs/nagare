use std::fs;
use std::path::PathBuf;

use crate::*;

#[derive(Debug, Clone)]
pub struct StaticUiExportInput {
    pub out_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct StaticUiExportResult {
    pub out_dir: PathBuf,
    pub index_path: PathBuf,
    pub item_paths: Vec<PathBuf>,
}

pub fn export_static_ui(
    root: impl Into<PathBuf>,
    input: StaticUiExportInput,
) -> Result<StaticUiExportResult, NagareError> {
    let root = root.into();
    ensure_project(&root)?;
    let items = list_work_items(&root)?;
    let snapshots = items
        .iter()
        .map(|item| get_work_item_snapshot(&root, &item.id))
        .collect::<Result<Vec<_>, _>>()?;
    let agents = list_agent_profiles(&root)?;
    let defaults = get_nagare_agent_settings(&root)?;
    let locale = get_locale_settings(&root)?;

    let out_dir = input.out_dir;
    let items_dir = out_dir.join("items");
    fs::create_dir_all(&items_dir)?;
    fs::write(out_dir.join("logo.png"), logo_png())?;
    fs::write(out_dir.join("styles.css"), stylesheet())?;

    let mut item_paths = Vec::new();
    for snapshot in &snapshots {
        let path = items_dir.join(format!("{}.html", snapshot.item.id));
        fs::write(
            &path,
            render_item_detail(snapshot, &defaults, locale.language.as_str()),
        )?;
        item_paths.push(path);
    }

    let index_path = out_dir.join("index.html");
    fs::write(
        &index_path,
        render_board(&snapshots, &agents, &defaults, locale.language.as_str()),
    )?;

    Ok(StaticUiExportResult {
        out_dir,
        index_path,
        item_paths,
    })
}

pub fn logo_png() -> &'static [u8] {
    include_bytes!("../../../logo.png")
}

fn render_board(
    snapshots: &[WorkItemSnapshot],
    agents: &[AgentProfile],
    defaults: &NagareAgentSettings,
    locale: &str,
) -> String {
    let rows = if snapshots.is_empty() {
        format!(
            "<tr><td colspan=\"7\" class=\"muted\">{}</td></tr>",
            h(ui_text(locale, "No work items", "Work Item はありません"))
        )
    } else {
        snapshots
            .iter()
            .map(|snapshot| render_board_row(snapshot, locale))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let attention_cards = snapshots
        .iter()
        .filter(|snapshot| needs_attention(snapshot))
        .map(|snapshot| render_attention_card(snapshot, locale))
        .collect::<Vec<_>>()
        .join("\n");
    let attention_body = if attention_cards.is_empty() {
        format!(
            "<p class=\"empty\">{}</p>",
            h(ui_text(
                locale,
                "No work items need confirmation.",
                "確認が必要な Work Item はありません。"
            ))
        )
    } else {
        format!(r#"<div class="attention-list">{attention_cards}</div>"#)
    };
    let attention = snapshots
        .iter()
        .filter(|snapshot| needs_attention(snapshot))
        .count();
    let failed = snapshots
        .iter()
        .filter(|snapshot| matches!(snapshot.item.status, WorkItemStatus::ChangesRequested))
        .count();
    let running = snapshots
        .iter()
        .filter(|snapshot| snapshot.item.status == WorkItemStatus::AgentRunning)
        .count();
    let recovery = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot
                .recovery_plans
                .iter()
                .any(|plan| plan.status != RecoveryPlanStatus::Superseded)
        })
        .count();

    page(
        ui_text(locale, "Nagare Work Queue", "Nagare Work Queue"),
        &format!(
            r##"<main class="app">
  <aside class="sidebar">
    <div class="brand"><img class="brand-logo" src="logo.png" alt=""><span class="brand-text">Nagare</span></div>
    <nav>
      <a class="active" href="index.html">{}</a>
      <a href="#agents">{}</a>
      <a href="#settings">{}</a>
    </nav>
  </aside>
  <section class="content">
    <header class="topbar">
      <div>
        <h1>{}</h1>
        <p class="muted">{} {}</p>
      </div>
      <div class="actions">
        <span class="badge blue">work {}</span>
        <span class="badge gray">review {}</span>
        <span class="badge gray">dispatch {}</span>
        <span class="badge amber">supervisor {}</span>
      </div>
    </header>
    <section class="composer">
      <h2>{}</h2>
      <p>{}</p>
      <code>nagare item create --title &lt;title&gt; --acceptance &lt;csv&gt;</code>
    </section>
    <section class="quick">
      <span class="badge amber">{} {}</span>
      <span class="badge red">{} {}</span>
      <span class="badge blue">{} {}</span>
      <span class="badge green">{} {}</span>
    </section>
    <section class="attention-panel panel">
      <div class="panel-head">
        <h2>{}</h2>
        <span class="badge amber">{} items</span>
      </div>
      {}
    </section>
    <section class="panel">
      <div class="panel-head">
        <h2>{}</h2>
        <span class="muted">{} items / {} agents</span>
      </div>
      <table>
        <thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead>
        <tbody>{}</tbody>
      </table>
    </section>
    <section id="agents" class="panel">
      <h2>{}</h2>
      <div class="grid four">
        <div><b>work</b><span>{}</span></div>
        <div><b>review</b><span>{}</span></div>
        <div><b>dispatch</b><span>{}</span></div>
        <div><b>supervisor</b><span>{}</span></div>
      </div>
    </section>
  </section>
</main>"##,
            h(ui_text(locale, "Work Queue", "作業キュー")),
            h(ui_text(locale, "Agents", "Agent")),
            h(ui_text(locale, "Settings", "設定")),
            h(ui_text(locale, "Work Queue", "Work Queue")),
            h(ui_text(
                locale,
                "Static export / locale",
                "静的 export / locale"
            )),
            h(locale),
            h(&defaults.work_agent),
            h(&defaults.review_agent),
            h(&defaults.dispatch_agent),
            h(&defaults.supervisor_agent),
            h(ui_text(locale, "Request work", "仕事を依頼")),
            h(ui_text(
                locale,
                "Review CLI-created Work Items by status and next decision.",
                "CLIで作成したWork Itemを、次へ進める判断と状態で確認する。"
            )),
            h(ui_text(locale, "Attention", "要確認")),
            attention,
            h(ui_text(locale, "Failed", "失敗")),
            failed,
            h(ui_text(locale, "Recovery", "回復案")),
            recovery,
            h(ui_text(locale, "Running", "実行中")),
            running,
            h(ui_text(locale, "Confirmation queue", "確認キュー")),
            attention,
            attention_body,
            h(ui_text(locale, "Work queue", "作業キュー")),
            snapshots.len(),
            agents.len(),
            h(ui_text(locale, "Work Item", "Work Item")),
            h(ui_text(locale, "Status", "状態")),
            h(ui_text(locale, "Agent / Decision", "Agent / 判断")),
            h(ui_text(locale, "DoD / criteria", "DoD / criteria")),
            h(ui_text(locale, "Review", "Review")),
            h(ui_text(locale, "Recovery", "Recovery")),
            h(ui_text(locale, "Next", "Next")),
            rows,
            h(ui_text(locale, "Agent defaults", "Agent defaults")),
            h(&defaults.work_agent),
            h(&defaults.review_agent),
            h(&defaults.dispatch_agent),
            h(&defaults.supervisor_agent),
        ),
        locale,
    )
}

fn render_board_row(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let latest_decision = snapshot.workflow_decisions.last();
    let target = latest_decision
        .and_then(|decision| decision.target_agent_profile_id.as_deref())
        .or_else(|| latest_run_agent(snapshot))
        .unwrap_or("-");
    let criteria = criteria_summary(snapshot);
    let review = snapshot
        .review_results
        .last()
        .map(|review| review.verdict.to_string())
        .unwrap_or_else(|| "-".to_string());
    let recovery = latest_recovery_label(snapshot);
    let row_class = if needs_attention(snapshot) {
        " class=\"attention-row\""
    } else {
        ""
    };
    format!(
        r#"<tr{}>
  <td><a href="items/{}.html">{}</a><div class="muted">{}</div></td>
  <td><span class="badge {}">{}</span></td>
  <td>{}<div class="muted">{}</div></td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td><code>{}</code></td>
</tr>"#,
        row_class,
        h(&snapshot.item.id),
        h(&snapshot.item.id),
        h(&snapshot.item.title),
        status_kind(snapshot.item.status),
        h(&status_label(locale, snapshot.item.status)),
        h(target),
        latest_decision
            .map(|decision| h(&decision.action.to_string()))
            .unwrap_or_else(|| "-".to_string()),
        h(&criteria),
        h(&review),
        h(&recovery),
        h(&snapshot.completion.next_action),
    )
}

fn render_attention_card(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let reason = attention_reason(snapshot);
    format!(
        r#"<article class="attention-card">
  <div>
    <a href="items/{}.html">{}</a>
    <div class="muted">{}</div>
  </div>
  <span class="badge {}">{}</span>
  <code>{}</code>
</article>"#,
        h(&snapshot.item.id),
        h(&snapshot.item.title),
        h(&reason),
        status_kind(snapshot.item.status),
        h(&status_label(locale, snapshot.item.status)),
        h(&snapshot.completion.next_action),
    )
}

fn render_item_detail(
    snapshot: &WorkItemSnapshot,
    defaults: &NagareAgentSettings,
    locale: &str,
) -> String {
    let history = snapshot
        .history_steps
        .iter()
        .enumerate()
        .map(|(index, step)| render_history_step_event(step, index + 1, locale))
        .collect::<Vec<_>>()
        .join("\n");
    page(
        &format!("{} / {}", snapshot.item.id, snapshot.item.title),
        &format!(
            r##"<main class="app">
  <aside class="sidebar">
    <div class="brand"><img class="brand-logo" src="../logo.png" alt=""><span class="brand-text">Nagare</span></div>
    <nav><a class="active" href="../index.html">{}</a></nav>
  </aside>
  <section class="content">
    <nav class="breadcrumbs" aria-label="Breadcrumb"><a href="../index.html">{}</a><span>/</span><span>{}</span></nav>
    <header class="topbar">
      <div>
        <h1>{}</h1>
        <p class="muted">{} / locale {}</p>
      </div>
      <div class="actions">
        <span class="badge {}">{}</span>
        <span class="badge blue">{}</span>
        <span class="badge gray">{}</span>
      </div>
    </header>
    <div class="detail-layout">
      <section class="summary panel">
        <h2>{}</h2>
        <dl>
          <dt>work_folder</dt><dd>{}</dd>
          <dt>acceptance criteria</dt><dd>{}</dd>
          <dt>expected artifacts</dt><dd>{}</dd>
          <dt>constraints</dt><dd>{}</dd>
          <dt>workflow mode</dt><dd>{}</dd>
          <dt>defaults</dt><dd>work={} review={} supervisor={}</dd>
        </dl>
        <h3>{}</h3>
        <p>{}</p>
        <code>{}</code>
        {}
        {}
      </section>
      <section class="timeline panel">
        <h2>{}</h2>
        {}
      </section>
      <section class="inspector panel">
        {}
        {}
        {}
        {}
        {}
        {}
      </section>
    </div>
  </section>
</main>"##,
            h(ui_text(locale, "Work Queue", "作業キュー")),
            h(ui_text(locale, "Work Queue", "作業キュー")),
            h(ui_text(locale, "Detail", "詳細")),
            h(&snapshot.item.title),
            h(&snapshot.item.id),
            h(locale),
            status_kind(snapshot.item.status),
            h(&status_label(locale, snapshot.item.status)),
            h(ui_text(locale, "Advance", "次へ進める")),
            h(ui_text(locale, "Review decision", "判断を確認")),
            h(ui_text(locale, "Summary", "概要")),
            h(snapshot.item.work_folder.as_deref().unwrap_or("-")),
            h(&list_label(&snapshot.item.acceptance_criteria)),
            h(&list_label(&snapshot.item.expected_artifacts)),
            h(&list_label(&snapshot.item.constraints)),
            h(&snapshot.item.workflow_mode.to_string()),
            h(&defaults.work_agent),
            h(&defaults.review_agent),
            h(&defaults.supervisor_agent),
            h(ui_text(locale, "Required next", "次に必要")),
            h(&snapshot.completion.next_action),
            h(snapshot
                .completion
                .next_command_hint
                .as_deref()
                .unwrap_or("-")),
            render_next_action_panel(snapshot, locale),
            render_human_input_panel(snapshot, locale),
            h(ui_text(locale, "Processing History", "処理履歴")),
            history,
            render_workflow_decision_inspector(snapshot, locale),
            render_approval_gate_inspector(snapshot, locale),
            render_agent_notes_inspector(snapshot, locale),
            render_review_inspector(snapshot, locale),
            render_recovery_inspector(snapshot, locale),
            render_handoff_inspector(snapshot, locale),
        ),
        locale,
    )
}

fn render_human_input_panel(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let (mode, target, command_prefix, placeholder) =
        if snapshot.item.status == WorkItemStatus::NeedsInput {
            (
                "answer",
                latest_open_question(snapshot).unwrap_or_else(|| "needs_input".to_string()),
                format!("nagare item answer {} --answer", snapshot.item.id),
                ui_text(locale, "Enter an answer", "回答を入力"),
            )
        } else if snapshot.completion.next_action == "run_agent" {
            (
                "run_agent",
                snapshot
                    .completion
                    .blocking_reason
                    .clone()
                    .unwrap_or_else(|| "agent prompt".to_string()),
                format!("nagare item run {} --prompt", snapshot.item.id),
                ui_text(
                    locale,
                    "Enter additional instructions for the agent",
                    "Agentへの追加指示を入力",
                ),
            )
        } else {
            (
                "instruction",
                snapshot
                    .completion
                    .blocking_reason
                    .clone()
                    .or_else(|| Some(snapshot.completion.next_action.clone()))
                    .unwrap_or_else(|| "next_action".to_string()),
                format!(
                    "nagare item advance {} --until-blocked true --prompt",
                    snapshot.item.id
                ),
                ui_text(locale, "Enter additional instructions", "追加指示を入力"),
            )
        };
    let command_id = format!("human-command-{}", snapshot.item.id);
    let input_id = format!("human-input-{}", snapshot.item.id);
    format!(
        r#"<section class="human-input command-builder" data-command-prefix="{}" data-command-target="{}">
  <h2>{}</h2>
  <dl><dt>mode</dt><dd>{}</dd><dt>target</dt><dd>{}</dd></dl>
  <textarea id="{}" rows="4" placeholder="{}"></textarea>
  <div class="command-row"><code id="{}">{} "&lt;text&gt;"</code><button type="button" data-copy-target="{}">{}</button></div>
</section>"#,
        h(&command_prefix),
        h(&command_id),
        h("Human Input Panel"),
        h(mode),
        h(&target),
        h(&input_id),
        h(placeholder),
        h(&command_id),
        h(&command_prefix),
        h(&command_id),
        h("Copy"),
    )
}

fn render_next_action_panel(snapshot: &WorkItemSnapshot, _locale: &str) -> String {
    let command = recommended_command(snapshot);
    let recovery = active_recovery(snapshot)
        .map(|plan| format!("{} / {}", plan.failure_class, plan.action))
        .unwrap_or_else(|| "-".to_string());
    let handoff = snapshot
        .handoffs
        .last()
        .map(|handoff| {
            format!(
                "{} -> {}",
                handoff.from_agent_profile, handoff.to_agent_profile
            )
        })
        .unwrap_or_else(|| "-".to_string());
    let notes = snapshot
        .agent_outputs
        .last()
        .map(|output| output_field_label(output, "next_notes"))
        .unwrap_or_else(|| "-".to_string());
    format!(
        r#"<section class="next-action">
  <h2>{}</h2>
  <span class="badge blue">{}</span>
  <span class="badge gray">{}</span>
  <code>{}</code>
  <dl><dt>workflow mode</dt><dd>{}</dd><dt>approval gate</dt><dd>{}</dd><dt>recovery</dt><dd>{}</dd><dt>handoff</dt><dd>{}</dd><dt>next notes</dt><dd>{}</dd></dl>
</section>"#,
        h("Next Action Panel"),
        h(&snapshot.completion.state),
        h(&snapshot.completion.next_action),
        h(&command),
        h(&snapshot.item.workflow_mode.to_string()),
        h(&snapshot.approval_gate.state),
        h(&recovery),
        h(&handoff),
        h(&notes),
    )
}

fn recommended_command(snapshot: &WorkItemSnapshot) -> String {
    if let Some(plan) = active_recovery(snapshot) {
        return match plan.status {
            RecoveryPlanStatus::Draft => {
                format!("nagare item recover accept {}", snapshot.item.id)
            }
            RecoveryPlanStatus::Accepted => {
                format!("nagare item recover apply {}", snapshot.item.id)
            }
            RecoveryPlanStatus::Superseded => fallback_command(snapshot),
        };
    }
    if snapshot.approval_gate.ready {
        return format!("nagare decision approve {}", snapshot.item.id);
    }
    fallback_command(snapshot)
}

fn fallback_command(snapshot: &WorkItemSnapshot) -> String {
    snapshot
        .completion
        .next_command_hint
        .clone()
        .unwrap_or_else(|| {
            format!(
                "nagare item advance {} --until-blocked true",
                snapshot.item.id
            )
        })
}

fn active_recovery(snapshot: &WorkItemSnapshot) -> Option<&RecoveryPlan> {
    snapshot
        .recovery_plans
        .iter()
        .rev()
        .find(|plan| plan.status != RecoveryPlanStatus::Superseded)
}

fn render_history_step_event(step: &WorkItemHistoryStep, sequence: usize, locale: &str) -> String {
    let timing = match (step.started_at.as_deref(), step.ended_at.as_deref()) {
        (Some(started), Some(ended)) if started != ended => format!("{started} -> {ended}"),
        (_, Some(ended)) => ended.to_string(),
        (Some(started), None) => started.to_string(),
        _ => "-".to_string(),
    };
    let facts = if step.facts.is_empty() {
        String::new()
    } else {
        let rows = step
            .facts
            .iter()
            .take(8)
            .map(|fact| format!("<span><b>{}</b> {}</span>", h(&fact.label), h(&fact.value)))
            .collect::<Vec<_>>()
            .join("");
        format!("<div class=\"history-facts\">{rows}</div>")
    };
    let sources = if step.source_record_ids.is_empty() {
        "-".to_string()
    } else {
        step.source_record_ids.join(", ")
    };
    format!(
        r#"<article class="event">
  <div class="node"></div>
  <div>
    <div class="event-head"><span class="badge gray">Step {}</span><b>{}</b><span class="badge {}">{}</span><span>{}</span></div>
    <p>{}</p>
    {}
    <small>{}: {} / {} {} / next {}</small>
  </div>
</article>"#,
        sequence,
        h(&step.title),
        history_step_badge(step),
        h(&history_step_status_label(&step.state, locale)),
        h(&timing),
        h(&step.summary),
        facts,
        h(ui_text(locale, "source", "source")),
        h(&sources),
        h(ui_text(locale, "actor", "actor")),
        h(step.actor.as_deref().unwrap_or("-")),
        h(step.next_action.as_deref().unwrap_or("-")),
    )
}

fn history_step_status_label(state: &str, locale: &str) -> String {
    if !is_ui_ja(locale) {
        return state.to_string();
    }
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

fn history_step_badge(step: &WorkItemHistoryStep) -> &'static str {
    match step.state.as_str() {
        "succeeded" | "passed" | "pass" | "accepted" | "answered" | "approve" | "done" => "green",
        "needs_input" | "draft" | "ready" => "amber",
        "contract_invalid" | "failed" | "request_changes" | "blocked" | "unparsed" => "red",
        "running" | "in_progress" => "blue",
        _ if step.kind == "work" || step.kind == "dispatch" => "blue",
        _ => "gray",
    }
}

fn render_agent_notes_inspector(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let Some(output) = snapshot.agent_outputs.last() else {
        return inspector_empty(
            ui_text(locale, "Agent Output Notes", "Agent Output Notes"),
            ui_text(
                locale,
                "No parsed output notes.",
                "parse 済み output notes はありません。",
            ),
        );
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge blue">{}</span>
  <dl><dt>completed</dt><dd>{}</dd><dt>next_notes</dt><dd>{}</dd><dt>next_action</dt><dd>{}</dd></dl>
</div>"#,
        h(ui_text(locale, "Agent Output Notes", "Agent Output Notes")),
        h(&output.purpose.to_string()),
        h(&output_field_label(output, "completed")),
        h(&output_field_label(output, "next_notes")),
        h(output.next_action.as_deref().unwrap_or("-")),
    )
}

fn render_approval_gate_inspector(snapshot: &WorkItemSnapshot, _locale: &str) -> String {
    let gate = &snapshot.approval_gate;
    let badge = if gate.ready {
        "green"
    } else if gate.state == "blocked" {
        "red"
    } else {
        "gray"
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge {}">{}</span>
  <span class="badge gray">criteria {}/{}</span>
  <dl><dt>review</dt><dd>{}</dd><dt>artifacts</dt><dd>{}</dd><dt>recoveries</dt><dd>{}</dd><dt>blockers</dt><dd>{}</dd><dt>hint</dt><dd>{}</dd></dl>
</div>"#,
        h("Approval Gate"),
        badge,
        h(&gate.state),
        gate.criteria_passed,
        gate.criteria_total,
        h(gate.latest_review_id.as_deref().unwrap_or("-")),
        gate.artifact_count,
        gate.recovery_count,
        h(&list_label(&gate.blockers)),
        h(gate.command_hint.as_deref().unwrap_or("-")),
    )
}

fn render_workflow_decision_inspector(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let Some(decision) = snapshot.workflow_decisions.last() else {
        return inspector_empty(
            ui_text(locale, "Workflow Decision", "Workflow Decision"),
            ui_text(
                locale,
                "No decision recorded yet.",
                "まだ判断は記録されていません。",
            ),
        );
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge blue">{}</span>
  <span class="badge amber">{}</span>
  <p>{}</p>
  <dl><dt>requires_human</dt><dd>{}</dd><dt>target</dt><dd>{}</dd><dt>confidence</dt><dd>{:.2}</dd><dt>hint</dt><dd>{}</dd></dl>
</div>"#,
        h(ui_text(locale, "Workflow Decision", "Workflow Decision")),
        h(&decision.action.to_string()),
        h(&decision.source.to_string()),
        h(&decision.reason),
        decision.requires_human,
        h(decision.target_agent_profile_id.as_deref().unwrap_or("-")),
        decision.confidence,
        h(decision.command_hint.as_deref().unwrap_or("-")),
    )
}

fn render_review_inspector(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let Some(review) = snapshot.review_results.last() else {
        return inspector_empty(
            ui_text(locale, "Review", "Review"),
            ui_text(
                locale,
                "No review recorded yet.",
                "まだ review は記録されていません。",
            ),
        );
    };
    let criteria = if review.criteria_results.is_empty() {
        format!(
            "<li class=\"muted\">{}</li>",
            h(ui_text(
                locale,
                "No criteria results",
                "criteria 結果はありません"
            ))
        )
    } else {
        review
            .criteria_results
            .iter()
            .map(|result| {
                format!(
                    "<li><b>{}</b> <span class=\"badge {}\">{}</span><p>{}</p></li>",
                    h(&result.criterion),
                    criteria_kind(result.status),
                    h(&result.status.to_string()),
                    h(&result.note)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge {}">{}</span>
  <p>{}</p>
  <h3>{}</h3>
  <ul>{}</ul>
</div>"#,
        h(ui_text(locale, "Review", "Review")),
        review_kind(review.verdict),
        h(&review.verdict.to_string()),
        h(&list_label(&review.findings)),
        h(ui_text(locale, "Criteria", "Criteria")),
        criteria,
    )
}

fn render_recovery_inspector(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let Some(plan) = snapshot
        .recovery_plans
        .iter()
        .rev()
        .find(|plan| plan.status != RecoveryPlanStatus::Superseded)
    else {
        return inspector_empty(
            ui_text(locale, "Recovery", "Recovery"),
            ui_text(
                locale,
                "No active recovery plan.",
                "有効な recovery plan はありません。",
            ),
        );
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge amber">{}</span>
  <span class="badge red">{}</span>
  <p>{}</p>
  <dl><dt>action</dt><dd>{}</dd><dt>target</dt><dd>{}</dd><dt>hint</dt><dd>{}</dd></dl>
</div>"#,
        h(ui_text(locale, "Recovery", "Recovery")),
        h(&plan.status.to_string()),
        h(&plan.failure_class),
        h(&plan.summary),
        h(&plan.action.to_string()),
        h(plan.target_agent_profile_id.as_deref().unwrap_or("-")),
        h(plan.command_hint.as_deref().unwrap_or("-")),
    )
}

fn render_handoff_inspector(snapshot: &WorkItemSnapshot, locale: &str) -> String {
    let Some(handoff) = snapshot.handoffs.last() else {
        return inspector_empty(
            ui_text(locale, "Handoff", "Handoff"),
            ui_text(
                locale,
                "No handoff packet.",
                "handoff packet はありません。",
            ),
        );
    };
    format!(
        r#"<div class="inspector-section">
  <h2>{}</h2>
  <span class="badge blue">{} -> {}</span>
  <p>{}</p>
  <dl><dt>current_state</dt><dd>{}</dd><dt>next_request</dt><dd>{}</dd><dt>artifacts</dt><dd>{}</dd></dl>
</div>"#,
        h(ui_text(locale, "Handoff", "Handoff")),
        h(&handoff.from_agent_profile),
        h(&handoff.to_agent_profile),
        h(&handoff.reason),
        h(&handoff.current_state),
        h(&handoff.next_request),
        h(&list_label(&handoff.artifact_ids)),
    )
}

fn inspector_empty(title: &str, message: &str) -> String {
    format!(
        r#"<div class="inspector-section empty"><h2>{}</h2><p>{}</p></div>"#,
        h(title),
        h(message)
    )
}

fn output_field_label(output: &AgentOutputRecord, key: &str) -> String {
    output
        .fields
        .get(key)
        .map(|values| list_label(values))
        .unwrap_or_else(|| "-".to_string())
}

fn latest_run_agent(snapshot: &WorkItemSnapshot) -> Option<&str> {
    snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work)
        .map(|run| run.agent_profile_id.as_str())
}

fn criteria_summary(snapshot: &WorkItemSnapshot) -> String {
    let total = snapshot.item.acceptance_criteria.len();
    if total == 0 {
        return "-".to_string();
    }
    let passed = snapshot
        .review_results
        .last()
        .map(|review| {
            review
                .criteria_results
                .iter()
                .filter(|result| result.status == CriteriaReviewStatus::Passed)
                .count()
        })
        .unwrap_or(0);
    format!("{passed}/{total}")
}

fn latest_recovery_label(snapshot: &WorkItemSnapshot) -> String {
    snapshot
        .recovery_plans
        .iter()
        .rev()
        .find(|plan| plan.status != RecoveryPlanStatus::Superseded)
        .map(|plan| format!("{} / {}", plan.status, plan.failure_class))
        .unwrap_or_else(|| "-".to_string())
}

fn needs_attention(snapshot: &WorkItemSnapshot) -> bool {
    !matches!(
        snapshot.item.status,
        WorkItemStatus::Done | WorkItemStatus::AgentRunning
    )
}

fn attention_reason(snapshot: &WorkItemSnapshot) -> String {
    snapshot
        .completion
        .blocking_reason
        .clone()
        .or_else(|| snapshot.approval_gate.blockers.first().cloned())
        .unwrap_or_else(|| snapshot.completion.next_action.clone())
}

fn latest_open_question(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| output.questions.first().cloned())
}

fn list_label(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}

fn status_kind(status: WorkItemStatus) -> &'static str {
    match status {
        WorkItemStatus::Done | WorkItemStatus::ReadyForReview => "green",
        WorkItemStatus::ChangesRequested => "red",
        WorkItemStatus::NeedsInput | WorkItemStatus::NeedsHandoff => "amber",
        WorkItemStatus::AgentRunning => "blue",
        _ => "gray",
    }
}

fn review_kind(verdict: ReviewVerdict) -> &'static str {
    match verdict {
        ReviewVerdict::Pass => "green",
        ReviewVerdict::RequestChanges => "red",
        ReviewVerdict::Blocked | ReviewVerdict::Unknown => "amber",
    }
}

fn criteria_kind(status: CriteriaReviewStatus) -> &'static str {
    match status {
        CriteriaReviewStatus::Passed => "green",
        CriteriaReviewStatus::Failed => "red",
        CriteriaReviewStatus::Unknown => "amber",
    }
}

fn page(title: &str, body: &str, locale: &str) -> String {
    format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"{}styles.css\"></head><body>{}<script>{}</script></body></html>",
        html_lang(locale),
        h(title),
        if title.starts_with("work_") {
            "../"
        } else {
            ""
        },
        body,
        static_ui_script()
    )
}

fn status_label(locale: &str, status: WorkItemStatus) -> String {
    if !is_ui_ja(locale) {
        return status.to_string();
    }
    match status {
        WorkItemStatus::Done => "完了".to_string(),
        WorkItemStatus::ReadyForReview => "レビュー待ち".to_string(),
        WorkItemStatus::ChangesRequested => "修正依頼".to_string(),
        WorkItemStatus::NeedsInput => "入力待ち".to_string(),
        WorkItemStatus::NeedsHandoff => "引き継ぎ待ち".to_string(),
        WorkItemStatus::AgentRunning => "Agent 実行中".to_string(),
        _ => status.to_string(),
    }
}

fn ui_text<'a>(locale: &str, en: &'a str, ja: &'a str) -> &'a str {
    if is_ui_ja(locale) { ja } else { en }
}

fn is_ui_ja(locale: &str) -> bool {
    locale.to_ascii_lowercase().starts_with("ja")
}

fn html_lang(locale: &str) -> &'static str {
    if is_ui_ja(locale) { "ja" } else { "en" }
}

fn stylesheet() -> &'static str {
    r#":root{color-scheme:light;--bg:#f8fafc;--surface:#fff;--surface2:#f1f5f9;--text:#020617;--muted:#475569;--line:#e2e8f0;--blue:#4338ca;--green:#047857;--amber:#b45309;--red:#b91c1c}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:14px/1.45 Inter,"Yu Gothic UI",Meiryo,Arial,sans-serif}.app{display:grid;grid-template-columns:200px 1fr;min-height:100vh}.sidebar{background:#fff;border-right:1px solid var(--line);padding:24px 18px}.brand{display:block;margin:0 0 24px}.brand-logo{display:block;width:132px;height:auto}.brand-text{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}nav a{display:block;padding:9px 14px;border-radius:7px;color:var(--muted);text-decoration:none;font-weight:700}nav a.active{background:#eef2ff;color:var(--blue)}.content{padding:26px 32px}.breadcrumbs{display:flex;gap:8px;align-items:center;color:var(--muted);font-size:12px;font-weight:800;margin:0 0 12px}.breadcrumbs a{padding:0;border-radius:0}.breadcrumbs span{color:var(--muted)}.topbar{display:flex;justify-content:space-between;gap:24px;align-items:flex-start;margin-bottom:22px}h1{font-size:24px;margin:0 0 4px}h2{font-size:17px;margin:0 0 12px}h3{font-size:14px;margin:18px 0 8px}.muted{color:var(--muted);font-size:12px}.panel,.composer{background:var(--surface);border:1px solid var(--line);border-radius:8px;padding:20px;margin-bottom:18px}.quick{display:flex;gap:10px;margin-bottom:18px;flex-wrap:wrap}.panel-head{display:flex;justify-content:space-between;align-items:center;gap:12px}.badge{display:inline-flex;align-items:center;min-height:24px;border-radius:12px;padding:4px 9px;font-size:11px;font-weight:800;margin-right:6px}.blue{background:#eef2ff;color:var(--blue)}.green{background:#ecfdf5;color:var(--green)}.amber{background:#fffbeb;color:var(--amber)}.red{background:#fef2f2;color:var(--red)}.gray{background:#f1f5f9;color:var(--muted)}.attention-list{display:grid;gap:10px}.attention-card{display:grid;grid-template-columns:minmax(180px,1fr) auto minmax(130px,220px);gap:12px;align-items:center;background:#fffbeb;border:1px solid #fde68a;border-radius:7px;padding:12px}.attention-row{background:#fffbeb}table{width:100%;border-collapse:collapse}th{text-align:left;color:var(--muted);font-size:11px;padding:10px;border-bottom:1px solid var(--line)}td{padding:12px 10px;border-bottom:1px solid var(--line);vertical-align:top}a{color:var(--blue);font-weight:800;text-decoration:none}code{display:inline-block;max-width:100%;overflow-wrap:anywhere;background:var(--surface2);border:1px solid var(--line);border-radius:6px;padding:5px 7px;font-family:Consolas,Menlo,monospace;font-size:12px}.grid{display:grid;gap:12px}.grid.four{grid-template-columns:repeat(4,1fr)}.grid div{background:var(--surface2);border:1px solid var(--line);border-radius:7px;padding:12px}.grid span{display:block;margin-top:6px}.detail-layout{display:grid;grid-template-columns:300px minmax(380px,1fr) 360px;gap:18px}.summary{position:sticky;top:18px;align-self:start}.next-action,.human-input{border-top:1px solid var(--line);margin-top:18px;padding-top:18px}.next-action code{display:block;margin:10px 0 12px}.human-input textarea{width:100%;min-height:96px;resize:vertical;border:1px solid var(--line);border-radius:7px;padding:10px;font:inherit;margin:10px 0}.command-row{display:grid;grid-template-columns:1fr auto;gap:8px;align-items:start}.command-row code{display:block}.command-row button{border:1px solid var(--line);background:var(--text);color:#fff;border-radius:7px;padding:7px 10px;font-weight:800;cursor:pointer}.timeline{position:relative}.event{display:grid;grid-template-columns:24px 1fr;gap:12px;padding:10px 0;border-bottom:1px solid var(--line)}.node{width:12px;height:12px;border:2px solid var(--blue);background:#eef2ff;border-radius:50%;margin-top:8px}.event-head{display:flex;gap:8px;align-items:center;flex-wrap:wrap}.event p{margin:6px 0}.history-facts{display:flex;gap:6px;flex-wrap:wrap;margin:8px 0}.history-facts span{background:var(--surface2);border:1px solid var(--line);border-radius:7px;padding:5px 8px;font-size:12px}.inspector-section{border-bottom:1px solid var(--line);padding-bottom:16px;margin-bottom:16px}.empty{color:var(--muted)}dl{display:grid;grid-template-columns:120px 1fr;gap:8px 10px}dt{color:var(--muted);font-size:12px}dd{margin:0;overflow-wrap:anywhere}ul{padding-left:18px}@media(max-width:1000px){.app{grid-template-columns:1fr}.sidebar{display:none}.detail-layout{grid-template-columns:1fr}.topbar{display:block}.actions{margin-top:12px}.grid.four{grid-template-columns:1fr 1fr}.attention-card{grid-template-columns:1fr}.command-row{grid-template-columns:1fr}}"#
}

fn static_ui_script() -> &'static str {
    r#"function nagareQuote(value){return '"' + value.replace(/\\/g,'\\\\').replace(/"/g,'\\"').replace(/\s+/g,' ').trim() + '"';}
document.querySelectorAll('.command-builder').forEach(function(builder){
  var input = builder.querySelector('textarea');
  var target = document.getElementById(builder.dataset.commandTarget);
  var prefix = builder.dataset.commandPrefix;
  if(!input || !target || !prefix){return;}
  function update(){target.textContent = prefix + ' ' + nagareQuote(input.value || '<text>');}
  input.addEventListener('input', update);
  update();
});
document.querySelectorAll('[data-copy-target]').forEach(function(button){
  button.addEventListener('click', function(){
    var target = document.getElementById(button.dataset.copyTarget);
    if(target && navigator.clipboard){navigator.clipboard.writeText(target.textContent);}
  });
});"#
}

fn h(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
