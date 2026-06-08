use std::path::Path;

use nagare_core::{
    AgentRun, AgentRunPurpose, DispatchPlan, DispatchPlanStatus, RecoveryPlanStatus,
    WorkItemStatus, get_work_item_snapshot, list_agent_profiles,
};

use crate::ui::read_ui_running_state;
use crate::ui_agent::{agent_label, agent_label_with_meta};
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
        return format!("処理中: {running}");
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
        "done" => "完了しています".to_string(),
        "none" => "追加対応は不要です".to_string(),
        other => other.to_string(),
    }
}

fn render_dispatch_panel(
    plan: Option<&DispatchPlan>,
    next_action: &str,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    let Some(plan) = plan else {
        return r#"<section class="panel workflow-panel"><div class="panel-head"><h2>Dispatch Plan</h2><span class="badge gray">not run</span></div><p class="muted">No dispatch plan has been created yet.</p></section>"#.to_string();
    };
    let mut optional_rows = String::new();
    if !plan.selection_warnings.is_empty() {
        optional_rows.push_str(&format!(
            "<dt>Warnings</dt><dd>{}</dd>",
            list_or_dash(&plan.selection_warnings)
        ));
    }
    if !plan.risks.is_empty() {
        optional_rows.push_str(&format!(
            "<dt>Risks</dt><dd>{}</dd>",
            list_or_dash(&plan.risks)
        ));
    }
    if !plan.missing_information.is_empty() {
        optional_rows.push_str(&format!(
            "<dt>Missing info</dt><dd>{}</dd>",
            list_or_dash(&plan.missing_information)
        ));
    }
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
    {}
  </dl>
</section>"#,
        display_class,
        h(&display_status),
        h(&plan.id),
        h(&agent_label_with_meta(
            profiles,
            &plan.target_agent_profile_id
        )),
        h(&agent_label_with_meta(
            profiles,
            &plan.dispatch_agent_profile_id
        )),
        h(&plan.summary),
        optional_rows
    )
}

fn dispatch_status_class(status: DispatchPlanStatus) -> &'static str {
    match status {
        DispatchPlanStatus::Draft => "amber",
        DispatchPlanStatus::Accepted => "green",
        DispatchPlanStatus::Superseded => "gray",
    }
}

fn work_item_match_text(item: &nagare_core::WorkItem) -> String {
    format!(
        "{}\n{}\n{}",
        item.title,
        item.description,
        item.acceptance_criteria.join("\n")
    )
    .to_lowercase()
}

fn matched_terms(values: &[String], item_text: &str) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return None;
            }
            if item_text.contains(&trimmed.to_lowercase()) {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn domain_match_reason(
    profile: &nagare_core::AgentProfile,
    item: &nagare_core::WorkItem,
) -> String {
    if item.domain_id.is_none() && item.domain_group_id.is_none() {
        if profile.domain_ids.is_empty() && profile.domain_group_ids.is_empty() {
            return "ドメイン指定なし: 制約なしAgent".to_string();
        }
        if profile.domain_ids.iter().any(|domain| domain == "general")
            || profile
                .domain_group_ids
                .iter()
                .any(|group| group == "general")
        {
            return "ドメイン指定なし: 汎用Agent".to_string();
        }
        return "ドメイン指定なし: 専用ドメインAgent".to_string();
    }
    let domain_match = item.domain_id.as_deref().is_some_and(|domain| {
        profile
            .domain_ids
            .iter()
            .any(|profile_domain| profile_domain == domain)
    });
    let group_match = item.domain_group_id.as_deref().is_some_and(|group| {
        profile
            .domain_group_ids
            .iter()
            .any(|profile_group| profile_group == group)
    });
    if domain_match || group_match {
        return "ドメイン一致".to_string();
    }
    if profile.domain_ids.is_empty() && profile.domain_group_ids.is_empty() {
        return "ドメイン制約なし".to_string();
    }
    if profile.domain_ids.iter().any(|domain| domain == "general")
        || profile
            .domain_group_ids
            .iter()
            .any(|group| group == "general")
    {
        return "汎用Agentとして候補".to_string();
    }
    "ドメイン不一致".to_string()
}

fn candidate_score(
    profile: &nagare_core::AgentProfile,
    item: &nagare_core::WorkItem,
    item_text: &str,
) -> i32 {
    if profile.role != "worker" {
        return -1000;
    }
    let mut score = 10;
    score += (matched_terms(&profile.specialties, item_text).len() as i32) * 40;
    score += (matched_terms(&profile.skill_set_ids, item_text).len() as i32) * 20;
    let domain_reason = domain_match_reason(profile, item);
    if domain_reason == "ドメイン一致" {
        score += 30;
    } else if domain_reason == "ドメイン制約なし" || domain_reason == "汎用Agentとして候補"
    {
        score += 8;
    }
    score
}

fn candidate_reason_parts(
    profile: &nagare_core::AgentProfile,
    item: &nagare_core::WorkItem,
    item_text: &str,
) -> Vec<String> {
    let mut parts = Vec::new();
    if profile.role == "worker" {
        parts.push("role=worker: 作業候補".to_string());
    } else {
        parts.push(format!("role={}: 作業実行候補ではありません", profile.role));
    }
    let specialties = matched_terms(&profile.specialties, item_text);
    if specialties.is_empty() {
        if profile.specialties.is_empty() {
            parts.push("専門性: 未設定".to_string());
        } else {
            parts.push(format!(
                "専門性: 一致なし ({})",
                profile.specialties.join(", ")
            ));
        }
    } else {
        parts.push(format!("専門性一致: {}", specialties.join(", ")));
    }
    parts.push(domain_match_reason(profile, item));
    let skills = matched_terms(&profile.skill_set_ids, item_text);
    if skills.is_empty() {
        if profile.skill_set_ids.is_empty() {
            parts.push("スキル: 未設定".to_string());
        } else {
            parts.push(format!("保有スキル: {}", profile.skill_set_ids.join(", ")));
        }
    } else {
        parts.push(format!("スキル一致: {}", skills.join(", ")));
    }
    parts
}

fn candidate_main_reason(
    profile: &nagare_core::AgentProfile,
    item: &nagare_core::WorkItem,
    item_text: &str,
) -> String {
    let parts = candidate_reason_parts(profile, item, item_text);
    parts
        .iter()
        .find(|part| part.starts_with("専門性一致:") || part.starts_with("スキル一致:"))
        .or_else(|| parts.iter().find(|part| part.as_str() == "ドメイン一致"))
        .or_else(|| parts.iter().find(|part| part.contains("汎用Agent")))
        .cloned()
        .unwrap_or_else(|| "選定理由は詳細ログで確認できます".to_string())
}

fn render_candidate_evaluation_panel(
    snapshot: &nagare_core::WorkItemSnapshot,
    latest_dispatch: Option<&DispatchPlan>,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    let selected_agent = latest_dispatch.map(|plan| plan.target_agent_profile_id.as_str());
    let item_text = work_item_match_text(&snapshot.item);
    let selected_score = selected_agent
        .and_then(|selected| profiles.iter().find(|profile| profile.id == selected))
        .map(|profile| candidate_score(profile, &snapshot.item, &item_text));
    let selected_summary = selected_agent
        .and_then(|selected| profiles.iter().find(|profile| profile.id == selected))
        .map(|profile| {
            format!(
                "{}: {}",
                agent_label(profiles, &profile.id),
                candidate_main_reason(profile, &snapshot.item, &item_text)
            )
        })
        .unwrap_or_else(|| "作業エージェントはまだ選定されていません".to_string());
    let mut rows = profiles
        .iter()
        .map(|profile| {
            let score = candidate_score(profile, &snapshot.item, &item_text);
            let selected = selected_agent == Some(profile.id.as_str());
            let (status_class, status_label) = if selected {
                ("green", "選定")
            } else if profile.role != "worker" {
                ("gray", "除外")
            } else if selected_score.is_some_and(|selected_score| score < selected_score) {
                ("gray", "未選定")
            } else {
                ("blue", "候補")
            };
            let reason = if selected {
                format!(
                    "選定理由: {}",
                    candidate_reason_parts(profile, &snapshot.item, &item_text).join(" / ")
                )
            } else if profile.role != "worker" {
                format!(
                    "除外理由: {}",
                    candidate_reason_parts(profile, &snapshot.item, &item_text).join(" / ")
                )
            } else {
                format!(
                    "未選定理由: {}",
                    candidate_reason_parts(profile, &snapshot.item, &item_text).join(" / ")
                )
            };
            (
                selected,
                score,
                profile.id.clone(),
                status_class,
                status_label,
                reason,
            )
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    let rows = rows
        .into_iter()
        .map(|(_, score, agent_id, status_class, status_label, reason)| {
            format!(
                r#"<article class="candidate-row">
  <div class="candidate-head">
    <b translate="no">{}</b>
    <span class="badge {}">{}</span>
  </div>
  <p>{}</p>
  <dl>
    <dt>Score</dt><dd>{}</dd>
    <dt>Agent</dt><dd>{}</dd>
  </dl>
</article>"#,
                h(&agent_label_with_meta(profiles, &agent_id)),
                h(status_class),
                h(status_label),
                h(&reason),
                h(&score.to_string()),
                h(&agent_id)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<details class="panel candidate-panel detail-disclosure">
  <summary><span>なぜこのエージェント？</span><small>{}</small></summary>
  <p class="muted">role、専門性、ドメイン、スキルから現在のAgent候補を評価しています。</p>
  <div class="candidate-list">{rows}</div>
  </details>"#,
        h(&selected_summary)
    )
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

fn latest_agent_result(
    snapshot: &nagare_core::WorkItemSnapshot,
    profiles: &[nagare_core::AgentProfile],
) -> String {
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
                        agent_label(profiles, &output.agent_profile_id),
                        purpose_label(output.purpose),
                        summary
                    )
                })
        })
        .unwrap_or_else(|| "No agent output has been recorded yet.".to_string())
}

fn latest_agent_line(
    snapshot: &nagare_core::WorkItemSnapshot,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    snapshot
        .runs
        .last()
        .map(|run| {
            format!(
                "{} / {} ({})",
                agent_label(profiles, &run.agent_profile_id),
                purpose_label(run.purpose),
                run_status_label(run.status)
            )
        })
        .unwrap_or_else(|| "まだエージェントは実行されていません。".to_string())
}

fn purpose_label(purpose: AgentRunPurpose) -> &'static str {
    match purpose {
        AgentRunPurpose::Work => "作業",
        AgentRunPurpose::DispatchPreview => "割り振り",
        AgentRunPurpose::Review => "レビュー",
        AgentRunPurpose::WorkflowSupervision => "進行管理",
    }
}

fn run_status_label(status: nagare_core::AgentRunStatus) -> &'static str {
    match status {
        nagare_core::AgentRunStatus::Succeeded => "完了",
        nagare_core::AgentRunStatus::Failed => "失敗",
    }
}

fn work_item_status_label(status: &WorkItemStatus) -> &'static str {
    match status {
        WorkItemStatus::Ready => "受付済み",
        WorkItemStatus::AgentRunning => "処理中",
        WorkItemStatus::NeedsInput => "入力待ち",
        WorkItemStatus::NeedsHandoff => "引き継ぎ待ち",
        WorkItemStatus::ReadyForReview => "レビュー待ち",
        WorkItemStatus::ChangesRequested => "修正依頼",
        WorkItemStatus::Done => "完了",
    }
}

fn assigned_agent_line(
    latest_dispatch: Option<&DispatchPlan>,
    profiles: &[nagare_core::AgentProfile],
) -> String {
    latest_dispatch
        .map(|plan| agent_label(profiles, &plan.target_agent_profile_id))
        .unwrap_or_else(|| "未選定".to_string())
}

fn assigned_agent_context(
    latest_dispatch: Option<&DispatchPlan>,
    next_action: &str,
    _profiles: &[nagare_core::AgentProfile],
) -> String {
    let Some(plan) = latest_dispatch else {
        return "作業エージェントはまだ選定されていません。".to_string();
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
    format!("割り振りは{status}です。理由は下の「なぜこのエージェント？」で確認できます。")
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
    profiles: &[nagare_core::AgentProfile],
) -> String {
    let (dispatch_class, dispatch_state) =
        dispatch_flow_state(latest_dispatch, &snapshot.completion.next_action);
    let dispatch_title = latest_dispatch
        .map(|plan| agent_label(profiles, &plan.dispatch_agent_profile_id))
        .unwrap_or_else(|| "Dispatcher".to_string());
    let dispatch_detail = latest_dispatch
        .map(|plan| {
            let reason = profiles
                .iter()
                .find(|profile| profile.id == plan.target_agent_profile_id)
                .map(|profile| {
                    candidate_main_reason(
                        profile,
                        &snapshot.item,
                        &work_item_match_text(&snapshot.item),
                    )
                })
                .unwrap_or_else(|| "選定理由は詳細ログで確認できます".to_string());
            format!(
                "{} を作業エージェントに選定。{}",
                agent_label(profiles, &plan.target_agent_profile_id),
                reason
            )
        })
        .unwrap_or_else(|| "まだ作業エージェントは選定されていません。".to_string());

    let work_run = latest_run_for_purpose(snapshot, AgentRunPurpose::Work);
    let work_target = latest_dispatch
        .map(|plan| agent_label(profiles, &plan.target_agent_profile_id))
        .unwrap_or_else(|| "未選定".to_string());
    let (work_class, work_title, work_detail) = if let Some(run) = work_run {
        let detail = output_summary_for_run(snapshot, run).unwrap_or_else(|| {
            format!(
                "作業実行は{}で終了しました。終了コード{}",
                run_status_label(run.status),
                run.exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "-".to_string())
            )
        });
        (
            run_status_class(run.status),
            agent_label(profiles, &run.agent_profile_id),
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
            "未選定".to_string(),
            "Dispatcher の選定後に作業エージェントが表示されます。".to_string(),
        )
    };

    let review_run = latest_run_for_purpose(snapshot, AgentRunPurpose::Review);
    let latest_review = snapshot.review_results.iter().rev().next();
    let (review_class, review_title, review_detail) = if let Some(review) = latest_review {
        (
            "done",
            agent_label(profiles, &review.agent_profile_id),
            format!("レビュー結果: {}", review.verdict),
        )
    } else if let Some(run) = review_run {
        (
            run_status_class(run.status),
            agent_label(profiles, &run.agent_profile_id),
            output_summary_for_run(snapshot, run).unwrap_or_else(|| {
                format!(
                    "レビュー実行は{}で終了しました。",
                    run_status_label(run.status)
                )
            }),
        )
    } else if snapshot.completion.next_action == "review" {
        (
            "active",
            "レビュー待ち".to_string(),
            "作業エージェントの出力後、レビュー実行待ちです。".to_string(),
        )
    } else if snapshot.completion.next_action == "approve" {
        (
            "active",
            "承認待ち".to_string(),
            "レビュー後、最終承認待ちです。".to_string(),
        )
    } else if snapshot.item.status == WorkItemStatus::Done {
        (
            "done",
            "完了".to_string(),
            "Work Item は完了しています。".to_string(),
        )
    } else {
        (
            "pending",
            "未実施".to_string(),
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
            &dispatch_title,
            &dispatch_detail
        ),
        render_flow_node("2", work_class, "作業", &work_title, &work_detail),
        render_flow_node(
            "3",
            review_class,
            "レビュー/承認",
            &review_title,
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
        return format!("{running} を実行中です。このページで待機できます。");
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput {
        return latest_question
            .map(|question| format!("エージェントから確認があります: {question}"))
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
        return "処理中".to_string();
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput {
        return if latest_question.is_some() {
            "人の入力待ち".to_string()
        } else {
            "確認が必要".to_string()
        };
    }
    if snapshot.item.status == WorkItemStatus::ChangesRequested {
        return "修正対応待ち".to_string();
    }
    match snapshot.completion.next_action.as_str() {
        "review" => "レビュー待ち".to_string(),
        "approve" => "承認待ち".to_string(),
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
        return "処理完了を待つ".to_string();
    }
    if snapshot.item.status == WorkItemStatus::NeedsInput && latest_question.is_none() {
        return "回答できる質問がありません。最新のエージェント出力を確認してください。"
            .to_string();
    }
    match snapshot.completion.next_action.as_str() {
        "dispatch" => "作業エージェントを選定".to_string(),
        "accept_dispatch" => "割り振りを承認".to_string(),
        "run_agent" => "作業エージェントを実行".to_string(),
        "answer_question" => "エージェントの質問に回答".to_string(),
        "review" => "レビューを実行".to_string(),
        "approve" => "最終結果を承認".to_string(),
        "recover" => "復旧を作成または適用".to_string(),
        "apply_recovery" => "復旧プランを適用".to_string(),
        "done" => "対応不要".to_string(),
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
    profiles: &[nagare_core::AgentProfile],
) -> String {
    let reason = judgment_reason(snapshot, current_state, latest_question, running);
    let current = judgment_label(snapshot, current_state, latest_question, running);
    let next = next_action_label(snapshot, latest_question, running);
    let result = latest_agent_result(snapshot, profiles);
    let last_agent = latest_agent_line(snapshot, profiles);
    let assigned_agent = assigned_agent_line(latest_dispatch, profiles);
    let assigned_context =
        assigned_agent_context(latest_dispatch, &snapshot.completion.next_action, profiles);
    format!(
        r#"<section id="detail" class="panel summary">
  <div class="panel-head">
    <div>
      <h2>状況</h2>
      <p class="muted">このWork Itemが誰に割り振られ、次に何を待っているかを表示しています。</p>
    </div>
    <span class="badge blue">現在</span>
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
      <small>この画面で実行できる次の操作です</small>
    </div>
    <div class="status-card">
      <span>直近の実行</span>
      <b translate="no">{}</b>
      <small>{}</small>
    </div>
  </div>
</section>"#,
        h(&assigned_agent),
        h(&assigned_context),
        h(&current),
        h(&reason),
        h(&next),
        h(&last_agent),
        h(&result)
    )
}

pub(crate) fn render_serve_item_detail(root: &Path, work_item_id: &str) -> Result<String, String> {
    let snapshot = get_work_item_snapshot(root, work_item_id).map_err(|error| error.to_string())?;
    let agent_profiles = list_agent_profiles(root).unwrap_or_default();
    let latest_dispatch = snapshot.dispatch_plans.iter().rev().next();
    let dispatch_panel = render_dispatch_panel(
        latest_dispatch,
        &snapshot.completion.next_action,
        &agent_profiles,
    );
    let running_state = read_ui_running_state(root, work_item_id);
    let run_history_panel =
        render_run_history_panel(&snapshot, running_state.as_deref(), &agent_profiles);
    let answer_panel = render_answer_panel(&answer_view(&snapshot, &agent_profiles));
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
  <label>質問<textarea readonly rows="3">{}</textarea></label>
  <label>回答<textarea name="answer" rows="4" required></textarea></label>
  <button type="submit">回答を送信</button>
  <p id="answer-status" class="muted" role="status"></p>
</form>"#,
                h(&snapshot.item.id),
                h(latest_question),
                h(latest_question)
            )
        } else {
            "<p class=\"muted\">現在、回答が必要な質問はありません。</p>".to_string()
        };
    let approve_form = if snapshot.approval_gate.ready {
        r#"<form id="approve-form">
  <button type="submit">承認して完了</button>
  <p id="approve-status" class="muted" role="status"></p>
</form>
<form id="reject-form">
  <label>差し戻し理由<textarea name="rationale" rows="3" required placeholder="差し戻す理由"></textarea></label>
  <button class="danger" type="submit">差し戻して再割り振り</button>
  <p id="reject-status" class="muted" role="status"></p>
</form>"#
            .to_string()
    } else {
        "<p class=\"muted\">承認できる状態ではありません。</p>".to_string()
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
                "<dl><dt>プラン</dt><dd>{}</dd><dt>状態</dt><dd>{}</dd><dt>操作</dt><dd>{}</dd><dt>理由</dt><dd>{}</dd><dt>概要</dt><dd>{}</dd></dl>",
                h(&plan.id),
                h(&plan.status.to_string()),
                h(&plan.action.to_string()),
                h(&plan.reason),
                h(&plan.summary)
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">復旧プランはまだありません。</p>".to_string());
    let recover_create_form = if snapshot.completion.next_action == "recover" {
        r#"<form id="recover-form">
  <button type="submit">復旧プランを作成</button>
  <p id="recover-status" class="muted" role="status"></p>
</form>"#
            .to_string()
    } else {
        "<p class=\"muted\">現在の次アクションは復旧ではありません。</p>".to_string()
    };
    let recover_accept_form = latest_draft_recovery
        .map(|plan| {
            format!(
                r#"<form id="recover-accept-form">
  <input type="hidden" name="recovery_plan" value="{}">
  <button type="submit">復旧プランを承認</button>
  <p id="recover-accept-status" class="muted" role="status"></p>
</form>"#,
                h(&plan.id)
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">承認待ちの復旧プランはありません。</p>".to_string());
    let recover_apply_form = latest_accepted_recovery
        .map(|plan| {
            format!(
                r#"<form id="recover-apply-form">
  <input type="hidden" name="recovery_plan" value="{}">
  <label>プロンプト<textarea name="prompt" rows="4" placeholder="復旧エージェントへの指示…">{}</textarea></label>
  <label>コマンド<textarea name="command" rows="2" placeholder="E2E/dev用コマンド…"></textarea></label>
  <button type="submit">復旧プランを適用</button>
  <p id="recover-apply-status" class="muted" role="status"></p>
</form>"#,
                h(&plan.id),
                h(plan.prompt_hint.as_deref().unwrap_or(""))
            )
        })
        .unwrap_or_else(|| "<p class=\"muted\">適用できる復旧プランはありません。</p>".to_string());
    let recovery_panel = format!(
        "{}{}{}{}",
        recovery_summary, recover_create_form, recover_accept_form, recover_apply_form
    );
    let mut action_sections = Vec::new();
    if snapshot.item.status == WorkItemStatus::NeedsInput && latest_question.is_some() {
        action_sections.push(format!(
            r#"<section class="panel primary-action"><h2>質問に回答</h2>{answer_form}</section>"#
        ));
    }
    if snapshot.completion.next_action == "review" {
        action_sections.push(
            r#"<section class="panel primary-action"><h2>レビュー</h2>
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
            r#"<section class="panel primary-action"><h2>承認</h2>{approve_form}</section>"#
        ));
    }
    if snapshot.completion.next_action == "recover" || latest_recovery.is_some() {
        action_sections.push(format!(
            r#"<section class="panel primary-action"><h2>復旧</h2>{recovery_panel}</section>"#
        ));
    }
    if let Some(running) = running_state.as_deref() {
        action_sections.insert(
            0,
            format!(
                r#"<section class="panel primary-action"><h2>処理中</h2><p>{}</p><p class="muted">ワークフローを続行できる間、このページは自動更新されます。</p></section>"#,
                h(&format!("{running} を実行中です。"))
            ),
        );
    }
    let action_sections = if action_sections.is_empty() {
        if matches!(snapshot.completion.next_action.as_str(), "none" | "done") {
            r#"<section class="panel primary-action"><h2>次の操作</h2><p class="muted">現在必要な操作はありません。</p></section>"#
                .to_string()
        } else {
            format!(
                r#"<section class="panel primary-action"><h2>次の操作</h2><p>{}</p><p class="muted">現在のステップが実行可能になると、Nagare が続行できます。</p></section>"#,
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
    let technical_panel = format!(
        r#"<details id="workflow" class="panel detail-disclosure technical-details">
  <summary><span>詳細ログ</span><small>実行IDや内部イベントを確認できます</small></summary>
  <div class="details-stack">{run_history_panel}{dispatch_panel}</div>
</details>"#
    );
    let summary_panel = render_summary_panel(
        &snapshot,
        &current_state,
        latest_dispatch,
        latest_question.as_deref(),
        running_state.as_deref(),
        &agent_profiles,
    );
    let progress_panel = render_progress_panel(
        &snapshot,
        latest_dispatch,
        running_state.as_deref(),
        &agent_profiles,
    );
    let candidate_panel =
        render_candidate_evaluation_panel(&snapshot, latest_dispatch, &agent_profiles);
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
          <span class="badge gray">次: {}</span>
        </div>
      </header>
      <div class="detail-layout">
        {}
        <section id="human-action" class="action-stack">{}</section>
        {}
        {}
        {}
        {}
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
        h(&work_item_status_label(&snapshot.item.status)),
        h(&next_action_label(
            &snapshot,
            latest_question.as_deref(),
            running_state.as_deref()
        )),
        summary_panel,
        action_sections,
        progress_panel,
        answer_panel,
        candidate_panel,
        technical_panel,
        serve_script()
    ))
}
