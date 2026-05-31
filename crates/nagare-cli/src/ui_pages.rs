use std::fs;
use std::path::Path;

use nagare_core::{
    ApprovalPolicy, WorkItemStatus, WorkflowMode, WorkflowSettings, get_domain_group,
    get_domain_profile, get_work_item_snapshot, get_workflow_settings, list_agent_profiles,
    list_domain_groups, list_domain_profiles, list_work_items,
};

use crate::ui::read_ui_running_state;
use crate::ui_answer::{answer_view, render_answer_preview};
use crate::ui_assets::{serve_script, serve_stylesheet};
use crate::ui_html::h;
pub(crate) fn render_serve_home(root: &Path) -> Result<String, String> {
    let items = list_work_items(root).map_err(|error| error.to_string())?;
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let mut queue_signals = QueueSignals::default();
    let rows = if items.is_empty() {
        "<tr><td colspan=\"8\" class=\"muted\">No work items yet</td></tr>".to_string()
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
                    r#"<tr class="{}" data-queue-state="{}"><td><a href="/items/{}">{}</a><div class="muted">{}</div></td><td>{}</td><td>{}</td><td><span class="badge {}">{}</span><div class="muted">{}</div></td><td>{}</td><td>{}</td><td><form class="delete-work-form" data-work-id="{}" data-work-title="{}"><button class="danger" type="submit">Delete</button></form></td></tr>"#,
                    h(&format!("state-{}", state_label.to_ascii_lowercase().replace(' ', "-"))),
                    h(filter_state),
                    h(&item.id),
                    h(&item.id),
                    h(item.work_folder.as_deref().unwrap_or(".")),
                    h(&item.title),
                    render_answer_preview(answer.as_ref()),
                    state_class,
                    h(&state_label),
                    h(&state_detail),
                    h(next_action),
                    h(&item.workflow_mode.to_string()),
                    h(&item.id),
                    h(&item.title)
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
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a class="active" href="/">Work Queue</a>
        <a href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>Work Queue</h1>
          <p class="muted">Monitor, filter, and continue agent work from one queue.</p>
        </div>
        <div class="actions">
          <a class="button-link" href="/new">Create New Item</a>
          <span class="badge blue">work {}</span>
          <span class="badge gray">agents {}</span>
        </div>
      </header>
      <section class="queue-layout">
        <section class="panel queue-panel">
          <div class="panel-head">
            <h2>Work Queue</h2>
            <span class="badge gray">manual continuation</span>
          </div>
          <div class="status-strip">
            <button class="queue-chip active" type="button" data-filter-state="all">All <b>{}</b></button>
            <button class="queue-chip attention" type="button" data-filter-state="attention">Needs attention <b>{}</b></button>
            <button class="queue-chip failed" type="button" data-filter-state="failed">Failed <b>{}</b></button>
            <button class="queue-chip approval" type="button" data-filter-state="approval">Approval <b>{}</b></button>
            <button class="queue-chip running" type="button" data-filter-state="running">Running <b>{}</b></button>
          </div>
          <table><thead><tr><th>ID / Folder</th><th>Title</th><th>Answer</th><th>State</th><th>Next</th><th>Mode</th><th></th></tr></thead><tbody id="work-items">{}</tbody></table>
        </section>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        serve_stylesheet(),
        items.len(),
        agents.len(),
        items.len(),
        queue_signals.attention,
        queue_signals.failed,
        queue_signals.approval,
        queue_signals.running,
        rows,
        serve_script()
    ))
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
    let items = list_work_items(root).map_err(|error| error.to_string())?;
    let domain_groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let domain_group_options = domain_group_select_options(&domain_groups, None, "Project default");
    let domain_options = domain_select_options(&domains, None, "Project default");
    let workflow_options = workflow_mode_options(Some(settings.default_progress_mode), false);
    let approval_options = approval_policy_options(Some(settings.approval_policy), false);
    Ok(format!(
        r#"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>New Work Item</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">Work Queue</a>
        <a href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>Create New Item</h1>
          <p class="muted">Add one work item; execution continues in the background</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/">Work Queue</a>
          <span class="badge blue">work {}</span>
        </div>
      </header>
      <section class="composer">
        <h2>New Work Item</h2>
        <form id="create-work-form">
          <label>Prompt<textarea name="description" rows="4" required placeholder="エージェントへの依頼内容"></textarea></label>
          <label>Work folder<input name="work_folder" placeholder="crates/nagare-core"></label>
          <label>Acceptance criteria<textarea name="acceptance" rows="3" placeholder="1行に1条件"></textarea></label>
          <details class="advanced-form" open>
            <summary>More context</summary>
            <label>Expected artifacts<textarea name="artifacts" rows="2" placeholder="README, tests, screenshots"></textarea></label>
            <label>Constraints<textarea name="constraints" rows="2" placeholder="破壊的操作を避ける、既存APIを維持する"></textarea></label>
            <label>Domain Group<select name="domain_group_id">{}</select></label>
            <label>Domain<select name="domain_id">{}</select></label>
            <label>Progress mode<select name="workflow_mode">{}</select></label>
            <label>Final approval<select name="approval_policy">{}</select></label>
          </details>
          <input type="hidden" name="max_steps" value="8">
          <input type="hidden" name="command" value="">
          <input type="hidden" name="review_command" value="">
          <button type="submit">Create Work Item</button>
          <p id="form-status" class="muted" role="status"></p>
        </form>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"#,
        serve_stylesheet(),
        items.len(),
        domain_group_options,
        domain_options,
        workflow_options,
        approval_options,
        serve_script()
    ))
}

pub(crate) fn render_serve_settings(root: &Path) -> Result<String, String> {
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let domain_groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let workflow_settings = get_workflow_settings(root).map_err(|error| error.to_string())?;
    let agent_rows = agent_profile_rows(&agents, &domain_groups, &domains);
    let group_rows = domain_group_rows(&domain_groups);
    let domain_rows = domain_profile_rows(&domains, &domain_groups);
    let workflow_form = render_workflow_settings_form(workflow_settings);
    Ok(format!(
        r##"<!doctype html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>Nagare Settings</title>
  <style>{}</style>
</head>
<body>
  <main class="app">
    <aside class="sidebar">
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">Work Queue</a>
        <a class="active" href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>Settings</h1>
          <p class="muted">Workflow policy, domains, and agent profiles</p>
        </div>
        <div class="actions">
          <a class="button-link" href="/settings/agents/new">Create New Agent</a>
          <span class="badge gray">agents {}</span>
          <span class="badge gray">groups {}</span>
          <span class="badge gray">domains {}</span>
          <span class="badge blue">profiles</span>
        </div>
      </header>
      <div class="settings-tabs" role="tablist" aria-label="Settings sections">
        <button class="settings-tab active" type="button" role="tab" data-settings-tab="workflow">Workflow</button>
        <button class="settings-tab" type="button" role="tab" data-settings-tab="domains">Domains</button>
        <button class="settings-tab" type="button" role="tab" data-settings-tab="agents">Agents</button>
      </div>
      <section class="settings-panel" data-settings-panel="workflow">
        {}
      </section>
      <section class="settings-panel" data-settings-panel="domains" hidden>
        <section class="panel">
          <div class="panel-head">
            <div>
              <h2>Domain Groups</h2>
              <p class="muted">Shared knowledge, common rubric, dispatch hints, and workflow defaults</p>
            </div>
            <a class="button-link secondary" href="/settings/domain-groups/new">Create New Domain Group</a>
          </div>
          <table><thead><tr><th>Group</th><th>Description</th><th>Shared knowledge</th><th>Rubric</th><th>Dispatch hints</th><th>Workflow</th><th>Source</th><th>Actions</th></tr></thead><tbody id="domain-groups">{}</tbody></table>
        </section>
        <section class="panel">
          <div class="panel-head">
            <div>
              <h2>Domains</h2>
              <p class="muted">Domain Group membership, artifact types, rubric, dispatch hints, and workflow overrides</p>
            </div>
            <a class="button-link secondary" href="/settings/domains/new">Create New Domain</a>
          </div>
          <table><thead><tr><th>Domain</th><th>Group</th><th>Description</th><th>Rubric</th><th>Dispatch hints</th><th>Workflow</th><th>Source</th><th>Actions</th></tr></thead><tbody id="domain-profiles">{}</tbody></table>
        </section>
      </section>
      <section class="panel settings-panel" data-settings-panel="agents" hidden>
        <div class="panel-head">
          <h2>Agents</h2>
          <span class="badge gray">registered</span>
        </div>
        <table><thead><tr><th>Agent</th><th>Type</th><th>Domain scope</th><th>Workdir</th><th>Instruction</th><th>Source</th></tr></thead><tbody id="agent-profiles">{}</tbody></table>
      </section>
    </section>
  </main>
  <script>{}</script>
</body>
</html>"##,
        serve_stylesheet(),
        agents.len(),
        domain_groups.len(),
        domains.len(),
        workflow_form,
        group_rows,
        domain_rows,
        agent_rows,
        serve_script()
    ))
}

fn domain_group_rows(groups: &[nagare_core::DomainGroup]) -> String {
    if groups.is_empty() {
        return "<tr><td colspan=\"8\" class=\"muted\">No domain groups registered.</td></tr>"
            .to_string();
    }
    let mut sorted_groups = groups.iter().collect::<Vec<_>>();
    sorted_groups.sort_by_key(|group| group.display_name.as_str());
    sorted_groups
        .into_iter()
        .map(|group| {
            format!(
            r#"<tr>
  <td><a href="/settings/domain-groups/{}">{}</a><div class="muted">{}</div></td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td><div class="row-actions"><a class="button-link secondary" href="/settings/domain-groups/{}">Edit</a><form class="delete-domain-group-form" data-domain-group-id="{}" data-domain-group-name="{}"><button class="danger" type="submit">Delete</button></form></div></td>
</tr>"#,
                h(&group.id),
                h(&group.display_name),
                h(&group.id),
                h(&compact_instruction(&group.description)),
                h(&group.shared_knowledge.len().to_string()),
                h(&group.common_rubric.len().to_string()),
                h(&group.dispatch_hints.len().to_string()),
                h(&domain_group_workflow_label(group)),
                h(&group.source.to_string()),
                h(&group.id),
                h(&group.id),
                h(&group.display_name)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn domain_profile_rows(
    domains: &[nagare_core::DomainProfile],
    groups: &[nagare_core::DomainGroup],
) -> String {
    if domains.is_empty() {
        return "<tr><td colspan=\"8\" class=\"muted\">No domains registered.</td></tr>"
            .to_string();
    }
    let mut domains = domains.iter().collect::<Vec<_>>();
    domains.sort_by_key(|domain| domain.display_name.as_str());
    domains
        .into_iter()
        .map(|domain| {
            format!(
            r#"<tr>
  <td><a href="/settings/domains/{}">{}</a><div class="muted">{}</div></td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td><div class="row-actions"><a class="button-link secondary" href="/settings/domains/{}">Edit</a><form class="delete-domain-form" data-domain-id="{}" data-domain-name="{}"><button class="danger" type="submit">Delete</button></form></div></td>
</tr>"#,
                h(&domain.id),
                h(&domain.display_name),
                h(&domain.id),
                h(&domain_group_label(groups, domain.group_id.as_deref())),
                h(&compact_instruction(&domain.description)),
                h(&domain.rubric.len().to_string()),
                h(&domain.dispatch_hints.len().to_string()),
                h(&domain_workflow_label(domain)),
                h(&domain.source.to_string()),
                h(&domain.id),
                h(&domain.id),
                h(&domain.display_name)
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

fn render_workflow_settings_form(settings: WorkflowSettings) -> String {
    format!(
        r#"<section class="panel">
        <div class="panel-head">
          <h2>Workflow</h2>
          <span class="badge gray">project default</span>
        </div>
        <form id="workflow-settings-form" data-action="/api/workflow-settings">
          <div class="form-grid">
            <label>Progress mode<select name="default_progress_mode">{}</select></label>
            <label>Final approval<select name="approval_policy">{}</select></label>
          </div>
          <button type="submit">Save Workflow Settings</button>
          <p id="workflow-settings-status" class="muted" role="status"></p>
        </form>
      </section>"#,
        workflow_mode_options(Some(settings.default_progress_mode), false),
        approval_policy_options(Some(settings.approval_policy), false)
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

fn workflow_mode_options(selected: Option<WorkflowMode>, include_inherit: bool) -> String {
    let mut options = Vec::new();
    if include_inherit {
        options.push(r#"<option value="">Inherit project default</option>"#.to_string());
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
            h(&mode.to_string())
        ));
    }
    options.join("")
}

fn approval_policy_options(selected: Option<ApprovalPolicy>, include_inherit: bool) -> String {
    let mut options = Vec::new();
    if include_inherit {
        options.push(r#"<option value="">Inherit project default</option>"#.to_string());
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
            h(&policy.to_string())
        ));
    }
    options.join("")
}

fn domain_workflow_label(domain: &nagare_core::DomainProfile) -> String {
    let mode = domain
        .workflow
        .progress_mode
        .map(|mode| mode.to_string())
        .unwrap_or_else(|| "inherit".to_string());
    let approval = domain
        .workflow
        .approval_policy
        .map(|policy| policy.to_string())
        .unwrap_or_else(|| "inherit".to_string());
    format!("{mode} / {approval}")
}

fn domain_group_workflow_label(group: &nagare_core::DomainGroup) -> String {
    let mode = group
        .workflow
        .progress_mode
        .map(|mode| mode.to_string())
        .unwrap_or_else(|| "inherit".to_string());
    let approval = group
        .workflow
        .approval_policy
        .map(|policy| policy.to_string())
        .unwrap_or_else(|| "inherit".to_string());
    format!("{mode} / {approval}")
}

fn agent_profile_rows(
    agents: &[nagare_core::AgentProfile],
    groups: &[nagare_core::DomainGroup],
    domains: &[nagare_core::DomainProfile],
) -> String {
    if agents.is_empty() {
        return "<tr><td colspan=\"6\" class=\"muted\">No agents registered.</td></tr>".to_string();
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
            format!(
                r#"<tr>
  <td><a href="/settings/agents/{}">{}</a><div class="muted">{}</div></td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
  <td>{}</td>
</tr>"#,
                h(&agent.id),
                h(&agent.display_name),
                h(&agent.id),
                h(&agent_kind_label(&agent.runtime, &agent.adapter)),
                h(&agent_domain_scope_label(agent, groups, domains)),
                h(&agent.working_dir),
                h(&compact_instruction(&agent.description)),
                h(&agent.source.to_string())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn agent_domain_scope_label(
    agent: &nagare_core::AgentProfile,
    groups: &[nagare_core::DomainGroup],
    domains: &[nagare_core::DomainProfile],
) -> String {
    let mut parts = Vec::new();
    if !agent.domain_group_ids.is_empty() {
        let labels = agent
            .domain_group_ids
            .iter()
            .map(|id| domain_group_label(groups, Some(id)))
            .collect::<Vec<_>>()
            .join(", ");
        parts.push(format!("groups: {labels}"));
    }
    if !agent.domain_ids.is_empty() {
        let labels = agent
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
            .join(", ");
        parts.push(format!("domains: {labels}"));
    }
    if parts.is_empty() {
        "any".to_string()
    } else {
        parts.join(" / ")
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

fn agent_kind_label(runtime: &str, adapter: &str) -> String {
    match (runtime, adapter) {
        ("codex-local", "process.codex-cli") | ("codex", "process.codex-cli") => {
            "Codex CLI".to_string()
        }
        ("codex-app-local", "stdio.codex-app-server")
        | ("codex-app-server", "stdio.codex-app-server") => "Codex App Server".to_string(),
        _ => format!("{runtime} / {adapter}"),
    }
}

fn agent_kind_value(runtime: &str, adapter: &str) -> &'static str {
    match (runtime, adapter) {
        ("codex-app-local", "stdio.codex-app-server")
        | ("codex-app-server", "stdio.codex-app-server") => "codex_app_server",
        _ => "codex_cli",
    }
}

fn compact_instruction(instruction: &str) -> String {
    let instruction = instruction.split_whitespace().collect::<Vec<_>>().join(" ");
    if instruction.chars().count() <= 96 {
        return instruction;
    }
    let mut compact = instruction.chars().take(96).collect::<String>();
    compact.push_str("...");
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
    let groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domain = match domain_id {
        Some(id) => Some(get_domain_profile(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = domain.is_none();
    let domain = domain.as_ref();
    let title = if is_new {
        "Create New Domain".to_string()
    } else {
        format!(
            "Edit Domain: {}",
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
        "No group",
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
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">Work Queue</a>
        <a class="active" href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">Configure domain rubric and dispatch hints</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">Settings</a>
        </div>
      </header>
      <section class="composer">
        <form id="domain-profile-form" data-action="{}" data-redirect="/settings#domains">
          {}
          <label>Domain Group<select name="group_id">{}</select></label>
          <label>Name<input name="display_name" required value="{}"></label>
          <label>Description<textarea name="description" rows="4" placeholder="このドメインが扱う作成物や判断対象">{}</textarea></label>
          <label>Artifact types<textarea name="artifact_types" rows="3" placeholder="1行に1種類。例: html, ui screenshot, rust cli">{}</textarea></label>
          <label>Rubric<textarea name="rubric" rows="7" placeholder="1行に1基準。例: 主要導線が迷わず使える">{}</textarea></label>
          <label>Dispatch hints<textarea name="dispatch_hints" rows="4" placeholder="1行に1ヒント。例: UI変更ならfrontend-ui domainを候補にする">{}</textarea></label>
          <div class="form-grid">
            <label>Progress mode override<select name="workflow_progress_mode">{}</select></label>
            <label>Final approval override<select name="workflow_approval_policy">{}</select></label>
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
        h(&title),
        h(&action),
        id_field,
        group_options,
        h(display_name),
        h(description),
        h(&artifact_types),
        h(&rubric),
        h(&dispatch_hints),
        workflow_mode_options(progress_mode, true),
        approval_policy_options(approval_policy, true),
        if is_new {
            "Create Domain"
        } else {
            "Save Domain"
        },
        serve_script()
    ))
}

pub(crate) fn render_serve_domain_group_form(
    root: &Path,
    group_id: Option<&str>,
) -> Result<String, String> {
    let group = match group_id {
        Some(id) => Some(get_domain_group(root, id).map_err(|error| error.to_string())?),
        None => None,
    };
    let is_new = group.is_none();
    let group = group.as_ref();
    let title = if is_new {
        "Create New Domain Group".to_string()
    } else {
        format!(
            "Edit Domain Group: {}",
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
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">Work Queue</a>
        <a class="active" href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">Configure shared knowledge, rubric, and workflow defaults</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">Settings</a>
        </div>
      </header>
      <section class="composer">
        <form id="domain-group-form" data-action="{}" data-redirect="/settings#domains">
          {}
          <label>Name<input name="display_name" required value="{}"></label>
          <label>Description<textarea name="description" rows="4" placeholder="このグループに含めるドメインの共通知識">{}</textarea></label>
          <label>Shared knowledge<textarea name="shared_knowledge" rows="4" placeholder="1行に1知識。例: 変更は小さく検証可能にする">{}</textarea></label>
          <label>Common rubric<textarea name="common_rubric" rows="7" placeholder="1行に1基準。例: 主要な品質基準を満たす">{}</textarea></label>
          <label>Dispatch hints<textarea name="dispatch_hints" rows="4" placeholder="1行に1ヒント。例: UIならFrontend UI Domainを優先">{}</textarea></label>
          <div class="form-grid">
            <label>Progress mode default<select name="workflow_progress_mode">{}</select></label>
            <label>Final approval default<select name="workflow_approval_policy">{}</select></label>
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
        h(&title),
        h(&action),
        id_field,
        h(display_name),
        h(description),
        h(&shared_knowledge),
        h(&common_rubric),
        h(&dispatch_hints),
        workflow_mode_options(progress_mode, true),
        approval_policy_options(approval_policy, true),
        if is_new {
            "Create Domain Group"
        } else {
            "Save Domain Group"
        },
        serve_script()
    ))
}

pub(crate) fn render_serve_agent_form(
    root: &Path,
    agent_id: Option<&str>,
) -> Result<String, String> {
    let agents = list_agent_profiles(root).map_err(|error| error.to_string())?;
    let groups = list_domain_groups(root).map_err(|error| error.to_string())?;
    let domains = list_domain_profiles(root).map_err(|error| error.to_string())?;
    let agent = agent_id.and_then(|id| agents.iter().find(|agent| agent.id == id));
    let is_new = agent.is_none();
    let title = if is_new {
        "Create New Agent".to_string()
    } else {
        format!("Edit Agent: {}", agent.unwrap().display_name)
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
    let selected_domain_group_ids = agent
        .map(|agent| agent.domain_group_ids.clone())
        .unwrap_or_default();
    let selected_domain_ids = agent
        .map(|agent| agent.domain_ids.clone())
        .unwrap_or_default();
    let domain_group_options = domain_group_multi_options(&groups, &selected_domain_group_ids);
    let domain_options = domain_multi_options(&domains, &selected_domain_ids);
    let kind = agent
        .map(|agent| agent_kind_value(&agent.runtime, &agent.adapter))
        .unwrap_or("codex_cli");
    let action = if is_new {
        "/api/agents".to_string()
    } else {
        format!("/api/agents/{}", h(id_value))
    };
    let delete_button = if is_new {
        String::new()
    } else {
        format!(
            r#"<button class="danger" type="button" id="delete-agent-button" data-action="/api/agents/{}/delete" data-agent-name="{}">Delete Agent</button>"#,
            h(id_value),
            h(display_name)
        )
    };
    let id_field = if is_new {
        format!(
            r#"<label>ID<input name="id" required placeholder="my-agent" value="{}"></label>"#,
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
      <h1 class="brand"><img class="brand-logo" src="/assets/logo.png" alt=""><span class="brand-text">Nagare</span></h1>
      <nav>
        <a href="/">Work Queue</a>
        <a class="active" href="/settings">Settings</a>
      </nav>
    </aside>
    <section class="content">
      <header class="topbar">
        <div>
          <h1>{}</h1>
          <p class="muted">Configure this agent profile</p>
        </div>
        <div class="actions">
          <a class="button-link secondary" href="/settings">Agents</a>
        </div>
      </header>
      <section class="composer">
        <form id="agent-profile-form" data-action="{}" data-redirect="/settings#agents">
          {}
          <label>External Agent Type
            <select name="agent_kind" id="agent-kind">
              <option value="codex_cli"{}>Codex CLI</option>
              <option value="codex_app_server"{}>Codex App Server</option>
            </select>
          </label>
          <input type="hidden" name="runtime" value="">
          <input type="hidden" name="adapter" value="">
          <label>Display Name<input name="display_name" required value="{}"></label>
          <label>Role<input name="role" placeholder="planner / worker / reviewer" value="{}"></label>
          <label>Workdir<select name="working_dir">{}</select></label>
          <label>Domain Groups<select name="domain_group_ids" multiple size="4">{}</select></label>
          <label>Domains<select name="domain_ids" multiple size="5">{}</select></label>
          <label>Instructions<textarea name="description" rows="6">{}</textarea></label>
          <label>Specialties<textarea name="specialties" rows="2" placeholder="カンマまたは改行区切り">{}</textarea></label>
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
        h(&title),
        action,
        id_field,
        if kind == "codex_cli" { " selected" } else { "" },
        if kind == "codex_app_server" {
            " selected"
        } else {
            ""
        },
        h(display_name),
        h(role),
        workdir_options,
        domain_group_options,
        domain_options,
        h(&description),
        h(&specialties),
        if is_new { "Create Agent" } else { "Save Agent" },
        delete_button,
        serve_script()
    ))
}
