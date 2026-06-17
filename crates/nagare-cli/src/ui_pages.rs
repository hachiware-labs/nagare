use std::path::Path;

use nagare_core::{
    ApprovalPolicy, I18n, UiTextKey, WorkItemStatus, WorkflowMode, WorkflowSettings,
    get_artifact_type, get_domain, get_locale_settings, get_work_item_snapshot,
    get_workflow_settings, list_agent_profiles, list_artifact_types, list_domains,
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
    let (rows, cards) = if items.is_empty() {
        (
            format!(
                "<tr><td colspan=\"8\" class=\"muted\">{}</td></tr>",
                h(i18n.ui(UiTextKey::NoWorkItemsYet))
            ),
            format!(
                r#"<div class="queue-card empty"><p class="muted">{}</p></div>"#,
                h(i18n.ui(UiTextKey::NoWorkItemsYet))
            ),
        )
    } else {
        let mut row_entries = Vec::new();
        let mut card_entries = Vec::new();
        for item in &items {
            let snapshot = get_work_item_snapshot(root, &item.id).ok();
            let running = read_ui_running_state(root, &item.id);
            let next_action = snapshot
                .as_ref()
                .map(|snapshot| snapshot.completion.next_action.as_str())
                .unwrap_or("-");
            let (state_label, state_class, state_detail) =
                work_item_list_state(item, snapshot.as_ref(), running.as_deref());
            queue_signals.observe(&state_label);
            let answer = snapshot
                .as_ref()
                .map(|snapshot| answer_view(snapshot, &agents));
            let answer_preview = render_answer_preview(answer.as_ref());
            let filter_state = queue_filter_state(&state_label);
            let row_class = format!(
                "state-{}",
                state_label.to_ascii_lowercase().replace(' ', "-")
            );
            let state_text = localized_queue_state(&i18n, &state_label);
            let state_detail_text = localized_queue_detail(&i18n, &state_detail);
            let delete_form = format!(
                r#"<form class="delete-work-form" data-work-id="{}" data-work-title="{}"><button class="danger" type="submit">{}</button></form>"#,
                h(&item.id),
                h(&item.title),
                h(i18n.ui(UiTextKey::Delete))
            );
            row_entries.push(format!(
                r#"<tr class="{}" data-work-record="{}" data-queue-state="{}"><td data-label="{}"><a href="/items/{}">{}</a><div class="muted">{}</div></td><td data-label="{}">{}</td><td data-label="Answer">{}</td><td data-label="{}"><span class="badge {}">{}</span><div class="muted">{}</div></td><td data-label="{}">{}</td><td data-label="{}">{}</td><td data-label="">{}</td></tr>"#,
                h(&row_class),
                h(&item.id),
                h(filter_state),
                h(i18n.ui(UiTextKey::IdFolder)),
                h(&item.id),
                h(&item.id),
                h(item.work_folder.as_deref().unwrap_or(".")),
                h(i18n.ui(UiTextKey::Title)),
                h(&item.title),
                answer_preview,
                h(i18n.ui(UiTextKey::State)),
                state_class,
                h(&state_text),
                h(&state_detail_text),
                h(i18n.ui(UiTextKey::Next)),
                h(next_action),
                h(i18n.ui(UiTextKey::Mode)),
                h(&item.workflow_mode.to_string()),
                delete_form
            ));
            card_entries.push(format!(
                r#"<article class="queue-card {}" data-work-record="{}" data-queue-state="{}">
  <div class="queue-card-head">
    <div>
      <a href="/items/{}">{}</a>
      <p class="muted">{}</p>
    </div>
    <span class="badge {}">{}</span>
  </div>
  <h3>{}</h3>
  <div class="queue-card-answer">{}</div>
  <dl class="queue-card-meta">
    <div><dt>{}</dt><dd>{}</dd></div>
    <div><dt>{}</dt><dd>{}</dd></div>
  </dl>
  <div class="queue-card-actions">{}</div>
</article>"#,
                h(&row_class),
                h(&item.id),
                h(filter_state),
                h(&item.id),
                h(&item.id),
                h(item.work_folder.as_deref().unwrap_or(".")),
                state_class,
                h(&state_text),
                h(&item.title),
                answer_preview,
                h(i18n.ui(UiTextKey::Next)),
                h(next_action),
                h(i18n.ui(UiTextKey::Mode)),
                h(&item.workflow_mode.to_string()),
                delete_form
            ));
        }
        (row_entries.join("\n"), card_entries.join("\n"))
    };
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Nagare UI Server</title>
  <style>{}{}</style>
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
          <table class="queue-table"><thead><tr><th>{}</th><th>{}</th><th>Answer</th><th>{}</th><th>{}</th><th>{}</th><th></th></tr></thead><tbody id="work-items">{}</tbody></table>
          <div class="queue-card-list" aria-label="{}">{}</div>
        </section>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        serve_stylesheet(),
        serve_responsive_stylesheet(),
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
        i18n.ui(UiTextKey::WorkQueue),
        cards,
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
    let domains = list_domains(root).map_err(|error| error.to_string())?;
    let artifact_types = list_artifact_types(root).map_err(|error| error.to_string())?;
    let settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let domain_options = domain_select_options(&domains, None, i18n.ui(UiTextKey::ProjectDefault));
    let artifact_type_options =
        artifact_type_select_options(&artifact_types, None, i18n.ui(UiTextKey::ProjectDefault));
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
  <style>{}{}</style>
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
            <div class="form-grid">
              <label>{}<select name="domain_id" id="work-domain-group">{}</select></label>
              <label>{}<select name="artifact_type_id" id="work-domain">{}</select></label>
            </div>
            <label>{}<select name="domain_agent_policy">{}</select></label>
            <label>{}<select name="workflow_mode">{}</select></label>
            <label>{}<select name="approval_policy">{}</select></label>
            <aside class="routing-preview" data-routing-preview>
              <div>
                <span>{}</span>
                <b data-routing-domain>{}</b>
              </div>
              <p data-routing-policy>{}</p>
              <small>{}</small>
            </aside>
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
        serve_responsive_stylesheet(),
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
        i18n.ui(UiTextKey::Domain),
        artifact_type_options,
        i18n.ui(UiTextKey::Domain),
        domain_options,
        i18n.ui(UiTextKey::DomainAgentPolicy),
        domain_agent_policy_options,
        i18n.ui(UiTextKey::ProgressMode),
        workflow_options,
        i18n.ui(UiTextKey::FinalApproval),
        approval_options,
        localized(&i18n, "処理の見通し", "Routing Preview"),
        localized(
            &i18n,
            "プロジェクト既定で判定します",
            "Project default routing"
        ),
        localized(
            &i18n,
            "作成後、Dispatcherが依頼内容とドメイン設定から担当候補を確認します。",
            "After creation, the dispatcher checks the request and domain settings for a target agent."
        ),
        localized(
            &i18n,
            "実際の担当と理由は詳細画面のステップで確認できます。",
            "The chosen target and reason appear in the item detail steps."
        ),
        i18n.ui(UiTextKey::CreateNewItem),
        serve_script()
    ))
}

pub(crate) fn render_serve_settings(root: &Path) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let domains = list_domains(root).map_err(|error| error.to_string())?;
    let artifact_types = list_artifact_types(root).map_err(|error| error.to_string())?;
    let workflow_settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let agent_rows = agent_profile_rows(&agents, &domains, &artifact_types, &i18n);
    let group_rows = domain_rows(&domains, &i18n);
    let domain_rows = artifact_type_rows(&artifact_types, &domains, &i18n);
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
            <a class="button-link secondary" href="/settings/domains/new">{}</a>
          </div>
          <table class="domain-table"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody id="domains">{}</tbody></table>
        </section>
        <section class="panel">
          <div class="panel-head">
            <div>
              <h2>{}</h2>
              <p class="muted">{}</p>
            </div>
            <a class="button-link secondary" href="/settings/artifact-types/new">{}</a>
          </div>
          <table class="domain-table"><thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody id="domain-profiles">{}</tbody></table>
        </section>
      </section>
      <section id="settings-panel-agents" class="panel settings-panel" role="tabpanel" aria-labelledby="settings-tab-agents" data-settings-panel="agents" hidden>
        <div class="panel-head">
          <h2>{}</h2>
          <a class="button-link secondary" href="/settings/agents/new">{}</a>
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
        i18n.ui(UiTextKey::Settings),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::Agents),
        workflow_form,
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::DomainsLead),
        i18n.ui(UiTextKey::CreateNewDomain),
        i18n.ui(UiTextKey::Domain),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::SharedKnowledge),
        i18n.ui(UiTextKey::Rubric),
        i18n.ui(UiTextKey::DispatchHints),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Source),
        i18n.ui(UiTextKey::Actions),
        group_rows,
        i18n.ui(UiTextKey::ArtifactTypes),
        i18n.ui(UiTextKey::ArtifactTypesLead),
        i18n.ui(UiTextKey::CreateNewArtifactType),
        i18n.ui(UiTextKey::ArtifactType),
        i18n.ui(UiTextKey::Domain),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::Rubric),
        i18n.ui(UiTextKey::DispatchHints),
        i18n.ui(UiTextKey::Workflow),
        i18n.ui(UiTextKey::Source),
        i18n.ui(UiTextKey::Actions),
        domain_rows,
        i18n.ui(UiTextKey::Agents),
        i18n.ui(UiTextKey::CreateNewAgent),
        agent_filters(&domains, &artifact_types, &i18n),
        i18n.ui(UiTextKey::Agent),
        i18n.ui(UiTextKey::Description),
        i18n.ui(UiTextKey::Domains),
        i18n.ui(UiTextKey::ArtifactTypes),
        i18n.ui(UiTextKey::Actions),
        agent_rows,
        serve_script()
    ))
}

fn domain_rows(groups: &[nagare_core::Domain], i18n: &I18n) -> String {
    if groups.is_empty() {
        return format!(
            "<tr><td colspan=\"5\" class=\"muted\">{}.</td></tr>",
            h(i18n.ui(UiTextKey::Domains))
        );
    }
    let mut sorted_groups = groups.iter().collect::<Vec<_>>();
    sorted_groups.sort_by_key(|group| group.display_name.as_str());
    sorted_groups
        .into_iter()
        .map(|group| {
            format!(
            r#"<tr>
  <td data-label="{}"><a href="/settings/domains/{}">{}</a><div class="muted">{}</div></td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/domains/{}">{}</a><form class="delete-domain-group-form" data-domain-group-id="{}" data-domain-group-name="{}"><button class="danger" type="submit">{}</button></form></div></td>
</tr>"#,
                h(i18n.ui(UiTextKey::Domain)),
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
                h(&domain_workflow_label(group, i18n)),
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

fn artifact_type_rows(
    domains: &[nagare_core::ArtifactType],
    groups: &[nagare_core::Domain],
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
  <td data-label="{}"><a href="/settings/artifact-types/{}">{}</a><div class="muted">{}</div></td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/artifact-types/{}">{}</a><form class="delete-domain-form" data-domain-id="{}" data-domain-name="{}"><button class="danger" type="submit">{}</button></form></div></td>
</tr>"#,
                h(i18n.ui(UiTextKey::Domain)),
                h(&domain.id),
                h(&domain.display_name),
                h(&domain.id),
                h(i18n.ui(UiTextKey::Group)),
                h(&domain_label(groups, domain.domain_id.as_deref())),
                h(i18n.ui(UiTextKey::Description)),
                h(&compact_instruction(&domain.description)),
                h(i18n.ui(UiTextKey::Rubric)),
                h(&domain.rubric.len().to_string()),
                h(i18n.ui(UiTextKey::DispatchHints)),
                h(&domain.dispatch_hints.len().to_string()),
                h(i18n.ui(UiTextKey::Workflow)),
                h(&artifact_type_workflow_label(domain, i18n)),
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

fn domain_label(groups: &[nagare_core::Domain], id: Option<&str>) -> String {
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
            "project_domain_directory" => "プロジェクトのドメイン定義",
            "project_artifact_type_directory" => "プロジェクトの成果物種別定義",
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

fn artifact_type_select_options(
    domains: &[nagare_core::ArtifactType],
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
            r#"<option value="{}" data-domain-group="{}"{}>{}</option>"#,
            h(&domain.id),
            h(domain.domain_id.as_deref().unwrap_or("")),
            selected_attr,
            h(&domain.display_name)
        )
    }));
    options.join("")
}

fn domain_select_options(
    groups: &[nagare_core::Domain],
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

fn agent_artifact_type_select_options(
    domains: &[nagare_core::ArtifactType],
    selected: Option<&str>,
) -> String {
    artifact_type_select_options(domains, selected, "-")
}

fn agent_domain_select_options(groups: &[nagare_core::Domain], selected: Option<&str>) -> String {
    domain_select_options(groups, selected, "-")
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
    groups: &[nagare_core::Domain],
    domains: &[nagare_core::ArtifactType],
    i18n: &I18n,
) -> String {
    let group_filters = agent_domain_filter_options(groups, i18n);
    let domain_filters = agent_artifact_type_filter_options(domains, i18n);
    format!(
        r#"<div class="filter-panel" data-agent-filters>
          <div>
            <h3>{}</h3>
            <div class="checkbox-grid">{}</div>
          </div>
          <div>
            <h3>{}</h3>
            <div class="checkbox-grid" data-agent-domain-filter-options>{}</div>
            <span class="muted" data-agent-domain-filter-empty>{}</span>
          </div>
          <div class="filter-actions">
            <button class="secondary-button" type="button" data-clear-agent-filters>{}</button>
            <span class="muted" data-agent-filter-count></span>
          </div>
        </div>"#,
        i18n.ui(UiTextKey::ArtifactTypes),
        group_filters,
        i18n.ui(UiTextKey::Domains),
        domain_filters,
        localized(
            i18n,
            "ドメインを選択すると成果物種別の候補が表示されます。",
            "Select a domain to show artifact type choices."
        ),
        i18n.ui(UiTextKey::ClearFilters)
    )
}

fn agent_domain_filter_options(groups: &[nagare_core::Domain], i18n: &I18n) -> String {
    if groups.is_empty() {
        return format!(
            r#"<span class="muted">{}.</span>"#,
            h(i18n.ui(UiTextKey::Domains))
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

fn agent_artifact_type_filter_options(
    domains: &[nagare_core::ArtifactType],
    i18n: &I18n,
) -> String {
    if domains.is_empty() {
        return format!(
            r#"<span class="muted">{}.</span>"#,
            h(i18n.ui(UiTextKey::ArtifactTypes))
        );
    }
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    domains
        .into_iter()
        .map(|domain| {
            format!(
                r#"<label class="check-option" data-agent-filter-domain-option data-domain-group="{}" hidden><input type="checkbox" data-agent-filter-domain value="{}" disabled><span>{}</span></label>"#,
                h(domain.domain_id.as_deref().unwrap_or("")),
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
    agent_id: Option<&str>,
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
        r#"<div class="skill-picker" data-skill-picker data-empty-label="{}" data-agent-id="{}" data-uninstall-label="{}" data-uninstall-confirm="{}">
  <label class="skill-search">{}<input type="search" data-skill-search autocomplete="off" placeholder="{}"></label>
  <div class="skill-selected" data-skill-selected aria-live="polite">{}</div>
  <div class="skill-picker-list">{}</div>
</div>"#,
        h(localized(i18n, "スキル未選択", "No skills selected")),
        h(agent_id.unwrap_or("")),
        h(localized(i18n, "アンインストール", "Uninstall")),
        h(localized(
            i18n,
            "このエージェントから外し、他で未使用ならこのツール向けのスキル本体も削除します。実行しますか？",
            "Remove this skill from the agent and uninstall the package for this tool if unused elsewhere?"
        )),
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

fn artifact_type_workflow_label(domain: &nagare_core::ArtifactType, i18n: &I18n) -> String {
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

fn domain_workflow_label(group: &nagare_core::Domain, i18n: &I18n) -> String {
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
    groups: &[nagare_core::Domain],
    domains: &[nagare_core::ArtifactType],
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
            let domain_ids = agent.domain_ids.join(" ");
            let artifact_type_ids = agent.artifact_type_ids.join(" ");
            format!(
                r#"<tr data-agent-row data-agent-domains="{}" data-agent-artifact-types="{}">
  <td data-label="{}">
    <a href="/settings/agents/{}">{}</a>
    <div class="muted" translate="no">{}</div>
  </td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}">{}</td>
  <td data-label="{}"><div class="row-actions"><a class="button-link secondary" href="/settings/agents/{}">{}</a></div></td>
</tr>"#,
                h(&domain_ids),
                h(&artifact_type_ids),
                h(i18n.ui(UiTextKey::Agent)),
                h(&agent.id),
                h(&agent.display_name),
                h(&agent.id),
                h(i18n.ui(UiTextKey::Description)),
                h(&compact_instruction(&agent.description)),
                h(i18n.ui(UiTextKey::Domains)),
                h(&agent_domain_label(agent, groups, i18n)),
                h(i18n.ui(UiTextKey::ArtifactTypes)),
                h(&agent_artifact_type_label(agent, domains, i18n)),
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

fn agent_domain_label(
    agent: &nagare_core::AgentProfile,
    groups: &[nagare_core::Domain],
    i18n: &I18n,
) -> String {
    if agent.domain_ids.is_empty() {
        return any_scope_label(i18n);
    }
    agent
        .domain_ids
        .iter()
        .map(|id| domain_label(groups, Some(id)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn agent_artifact_type_label(
    agent: &nagare_core::AgentProfile,
    domains: &[nagare_core::ArtifactType],
    i18n: &I18n,
) -> String {
    if agent.artifact_type_ids.is_empty() {
        return any_scope_label(i18n);
    }
    agent
        .artifact_type_ids
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

pub(crate) fn render_serve_artifact_type_form(
    root: &Path,
    artifact_type_id: Option<&str>,
) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let groups = list_domains(root).map_err(|error| error.to_string())?;
    let domain = match artifact_type_id {
        Some(id) => Some(get_artifact_type(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = domain.is_none();
    let domain = domain.as_ref();
    let title = if is_new {
        i18n.ui(UiTextKey::CreateNewArtifactType).to_string()
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
    let group_options = domain_select_options(
        &groups,
        domain.and_then(|domain| domain.domain_id.as_deref()),
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
        "/api/artifact-types".to_string()
    } else {
        format!("/api/artifact-types/{}", h(id_value))
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
          <label>{}<select name="domain_id">{}</select></label>
          <label>{}<input name="display_name" required value="{}"></label>
          <label>{}<textarea name="description" rows="4" placeholder="このドメインが扱う作成物や判断対象">{}</textarea></label>
          <label>{}<textarea name="artifact_types" rows="3" placeholder="1行に1種類。例: html, ui screenshot, rust cli">{}</textarea></label>
          <section class="form-section domain-rubric-builder">
            <div class="field-group-head">
              <h2>Rubric Builder</h2>
              <p class="muted">ドメイン説明、サンプル、評価ポイントから100点満点の判断基準を作ります。</p>
            </div>
            <div class="form-grid">
              <label>良いサンプル<input type="file" name="sample_good_files" multiple></label>
              <label>悪いサンプル<input type="file" name="sample_bad_files" multiple></label>
            </div>
            <label>参考サンプル<input type="file" name="sample_reference_files" multiple></label>
            <div class="form-grid">
              <label>サンプルメモ<textarea name="sample_note" rows="4" placeholder="サンプルから読み取ってほしい品質、失敗例、比較観点"></textarea></label>
              <label>一般的な評価ポイント<textarea name="general_points" rows="4" placeholder="世間一般で重視される評価観点"></textarea></label>
            </div>
            <div class="form-grid">
              <label>プロジェクト固有の評価ポイント<textarea name="project_points" rows="4" placeholder="このプロジェクトで特に重視する判断基準"></textarea></label>
              <label>NG例<textarea name="ng_examples" rows="4" placeholder="低評価にしたいパターン、避けたい成果物"></textarea></label>
            </div>
            <div class="rubric-builder-actions">
              <button class="secondary-button" type="button" data-generate-domain-rubric>100点Rubricを生成</button>
              <p class="muted">採点結果に応じた次のAgent処理は、Nagareの共通Review Policyに従います。</p>
            </div>
          </section>
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
        i18n.ui(UiTextKey::ArtifactTypeFormLead),
        i18n.ui(UiTextKey::Settings),
        h(&action),
        id_field,
        i18n.ui(UiTextKey::Domain),
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
            i18n.ui(UiTextKey::CreateArtifactType)
        } else {
            i18n.ui(UiTextKey::SaveArtifactType)
        },
        serve_script()
    ))
}

pub(crate) fn render_serve_domain_form(
    root: &Path,
    domain_id: Option<&str>,
) -> Result<String, String> {
    let i18n = i18n_for_root(root)?;
    let group = match domain_id {
        Some(id) => Some(get_domain(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = group.is_none();
    let group = group.as_ref();
    let title = if is_new {
        i18n.ui(UiTextKey::CreateNewDomain).to_string()
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
        "/api/domains".to_string()
    } else {
        format!("/api/domains/{}", h(id_value))
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
        i18n.ui(UiTextKey::DomainFormLead),
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
            i18n.ui(UiTextKey::CreateDomain)
        } else {
            i18n.ui(UiTextKey::SaveDomain)
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
    let groups = list_domains(root).map_err(|error| error.to_string())?;
    let domains = list_artifact_types(root).map_err(|error| error.to_string())?;
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
    let description = agent.map(|agent| agent.description.as_str()).unwrap_or("");
    let specialties = agent
        .map(|agent| agent.specialties.join(", "))
        .unwrap_or_default();
    let selected_skill_set_ids = agent
        .map(|agent| agent.skill_set_ids.clone())
        .unwrap_or_default();
    let selected_domain_ids = agent
        .map(|agent| agent.domain_ids.clone())
        .unwrap_or_default();
    let selected_artifact_type_ids = agent
        .map(|agent| agent.artifact_type_ids.clone())
        .unwrap_or_default();
    let selected_domain_id = selected_domain_ids.first().map(String::as_str);
    let selected_artifact_type_id = selected_artifact_type_ids.first().map(String::as_str);
    let domain_options = agent_domain_select_options(&groups, selected_domain_id);
    let artifact_type_options =
        agent_artifact_type_select_options(&domains, selected_artifact_type_id);
    let skill_picker = skill_set_picker(&skill_sets, &selected_skill_set_ids, &i18n, agent_id);
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
          <aside class="routing-preview compact" data-agent-kind-hint>
            <div>
              <span>{}</span>
              <b data-agent-kind-title></b>
            </div>
            <p data-agent-kind-copy></p>
          </aside>
          <input type="hidden" name="runtime" value="">
          <input type="hidden" name="adapter" value="">
          <input type="hidden" name="external_provider" value="">
          <input type="hidden" name="external_agent_id" id="external-agent-id" value="{}">
          <input type="hidden" name="external_managed" value="true">
          <input type="hidden" name="external_source" value="{}">
          <input type="hidden" name="api_key_env" value="">
          <label>{}<input name="display_name" required autocomplete="off" value="{}"></label>
          <label>{}<select name="role">{}</select></label>
          <label>{}<input name="working_dir" value="{}" spellcheck="false" autocomplete="off" placeholder="., crates/nagare-cli, packages/app…"></label>
          <div data-model-section="model">
            <div class="field-group-head">
              <h2>{}</h2>
              <p class="muted" data-model-help>{}</p>
            </div>
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
          <label>{}<select name="domain_ids" id="agent-domain-group">{}</select></label>
          <label>{}<select name="artifact_type_ids" id="agent-domain">{}</select></label>
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
        localized(&i18n, "ツール設定", "Tool Settings"),
        h(external_agent_id),
        h(external_source),
        i18n.ui(UiTextKey::DisplayName),
        h(display_name),
        i18n.ui(UiTextKey::Role),
        role_options(role),
        i18n.ui(UiTextKey::Workdir),
        h(working_dir),
        localized(&i18n, "モデル", "Model"),
        localized(
            &i18n,
            "選んだ外部エージェント種別に応じて必要なモデル設定だけを表示します。",
            "Only model fields needed for the selected external agent type are shown."
        ),
        i18n.ui(UiTextKey::ModelProvider),
        openclaw_provider_options(model_provider),
        i18n.ui(UiTextKey::Model),
        h(model_id),
        i18n.ui(UiTextKey::BaseUrl),
        h(base_url),
        i18n.ui(UiTextKey::Domains),
        artifact_type_options,
        i18n.ui(UiTextKey::Domains),
        domain_options,
        localized(&i18n, "スキル", "Skills"),
        localized(
            &i18n,
            "このエージェントで使う能力",
            "Capabilities used by this agent"
        ),
        localized(&i18n, "登録済みスキル", "Registered Skills"),
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
          <input type="hidden" name="install" value="true">
          <section class="source-choice-section">
            <div class="field-group-head">
              <h2>{}</h2>
              <p class="muted">{}</p>
            </div>
            <div class="source-choice-grid" data-skill-source-choices>
              {}
            </div>
            <label class="visually-hidden">{}<select name="source_kind" id="skill-source-kind">{}</select></label>
          </section>
          <aside class="routing-preview compact" data-skill-source-summary>
            <div>
              <span>{}</span>
              <b data-skill-source-title></b>
            </div>
            <p data-skill-source-copy></p>
            <small data-skill-source-fields></small>
          </aside>
          <label data-skill-primary-field><span data-skill-primary-label>{}</span><input name="id" spellcheck="false" autocomplete="off" placeholder="react-review…"></label>
          <label data-skill-source-field="source"><span data-skill-source-label>{}</span><input name="source" spellcheck="false" autocomplete="off" placeholder="https://github.com/owner/repo…"></label>
          <label data-skill-source-field="path"><span data-skill-path-label>{}</span><input name="path" spellcheck="false" autocomplete="off" placeholder="./skills/react-review…"></label>
          <div class="form-section" data-skill-source-field="vercel_options">
            <div class="field-group-head">
              <h2>{}</h2>
              <p class="muted">{}</p>
            </div>
            <div class="form-grid">
              <label>{}<select name="install_scope"><option value="project" selected>{}</option><option value="global">{}</option></select></label>
              <label>{}<span class="checkbox-grid"><span class="check-option"><input type="checkbox" name="install_targets" value="codex" checked><span>{}</span></span><span class="check-option"><input type="checkbox" name="install_targets" value="openclaw"><span>{}</span></span></span></label>
            </div>
          </div>
          <details class="advanced-form">
            <summary>{}</summary>
            <div class="form-grid">
              <label>{}<input name="reference" spellcheck="false" autocomplete="off" placeholder="main, v1.0.0, commit sha…"></label>
              <label>{}<input name="checksum" spellcheck="false" autocomplete="off" placeholder="sha256:…"></label>
            </div>
            <label>{}<input name="skill_set_id" spellcheck="false" autocomplete="off" placeholder="通常はスキル名と同じ…"></label>
            <label>{}<textarea name="skill_paths" rows="2" placeholder="必要な場合だけ。例: src, tests…"></textarea></label>
            <div class="form-grid">
              <label>{}<textarea name="required_capabilities" rows="2" placeholder="カタログにない場合だけ。例: repo_read…"></textarea></label>
              <label>{}<textarea name="optional_capabilities" rows="2" placeholder="必要な場合だけ。例: shell_command…"></textarea></label>
            </div>
          </details>
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
            "skill package をProjectに取り込み、スキルカタログへ登録します。",
            "Import a skill package into the project and register it in the skill catalog."
        )),
        h(i18n.ui(UiTextKey::Agents)),
        h(localized(&i18n, "追加方法", "Add Method")),
        h(localized(
            &i18n,
            "探す、作る、持ち込むのどれかを選ぶと、必要な入力だけを表示します。",
            "Choose discover, create, or import; only the required fields are shown."
        )),
        skill_source_choice_cards(),
        h(localized(&i18n, "追加元", "Source")),
        skill_source_options(),
        h(localized(&i18n, "次に必要な入力", "Required Next Inputs")),
        h(localized(&i18n, "スキル名", "Skill Name")),
        h(localized(&i18n, "Repo URL", "Repo URL")),
        h(localized(&i18n, "フォルダPath", "Folder Path")),
        h(localized(
            &i18n,
            "Vercelインストール先",
            "Vercel Install Target"
        )),
        h(localized(
            &i18n,
            "Project範囲を既定にし、必要なツールだけにインストールします。",
            "Project scope is the default; install only to the tools you need."
        )),
        h(localized(&i18n, "範囲", "Scope")),
        h(localized(&i18n, "Project", "Project")),
        h(localized(&i18n, "Global", "Global")),
        h(localized(&i18n, "対象ツール", "Target Tools")),
        h(localized(&i18n, "Codex", "Codex")),
        h(localized(&i18n, "OpenClaw", "OpenClaw")),
        h(localized(&i18n, "詳細設定", "Advanced Settings")),
        h(localized(&i18n, "Ref / Version", "Ref / Version")),
        h(localized(&i18n, "Checksum", "Checksum")),
        h(localized(&i18n, "Skill Set ID", "Skill Set ID")),
        h(localized(&i18n, "対象Path", "Target Paths")),
        h(localized(&i18n, "必須能力", "Required Capabilities")),
        h(localized(&i18n, "追加能力", "Optional Capabilities")),
        h(localized(&i18n, "取り込んで登録", "Import and Register")),
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

fn skill_source_choice_cards() -> String {
    [
        (
            "skill-creator",
            "Skill Creator",
            "Skill Creatorで作成したスキルフォルダを登録する",
        ),
        (
            "clawhub",
            "ClawHub",
            "公開カタログのスキルIDを取り込む",
        ),
        (
            "vercel",
            "Vercel Skills",
            "Vercel Skillsのpackage IDを取り込む",
        ),
        ("local", "Local", "手元のスキルフォルダを登録する"),
        ("git", "Git", "Gitリポジトリ上のスキルを参照して登録する"),
    ]
    .into_iter()
    .map(|(value, title, copy)| {
        format!(
            r#"<button class="source-choice" type="button" data-skill-source-choice="{}" aria-pressed="false"><b>{}</b><span>{}</span></button>"#,
            h(value),
            h(title),
            h(copy)
        )
    })
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
