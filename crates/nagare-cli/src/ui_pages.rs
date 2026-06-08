use std::fs;
use std::path::Path;

use nagare_core::{
    ApprovalPolicy, I18n, UiTextKey, WorkItemStatus, WorkflowMode, WorkflowSettings,
    get_domain_group, get_domain_profile, get_locale_settings, get_work_item_snapshot,
    get_workflow_settings, list_agent_profiles, list_domain_groups, list_domain_profiles,
    list_skill_set_catalog, list_work_items,
};

use crate::ui::read_ui_running_state;
use crate::ui_answer::{answer_view, render_answer_preview};
use crate::ui_assets::{serve_responsive_stylesheet, serve_script, serve_stylesheet};
use crate::ui_html::h;

fn i18n_for_root(root: &Path) -> Result<I18n, String> {
    let locale = get_locale_settings(root).map_err(|error| error.to_string())?;
    Ok(I18n::new(locale.language))
}

pub(crate) fn render_serve_home(root: &Path) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let items = list_work_items(root).map_err(|error| error.to_string())?;
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let mut queue_signals = QueueSignals::default();
    let rows = if items.is_empty() {
        format!(
            "<tr><td colspan=\"8\" class=\"muted\">{}</td></tr>",
            h(i18n.ui(UiTextKey::NoWorkItemsYet))
        )
    } else {
        items
            .iter()
            .map(|item| {
                let snapshot = get_work_item_snapshot(root, &item.id).ok();
                let running = read_ui_running_state(root, &item.id);
                let next_action = snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.completion.next_action.as_str())
                    .unwrap_or("-");
                let (state_label, state_class, state_detail) = work_item_list_state(
                    item,
                    snapshot.as_ref(),
                    running.as_deref(),
                );
                queue_signals.observe(&state_label);
                let answer = snapshot.as_ref().map(answer_view);
                let filter_state = queue_filter_state(&state_label);
                format!(
                    r#"<tr class="{}" data-queue-state="{}"><td><a href="/items/{}">{}</a><div class="muted">{}</div></td><td>{}</td><td>{}</td><td><span class="badge {}">{}</span><div class="muted">{}</div></td><td>{}</td><td>{}</td><td><form class="delete-work-form" data-work-id="{}" data-work-title="{}"><button class="danger" type="submit">{}</button></form></td></tr>"#,
                    h(&format!("state-{}", state_label.to_ascii_lowercase().replace(' ', "-"))),
                    h(filter_state),
                    h(&item.id),
                    h(&item.id),
                    h(item.work_folder.as_deref().unwrap_or(".")),
                    h(&item.title),
                    render_answer_preview(answer.as_ref()),
                    state_class,
                    h(&localized_queue_state(&i18n, &state_label)),
                    h(&localized_queue_detail(&i18n, &state_detail)),
                    h(next_action),
                    h(&item.workflow_mode.to_string()),
                    h(&item.id),
                    h(&item.title),
                    h(i18n.ui(UiTextKey::Delete))
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Nagare UI Server</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a class="active" href="/">{}</a>
        <a href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link" href="/new">{}</a>
            <span class="badge blue">{} {}</span>
            <span class="badge gray">{} {}</span>
        </div>
      </header>
      <section class="queue-layout">
        <section class="panel queue-panel">
          <div class="panel-head">
            <h2>{}</h2>
            <span class="badge gray">{}</span>
          </div>
          <div class="status-strip">
            <button class="queue-chip active" type="button" data-filter-state="all">{} <b>{}</b></button>
            <button class="queue-chip attention" type="button" data-filter-state="attention">{} <b>{}</b></button>
            <button class="queue-chip failed" type="button" data-filter-state="failed">{} <b>{}</b></button>
            <button class="queue-chip approval" type="button" data-filter-state="approval">{} <b>{}</b></button>
            <button class="queue-chip running" type="button" data-filter-state="running">{} <b>{}</b></button>
          </div>
          <table><thead><tr><th>{}</th><th>{}</th><th>Answer</th><th>{}</th><th>{}</th><th>{}</th><th></th></tr></thead><tbody id="work-items">{}</tbody></table>
        </section>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        serve_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::WorkQueue),
        "作業の一覧、フィルタ、継続操作をまとめて確認します",
        i18n.ui(UiTextKey::CreateNewItem),
        i18n.ui(UiTextKey::Work),
        items.len(),
        i18n.ui(UiTextKey::Agents),
        agents.len(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::ManualContinuation),
        i18n.ui(UiTextKey::All),
        items.len(),
        i18n.ui(UiTextKey::NeedsAttention),
        queue_signals.attention,
        i18n.ui(UiTextKey::Failed),
        queue_signals.failed,
        i18n.ui(UiTextKey::Approval),
        queue_signals.approval,
        i18n.ui(UiTextKey::Running),
        queue_signals.running,
        i18n.ui(UiTextKey::IdFolder),
        i18n.ui(UiTextKey::Title),
        i18n.ui(UiTextKey::State),
        i18n.ui(UiTextKey::Next),
        i18n.ui(UiTextKey::Mode),
        rows,
        serve_script()
    ))
}

fn localized_queue_state(i18n: &I18n, label: &str) -> String {
    if i18n.language().is_ja() {
        match label {
            "Done" => "完了",
            "Running" => "処理中",
            "Needs input" => "入力待ち",
            "Needs approval" => "承認待ち",
            "Queued" => "待機中",
            "In review" => "レビュー中",
            "Needs handoff" => "引き継ぎ待ち",
            "Changes requested" => "変更要求",
            "Failed" => "失敗",
            other => other,
        }
        .to_string()
    } else {
        label.to_string()
    }
}

fn localized_queue_detail(i18n: &I18n, detail: &str) -> String {
    if i18n.language().is_ja() {
        match detail {
            "Completed" => "完了しました",
            "Waiting for your answer" => "回答待ちです",
            "Ready for final approval" => "最終承認待ちです",
            "Agent is running" => "エージェントが処理中です",
            "Waiting for background processing" => "バックグラウンド処理待ちです",
            "Work is ready for review or approval" => "レビューまたは承認待ちです",
            "Handoff decision is required" => "引き継ぎ判断が必要です",
            "Agent should address review feedback" => "レビュー指摘への対応待ちです",
            other if other.starts_with("Processing: ") => {
                return format!("処理中: {}", other.trim_start_matches("Processing: "));
            }
            other => other,
        }
        .to_string()
    } else {
        detail.to_string()
    }
}

fn queue_filter_state(label: &str) -> &'static str {
    match label {
        "Failed" => "failed attention",
        "Needs approval" => "approval attention",
        "Running" => "running",
        "Needs input" | "Needs handoff" | "Changes requested" => "attention",
        _ => "normal",
    }
}

#[derive(Default)]
struct QueueSignals {
    attention: usize,
    failed: usize,
    approval: usize,
    running: usize,
}

impl QueueSignals {
    fn observe(&mut self, label: &str) {
        match label {
            "Running" => self.running += 1,
            "Failed" => {
                self.failed += 1;
                self.attention += 1;
            }
            "Needs approval" => {
                self.approval += 1;
                self.attention += 1;
            }
            "Needs input" | "Needs handoff" | "Changes requested" => {
                self.attention += 1;
            }
            _ => {}
        }
    }
}

fn work_item_list_state(
    item: &nagare_core::WorkItem,
    snapshot: Option<&nagare_core::WorkItemSnapshot>,
    running: Option<&str>,
) -> (String, &'static str, String) {
    if item.status == WorkItemStatus::Done {
        return ("Done".to_string(), "green", "Completed".to_string());
    }
    if let Some(running) = running {
        return (
            "Running".to_string(),
            "blue",
            format!("Processing: {running}"),
        );
    }
    if item.status == WorkItemStatus::NeedsInput {
        return (
            "Needs input".to_string(),
            "amber",
            "Waiting for your answer".to_string(),
        );
    }
    if let Some(snapshot) = snapshot {
        if snapshot.approval_gate.ready {
            return (
                "Needs approval".to_string(),
                "amber",
                "Ready for final approval".to_string(),
            );
        }
        match snapshot.completion.next_action.as_str() {
            "answer_question" => {
                return (
                    "Needs input".to_string(),
                    "amber",
                    "Waiting for your answer".to_string(),
                );
            }
            "approve" => {
                return (
                    "Needs approval".to_string(),
                    "amber",
                    "Ready for final approval".to_string(),
                );
            }
            "none" => {}
            _ => {}
        }
    }
    match item.status {
        WorkItemStatus::Done => ("Done".to_string(), "green", "Completed".to_string()),
        WorkItemStatus::AgentRunning => (
            "Running".to_string(),
            "blue",
            "Agent is running".to_string(),
        ),
        WorkItemStatus::Ready => (
            "Queued".to_string(),
            "gray",
            "Waiting for background processing".to_string(),
        ),
        WorkItemStatus::ReadyForReview => (
            "In review".to_string(),
            "blue",
            "Work is ready for review or approval".to_string(),
        ),
        WorkItemStatus::NeedsHandoff => (
            "Needs handoff".to_string(),
            "amber",
            "Handoff decision is required".to_string(),
        ),
        WorkItemStatus::ChangesRequested => (
            "Changes requested".to_string(),
            "amber",
            "Agent should address review feedback".to_string(),
        ),
        WorkItemStatus::NeedsInput => (
            "Needs input".to_string(),
            "amber",
            "Waiting for your answer".to_string(),
        ),
    }
}

pub(crate) fn render_serve_new_item(root: &Path) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let items = list_work_items(root).map_err(|error| error.to_string())?;
    let domain_groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let domain_group_options =
        domain_group_select_options(&domain_groups, None, i18n.ui(UiTextKey::ProjectDefault));
    let domain_options = domain_select_options(&domains, None, i18n.ui(UiTextKey::ProjectDefault));
    let workflow_options =
        workflow_mode_options(Some(settings.default_progress_mode), false, &i18n);
    let approval_options = approval_policy_options(Some(settings.approval_policy), false, &i18n);
    let domain_agent_policy_options = domain_agent_policy_options(&i18n);
    Ok(format!(
        r#"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/">{}</a>
          <span class="badge blue">{} {}</span>
        </div>
      </header>
      <section class="composer">
        <h2>{}</h2>
        <form id="create-work-form">
          <label>{}<textarea name="description" rows="4" required placeholder="エージェントへの依頼内容"></textarea></label>
          <label>{}<input name="work_folder" placeholder="crates/nagare-core"></label>
          <label>{}<textarea name="acceptance" rows="3" placeholder="1行に1条件"></textarea></label>
          <details class="advanced-form" open>
            <summary>{}</summary>
            <label>{}<textarea name="artifacts" rows="2" placeholder="README, tests, screenshots"></textarea></label>
            <label>{}<textarea name="constraints" rows="2" placeholder="破壊的操作を避ける、既存APIを維持する"></textarea></label>
            <label>{}<select name="domain_group_id">{}</select></label>
            <label>{}<select name="domain_id">{}</select></label>
            <label>{}<select name="domain_agent_policy">{}</select></label>
            <label>{}<select name="workflow_mode">{}</select></label>
            <label>{}<select name="approval_policy">{}</select></label>
          </details>
          <input type="hidden" name="max_steps" value="8">
          <input type="hidden" name="command" value="">
          <input type="hidden" name="review_command" value="">
          <button type="submit">{}</button>
          <p id="form-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"#,
        i18n.ui(UiTextKey::CreateNewItem),
        serve_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::CreateNewItem),
        i18n.ui(UiTextKey::AddItemLead),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Work),
        items.len(),
        i18n.ui(UiTextKey::CreateNewItem),
        i18n.ui(UiTextKey::Prompt),
        i18n.ui(UiTextKey::WorkFolder),
        i18n.ui(UiTextKey::AcceptanceCriteria),
        i18n.ui(UiTextKey::MoreContext),
        i18n.ui(UiTextKey::ExpectedArtifacts),
        i18n.ui(UiTextKey::Constraints),
        i18n.ui(UiTextKey::DomainGroup),
        domain_group_options,
        i18n.ui(UiTextKey::Domain),
        domain_options,
        i18n.ui(UiTextKey::DomainAgentPolicy),
        domain_agent_policy_options,
        i18n.ui(UiTextKey::ProgressMode),
        workflow_options,
        i18n.ui(UiTextKey::FinalApproval),
        approval_options,
        i18n.ui(UiTextKey::CreateNewItem),
        serve_script()
    ))
}

pub(crate) fn render_serve_settings(root: &Path) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let domain_groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let workflow_settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let agent_rows = agent_profile_rows(&agents, &domain_groups, &domains, &i18n);
    let group_rows = domain_group_rows(&domain_groups, &i18n);
    let domain_rows = domain_profile_rows(&domains, &domain_groups, &i18n);
    let workflow_form = render_workflow_settings_form(workflow_settings, &i18n);
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a class="active" href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link" href="/settings/agents/new">{}</a>
          <span class="badge gray">{} {}</span>
          <span class="badge gray">{} {}</span>
          <span class="badge gray">{} {}</span>
          <span class="badge blue">{}</span>
        </div>
      </header>
      <div class="settings-tabs" role="tablist" aria-label="{}">
        <button id="settings-tab-workflow" class="settings-tab active" type="button" role="tab" aria-selected="true" aria-controls="settings-panel-workflow" data-settings-tab="workflow">{}</button>
        <button id="settings-tab-domains" class="settings-tab" type="button" role="tab" aria-selected="false" aria-controls="settings-panel-domains" data-settings-tab="domains">{}</button>
        <button id="settings-tab-agents" class="settings-tab" type="button" role="tab" aria-selected="false" aria-controls="settings-panel-agents" data-settings-tab="agents">{}</button>
      </div>
      <section id="settings-panel-workflow" class="settings-panel" role="tabpanel" aria-labelledby="settings-tab-workflow" data-settings-panel="workflow">
        {}
      </section>
      <section id="settings-panel-domains" class="settings-panel" role="tabpanel" aria-labelledby="settings-tab-domains" data-settings-panel="domains" hidden>
        <section class="panel">
          <div class="panel-head">
            <div>
              <h2>{}</h2>
              <p class="muted">{}</p>
            </div>
            <a class="button-link secondary" href="/settings/domain-groups/new">{}</a>
          </div>
          <table class="domain-table"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody id="domain-groups">{}</tbody></table>
        </section>
        <section class="panel">
          <div class="panel-head">
            <div>
              <h2>{}</h2>
              <p class="muted">{}</p>
            </div>
            <a class="button-link secondary" href="/settings/domains/new">{}</a>
          </div>
          <table class="domain-table"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody id="domain-profiles">{}</tbody></table>
        </section>
      </section>
      <section id="settings-panel-agents" class="panel settings-panel" role="tabpanel" aria-labelledby="settings-tab-agents" data-settings-panel="agents" hidden>
        <div class="panel-head">
          <h2>{}</h2>
        </div>
        {}
        <table class="agent-table"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody id="agent-profiles">{}</tbody></table>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        i18n.ui(UiTextKey::Settings),
        serve_stylesheet(),
        serve_responsive_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::SettingsLead),
        i18n.ui(UiTextKey::CreateNewAgent),
        i18n.ui(UiTextKey::Agents),
        agents.len(),
        i18n.ui(UiTextKey::Groups),
        domain_groups.len(),
        i18n.ui(UiTextKey::Domains),
        domains.len(),
        i18n.ui(UiTextKey::Profiles),
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::Agents),
        workflow_form,
        i18n.ui(UiTextKey::DomainGroups),
        i18n.ui(UiTextKey::DomainGroupsLead),
        i18n.ui(UiTextKey::CreateNewDomainGroup),
        i18n.ui(UiTextKey::Group),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::SharedKnowledge),
        i18n.ui(UiTextKey::Rubric),
        i18n.ui(UiTextKey::DispatchHints),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Source),
        i18n.ui(UiTextKey::Actions),
        group_rows,
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::DomainsLead),
        i18n.ui(UiTextKey::CreateNewDomain),
        i18n.ui(UiTextKey::Domain),
        i18n.ui(UiTextKey::Group),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::Rubric),
        i18n.ui(UiTextKey::DispatchHints),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Source),
        i18n.ui(UiTextKey::Actions),
        domain_rows,
        i18n.ui(UiTextKey::Agents),
        agent_filters(&domain_groups, &domains, &i18n),
        i18n.ui(UiTextKey::Agent),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::Groups),
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::Actions),
        agent_rows,
        serve_script()
    ))
}

fn domain_group_rows(groups: &[nagare_core::DomainGroup], i18n: &I18n) -> String {
    if groups.is_empty() {
        return format!(
            "<tr><td colspan=\"5\" class=\"muted\">{}.</td></tr>",
            h(i18n.ui(UiTextKey::DomainGroups))
        );
    }
    let mut sorted_groups = groups.iter().collect::<Vec<_>>();
    sorted_groups.sort_by_key(|group| group.display_name.as_str());
    sorted_groups
        .into_iter()
        .map(|group| {
            format!(
            r#"<tr>
  <td data-label="{}"><a href="/settings/domain-groups/{}">{}</a><div class="muted">{}</div></td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/domain-groups/{}">{}</a><form class="delete-domain-group-form" data-domain-group-id="{}" data-domain-group-name="{}"><button class="danger" type="submit">{}</button></form></div></td>
</tr>"#,
                h(i18n.ui(UiTextKey::Group)),
                h(&group.id),
                h(&group.display_name),
                h(&group.id),
                h(i18n.ui(UiTextKey::Description)),
                h(&compact_instruction(&group.description)),
                h(i18n.ui(UiTextKey::SharedKnowledge)),
                h(&group.shared_knowledge.len().to_string()),
                h(i18n.ui(UiTextKey::Rubric)),
                h(&group.common_rubric.len().to_string()),
                h(i18n.ui(UiTextKey::DispatchHints)),
                h(&group.dispatch_hints.len().to_string()),
                h(i18n.ui(UiTextKey::Workflow)),
                h(&domain_group_workflow_label(group, i18n)),
                h(i18n.ui(UiTextKey::Source)),
                h(&source_label(&group.source.to_string(), i18n)),
                h(i18n.ui(UiTextKey::Actions)),
                h(&group.id),
                h(i18n.ui(UiTextKey::Edit)),
                h(&group.id),
                h(&group.display_name),
                h(i18n.ui(UiTextKey::Delete))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn domain_profile_rows(
    domains: &[nagare_core::DomainProfile],
    groups: &[nagare_core::DomainGroup],
    i18n: &I18n,
) -> String {
    if domains.is_empty() {
        return format!(
            "<tr><td colspan=\"8\" class=\"muted\">{}.</td></tr>",
            h(i18n.ui(UiTextKey::Domains))
        );
    }
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    domains
        .into_iter()
        .map(|domain| {
            format!(
            r#"<tr>
  <td data-label="{}"><a href="/settings/domains/{}">{}</a><div class="muted">{}</div></td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/domains/{}">{}</a><form class="delete-domain-form" data-domain-id="{}" data-domain-name="{}"><button class="danger" type="submit">{}</button></form></div></td>
</tr>"#,
                h(i18n.ui(UiTextKey::Domain)),
                h(&domain.id),
                h(&domain.display_name),
                h(&domain.id),
                h(i18n.ui(UiTextKey::Group)),
                h(&domain_group_label(groups, domain.group_id.as_deref())),
                h(i18n.ui(UiTextKey::Description)),
                h(&compact_instruction(&domain.description)),
                h(i18n.ui(UiTextKey::Rubric)),
                h(&domain.rubric.len().to_string()),
                h(i18n.ui(UiTextKey::DispatchHints)),
                h(&domain.dispatch_hints.len().to_string()),
                h(i18n.ui(UiTextKey::Workflow)),
                h(&domain_workflow_label(domain, i18n)),
                h(i18n.ui(UiTextKey::Source)),
                h(&source_label(&domain.source.to_string(), i18n)),
                h(i18n.ui(UiTextKey::Actions)),
                h(&domain.id),
                h(i18n.ui(UiTextKey::Edit)),
                h(&domain.id),
                h(&domain.display_name),
                h(i18n.ui(UiTextKey::Delete))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn domain_group_label(groups: &[nagare_core::DomainGroup], id: Option<&str>) -> String {
    let Some(id) = id else {
        return "-".to_string();
    };
    groups
        .iter()
        .find(|group| group.id == id)
        .map(|group| group.display_name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn source_label(source: &str, i18n: &I18n) -> String {
    if i18n.language().is_ja() {
        match source {
            "project_domain_group_directory" => "プロジェクトのドメイングループ定義",
            "project_domain_directory" => "プロジェクトのドメイン定義",
            "project_config" => "プロジェクト設定",
            "default_config" => "既定設定",
            other => other,
        }
        .to_string()
    } else {
        source.to_string()
    }
}

fn render_workflow_settings_form(settings: WorkflowSettings, i18n: &I18n) -> String {
    format!(
        r#"<section class="panel">
        <div class="panel-head">
          <h2>{}</h2>
          <span class="badge gray">{}</span>
        </div>
        <form id="workflow-settings-form" data-action="/api/workflow-settings">
          <div class="form-grid">
            <label>{}<select name="default_progress_mode">{}</select></label>
            <label>{}<select name="approval_policy">{}</select></label>
          </div>
          <button type="submit">{}</button>
          <p id="workflow-settings-status" class="muted" role="status"></p>
        </form>
      </section>"#,
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::ProjectDefault),
        i18n.ui(UiTextKey::ProgressMode),
        workflow_mode_options(Some(settings.default_progress_mode), false, i18n),
        i18n.ui(UiTextKey::FinalApproval),
        approval_policy_options(Some(settings.approval_policy), false, i18n),
        i18n.ui(UiTextKey::SaveWorkflowSettings)
    )
}

fn domain_select_options(
    domains: &[nagare_core::DomainProfile],
    selected: Option<&str>,
    empty_label: &str,
) -> String {
    let mut options = vec![format!(r#"<option value="">{}</option>"#, h(empty_label))];
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    options.extend(domains.into_iter().map(|domain| {
        let selected_attr = if selected == Some(domain.id.as_str()) {
            " selected"
        } else {
            ""
        };
        format!(
            r#"<option value="{}"{}>{}</option>"#,
            h(&domain.id),
            selected_attr,
            h(&domain.display_name)
        )
    }));
    options.join("")
}

fn domain_group_select_options(
    groups: &[nagare_core::DomainGroup],
    selected: Option<&str>,
    empty_label: &str,
) -> String {
    let mut options = vec![format!(r#"<option value="">{}</option>"#, h(empty_label))];
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by_key(|group| group.display_name.as_str());
    options.extend(groups.into_iter().map(|group| {
        let selected_attr = if selected == Some(group.id.as_str()) {
            " selected"
        } else {
            ""
        };
        format!(
            r#"<option value="{}"{}>{}</option>"#,
            h(&group.id),
            selected_attr,
            h(&group.display_name)
        )
    }));
    options.join("")
}

fn domain_group_multi_options(groups: &[nagare_core::DomainGroup], selected: &[String]) -> String {
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by_key(|group| group.display_name.as_str());
    groups
        .into_iter()
        .map(|group| {
            let selected_attr = if selected.iter().any(|id| id == &group.id) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                h(&group.id),
                selected_attr,
                h(&group.display_name)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn domain_multi_options(domains: &[nagare_core::DomainProfile], selected: &[String]) -> String {
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    domains
        .into_iter()
        .map(|domain| {
            let selected_attr = if selected.iter().any(|id| id == &domain.id) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                h(&domain.id),
                selected_attr,
                h(&domain.display_name)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn workflow_mode_options(
    selected: Option<WorkflowMode>,
    include_inherit: bool,
    i18n: &I18n,
) -> String {
    let mut options = Vec::new();
    if include_inherit {
        options.push(format!(
            r#"<option value="">{}</option>"#,
            h(i18n.ui(UiTextKey::InheritProjectDefault))
        ));
    }
    for mode in [WorkflowMode::ConfirmFirst, WorkflowMode::FinishFirst] {
        let selected_attr = if selected == Some(mode) {
            " selected"
        } else {
            ""
        };
        options.push(format!(
            r#"<option value="{}"{}>{}</option>"#,
            h(&mode.to_string()),
            selected_attr,
            h(&workflow_mode_label(mode, i18n))
        ));
    }
    options.join("")
}

fn approval_policy_options(
    selected: Option<ApprovalPolicy>,
    include_inherit: bool,
    i18n: &I18n,
) -> String {
    let mut options = Vec::new();
    if include_inherit {
        options.push(format!(
            r#"<option value="">{}</option>"#,
            h(i18n.ui(UiTextKey::InheritProjectDefault))
        ));
    }
    for policy in [
        ApprovalPolicy::ManualFinalApproval,
        ApprovalPolicy::AutoCompleteOnReviewPass,
    ] {
        let selected_attr = if selected == Some(policy) {
            " selected"
        } else {
            ""
        };
        options.push(format!(
            r#"<option value="{}"{}>{}</option>"#,
            h(&policy.to_string()),
            selected_attr,
            h(&approval_policy_label(policy, i18n))
        ));
    }
    options.join("")
}

fn workflow_mode_label(mode: WorkflowMode, i18n: &I18n) -> String {
    if i18n.language().is_ja() {
        match mode {
            WorkflowMode::ConfirmFirst => "確認してから進める",
            WorkflowMode::FinishFirst => "完了まで進める",
        }
        .to_string()
    } else {
        mode.to_string()
    }
}

fn approval_policy_label(policy: ApprovalPolicy, i18n: &I18n) -> String {
    if i18n.language().is_ja() {
        match policy {
            ApprovalPolicy::ManualFinalApproval => "最終承認を手動で行う",
            ApprovalPolicy::AutoCompleteOnReviewPass => "レビュー通過で自動完了",
        }
        .to_string()
    } else {
        policy.to_string()
    }
}

fn domain_agent_policy_options(i18n: &I18n) -> String {
    let labels = if i18n.language().is_ja() {
        [
            ("auto_general_fallback", "専門Agentがなければ汎用で自動実行"),
            (
                "confirm_general_fallback",
                "専門Agentがなければ確認して汎用で実行",
            ),
            ("require_domain_agent", "専門Agentを必須にする"),
        ]
    } else {
        [
            ("auto_general_fallback", "Auto general fallback"),
            ("confirm_general_fallback", "Confirm general fallback"),
            ("require_domain_agent", "Require domain agent"),
        ]
    };
    labels
        .into_iter()
        .map(|(value, label)| format!(r#"<option value="{}">{}</option>"#, h(value), h(label)))
        .collect::<Vec<_>>()
        .join("")
}

fn agent_filters(
    groups: &[nagare_core::DomainGroup],
    domains: &[nagare_core::DomainProfile],
    i18n: &I18n,
) -> String {
    let group_filters = agent_domain_group_filter_options(groups, i18n);
    let domain_filters = agent_domain_filter_options(domains, i18n);
    format!(
        r#"<div class="filter-panel" data-agent-filters>
          <div>
            <h3>{}</h3>
            <div class="checkbox-grid">{}</div>
          </div>
          <div>
            <h3>{}</h3>
            <div class="checkbox-grid">{}</div>
          </div>
          <div class="filter-actions">
            <button class="secondary-button" type="button" data-clear-agent-filters>{}</button>
            <span class="muted" data-agent-filter-count></span>
          </div>
        </div>"#,
        i18n.ui(UiTextKey::DomainGroups),
        group_filters,
        i18n.ui(UiTextKey::Domains),
        domain_filters,
        i18n.ui(UiTextKey::ClearFilters)
    )
}

fn agent_domain_group_filter_options(groups: &[nagare_core::DomainGroup], i18n: &I18n) -> String {
    if groups.is_empty() {
        return format!(
            r#"<span class="muted">{}.</span>"#,
            h(i18n.ui(UiTextKey::DomainGroups))
        );
    }
    let mut groups = groups.iter().collect::<Vec<_>>();
    groups.sort_by_key(|group| group.display_name.as_str());
    groups
        .into_iter()
        .map(|group| {
            format!(
                r#"<label class="check-option"><input type="checkbox" data-agent-filter-group value="{}"><span>{}</span></label>"#,
                h(&group.id),
                h(&group.display_name)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn agent_domain_filter_options(domains: &[nagare_core::DomainProfile], i18n: &I18n) -> String {
    if domains.is_empty() {
        return format!(
            r#"<span class="muted">{}.</span>"#,
            h(i18n.ui(UiTextKey::Domains))
        );
    }
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    domains
        .into_iter()
        .map(|domain| {
            format!(
                r#"<label class="check-option"><input type="checkbox" data-agent-filter-domain value="{}"><span>{}</span></label>"#,
                h(&domain.id),
                h(&domain.display_name)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn skill_set_picker(
    skill_sets: &[nagare_core::SkillSetCatalogEntry],
    selected: &[String],
    i18n: &I18n,
) -> String {
    if skill_sets.is_empty() {
        return format!(
            r#"<div class="skill-picker empty"><p class="muted">{}</p></div>"#,
            h(localized(
                i18n,
                "このプロジェクトにはインストール済みスキルがありません。",
                "This project has no installed skills."
            ))
        );
    }
    let mut skill_sets = skill_sets.iter().collect::<Vec<_>>();
    skill_sets.sort_by_key(|skill| skill.id.as_str());
    let options = skill_sets
        .iter()
        .map(|skill| skill_set_picker_option(skill, selected, i18n))
        .collect::<Vec<_>>()
        .join("");
    let selected_summary = selected_skill_set_summary(&skill_sets, selected, i18n);
    format!(
        r#"<div class="skill-picker" data-skill-picker data-empty-label="{}">
  <label class="skill-search">{}<input type="search" data-skill-search autocomplete="off" placeholder="{}"></label>
  <div class="skill-selected" data-skill-selected aria-live="polite">{}</div>
  <div class="skill-picker-list">{}</div>
</div>"#,
        h(localized(i18n, "スキル未選択", "No skills selected")),
        h(localized(i18n, "スキルを検索", "Search Skills")),
        h(localized(
            i18n,
            "skill id / capability…",
            "skill id / capability…"
        )),
        selected_summary,
        options
    )
}

fn skill_set_picker_option(
    skill: &nagare_core::SkillSetCatalogEntry,
    selected: &[String],
    i18n: &I18n,
) -> String {
    let checked_attr = if selected.iter().any(|id| id == &skill.id) {
        " checked"
    } else {
        ""
    };
    let details = skill_set_option_details(skill, i18n);
    let search_text = skill_set_search_text(skill);
    format!(
        r#"<label class="skill-option" data-skill-option data-skill-search-text="{}">
  <input type="checkbox" name="skill_set_ids" value="{}"{}>
  <span class="skill-option-body">
    <span class="skill-option-title"><span translate="no">{}</span><span class="badge gray">{}</span></span>
    <span class="skill-option-details">{}</span>
  </span>
</label>"#,
        h(&search_text),
        h(&skill.id),
        checked_attr,
        h(&skill.id),
        h(localized(i18n, "スキルセット", "Skill Set")),
        h(&details)
    )
}

fn selected_skill_set_summary(
    skill_sets: &[&nagare_core::SkillSetCatalogEntry],
    selected: &[String],
    i18n: &I18n,
) -> String {
    let selected = skill_sets
        .iter()
        .filter(|skill| selected.iter().any(|id| id == &skill.id))
        .map(|skill| {
            format!(
                r#"<span class="skill-chip" translate="no">{}</span>"#,
                h(&skill.id)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    if selected.is_empty() {
        format!(
            r#"<span class="muted">{}</span>"#,
            h(localized(i18n, "スキル未選択", "No skills selected"))
        )
    } else {
        selected
    }
}

fn skill_set_option_details(skill: &nagare_core::SkillSetCatalogEntry, i18n: &I18n) -> String {
    let mut details = Vec::new();
    if !skill.paths.is_empty() {
        details.push(format!(
            "{}: {}",
            localized(i18n, "対象", "Paths"),
            skill.paths.join(", ")
        ));
    }
    if !skill.required_capabilities.is_empty() {
        details.push(format!(
            "{}: {}",
            localized(i18n, "必須能力", "Requires"),
            skill.required_capabilities.join(", ")
        ));
    }
    if !skill.optional_capabilities.is_empty() {
        details.push(format!(
            "{}: {}",
            localized(i18n, "追加能力", "Optional"),
            skill.optional_capabilities.join(", ")
        ));
    }
    if details.is_empty() {
        localized(i18n, "追加情報なし", "No additional details").to_string()
    } else {
        details.join(" / ")
    }
}

fn skill_set_search_text(skill: &nagare_core::SkillSetCatalogEntry) -> String {
    let mut text = vec![skill.id.clone()];
    text.extend(skill.paths.iter().cloned());
    text.extend(skill.required_capabilities.iter().cloned());
    text.extend(skill.optional_capabilities.iter().cloned());
    text.join(" ").to_lowercase()
}

fn role_options(selected: &str) -> String {
    let known_roles = [
        "planner",
        "worker",
        "reviewer",
        "dispatcher",
        "supervisor",
        "implementer",
    ];
    let mut options = known_roles
        .iter()
        .map(|role| {
            let selected_attr = if selected == *role { " selected" } else { "" };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                h(role),
                selected_attr,
                h(role)
            )
        })
        .collect::<Vec<_>>();
    if !selected.trim().is_empty() && !known_roles.contains(&selected) {
        options.push(format!(
            r#"<option value="{}" selected>{}</option>"#,
            h(selected),
            h(selected)
        ));
    }
    options.join("")
}

fn domain_workflow_label(domain: &nagare_core::DomainProfile, i18n: &I18n) -> String {
    let mode = domain
        .workflow
        .progress_mode
        .map(|mode| workflow_mode_label(mode, i18n))
        .unwrap_or_else(|| i18n.ui(UiTextKey::InheritProjectDefault).to_string());
    let approval = domain
        .workflow
        .approval_policy
        .map(|policy| approval_policy_label(policy, i18n))
        .unwrap_or_else(|| i18n.ui(UiTextKey::InheritProjectDefault).to_string());
    format!("{mode} / {approval}")
}

fn domain_group_workflow_label(group: &nagare_core::DomainGroup, i18n: &I18n) -> String {
    let mode = group
        .workflow
        .progress_mode
        .map(|mode| workflow_mode_label(mode, i18n))
        .unwrap_or_else(|| i18n.ui(UiTextKey::InheritProjectDefault).to_string());
    let approval = group
        .workflow
        .approval_policy
        .map(|policy| approval_policy_label(policy, i18n))
        .unwrap_or_else(|| i18n.ui(UiTextKey::InheritProjectDefault).to_string());
    format!("{mode} / {approval}")
}

fn agent_profile_rows(
    agents: &[nagare_core::AgentProfile],
    groups: &[nagare_core::DomainGroup],
    domains: &[nagare_core::DomainProfile],
    i18n: &I18n,
) -> String {
    if agents.is_empty() {
        return format!(
            "<tr><td colspan=\"5\" class=\"muted\">{}.</td></tr>",
            h(i18n.ui(UiTextKey::Agents))
        );
    }
    let mut agents = agents.iter().collect::<Vec<_>>();
    agents.sort_by_key(|agent| {
        (
            agent_profile_sort_key(&agent.id),
            agent.display_name.as_str(),
        )
    });
    agents
        .into_iter()
        .map(|agent| {
            let group_ids = agent.domain_group_ids.join(" ");
            let domain_ids = agent.domain_ids.join(" ");
            format!(
                r#"<tr data-agent-row data-agent-domain-groups="{}" data-agent-domains="{}">
  <td data-label="{}">
    <a href="/settings/agents/{}">{}</a>
    <div class="muted" translate="no">{}</div>
  </td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/agents/{}">{}</a></div></td>
</tr>"#,
                h(&group_ids),
                h(&domain_ids),
                h(i18n.ui(UiTextKey::Agent)),
                h(&agent.id),
                h(&agent.display_name),
                h(&agent.id),
                h(i18n.ui(UiTextKey::Description)),
                h(&compact_instruction(&agent.description)),
                h(i18n.ui(UiTextKey::Groups)),
                h(&agent_domain_group_label(agent, groups, i18n)),
                h(i18n.ui(UiTextKey::Domains)),
                h(&agent_domain_label(agent, domains, i18n)),
                h(i18n.ui(UiTextKey::Actions)),
                h(&agent.id),
                h(i18n.ui(UiTextKey::Edit))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn localized(i18n: &I18n, ja: &'static str, en: &'static str) -> &'static str {
    if i18n.language().is_ja() { ja } else { en }
}

fn agent_domain_group_label(
    agent: &nagare_core::AgentProfile,
    groups: &[nagare_core::DomainGroup],
    i18n: &I18n,
) -> String {
    if agent.domain_group_ids.is_empty() {
        return any_scope_label(i18n);
    }
    agent
        .domain_group_ids
        .iter()
        .map(|id| domain_group_label(groups, Some(id)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn agent_domain_label(
    agent: &nagare_core::AgentProfile,
    domains: &[nagare_core::DomainProfile],
    i18n: &I18n,
) -> String {
    if agent.domain_ids.is_empty() {
        return any_scope_label(i18n);
    }
    agent
        .domain_ids
        .iter()
        .map(|id| {
            domains
                .iter()
                .find(|domain| &domain.id == id)
                .map(|domain| domain.display_name.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn any_scope_label(i18n: &I18n) -> String {
    match i18n.language() {
        nagare_core::NagareLanguage::Ja => "任意".to_string(),
        nagare_core::NagareLanguage::En => "any".to_string(),
    }
}

fn agent_profile_sort_key(id: &str) -> u8 {
    match id {
        "worker" => 0,
        "reviewer" => 1,
        "dispatcher" => 2,
        "supervisor" => 3,
        _ => 4,
    }
}

fn agent_kind_value(runtime: &str, adapter: &str) -> &'static str {
    match (runtime, adapter) {
        ("codex-app-local", "stdio.codex-app-server")
        | ("codex-app-server", "stdio.codex-app-server") => "codex_app_server",
        ("openclaw-local", "process.openclaw-agent") => "openclaw",
        _ => "codex_cli",
    }
}

fn compact_instruction(instruction: &str) -> String {
    let instruction = instruction.split_whitespace().collect::<Vec<_>>().join(" ");
    if instruction.chars().count() <= 96 {
        return instruction;
    }
    let mut compact = instruction.chars().take(96).collect::<String>();
    compact.push('…');
    compact
}

fn workdir_select_options(root: &Path, selected: &str) -> String {
    let mut dirs = vec![".".to_string()];
    collect_workdir_options(root, Path::new(""), 0, &mut dirs);
    dirs.sort();
    dirs.dedup();
    if !dirs.iter().any(|dir| dir == selected) {
        dirs.push(selected.to_string());
    }
    dirs.into_iter()
        .map(|dir| {
            let selected_attr = if dir == selected { " selected" } else { "" };
            format!(
                r#"<option value="{}"{}>{}</option>"#,
                h(&dir),
                selected_attr,
                h(&dir)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn collect_workdir_options(root: &Path, relative: &Path, depth: usize, dirs: &mut Vec<String>) {
    if depth >= 2 {
        return;
    }
    let base = root.join(relative);
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if matches!(
            name.as_str(),
            ".git" | ".nagare" | "target" | "node_modules"
        ) {
            continue;
        }
        let child = relative.join(&name);
        if let Some(value) = child.to_str() {
            dirs.push(value.replace('\\', "/"));
        }
        collect_workdir_options(root, &child, depth + 1, dirs);
    }
}

pub(crate) fn render_serve_domain_form(
    root: &Path,
    domain_id: Option<&str>,
) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domain = match domain_id {
        Some(id) => Some(get_domain_profile(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = domain.is_none();
    let domain = domain.as_ref();
    let title = if is_new {
        i18n.ui(UiTextKey::CreateNewDomain).to_string()
    } else {
        format!(
            "{}: {}",
            i18n.ui(UiTextKey::Edit),
            domain
                .map(|domain| domain.display_name.as_str())
                .unwrap_or("")
        )
    };
    let id_value = domain.map(|domain| domain.id.as_str()).unwrap_or("");
    let display_name = domain
        .map(|domain| domain.display_name.as_str())
        .unwrap_or("");
    let description = domain
        .map(|domain| domain.description.as_str())
        .unwrap_or("");
    let group_options = domain_group_select_options(
        &groups,
        domain.and_then(|domain| domain.group_id.as_deref()),
        i18n.ui(UiTextKey::NoGroup),
    );
    let artifact_types = domain
        .map(|domain| domain.artifact_types.join("\n"))
        .unwrap_or_default();
    let rubric = domain
        .map(|domain| domain.rubric.join("\n"))
        .unwrap_or_default();
    let dispatch_hints = domain
        .map(|domain| domain.dispatch_hints.join("\n"))
        .unwrap_or_default();
    let progress_mode = domain.and_then(|domain| domain.workflow.progress_mode);
    let approval_policy = domain.and_then(|domain| domain.workflow.approval_policy);
    let action = if is_new {
        "/api/domains".to_string()
    } else {
        format!("/api/domains/{}", h(id_value))
    };
    let id_field = if is_new {
        format!(
            r#"<label>ID<input name="id" required placeholder="frontend-ui" value="{}"></label>"#,
            h(id_value)
        )
    } else {
        format!(
            r#"<label>ID<input name="id" readonly value="{}"></label>"#,
            h(id_value)
        )
    };
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a class="active" href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">{}</a>
        </div>
      </header>
      <section class="composer">
        <form id="domain-profile-form" data-action="{}" data-redirect="/settings#domains">
          {}
          <label>{}<select name="group_id">{}</select></label>
          <label>{}<input name="display_name" required value="{}"></label>
          <label>{}<textarea name="description" rows="4" placeholder="このドメインが扱う作成物や判断対象">{}</textarea></label>
          <label>{}<textarea name="artifact_types" rows="3" placeholder="1行に1種類。例: html, ui screenshot, rust cli">{}</textarea></label>
          <label>{}<textarea name="rubric" rows="7" placeholder="1行に1基準。例: 主要導線が迷わず使える">{}</textarea></label>
          <label>{}<textarea name="dispatch_hints" rows="4" placeholder="1行に1ヒント。例: UI変更ならfrontend-ui domainを候補にする">{}</textarea></label>
          <div class="form-grid">
            <label>{}<select name="workflow_progress_mode">{}</select></label>
            <label>{}<select name="workflow_approval_policy">{}</select></label>
          </div>
          <button type="submit">{}</button>
          <p id="domain-profile-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        h(&title),
        serve_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        h(&title),
        i18n.ui(UiTextKey::DomainFormLead),
        i18n.ui(UiTextKey::Settings),
        h(&action),
        id_field,
        i18n.ui(UiTextKey::DomainGroup),
        group_options,
        i18n.ui(UiTextKey::Name),
        h(display_name),
        i18n.ui(UiTextKey::Description),
        h(description),
        i18n.ui(UiTextKey::ArtifactTypes),
        h(&artifact_types),
        i18n.ui(UiTextKey::Rubric),
        h(&rubric),
        i18n.ui(UiTextKey::DispatchHints),
        h(&dispatch_hints),
        i18n.ui(UiTextKey::ProgressModeOverride),
        workflow_mode_options(progress_mode, true, &i18n),
        i18n.ui(UiTextKey::FinalApprovalOverride),
        approval_policy_options(approval_policy, true, &i18n),
        if is_new {
            i18n.ui(UiTextKey::CreateDomain)
        } else {
            i18n.ui(UiTextKey::SaveDomain)
        },
        serve_script()
    ))
}

pub(crate) fn render_serve_domain_group_form(
    root: &Path,
    group_id: Option<&str>,
) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let group = match group_id {
        Some(id) => Some(get_domain_group(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = group.is_none();
    let group = group.as_ref();
    let title = if is_new {
        i18n.ui(UiTextKey::CreateNewDomainGroup).to_string()
    } else {
        format!(
            "{}: {}",
            i18n.ui(UiTextKey::Edit),
            group.map(|group| group.display_name.as_str()).unwrap_or("")
        )
    };
    let id_value = group.map(|group| group.id.as_str()).unwrap_or("");
    let display_name = group.map(|group| group.display_name.as_str()).unwrap_or("");
    let description = group.map(|group| group.description.as_str()).unwrap_or("");
    let shared_knowledge = group
        .map(|group| group.shared_knowledge.join("\n"))
        .unwrap_or_default();
    let common_rubric = group
        .map(|group| group.common_rubric.join("\n"))
        .unwrap_or_default();
    let dispatch_hints = group
        .map(|group| group.dispatch_hints.join("\n"))
        .unwrap_or_default();
    let progress_mode = group.and_then(|group| group.workflow.progress_mode);
    let approval_policy = group.and_then(|group| group.workflow.approval_policy);
    let action = if is_new {
        "/api/domain-groups".to_string()
    } else {
        format!("/api/domain-groups/{}", h(id_value))
    };
    let id_field = if is_new {
        format!(
            r#"<label>ID<input name="id" required placeholder="software-development" value="{}"></label>"#,
            h(id_value)
        )
    } else {
        format!(
            r#"<label>ID<input name="id" readonly value="{}"></label>"#,
            h(id_value)
        )
    };
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a class="active" href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">{}</a>
        </div>
      </header>
      <section class="composer">
        <form id="domain-group-form" data-action="{}" data-redirect="/settings#domains">
          {}
          <label>{}<input name="display_name" required value="{}"></label>
          <label>{}<textarea name="description" rows="4" placeholder="このグループに含めるドメインの共通知識">{}</textarea></label>
          <label>{}<textarea name="shared_knowledge" rows="4" placeholder="1行に1知識。例: 変更は小さく検証可能にする">{}</textarea></label>
          <label>{}<textarea name="common_rubric" rows="7" placeholder="1行に1基準。例: 主要な品質基準を満たす">{}</textarea></label>
          <label>{}<textarea name="dispatch_hints" rows="4" placeholder="1行に1ヒント。例: UIならFrontend UI Domainを優先">{}</textarea></label>
          <div class="form-grid">
            <label>{}<select name="workflow_progress_mode">{}</select></label>
            <label>{}<select name="workflow_approval_policy">{}</select></label>
          </div>
          <button type="submit">{}</button>
          <p id="domain-group-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        h(&title),
        serve_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        h(&title),
        i18n.ui(UiTextKey::DomainGroupFormLead),
        i18n.ui(UiTextKey::Settings),
        h(&action),
        id_field,
        i18n.ui(UiTextKey::Name),
        h(display_name),
        i18n.ui(UiTextKey::Description),
        h(description),
        i18n.ui(UiTextKey::SharedKnowledge),
        h(&shared_knowledge),
        i18n.ui(UiTextKey::CommonRubric),
        h(&common_rubric),
        i18n.ui(UiTextKey::DispatchHints),
        h(&dispatch_hints),
        i18n.ui(UiTextKey::ProgressModeDefault),
        workflow_mode_options(progress_mode, true, &i18n),
        i18n.ui(UiTextKey::FinalApprovalDefault),
        approval_policy_options(approval_policy, true, &i18n),
        if is_new {
            i18n.ui(UiTextKey::CreateDomainGroup)
        } else {
            i18n.ui(UiTextKey::SaveDomainGroup)
        },
        serve_script()
    ))
}

pub(crate) fn render_serve_agent_form(
    root: &Path,
    agent_id: Option<&str>,
) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let skill_sets = list_skill_set_catalog(root).map_err(|error| error.to_string())?;
    let agent = agent_id.and_then(|id| agents.iter().find(|agent| agent.id == id));
    let is_new = agent.is_none();
    let title = if is_new {
        i18n.ui(UiTextKey::CreateNewAgent).to_string()
    } else {
        format!(
            "{}: {}",
            i18n.ui(UiTextKey::Edit),
            agent.unwrap().display_name
        )
    };
    let id_value = agent.map(|agent| agent.id.as_str()).unwrap_or("");
    let display_name = agent.map(|agent| agent.display_name.as_str()).unwrap_or("");
    let role = agent.map(|agent| agent.role.as_str()).unwrap_or("");
    let working_dir = agent.map(|agent| agent.working_dir.as_str()).unwrap_or(".");
    let workdir_options = workdir_select_options(root, working_dir);
    let description = agent.map(|agent| agent.description.as_str()).unwrap_or("");
    let specialties = agent
        .map(|agent| agent.specialties.join(", "))
        .unwrap_or_default();
    let selected_skill_set_ids = agent
        .map(|agent| agent.skill_set_ids.clone())
        .unwrap_or_default();
    let selected_domain_group_ids = agent
        .map(|agent| agent.domain_group_ids.clone())
        .unwrap_or_default();
    let selected_domain_ids = agent
        .map(|agent| agent.domain_ids.clone())
        .unwrap_or_default();
    let domain_group_options = domain_group_multi_options(&groups, &selected_domain_group_ids);
    let domain_options = domain_multi_options(&domains, &selected_domain_ids);
    let skill_picker = skill_set_picker(&skill_sets, &selected_skill_set_ids, &i18n);
    let kind = agent
        .map(|agent| agent_kind_value(&agent.runtime, &agent.adapter))
        .unwrap_or("codex_cli");
    let model_provider = agent
        .map(|agent| agent.model.provider.as_str())
        .unwrap_or("");
    let model_id = agent.map(|agent| agent.model.id.as_str()).unwrap_or("");
    let base_url = agent
        .map(|agent| agent.model.base_url.as_str())
        .unwrap_or("");
    let external_agent_id = agent
        .map(|agent| agent.external.agent_id.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(id_value);
    let external_source = agent
        .map(|agent| agent.external.source.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("created");
    let action = if is_new {
        "/api/agents".to_string()
    } else {
        format!("/api/agents/{}", h(id_value))
    };
    let delete_button = if is_new {
        String::new()
    } else {
        format!(
            r#"<button class="danger" type="button" id="delete-agent-button" data-action="/api/agents/{}/delete" data-agent-name="{}">{}</button>"#,
            h(id_value),
            h(display_name),
            h(i18n.ui(UiTextKey::DeleteAgent))
        )
    };
    let id_field = if is_new {
        format!(
            r#"<label>ID<input name="id" required spellcheck="false" autocomplete="off" placeholder="my-agent…" value="{}"></label>"#,
            h(id_value)
        )
    } else {
        format!(
            r#"<label>ID<input name="id" readonly spellcheck="false" autocomplete="off" value="{}"></label>"#,
            h(id_value)
        )
    };
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}{}</style>
</head>
  <body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a class="active" href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">{}</a>
        </div>
      </header>
      <section class="composer">
        <form id="agent-profile-form" data-action="{}" data-redirect="/settings#agents" autocomplete="off">
          {}
          <label>{}
            <select name="agent_kind" id="agent-kind">
              <option value="codex_cli"{}>Codex CLI</option>
              <option value="codex_app_server"{}>Codex App Server</option>
              <option value="openclaw"{}>OpenClaw</option>
            </select>
          </label>
          <input type="hidden" name="runtime" value="">
          <input type="hidden" name="adapter" value="">
          <input type="hidden" name="external_provider" value="">
          <input type="hidden" name="external_agent_id" id="external-agent-id" value="{}">
          <input type="hidden" name="external_managed" value="true">
          <input type="hidden" name="external_source" value="{}">
          <input type="hidden" name="api_key_env" value="">
          <label>{}<input name="display_name" required autocomplete="off" value="{}"></label>
          <label>{}<select name="role">{}</select></label>
          <label>{}<select name="working_dir">{}</select></label>
          <div data-model-section="model">
            <div class="form-grid">
              <label data-model-field="provider">{}<select name="model_provider" id="openclaw-model-provider">{}</select></label>
              <label>{}<input name="model_id" value="{}" spellcheck="false" autocomplete="off" placeholder="gpt-5.3-codex…" list="openai-model-options"></label>
            </div>
            <label data-model-field="base-url">{}<input type="url" name="base_url" value="{}" spellcheck="false" autocomplete="off" placeholder="http://127.0.0.1:11434/v1…"></label>
          </div>
          <datalist id="openai-model-options">
            <option value="gpt-5.3-codex"></option>
            <option value="gpt-5.2-codex"></option>
            <option value="gpt-5.3"></option>
          </datalist>
          <label>{}<select name="domain_group_ids" multiple size="4">{}</select></label>
          <label>{}<select name="domain_ids" multiple size="5">{}</select></label>
          <section class="form-section" data-agent-skills>
            <div class="form-section-head">
              <div>
                <h2>{}</h2>
                <p class="muted">{}</p>
              </div>
              <div class="row-actions"><span class="badge gray">{}</span><a class="button-link secondary" href="/settings/skills/new">{}</a></div>
            </div>
            {}
          </section>
          <label>{}<textarea name="description" rows="6">{}</textarea></label>
          <label>{}<textarea name="specialties" rows="2" placeholder="{}">{}</textarea></label>
          <button type="submit">{}</button>
          {}
          <p id="agent-profile-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        h(&title),
        serve_stylesheet(),
        serve_responsive_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        h(&title),
        i18n.ui(UiTextKey::AgentFormLead),
        i18n.ui(UiTextKey::Agents),
        action,
        id_field,
        i18n.ui(UiTextKey::ExternalAgentType),
        if kind == "codex_cli" { " selected" } else { "" },
        if kind == "codex_app_server" {
            " selected"
        } else {
            ""
        },
        if kind == "openclaw" { " selected" } else { "" },
        h(external_agent_id),
        h(external_source),
        i18n.ui(UiTextKey::DisplayName),
        h(display_name),
        i18n.ui(UiTextKey::Role),
        role_options(role),
        i18n.ui(UiTextKey::Workdir),
        workdir_options,
        i18n.ui(UiTextKey::ModelProvider),
        openclaw_provider_options(model_provider),
        i18n.ui(UiTextKey::Model),
        h(model_id),
        i18n.ui(UiTextKey::BaseUrl),
        h(base_url),
        i18n.ui(UiTextKey::DomainGroups),
        domain_group_options,
        i18n.ui(UiTextKey::Domains),
        domain_options,
        localized(&i18n, "スキル", "Skills"),
        localized(
            &i18n,
            "このエージェントで使う能力",
            "Capabilities used by this agent"
        ),
        localized(&i18n, "インストール済み", "Installed"),
        localized(&i18n, "スキルを追加", "Add Skill"),
        skill_picker,
        i18n.ui(UiTextKey::Instructions),
        h(&description),
        i18n.ui(UiTextKey::Specialties),
        h(localized(
            &i18n,
            "カンマまたは改行区切り…",
            "Comma or newline separated…"
        )),
        h(&specialties),
        if is_new {
            i18n.ui(UiTextKey::CreateAgent)
        } else {
            i18n.ui(UiTextKey::SaveAgent)
        },
        delete_button,
        serve_script()
    ))
}

pub(crate) fn render_serve_skill_form(root: &Path) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>{}</title>
  <style>{}{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" width="1254" height="1254" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">{}</a>
        <a class="active" href="/settings">{}</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">{}</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings/agents/new">{}</a>
        </div>
      </header>
      <section class="composer">
        <form id="skill-package-form" data-action="/api/skills" data-redirect="/settings/agents/new" autocomplete="off">
          <div class="form-grid">
            <label>{}<select name="source_kind">{}</select></label>
            <label>{}<input name="id" spellcheck="false" autocomplete="off" placeholder="react-review…"></label>
          </div>
          <label>{}<input name="source" spellcheck="false" autocomplete="off" placeholder="vercel-labs/agent-skills or skill name…"></label>
          <label>{}<input name="path" spellcheck="false" autocomplete="off" placeholder="./skills/react-review…"></label>
          <div class="form-grid">
            <label>{}<input name="reference" spellcheck="false" autocomplete="off" placeholder="main, v1.0.0, commit sha…"></label>
            <label>{}<input name="checksum" spellcheck="false" autocomplete="off" placeholder="sha256:…"></label>
          </div>
          <label>{}<input name="skill_set_id" spellcheck="false" autocomplete="off" placeholder="react-review…"></label>
          <label>{}<textarea name="skill_paths" rows="2" placeholder="src, tests…"></textarea></label>
          <div class="form-grid">
            <label>{}<textarea name="required_capabilities" rows="2" placeholder="repo_read, shell_command…"></textarea></label>
            <label>{}<textarea name="optional_capabilities" rows="2" placeholder="event_stream…"></textarea></label>
          </div>
          <button type="submit">{}</button>
          <p id="skill-package-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        h(localized(&i18n, "スキルを追加", "Add Skill")),
        serve_stylesheet(),
        serve_responsive_stylesheet(),
        i18n.ui(UiTextKey::WorkQueue),
        i18n.ui(UiTextKey::Settings),
        h(localized(&i18n, "スキルを追加", "Add Skill")),
        h(localized(
            &i18n,
            "ClawHub / Vercel Skills / skill-creator 由来の skill package をProjectに登録します。",
            "Register a skill package from ClawHub, Vercel Skills, skill-creator, local, or git."
        )),
        h(i18n.ui(UiTextKey::Agents)),
        h(localized(&i18n, "追加元", "Source")),
        skill_source_options(),
        h(localized(&i18n, "Package ID", "Package ID")),
        h(localized(&i18n, "Source", "Source")),
        h(localized(&i18n, "Path", "Path")),
        h(localized(&i18n, "Ref / Version", "Ref / Version")),
        h(localized(&i18n, "Checksum", "Checksum")),
        h(localized(&i18n, "Skill Set ID", "Skill Set ID")),
        h(localized(&i18n, "対象Path", "Target Paths")),
        h(localized(&i18n, "必須能力", "Required Capabilities")),
        h(localized(&i18n, "追加能力", "Optional Capabilities")),
        h(localized(&i18n, "スキルを登録", "Register Skill")),
        serve_script()
    ))
}

fn skill_source_options() -> String {
    [
        ("skill-creator", "Skill Creator"),
        ("clawhub", "ClawHub"),
        ("vercel", "Vercel Skills"),
        ("local", "Local"),
        ("git", "Git"),
    ]
    .into_iter()
    .map(|(value, label)| format!(r#"<option value="{}">{}</option>"#, h(value), h(label)))
    .collect::<Vec<_>>()
    .join("")
}

fn openclaw_provider_options(selected: &str) -> String {
    let selected = match selected {
        "ollama" | "lmstudio" | "openai" | "openai-codex" => selected,
        _ => "openai-codex",
    };
    [
        ("openai-codex", "OpenAI"),
        ("ollama", "Ollama"),
        ("lmstudio", "LM Studio"),
    ]
    .into_iter()
    .map(|(value, label)| {
        let selected_attr = if value == selected { " selected" } else { "" };
        format!(
            r#"<option value="{}"{}>{}</option>"#,
            h(value),
            selected_attr,
            h(label)
        )
    })
    .collect::<Vec<_>>()
    .join("")
}
