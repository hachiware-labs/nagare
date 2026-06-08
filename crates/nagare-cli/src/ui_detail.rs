use std::path::Path;

use nagare_core::{
    AgentRun, AgentRunPurpose, DispatchPlan, DispatchPlanStatus, RecoveryPlanStatus,
    WorkItemStatus, get_work_item_snapshot,
};

use crate::ui::read_ui_running_state;
use crate::ui_answer::{answer_view, render_answer_panel};
use crate::ui_assets::{serve_item_detail_stylesheet, serve_script, serve_stylesheet};
use crate::ui_history::render_run_history_panel;
use crate::ui_html::{h, is_empty_display_value, list_or_dash};
fn current_processing_state(
    status: &WorkItemStatus,
    next_action: &str,
    latest_dispatch: Option<&DispatchPlan>,
    latest_run: Option<&AgentRun>,
    running: Option<&str>,
) -> String {
    if let Some(running) = running {
        return format!("Processing: {running}");
    }
    if *status == WorkItemStatus::AgentRunning {
        return "エージェントが処理中です".to_string();
    }
    if let Some(run) = latest_run {
        if run.status.to_string() == "failed" {
            return format!("直近の {} 実行が失敗しました", run.purpose);
        }
    }
    if let Some(plan) = latest_dispatch
        .filter(|_| matches!(next_action, "dispatch" | "accept_dispatch" | "run_agent"))
    {
        return match plan.status {
            DispatchPlanStatus::Draft => {
                if next_action == "run_agent" {
                    format!(
                        "Dispatch が `{}` を選択しました。実行待ちです",
                        plan.target_agent_profile_id
                    )
                } else {
                    format!(
                        "Dispatch が `{}` を選択しました。承認待ちです",
                        plan.target_agent_profile_id
                    )
                }
            }
            DispatchPlanStatus::Accepted => {
                format!(
                    "Dispatch は承認済みです。次のエージェントは `{}` です",
                    plan.target_agent_profile_id
                )
            }
            DispatchPlanStatus::Superseded => "Dispatch plan was superseded".to_string(),
        };
    }
    match next_action {
        "dispatch" => "Waiting for dispatch".to_string(),
        "accept_dispatch" => "Dispatch plan の承認待ちです".to_string(),
        "run_agent" => "選択済みエージェントの実行待ちです".to_string(),
        "review" => "作業が完了し、レビュー待ちです".to_string(),
        "approve" => "承認待ちです".to_string(),
        "recover" => "復旧が必要です".to_string(),
        "none" => "追加対応は不要です".to_string(),
        other => other.to_string(),
    }
}

fn render_dispatch_panel(plan: Option<&DispatchPlan>, next_action: &str) -> String {
    let Some(plan) = plan else {
        return r#"<section class="panel workflow-panel"><div class="panel-head"><h2>Dispatch Plan</h2><span class="badge gray">not run</span></div><p class="muted">No dispatch plan has been created yet.</p></section>"#.to_string();
    };
    let warnings = list_or_dash(&plan.selection_warnings);
    let risks = list_or_dash(&plan.risks);
    let missing = list_or_dash(&plan.missing_information);
    let display_status = if plan.status == DispatchPlanStatus::Draft && next_action == "run_agent" {
        "selected".to_string()
    } else {
        plan.status.to_string()
    };
    let display_class = if display_status == "selected" {
        "green"
    } else {
        dispatch_status_class(plan.status)
    };
    format!(
        r#"<section class="panel workflow-panel">
  <div class="panel-head"><h2>Dispatch Plan</h2><span class="badge {}">{}</span></div>
  <dl>
    <dt>Plan</dt><dd>{}</dd>
    <dt>Selected agent</dt><dd><b>{}</b></dd>
    <dt>Dispatch Agent</dt><dd>{}</dd>
    <dt>Summary</dt><dd>{}</dd>
    <dt>Warnings</dt><dd>{}</dd>
    <dt>Risks</dt><dd>{}</dd>
    <dt>Missing info</dt><dd>{}</dd>
  </dl>
</section>"#,
        display_class,
        h(&display_status),
        h(&plan.id),
        h(&plan.target_agent_profile_id),
        h(&plan.dispatch_agent_profile_id),
        h(&plan.summary),
        warnings,
        risks,
        missing
    )
}

fn dispatch_status_class(status: DispatchPlanStatus) -> &'static str {
    match status {
        DispatchPlanStatus::Draft => "amber",
        DispatchPlanStatus::Accepted => "green",
        DispatchPlanStatus::Superseded => "gray",
    }
}

fn first_output_field(output: &nagare_core::AgentOutputRecord, key: &str) -> Option<String> {
    output
        .fields
        .get(key)
        .and_then(|values| values.iter().find(|value| !is_empty_display_value(value)))
        .cloned()
}

fn latest_valid_question(snapshot: &nagare_core::WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .flat_map(|output| output.questions.iter())
        .find(|question| !is_empty_display_value(question))
        .cloned()
}

fn latest_agent_result(snapshot: &nagare_core::WorkItemSnapshot) -> String {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| {
            first_output_field(output, "summary")
                .or_else(|| first_output_field(output, "completed"))
                .map(|summary| {
                    format!(
                        "{} / {}: {}",
                        output.agent_profile_id, output.purpose, summary
                    )
                })
        })
        .unwrap_or_else(|| "No agent output has been recorded yet.".to_string())
}

fn latest_agent_line(snapshot: &nagare_core::WorkItemSnapshot) -> String {
    snapshot
        .runs
        .last()
        .map(|run| {
            format!(
                "{} / {} ({})",
                run.agent_profile_id, run.purpose, run.status
            )
        })
        .unwrap_or_else(|| "まだエージェントは実行されていません。".to_string())
}

fn assigned_agent_line(latest_dispatch: Option<&DispatchPlan>) -> String {
    latest_dispatch
        .map(|plan| plan.target_agent_profile_id.clone())
        .unwrap_or_else(|| "未選定".to_string())
}

fn assigned_agent_context(latest_dispatch: Option<&DispatchPlan>, next_action: &str) -> String {
    let Some(plan) = latest_dispatch else {
        return "Dispatch plan はまだ作成されていません。".to_string();
    };
    let status = if plan.status == DispatchPlanStatus::Draft && next_action == "run_agent" {
        "選定済み"
    } else {
        match plan.status {
            DispatchPlanStatus::Draft => "承認待ち",
            DispatchPlanStatus::Accepted => "承認済み",
            DispatchPlanStatus::Superseded => "置き換え済み",
        }
    };
    if plan.summary.trim().is_empty() {
        format!("Dispatch plan は{status}です。選定理由は記録されていません。")
    } else {
        format!("Dispatch plan は{status}です。{}", plan.summary)
    }
}

fn run_status_class(status: nagare_core::AgentRunStatus) -> &'static str {
    match status {
        nagare_core::AgentRunStatus::Succeeded => "done",
        nagare_core::AgentRunStatus::Failed => "blocked",
    }
}

fn dispatch_flow_state(plan: Option<&DispatchPlan>, next_action: &str) -> (&'static str, String) {
    let Some(plan) = plan else {
        return ("pending", "未実行".to_string());
    };
    if plan.status == DispatchPlanStatus::Draft && next_action == "accept_dispatch" {
        return ("active", "承認待ち".to_string());
    }
    if plan.status == DispatchPlanStatus::Draft && next_action == "run_agent" {
        return ("done", "選定済み".to_string());
    }
    match plan.status {
        DispatchPlanStatus::Draft => ("active", "承認待ち".to_string()),
        DispatchPlanStatus::Accepted => ("done", "割り振り済み".to_string()),
        DispatchPlanStatus::Superseded => ("pending", "置き換え済み".to_string()),
    }
}

fn output_summary_for_run(
    snapshot: &nagare_core::WorkItemSnapshot,
    run: &AgentRun,
) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .find(|output| output.agent_run_id == run.id)
        .and_then(|output| {
            first_output_field(output, "summary")
                .or_else(|| first_output_field(output, "completed"))
        })
}

fn latest_run_for_purpose(
    snapshot: &nagare_core::WorkItemSnapshot,
    purpose: AgentRunPurpose,
) -> Option<&AgentRun> {
    snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == purpose)
}

fn render_flow_node(
    marker: &str,
    state_class: &str,
    label: &str,
    title: &str,
    detail: &str,
) -> String {
    format!(
        r#"<li class="flow-node {}">
  <span class="flow-marker">{}</span>
  <div>
    <span>{}</span>
    <b translate="no">{}</b>
    <small>{}</small>
  </div>
</li>"#,
        h(state_class),
        h(marker),
        h(label),
        h(title),
        h(detail)
    )
}

fn render_progress_panel(
    snapshot: &nagare_core::WorkItemSnapshot,
    latest_dispatch: Option<&DispatchPlan>,
    running: Option<&str>,
) -> String {
    let (dispatch_class, dispatch_state) =
        dispatch_flow_state(latest_dispatch, &snapshot.completion.next_action);
    let dispatch_title = latest_dispatch
        .map(|plan| plan.dispatch_agent_profile_id.as_str())
        .unwrap_or("Dispatcher");
    let dispatch_detail = latest_dispatch
        .map(|plan| {
            format!(
                "{} を作業エージェントに選定。{}",
                plan.target_agent_profile_id, plan.summary
            )
        })
        .unwrap_or_else(|| "まだ作業エージェントは選定されていません。".to_string());

    let work_run = latest_run_for_purpose(snapshot, AgentRunPurpose::Work);
    let work_target = latest_dispatch
        .map(|plan| plan.target_agent_profile_id.as_str())
        .unwrap_or("未選定");
    let (work_class, work_title, work_detail) = if let Some(run) = work_run {
        let detail = output_summary_for_run(snapshot, run).unwrap_or_else(|| {
            format!(
                "work run は {} で終了しました。exit {}",
                run.status,
                run.exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "-".to_string())
            )
        });
        (
            run_status_class(run.status),
            run.agent_profile_id.as_str(),
            detail,
        )
    } else if running == Some("run_agent") || snapshot.completion.next_action == "run_agent" {
        (
            "active",
            work_target,
            "選定済みエージェントの実行待ちです。".to_string(),
        )
    } else if latest_dispatch.is_some() {
        (
            "pending",
            work_target,
            "まだ作業実行は記録されていません。".to_string(),
        )
    } else {
        (
            "pending",
            "未選定",
            "Dispatcher の選定後に作業エージェントが表示されます。".to_string(),
        )
    };

    let review_run = latest_run_for_purpose(snapshot, AgentRunPurpose::Review);
    let latest_review = snapshot.review_results.iter().rev().next();
    let (review_class, review_title, review_detail) = if let Some(review) = latest_review {
        (
            "done",
            review.agent_profile_id.as_str(),
            format!("レビュー結果: {}", review.verdict),
        )
    } else if let Some(run) = review_run {
        (
            run_status_class(run.status),
            run.agent_profile_id.as_str(),
            output_summary_for_run(snapshot, run)
                .unwrap_or_else(|| format!("review run は {} で終了しました。", run.status)),
        )
    } else if snapshot.completion.next_action == "review" {
        (
            "active",
            "レビュー待ち",
            "作業エージェントの出力後、レビュー実行待ちです。".to_string(),
        )
    } else if snapshot.completion.next_action == "approve" {
        (
            "active",
            "承認待ち",
            "レビュー後、最終承認待ちです。".to_string(),
        )
    } else if snapshot.item.status == WorkItemStatus::Done {
        ("done", "完了", "Work Item は完了しています。".to_string())
    } else {
        (
            "pending",
            "未実施",
            "作業実行後にレビュー状態が表示されます。".to_string(),
        )
    };

    format!(
        r#"<section class="panel progress-panel">
  <div class="panel-head">
    <div>
      <h2>進行フロー</h2>
      <p class="muted">Dispatcher以降の受け渡しを、ユーザーが読む順番で表示しています。</p>
    </div>
  </div>
  <ol class="flow-list">
    {}
    {}
    {}
  </ol>
</section>"#,
        render_flow_node(
            "1",
            dispatch_class,
            &dispatch_state,
            dispatch_title,
            &dispatch_detail
        ),
        render_flow_node("2", work_class, "作業", work_title, &work_detail),
        render_flow_node(
            "3",
            review_class,
            "レビュー/承認",
            review_title,
            &review_detail
        )
    )
}

fn judgment_reason(
    snapshot: &nagare_core::WorkItemSnapshot,
    current_state: &str,
    latest_question: Option<&str>,
    running: Option<&str>,
) -> String {
    if let Some(running) = running {
        return format!("{running} is running. You can wait on this page.");
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput {
        return latest_question
            .map(|question| format!("The agent asked for human input: {question}"))
            .unwrap_or_else(|| "入力待ち状態ですが、有効な質問は記録されていません。".to_string());
    }
    if let Some(output) = snapshot.agent_outputs.iter().rev().find(|output| {
        !output.warnings.is_empty()
            || output.parse_status == nagare_core::AgentOutputParseStatus::Unparsed
    }) {
        if output.parse_status == nagare_core::AgentOutputParseStatus::Unparsed {
            return format!(
                "{} の出力を Nagare の出力契約として解析できませんでした。",
                output.agent_profile_id
            );
        }
        return format!(
            "{} の出力に契約警告があります: {}",
            output.agent_profile_id,
            output.warnings.join(", ")
        );
    }
    current_state.to_string()
}

fn judgment_label(
    snapshot: &nagare_core::WorkItemSnapshot,
    current_state: &str,
    latest_question: Option<&str>,
    running: Option<&str>,
) -> String {
    if running.is_some() {
        return "Processing".to_string();
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput {
        return if latest_question.is_some() {
            "人の入力待ち".to_string()
        } else {
            "確認が必要".to_string()
        };
    }
    match snapshot.completion.next_action.as_str() {
        "review" => "レビュー待ち".to_string(),
        "approve" => "Ready for approval".to_string(),
        "recover" => "復旧が必要".to_string(),
        "none" => "追加対応なし".to_string(),
        _ => current_state.to_string(),
    }
}

fn next_action_label(
    snapshot: &nagare_core::WorkItemSnapshot,
    latest_question: Option<&str>,
    running: Option<&str>,
) -> String {
    if running.is_some() {
        return "Wait for processing".to_string();
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput && latest_question.is_none() {
        return "回答できる質問がありません。最新のエージェント出力を確認してください。"
            .to_string();
    }
    match snapshot.completion.next_action.as_str() {
        "answer_question" => "エージェントの質問に回答".to_string(),
        "review" => "レビューを実行".to_string(),
        "approve" => "最終結果を承認".to_string(),
        "recover" => "復旧を作成または適用".to_string(),
        "none" => "対応不要".to_string(),
        other => other.to_string(),
    }
}

fn render_summary_panel(
    snapshot: &nagare_core::WorkItemSnapshot,
    current_state: &str,
    latest_dispatch: Option<&DispatchPlan>,
    latest_question: Option<&str>,
    running: Option<&str>,
) -> String {
    let reason = judgment_reason(snapshot, current_state, latest_question, running);
    let current = judgment_label(snapshot, current_state, latest_question, running);
    let next = next_action_label(snapshot, latest_question, running);
    let result = latest_agent_result(snapshot);
    let last_agent = latest_agent_line(snapshot);
    let assigned_agent = assigned_agent_line(latest_dispatch);
    let assigned_context =
        assigned_agent_context(latest_dispatch, &snapshot.completion.next_action);
    format!(
        r#"<section id="detail" class="panel summary">
  <div class="panel-head">
    <div>
      <h2>状況</h2>
      <p class="muted">このWork Itemが誰に割り振られ、次に何を待っているかを表示しています。</p>
    </div>
    <span class="badge blue">Current decision</span>
  </div>
  <div class="status-grid">
    <div class="status-card primary">
      <span>割り振り先</span>
      <b translate="no">{}</b>
      <small>{}</small>
    </div>
    <div class="status-card">
      <span>現在の状態</span>
      <b>{}</b>
      <small>{}</small>
    </div>
    <div class="status-card">
      <span>次に必要なこと</span>
      <b>{}</b>
      <small>System action: {}</small>
    </div>
    <div class="status-card">
      <span>直近の実行</span>
      <b translate="no">{}</b>
      <small>{}</small>
    </div>
  </div>
  <dl class="summary-meta">
    <dt>Status</dt><dd>{}</dd>
    <dt>Domain</dt><dd>{}</dd>
    <dt>Workflow mode</dt><dd>{}</dd>
    <dt>Final approval</dt><dd>{}</dd>
    <dt>ID</dt><dd>{}</dd>
  </dl>
</section>"#,
        h(&assigned_agent),
        h(&assigned_context),
        h(&current),
        h(&reason),
        h(&next),
        h(&snapshot.completion.next_action),
        h(&last_agent),
        h(&result),
        h(&snapshot.item.status.to_string()),
        h(snapshot
            .item
            .domain_id
            .as_deref()
            .unwrap_or("project default")),
        h(&snapshot.item.workflow_mode.to_string()),
        h(&snapshot.item.approval_policy.to_string()),
        h(&snapshot.item.id)
    )
}

pub(crate) fn render_serve_item_detail(root: &Path, work_item_id: &str) -> Result<String, String> {
    let snapshot = get_work_item_snapshot(root, work_item_id).map_err(|error| error.to_string())?;
    let latest_dispatch = snapshot.dispatch_plans.iter().rev().next();
    let dispatch_panel = render_dispatch_panel(latest_dispatch, &snapshot.completion.next_action);
    let running_state = read_ui_running_state(root, work_item_id);
    let run_history_panel = render_run_history_panel(&snapshot, running_state.as_deref());
    let answer_panel = render_answer_panel(&answer_view(&snapshot));
    let current_state = current_processing_state(
        &snapshot.item.status,
        &snapshot.completion.next_action,
        latest_dispatch,
        snapshot.runs.last(),
        running_state.as_deref(),
    );
    let latest_question = latest_valid_question(&snapshot);
    let answer_form =
        if snapshot.item.status == WorkItemStatus::NeedsInput && latest_question.is_some() {
            let latest_question = latest_question.as_deref().unwrap_or("");
            format!(
                r#"<form id="answer-form" data-work-id="{}">
  <input type="hidden" name="question" value="{}">
  <input type="hidden" name="workflow_mode" value="finish_first">
  <input type="hidden" name="max_steps" value="8">
  <input type="hidden" name="command" value="">
  <input type="hidden" name="review_command" value="">
  <label>Question<textarea readonly rows="3">{}</textarea></label>
  <label>Answer<textarea name="answer" rows="4" required></textarea></label>
  <button type="submit">Submit Answer</button>
  <p id="answer-status" class="muted" role="status"></p>
</form>"#,
                h(&snapshot.item.id),
                h(latest_question),
                h(latest_question)
            )
        } else {
            "<p class=\"muted\">No human input is currently actionable.</p>".to_string()
        };
    let approve_form = if snapshot.approval_gate.ready {
        r#"<form id="approve-form">
  <button type="submit">Approve and finish</button>
  <p id="approve-status" class="muted" role="status"></p>
</form>
<form id="reject-form">
  <label>Reject reason<textarea name="rationale" rows="3" required placeholder="差し戻す理由"></textarea></label>
  <button class="danger" type="submit">Reject and redispatch</button>
  <p id="reject-status" class="muted" role="status"></p>
</form>"#
            .to_string()
    } else {
        "<p class=\"muted\">Approval gate is not ready.</p>".to_string()
    };
    let latest_recovery = snapshot.recovery_plans.iter().rev().next();
    let latest_draft_recovery = snapshot
        .recovery_plans
        .iter()
        .rev()
        .find(|plan| plan.status == RecoveryPlanStatus::Draft);
    let latest_accepted_recovery = snapshot
        .recovery_plans
        .iter()
        .rev()
        .find(|plan| plan.status == RecoveryPlanStatus::Accepted);
    let recovery_summary = latest_recovery
        .map(|plan| {
            format!(
                "<dl><dt>Plan</dt><dd>{}</dd><dt>Status</dt><dd>{}</dd><dt>Action</dt><dd>{}</dd><dt>Reason</dt><dd>{}</dd><dt>Summary</dt><dd>{}</dd></dl>",
                h(&plan.id),
                h(&plan.status.to_string()),
                h(&plan.action.to_string()),
                h(&plan.reason),
                h(&plan.summary)
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">No recovery plan yet.</p>".to_string());
    let recover_create_form = if snapshot.completion.next_action == "recover" {
        r#"<form id="recover-form">
  <button type="submit">Create Recovery Plan</button>
  <p id="recover-status" class="muted" role="status"></p>
</form>"#
            .to_string()
    } else {
        "<p class=\"muted\">Recovery is not the next action.</p>".to_string()
    };
    let recover_accept_form = latest_draft_recovery
        .map(|plan| {
            format!(
                r#"<form id="recover-accept-form">
  <input type="hidden" name="recovery_plan" value="{}">
  <button type="submit">Accept Recovery Plan</button>
  <p id="recover-accept-status" class="muted" role="status"></p>
</form>"#,
                h(&plan.id)
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">No draft recovery plan to accept.</p>".to_string());
    let recover_apply_form = latest_accepted_recovery
        .map(|plan| {
            format!(
                r#"<form id="recover-apply-form">
  <input type="hidden" name="recovery_plan" value="{}">
  <label>Prompt<textarea name="prompt" rows="4" placeholder="Recovery agentへの指示">{}</textarea></label>
  <label>Command<textarea name="command" rows="2" placeholder="E2E/dev用 command"></textarea></label>
  <button type="submit">Apply Recovery Plan</button>
  <p id="recover-apply-status" class="muted" role="status"></p>
</form>"#,
                h(&plan.id),
                h(plan.prompt_hint.as_deref().unwrap_or(""))
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">No accepted recovery plan to apply.</p>".to_string());
    let recovery_panel = format!(
        "{}{}{}{}",
        recovery_summary, recover_create_form, recover_accept_form, recover_apply_form
    );
    let mut action_sections = Vec::new();
    if snapshot.item.status == WorkItemStatus::NeedsInput && latest_question.is_some() {
        action_sections.push(format!(
            r#"<section class="panel primary-action"><h2>Answer</h2>{answer_form}</section>"#
        ));
    }
    if snapshot.completion.next_action == "review" {
        action_sections.push(
            r#"<section class="panel primary-action"><h2>Review</h2>
<form id="review-form">
  <button type="submit">レビューを実行</button>
  <p id="review-status" class="muted" role="status"></p>
</form>
</section>"#
                .to_string(),
        );
    }
    if snapshot.approval_gate.ready {
        action_sections.push(format!(
            r#"<section class="panel primary-action"><h2>Approval</h2>{approve_form}</section>"#
        ));
    }
    if snapshot.completion.next_action == "recover" || latest_recovery.is_some() {
        action_sections.push(format!(
            r#"<section class="panel primary-action"><h2>Recovery</h2>{recovery_panel}</section>"#
        ));
    }
    if let Some(running) = running_state.as_deref() {
        action_sections.insert(
            0,
            format!(
                r#"<section class="panel primary-action"><h2>Processing</h2><p>{}</p><p class="muted">This page refreshes automatically while the workflow can continue.</p></section>"#,
                h(&format!("{running} is running."))
            ),
        );
    }
    let action_sections = if action_sections.is_empty() {
        if snapshot.completion.next_action == "none" {
            r#"<section class="panel primary-action"><h2>Next Action</h2><p class="muted">No action is currently required.</p></section>"#
                .to_string()
        } else {
            format!(
                r#"<section class="panel primary-action"><h2>Next Action</h2><p>{}</p><p class="muted">Nagare can continue when the current step is available.</p></section>"#,
                h(&next_action_label(
                    &snapshot,
                    latest_question.as_deref(),
                    running_state.as_deref()
                ))
            )
        }
    } else {
        action_sections.join("\n")
    };
    let workflow_panels = format!("{run_history_panel}{dispatch_panel}");
    let summary_panel = render_summary_panel(
        &snapshot,
        &current_state,
        latest_dispatch,
        latest_question.as_deref(),
        running_state.as_deref(),
    );
    let progress_panel =
        render_progress_panel(&snapshot, latest_dispatch, running_state.as_deref());
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}</style>
</head>
<body data-next-action="{}" data-running="{}">
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a class="active" href="/">Work Queue</a>
        <a href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <nav class="breadcrumbs" aria-label="Breadcrumb">
        <a href="/">Work Queue</a>
        <span>/</span>
        <span>Detail</span>
      </nav>
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <span class="badge blue">{}</span>
          <span class="badge gray">next {}</span>
        </div>
      </header>
      <div class="detail-layout">
        {}
        {}
        {}
        <section id="workflow" class="action-stack">{}</section>
        <section id="human-action" class="action-stack">{}</section>
      </div>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        h(&snapshot.item.title),
        format!("{}{}", serve_stylesheet(), serve_item_detail_stylesheet()),
        h(&snapshot.completion.next_action),
        h(running_state.as_deref().unwrap_or("")),
        h(&snapshot.item.title),
        h(&snapshot.item.id),
        h(&snapshot.item.status.to_string()),
        h(&snapshot.completion.next_action),
        summary_panel,
        progress_panel,
        answer_panel,
        workflow_panels,
        action_sections,
        serve_script()
    ))
}
