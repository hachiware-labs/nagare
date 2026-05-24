use std::env;
use std::fs;
use std::path::PathBuf;

use crate::*;

#[test]
fn layout_uses_nagare_directory() {
    let layout = ProjectLayout::new("repo");
    assert_eq!(layout.nagare_dir, PathBuf::from("repo").join(".nagare"));
    assert_eq!(
        layout.config_path,
        PathBuf::from("repo").join(".nagare").join("project.toml")
    );
    assert_eq!(
        layout.ledger_path,
        PathBuf::from("repo")
            .join(".nagare")
            .join("state")
            .join("ledger.json")
    );
    assert_eq!(
        layout.agents_dir,
        PathBuf::from("repo").join(".nagare").join("agents")
    );
}

#[test]
fn default_config_declares_initial_adapters() {
    let config = default_config();
    assert!(config.contains("process.codex-cli"));
    assert!(config.contains("stdio.codex-app-server"));
}

#[test]
fn first_scenario_reaches_done() {
    let root = env::temp_dir().join(format!("nagare-test-{}", timestamp()));
    let result = run_first_scenario(&root).expect("scenario should pass");
    assert_eq!(result.final_status, WorkItemStatus::Done);
    let snapshot =
        get_work_item_snapshot(&root, &result.work_item_id).expect("snapshot should load");
    assert_eq!(snapshot.runs.len(), 2);
    assert_eq!(snapshot.handoffs.len(), 1);
    assert_eq!(snapshot.verification_results.len(), 1);
    assert_eq!(snapshot.decisions.len(), 1);
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_can_be_registered_and_used_in_scenario() {
    let root = env::temp_dir().join(format!("nagare-agent-test-{}", timestamp()));
    let result = run_registered_agent_scenario(&root).expect("registered scenario should pass");
    assert_eq!(result.final_status, WorkItemStatus::Done);

    let profiles = list_agent_profiles(&root).expect("profiles should load");
    assert!(profiles.iter().any(|profile| profile.id == "codex-cli"));
    assert!(
        profiles
            .iter()
            .any(|profile| profile.id == "codex-impl-smoke")
    );

    let snapshot =
        get_work_item_snapshot(&root, &result.work_item_id).expect("snapshot should load");
    assert_eq!(snapshot.runs[0].agent_profile_id, "codex-impl-smoke");
    assert_eq!(snapshot.runs[0].adapter, "process.codex-cli");
    assert_eq!(snapshot.runs[1].agent_profile_id, "codex-app-smoke");
    assert_eq!(snapshot.runs[1].adapter, "stdio.codex-app-server");
    fs::remove_dir_all(root).ok();
}

#[test]
fn unknown_agent_profile_is_rejected() {
    let root = env::temp_dir().join(format!("nagare-unknown-agent-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Unknown profile", "")
        .expect("item should create")
        .item;
    let error = run_work_item(
        &root,
        &item.id,
        "missing-profile",
        scenario_command("should not run", true).as_str(),
    )
    .expect_err("unknown profile should be rejected");
    assert!(
        error
            .to_string()
            .contains("not found: agent profile `missing-profile`")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_probe_records_capability_snapshot() {
    let root = env::temp_dir().join(format!("nagare-probe-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let result = agent_probe(&root, "codex-cli").expect("probe should be recorded");
    assert_eq!(result.probe.agent_profile_id, "codex-cli");
    assert_eq!(result.probe.adapter_id, "process.codex-cli");
    assert!(
        result
            .probe
            .discovered_capabilities
            .contains(&"repo_read".to_string())
    );

    let layout = ProjectLayout::new(&root);
    let ledger = load_ledger(&layout).expect("ledger should load");
    assert_eq!(ledger.capability_probes.len(), 1);
    assert_eq!(ledger.capability_probes[0].id, result.probe.id);
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_auto_probes_missing_and_stale_capability_snapshot() {
    let root = env::temp_dir().join(format!("nagare-auto-probe-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Auto probe", "")
        .expect("item should create")
        .item;
    let command = scenario_command("auto probe", true);

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command.as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should create missing probe");

    let layout = ProjectLayout::new(&root);
    let mut ledger = load_ledger(&layout).expect("ledger should load");
    assert_eq!(ledger.capability_probes.len(), 1);
    let first_probe_id = ledger.capability_probes[0].id.clone();
    ledger.capability_probes[0].probed_at = "0".to_string();
    save_ledger(&layout, &ledger).expect("ledger should save");

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command.as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should refresh stale probe");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let ledger = load_ledger(&layout).expect("ledger should reload");
    assert_eq!(ledger.capability_probes.len(), 2);
    assert_ne!(ledger.capability_probes[1].id, first_probe_id);
    assert_eq!(
        snapshot.resolved_skill_contexts[1].capability_probe_id,
        Some(ledger.capability_probes[1].id.clone())
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_working_dir_is_used_for_runs() {
    let root = env::temp_dir().join(format!("nagare-working-dir-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let subdir = root.join("packages").join("app");
    fs::create_dir_all(&subdir).expect("subdir should be created");
    fs::write(subdir.join("marker.txt"), "ok").expect("marker should be written");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-subdir",
            display_name: "Codex Subdir",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: "packages/app",
            description: "",
            specialties: Vec::new(),
        },
    )
    .expect("profile should be added");
    let item = create_work_item(&root, "Check cwd", "")
        .expect("item should create")
        .item;
    let command = if cfg!(windows) {
        "if exist marker.txt (exit /B 0) else (exit /B 1)"
    } else {
        "test -f marker.txt"
    };
    let result = run_work_item(&root, &item.id, "codex-subdir", command)
        .expect("run should use profile cwd");
    assert_eq!(result.run.status, AgentRunStatus::Succeeded);
    assert!(result.run.command.contains("packages"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_routing_hints_are_persisted() {
    let root = env::temp_dir().join(format!("nagare-agent-hints-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "researcher",
            working_dir: ".",
            description: "Handles source gathering and synthesis.",
            specialties: vec!["research".to_string(), "synthesis".to_string()],
        },
    )
    .expect("profile should be added");

    let profile = get_agent_profile(&root, "research-agent").expect("profile should load");
    assert_eq!(
        profile.description,
        "Handles source gathering and synthesis."
    );
    assert_eq!(profile.specialties, vec!["research", "synthesis"]);
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_can_be_updated_as_project_local_override() {
    let root = env::temp_dir().join(format!("nagare-agent-update-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "draft-agent",
            display_name: "Draft Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "writer",
            working_dir: ".",
            description: "Initial profile.",
            specialties: vec!["drafting".to_string()],
        },
    )
    .expect("profile should be added");

    let updated = update_agent_profile(
        &root,
        "draft-agent",
        UpdateAgentProfileInput {
            display_name: Some("Research Writer"),
            role: Some("researcher"),
            working_dir: Some("."),
            description: Some("Research and writing profile."),
            specialties: Some(vec!["research".to_string(), "writing".to_string()]),
        },
    )
    .expect("profile should update");

    assert!(updated.path.ends_with(".nagare/agents/draft-agent.toml"));
    let profile = get_agent_profile(&root, "draft-agent").expect("profile should load");
    assert_eq!(profile.display_name, "Research Writer");
    assert_eq!(profile.role, "researcher");
    assert_eq!(profile.description, "Research and writing profile.");
    assert_eq!(profile.specialties, vec!["research", "writing"]);
    assert_eq!(profile.source, AgentProfileSource::ProjectAgentDirectory);
    fs::remove_dir_all(root).ok();
}

#[test]
fn nagare_agent_settings_can_select_default_work_agent() {
    let root = env::temp_dir().join(format!("nagare-agent-settings-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-work",
            display_name: "Codex Work",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
        },
    )
    .expect("profile should be added");

    let settings = set_nagare_agent_settings(
        &root,
        SetNagareAgentSettingsInput {
            work_agent: Some("codex-work"),
            review_agent: None,
            dispatch_agent: Some("codex-work"),
        },
    )
    .expect("settings should update");
    assert_eq!(settings.work_agent, "codex-work");
    assert_eq!(settings.review_agent, "codex-app-server");
    assert_eq!(settings.dispatch_agent, "codex-work");

    let loaded = get_nagare_agent_settings(&root).expect("settings should load");
    assert_eq!(loaded.work_agent, "codex-work");
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_and_review_runs_do_not_advance_item_status() {
    let root = env::temp_dir().join(format!("nagare-purpose-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Route and review", "")
        .expect("item should create")
        .item;
    let command = scenario_command("agent purpose run", true);

    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command.as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should run");
    assert_eq!(preview.run.purpose, AgentRunPurpose::DispatchPreview);
    assert_eq!(preview.item_status, WorkItemStatus::Ready);
    assert!(preview.dispatch_plan_id.is_some());

    let review = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-app-server",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command.as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should run");
    assert_eq!(review.run.purpose, AgentRunPurpose::Review);
    assert_eq!(review.item_status, WorkItemStatus::Ready);

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    assert_eq!(snapshot.item.status, WorkItemStatus::Ready);
    assert_eq!(snapshot.runs.len(), 2);
    assert_eq!(snapshot.runs[0].purpose, AgentRunPurpose::DispatchPreview);
    assert_eq!(snapshot.runs[1].purpose, AgentRunPurpose::Review);
    assert_eq!(snapshot.dispatch_plans.len(), 1);
    assert_eq!(snapshot.dispatch_plans[0].agent_run_id, preview.run.id);
    assert_eq!(snapshot.dispatch_plans[0].summary, "agent purpose run");
    fs::remove_dir_all(root).ok();
}

#[test]
fn handoff_dispatch_uses_same_plan_lifecycle() {
    let root = env::temp_dir().join(format!("nagare-handoff-dispatch-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "repair-agent",
            display_name: "Repair Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
            description: "Handles repair work.",
            specialties: vec!["repair".to_string()],
        },
    )
    .expect("profile should be added");
    let item = create_work_item(&root, "Handoff dispatch", "")
        .expect("item should create")
        .item;
    create_handoff(
        &root,
        &item.id,
        "codex-cli",
        "repair-agent",
        "Initial agent failed",
        "Use repair profile for retry.",
    )
    .expect("handoff should create");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"repair-agent","summary":"Retry with repair agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch output should write");
    let command = if cfg!(windows) {
        "type dispatch.json"
    } else {
        "cat dispatch.json"
    };

    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("handoff dispatch should create plan");
    let dispatch_plan_id = preview.dispatch_plan_id.expect("plan should exist");
    let accepted = accept_dispatch_plan(&root, &item.id, Some(&dispatch_plan_id))
        .expect("handoff dispatch plan should accept")
        .plan;
    assert_eq!(accepted.status, DispatchPlanStatus::Accepted);
    assert_eq!(accepted.target_agent_profile_id, "repair-agent");

    let selection = select_agent_for_work_item_run(
        &root,
        &item.id,
        SelectRunAgentInput {
            explicit_agent_profile_id: None,
            dispatch_plan_id: None,
            path: None,
        },
    )
    .expect("accepted handoff dispatch should select repair agent");
    assert_eq!(selection.agent_profile_id, "repair-agent");
    assert_eq!(selection.source, RunAgentSelectionSource::DispatchPlan);
    fs::remove_dir_all(root).ok();
}

#[test]
fn accepted_dispatch_plan_selects_target_for_work_run() {
    let root = env::temp_dir().join(format!("nagare-dispatch-accept-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "researcher",
            working_dir: ".",
            description: "Research and source synthesis.",
            specialties: vec!["research".to_string()],
        },
    )
    .expect("profile should be added");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[[project_rules]]
id = "docs-research"
match = ["docs/**"]
default_agent = "research-agent"
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");

    let item = create_work_item(&root, "Research documentation", "")
        .expect("item should create")
        .item;
    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: Some("docs/topic.md"),
            prompt: None,
            dev_command: Some(scenario_command("dispatch ok", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should run");
    let dispatch_plan_id = preview.dispatch_plan_id.expect("plan should exist");

    let accepted = accept_dispatch_plan(&root, &item.id, Some(&dispatch_plan_id))
        .expect("plan should be accepted")
        .plan;
    assert_eq!(accepted.status, DispatchPlanStatus::Accepted);
    assert_eq!(accepted.target_agent_profile_id, "research-agent");

    let selection = select_agent_for_work_item_run(
        &root,
        &item.id,
        SelectRunAgentInput {
            explicit_agent_profile_id: None,
            dispatch_plan_id: None,
            path: None,
        },
    )
    .expect("accepted plan should select run agent");
    assert_eq!(selection.agent_profile_id, "research-agent");
    assert_eq!(selection.source, RunAgentSelectionSource::DispatchPlan);
    assert_eq!(
        selection.dispatch_plan_id.as_deref(),
        Some(dispatch_plan_id.as_str())
    );

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: selection.agent_profile_id.as_str(),
            dispatch_plan_id: selection.dispatch_plan_id.as_deref(),
            path: None,
            prompt: None,
            dev_command: Some(scenario_command("accepted dispatch run", true).as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work run should use accepted dispatch plan");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let packet = snapshot
        .resolved_run_packets
        .iter()
        .find(|packet| packet.purpose == AgentRunPurpose::Work)
        .expect("work run packet should exist");
    assert_eq!(
        packet.dispatch_plan_id.as_deref(),
        Some(dispatch_plan_id.as_str())
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_agent_json_can_choose_between_writing_and_research_agents() {
    let root = env::temp_dir().join(format!("nagare-dispatch-json-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "writing-agent",
            display_name: "Writing Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "writer",
            working_dir: ".",
            description: "Drafts and edits user-facing prose.",
            specialties: vec!["writing".to_string(), "editing".to_string()],
        },
    )
    .expect("writing profile should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "researcher",
            working_dir: ".",
            description: "Collects sources and synthesizes findings.",
            specialties: vec!["research".to_string(), "synthesis".to_string()],
        },
    )
    .expect("research profile should be added");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[[project_rules]]
id = "docs-writing-default"
match = ["docs/**"]
default_agent = "writing-agent"
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"research-agent","summary":"Research is required before writing.","risks":["source quality"],"missing_information":["source list"]}"#,
    )
    .expect("dispatch output should write");

    let item = create_work_item(&root, "Research before writing", "")
        .expect("item should create")
        .item;
    let command = if cfg!(windows) {
        "type dispatch.json"
    } else {
        "cat dispatch.json"
    };
    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: Some("docs/topic.md"),
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should run");
    let dispatch_plan_id = preview.dispatch_plan_id.expect("plan should exist");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    assert_eq!(snapshot.dispatch_plans[0].status, DispatchPlanStatus::Draft);
    assert_eq!(
        snapshot.dispatch_plans[0].target_agent_profile_id,
        "research-agent"
    );
    assert_eq!(
        snapshot.dispatch_plans[0].summary,
        "Research is required before writing."
    );
    assert_eq!(snapshot.dispatch_plans[0].risks, vec!["source quality"]);
    assert_eq!(
        snapshot.dispatch_plans[0].missing_information,
        vec!["source list"]
    );

    accept_dispatch_plan(&root, &item.id, Some(&dispatch_plan_id))
        .expect("plan should be accepted");
    let selection = select_agent_for_work_item_run(
        &root,
        &item.id,
        SelectRunAgentInput {
            explicit_agent_profile_id: None,
            dispatch_plan_id: None,
            path: Some("docs/topic.md"),
        },
    )
    .expect("accepted plan should beat rule fallback");
    assert_eq!(selection.agent_profile_id, "research-agent");
    assert_eq!(selection.source, RunAgentSelectionSource::DispatchPlan);
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_contract_fallback_records_selection_warnings() {
    let root = env::temp_dir().join(format!("nagare-dispatch-contract-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Invalid dispatch output", "")
        .expect("item should create")
        .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"summary":"No target provided."}"#,
    )
    .expect("dispatch output should write");
    let command = if cfg!(windows) {
        "type dispatch.json"
    } else {
        "cat dispatch.json"
    };

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: Some("README.md"),
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should fallback");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(plan.target_agent_profile_id, "codex-cli");
    assert_eq!(plan.summary, "No target provided.");
    assert!(plan.selection_warnings.iter().any(|warning| {
        warning.contains("missing required target_agent_profile_id")
            && warning.contains("fallback target `codex-cli`")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_plan_suggestion_parses_agent_json() {
    let output = r#"item/completed: {"params":{"item":{"text":"```json\n{\"target_agent_profile_id\":\"research-agent\",\"summary\":\"Use the research agent.\",\"risks\":[\"needs sources\"],\"missing_information\":[\"source list\"]}\n```"}}}"#;
    let suggestion = parse_dispatch_plan_suggestion(output).expect("suggestion should parse");

    assert_eq!(
        suggestion.target_agent_profile_id.as_deref(),
        Some("research-agent")
    );
    assert_eq!(
        suggestion.summary.as_deref(),
        Some("Use the research agent.")
    );
    assert_eq!(suggestion.risks, vec!["needs sources"]);
    assert_eq!(suggestion.missing_information, vec!["source list"]);
}

#[test]
fn project_rule_resolution_selects_most_specific_rule() {
    let root = env::temp_dir().join(format!("nagare-rule-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-rust",
            display_name: "Codex Rust",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
        },
    )
    .expect("profile should be added");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_sets.rust-core]
paths = ["skills/rust-core"]
required_capabilities = ["repo_read"]
optional_capabilities = ["shell_command"]

[[project_rules]]
id = "rust-core"
match = ["crates/**"]
default_agent = "codex-rust"
review_agent = "codex-app-server"
skill_sets = ["rust-core"]
permission_policy = "medium-code-task"
workspace_policy = "project-root"
verification = ["cargo test --workspace"]
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");

    let rust_resolution = resolve_rule_for_path(&root, Some("crates/nagare-core/src/lib.rs"), None)
        .expect("rule should resolve");
    assert_eq!(
        rust_resolution.matched_rule_id.as_deref(),
        Some("rust-core")
    );
    assert_eq!(rust_resolution.agent_profile_id, "codex-rust");
    assert_eq!(rust_resolution.skill_set_ids, vec!["rust-core".to_string()]);
    assert_eq!(
        rust_resolution.verification,
        vec!["cargo test --workspace".to_string()]
    );

    let default_resolution =
        resolve_rule_for_path(&root, Some("README.md"), None).expect("rule should resolve");
    assert_eq!(
        default_resolution.matched_rule_id.as_deref(),
        Some("default")
    );
    assert_eq!(default_resolution.agent_profile_id, "codex-cli");
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_with_path_records_resolved_skill_context_and_run_packet() {
    let root = env::temp_dir().join(format!("nagare-run-packet-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Resolve packet", "")
        .expect("item should create")
        .item;
    let command = scenario_command("resolved packet", true);
    let result = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: Some("README.md"),
            prompt: None,
            dev_command: Some(command.as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed");
    assert_eq!(result.run.purpose, AgentRunPurpose::DispatchPreview);
    assert!(result.dispatch_plan_id.is_some());

    let layout = ProjectLayout::new(&root);
    let ledger = load_ledger(&layout).expect("ledger should load");
    assert_eq!(ledger.resolved_skill_contexts.len(), 1);
    assert_eq!(ledger.resolved_run_packets.len(), 1);
    assert_eq!(ledger.dispatch_plans.len(), 1);
    let context = &ledger.resolved_skill_contexts[0];
    let packet = &ledger.resolved_run_packets[0];
    assert_eq!(context.agent_profile_id, "codex-cli");
    assert_eq!(context.project_rule_ids, vec!["default".to_string()]);
    assert_eq!(
        context.applied_skill_set_ids,
        vec!["repo-default".to_string()]
    );
    assert_eq!(packet.resolved_skill_context_id, context.id);
    assert_eq!(packet.agent_profile_id, "codex-cli");
    assert_eq!(packet.adapter_id, "process.codex-cli");
    assert_eq!(packet.purpose, AgentRunPurpose::DispatchPreview);
    assert_eq!(packet.goal, "Resolve packet");
    assert!(packet.working_dir.contains("nagare-run-packet-test"));
    assert_eq!(packet.path.as_deref(), Some("README.md"));
    assert_eq!(packet.project_rule_ids, vec!["default".to_string()]);
    let plan = &ledger.dispatch_plans[0];
    assert_eq!(plan.resolved_run_packet_id, packet.id);
    assert_eq!(plan.dispatch_agent_profile_id, "codex-cli");
    assert_eq!(plan.target_agent_profile_id, "codex-cli");
    assert_eq!(plan.path.as_deref(), Some("README.md"));
    assert!(
        layout
            .artifacts_dir
            .join(format!("{}.json", context.id))
            .exists()
    );
    assert!(
        layout
            .artifacts_dir
            .join(format!("{}.json", packet.id))
            .exists()
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_records_skipped_skill_sets_when_required_capabilities_are_missing() {
    let root = env::temp_dir().join(format!("nagare-skill-skip-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_sets.network-only]
paths = ["skills/network-only"]
required_capabilities = ["network_access"]
optional_capabilities = []

[[project_rules]]
id = "network-skill"
match = ["secure/**"]
default_agent = "codex-cli"
skill_sets = ["network-only"]
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");

    let item = create_work_item(&root, "Resolve unavailable skill", "")
        .expect("item should create")
        .item;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
            dispatch_plan_id: None,
            path: Some("secure/config.rs"),
            prompt: None,
            dev_command: Some(scenario_command("skill skip", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed with skipped skill recorded");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let context = &snapshot.resolved_skill_contexts[0];
    assert_eq!(
        context.declared_skill_set_ids,
        vec!["network-only".to_string()]
    );
    assert!(context.applied_skill_set_ids.is_empty());
    assert_eq!(
        context.skipped_skill_set_ids,
        vec!["network-only".to_string()]
    );
    assert!(
        snapshot.resolved_run_packets[0]
            .constraints
            .iter()
            .any(|constraint| constraint.contains("network_access"))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn locale_is_recorded_and_used_for_generated_evidence() {
    let root = env::temp_dir().join(format!("nagare-locale-test-{}", timestamp()));
    init_project(&root).expect("project should init");
    set_locale_settings(
        &root,
        SetLocaleInput {
            language: Some("ja-JP"),
            timezone: Some("Asia/Tokyo"),
        },
    )
    .expect("locale should update");
    let item = create_work_item(&root, "Locale check", "")
        .expect("item should create")
        .item;
    let result = run_work_item(
        &root,
        &item.id,
        "codex-cli",
        scenario_command("locale run", true).as_str(),
    )
    .expect("run should succeed");
    assert_eq!(result.run.locale, "ja-JP");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    assert_eq!(snapshot.item.locale, "ja-JP");
    assert_eq!(snapshot.evidence[0].locale, "ja-JP");
    assert!(snapshot.evidence[0].claim.contains("成功"));
    fs::remove_dir_all(root).ok();
}
