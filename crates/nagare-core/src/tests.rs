use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::*;

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root(label: &str) -> PathBuf {
    let count = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!("nagare-{label}-{}-{count}", std::process::id()))
}

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
    assert!(config.contains("process.claude-code"));
    assert!(config.contains("process.opencode"));
    assert!(config.contains("process.openclaw-agent"));
    assert!(config.contains("role = \"worker\""));
    assert!(config.contains("role = \"reviewer\""));
    assert!(config.contains("role = \"organizer\""));
    assert!(!config.contains("role = \"dispatcher\""));
    assert!(!config.contains("role = \"supervisor\""));
}

#[test]
fn default_agents_use_localized_general_display_names() {
    let japanese = I18n::new("ja-JP");
    let english = I18n::new("en-US");

    assert_eq!(
        japanese.agent_default_name("organizer"),
        "汎用オーガナイザー"
    );
    assert_eq!(japanese.agent_default_name("worker"), "汎用ワーカー");
    assert_eq!(japanese.agent_default_name("reviewer"), "汎用レビュアー");
    assert_eq!(english.agent_default_name("organizer"), "General Organizer");
    assert_eq!(english.agent_default_name("worker"), "General Worker");
    assert_eq!(english.agent_default_name("reviewer"), "General Reviewer");
}

#[test]
fn runtime_mcp_capability_table_drives_agent_choices() {
    let codex = runtime_mcp_capability(AgentToolKind::Codex);
    assert_eq!(codex.scope, RuntimeMcpScope::Project);
    assert!(codex.agent_assignable);

    let codex_cli = runtime_mcp_capability(AgentToolKind::CodexCli);
    assert_eq!(codex_cli.scope, RuntimeMcpScope::Project);
    assert!(codex_cli.agent_assignable);

    let claude = runtime_mcp_capability(AgentToolKind::ClaudeCode);
    assert_eq!(claude.scope, RuntimeMcpScope::Project);
    assert!(!claude.agent_assignable);

    let opencode = runtime_mcp_capability(AgentToolKind::OpenCode);
    assert_eq!(opencode.scope, RuntimeMcpScope::Project);
    assert!(!opencode.agent_assignable);

    let openclaw = runtime_mcp_capability(AgentToolKind::OpenClaw);
    assert_eq!(openclaw.scope, RuntimeMcpScope::GlobalOnly);
    assert!(!openclaw.agent_assignable);
}

#[test]
fn mcp_connection_crud_records_test_status() {
    let root = test_root("mcp-connection-crud");
    init_project(&root).expect("project should init");

    let connection = add_mcp_connection(
        &root,
        AddMcpConnectionInput {
            id: "rustc-version",
            display_name: "Rustc Version MCP",
            tool_kind: AgentToolKind::CodexCli,
            command: "rustc",
            args: Vec::new(),
            env: Default::default(),
            test_args: vec!["--version".to_string()],
        },
    )
    .expect("connection should add");
    assert_eq!(connection.last_test_status, "");
    assert!(!connection.agent_assignable);

    let tested = test_mcp_connection(&root, "rustc-version").expect("connection should test");
    assert!(tested.success, "{}", tested.detail);
    assert_eq!(tested.connection.last_test_status, "passed");
    assert!(tested.connection.agent_assignable);

    let listed = list_mcp_connections(&root).expect("connections should list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "rustc-version");
    assert_eq!(listed[0].last_test_status, "passed");

    let deleted = delete_mcp_connection(&root, "rustc-version").expect("connection should delete");
    assert_eq!(deleted.id, "rustc-version");
    assert!(
        list_mcp_connections(&root)
            .expect("connections should list")
            .is_empty()
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn tested_mcp_connection_can_be_assigned_to_agent_and_detaches_on_delete() {
    let root = test_root("mcp-agent-assignment");
    init_project(&root).expect("project should init");

    add_mcp_connection(
        &root,
        AddMcpConnectionInput {
            id: "rustc-version",
            display_name: "Rustc Version MCP",
            tool_kind: AgentToolKind::CodexCli,
            command: "rustc",
            args: Vec::new(),
            env: Default::default(),
            test_args: vec!["--version".to_string()],
        },
    )
    .expect("connection should add");

    let untested_assignment = add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-with-untested-mcp",
            display_name: "Codex Untested MCP",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: vec!["rustc-version".to_string()],
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    );
    assert!(
        untested_assignment.is_err(),
        "untested MCP connections must not be assignable"
    );

    let tested = test_mcp_connection(&root, "rustc-version").expect("connection should test");
    assert!(tested.success, "{}", tested.detail);

    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-with-mcp",
            display_name: "Codex With MCP",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: vec!["rustc-version".to_string()],
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("tested MCP should assign to agent");

    let agent = get_agent_profile(&root, "codex-with-mcp").expect("agent should load");
    assert_eq!(agent.mcp_connection_ids, vec!["rustc-version".to_string()]);

    delete_mcp_connection(&root, "rustc-version").expect("connection should delete");
    let agent = get_agent_profile(&root, "codex-with-mcp").expect("agent should load");
    assert!(agent.mcp_connection_ids.is_empty());

    fs::remove_dir_all(root).ok();
}

#[test]
fn failed_mcp_connection_delete_restores_agent_assignments() {
    let root = test_root("mcp-delete-rollback");
    init_project(&root).expect("project should init");

    add_mcp_connection(
        &root,
        AddMcpConnectionInput {
            id: "rustc-version",
            display_name: "Rustc Version MCP",
            tool_kind: AgentToolKind::CodexCli,
            command: "rustc",
            args: Vec::new(),
            env: Default::default(),
            test_args: vec!["--version".to_string()],
        },
    )
    .expect("connection should add");
    let tested = test_mcp_connection(&root, "rustc-version").expect("connection should test");
    assert!(tested.success, "{}", tested.detail);

    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-with-mcp",
            display_name: "Codex With MCP",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: vec!["rustc-version".to_string()],
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("tested MCP should assign to agent");

    let layout = ProjectLayout::new(&root);
    let mut permissions = fs::metadata(&layout.config_path)
        .expect("config metadata")
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&layout.config_path, permissions).expect("config should become readonly");

    let result = delete_mcp_connection(&root, "rustc-version");

    let mut permissions = fs::metadata(&layout.config_path)
        .expect("config metadata")
        .permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&layout.config_path, permissions).expect("config should become writable");

    assert!(
        result.is_err(),
        "readonly project config should make MCP deletion fail"
    );
    let connections = list_mcp_connections(&root).expect("connections should list");
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].id, "rustc-version");
    let agent = get_agent_profile(&root, "codex-with-mcp").expect("agent should load");
    assert_eq!(agent.mcp_connection_ids, vec!["rustc-version".to_string()]);

    fs::remove_dir_all(root).ok();
}

#[test]
fn init_project_seeds_general_domain_context() {
    let root = test_root("default-domains");
    init_project(&root).expect("project should init");

    let groups = list_domains(&root).expect("Domains should load");
    let domains = list_artifact_types(&root).expect("domains should load");
    let general_group = groups
        .iter()
        .find(|group| group.id == "general")
        .expect("general group should be seeded");
    let general_domain = domains
        .iter()
        .find(|domain| domain.id == "general")
        .expect("general domain should be seeded");

    assert_eq!(
        general_group.display_name,
        I18n::environment().ui(UiTextKey::General)
    );
    assert_eq!(general_domain.domain_id.as_deref(), Some("general"));
    assert!(general_domain.artifact_types.contains(&"code".to_string()));
    assert!(
        general_domain
            .rubric
            .iter()
            .any(|line| line == "## 完了度 (40)" || line == "## Completeness (40)")
    );
    for agent_id in ["organizer", "worker", "reviewer"] {
        let profile = get_agent_profile(&root, agent_id).expect("default profile should load");
        assert_eq!(profile.domain_ids, vec!["general"]);
        assert_eq!(profile.artifact_type_ids, vec!["general"]);
    }
    assert!(
        root.join(".nagare")
            .join("domains")
            .join("general.toml")
            .exists()
    );
    assert!(
        root.join(".nagare")
            .join("artifact-types")
            .join("general.toml")
            .exists()
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn ensure_project_restores_directories_added_after_initialization() {
    let root = test_root("restore-project-directories");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    fs::remove_dir_all(&layout.artifact_types_dir).expect("artifact type directory should remove");
    fs::remove_dir_all(&layout.artifact_type_samples_dir)
        .expect("artifact type sample directory should remove");

    ensure_project(&root).expect("project directories should restore");

    assert!(layout.artifact_types_dir.join("general.toml").is_file());
    assert!(layout.artifact_type_samples_dir.is_dir());
    fs::remove_dir_all(root).ok();
}

#[test]
fn ensure_project_migrates_managed_dispatcher_and_supervisor_to_organizer() {
    let root = test_root("migrate-standard-organizer");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    fs::write(
        &layout.config_path,
        r#"[locale]
language = "en-US"

[nagare_agents]
work_agent = "worker"
review_agent = "reviewer"
dispatch_agent = "dispatcher"
supervisor_agent = "supervisor"

[agent_profiles.worker]
display_name = "Worker"
role = "worker"

[agent_profiles.reviewer]
display_name = "Reviewer"
role = "reviewer"

[agent_profiles.dispatcher]
display_name = "Dispatcher"
role = "dispatcher"
managed_by = "nagare"

[agent_profiles.dispatcher.external]
provider = "codex-cli"
agent_id = "dispatcher"
managed = true
source = "created"

[agent_profiles.supervisor]
display_name = "Supervisor"
role = "supervisor"
managed_by = "nagare"

[agent_profiles.supervisor.external]
provider = "codex-cli"
agent_id = "supervisor"
managed = true
source = "created"
"#,
    )
    .expect("legacy config should write");

    ensure_project(&root).expect("legacy config should migrate");
    let migrated = fs::read_to_string(&layout.config_path).expect("migrated config should read");

    assert!(migrated.contains("[agent_profiles.organizer]"));
    assert!(migrated.contains("display_name = \"General Organizer\""));
    assert!(migrated.contains("display_name = \"General Worker\""));
    assert!(migrated.contains("display_name = \"General Reviewer\""));
    assert!(migrated.contains("role = \"organizer\""));
    assert!(!migrated.contains("organizer_agent = \"organizer\""));
    assert!(migrated.contains("dispatch_agent = \"organizer\""));
    assert!(migrated.contains("supervisor_agent = \"organizer\""));
    assert!(!migrated.contains("[agent_profiles.dispatcher]"));
    assert!(!migrated.contains("[agent_profiles.supervisor]"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn ensure_project_renames_legacy_standard_names_after_role_migration() {
    let root = test_root("migrate-general-agent-names");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let legacy = I18n::new("en-US")
        .default_config_toml("UTC")
        .replace("General Organizer", "Organizer")
        .replace("General Worker", "Worker")
        .replace("General Reviewer", "Reviewer");
    fs::write(&layout.config_path, legacy).expect("legacy standard config should write");

    ensure_project(&root).expect("legacy standard names should migrate");
    let migrated = fs::read_to_string(&layout.config_path).expect("migrated config should read");

    assert!(migrated.contains("display_name = \"General Organizer\""));
    assert!(migrated.contains("display_name = \"General Worker\""));
    assert!(migrated.contains("display_name = \"General Reviewer\""));
    fs::remove_dir_all(root).ok();
}

#[test]
fn ensure_project_preserves_unmanaged_legacy_named_agents() {
    let root = test_root("preserve-custom-legacy-names");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    fs::write(
        &layout.config_path,
        r#"[nagare_agents]
work_agent = "worker"
review_agent = "reviewer"
dispatch_agent = "dispatcher"
supervisor_agent = "supervisor"

[agent_profiles.worker]
role = "worker"

[agent_profiles.reviewer]
role = "reviewer"

[agent_profiles.dispatcher]
display_name = "Custom dispatcher"
role = "dispatcher"

[agent_profiles.supervisor]
display_name = "Custom supervisor"
role = "supervisor"
"#,
    )
    .expect("custom config should write");

    ensure_project(&root).expect("custom config should remain usable");
    let current = fs::read_to_string(&layout.config_path).expect("current config should read");

    assert!(current.contains("[agent_profiles.dispatcher]"));
    assert!(current.contains("display_name = \"Custom dispatcher\""));
    assert!(current.contains("[agent_profiles.supervisor]"));
    assert!(current.contains("display_name = \"Custom supervisor\""));
    assert!(!current.contains("[agent_profiles.organizer]"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn ensure_project_repairs_missing_general_domain_context() {
    let root = test_root("repair-default-domains");
    init_project(&root).expect("project should init");
    fs::remove_file(root.join(".nagare").join("domains").join("general.toml"))
        .expect("general group should be removable");
    fs::remove_file(
        root.join(".nagare")
            .join("artifact-types")
            .join("general.toml"),
    )
    .expect("general domain should be removable");

    let groups = list_domains(&root).expect("Domains should reload");
    let domains = list_artifact_types(&root).expect("domains should reload");

    assert!(groups.iter().any(|group| group.id == "general"));
    assert!(domains.iter().any(|domain| domain.id == "general"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn first_scenario_reaches_done() {
    let root = test_root("first-scenario");
    let result = run_first_scenario(&root).expect("scenario should pass");
    assert_eq!(result.final_status, WorkItemStatus::Done);
    let snapshot =
        get_work_item_snapshot(&root, &result.work_item_id).expect("snapshot should load");
    assert_eq!(snapshot.runs.len(), 4);
    assert_eq!(snapshot.handoffs.len(), 1);
    assert_eq!(snapshot.review_results.len(), 1);
    assert_eq!(snapshot.decisions.len(), 1);
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_can_be_registered_and_used_in_scenario() {
    let root = test_root("agent");
    let result = run_registered_agent_scenario(&root).expect("registered scenario should pass");
    assert_eq!(result.final_status, WorkItemStatus::Done);

    let profiles = list_agent_profiles(&root).expect("profiles should load");
    assert!(profiles.iter().any(|profile| profile.id == "worker"));
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
    let root = test_root("unknown-agent");
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
    let root = test_root("probe");
    init_project(&root).expect("project should init");
    let result = agent_probe(&root, "worker").expect("probe should be recorded");
    assert_eq!(result.probe.agent_profile_id, "worker");
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
    let root = test_root("auto-probe");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Auto probe", "")
        .expect("item should create")
        .item;
    let command = scenario_command("auto probe", true);

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
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
            agent_profile_id: "worker",
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
    let root = test_root("working-dir");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    assert_eq!(
        snapshot.resolved_run_packets[0].output_contract.contract,
        "nagare.result.v1"
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_routing_hints_are_persisted() {
    let root = test_root("agent-hints");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
    let root = test_root("agent-update");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let updated = update_agent_profile(
        &root,
        "draft-agent",
        UpdateAgentProfileInput {
            display_name: Some("Research Writer"),
            avatar: Some("data:image/svg+xml;base64,PHN2Zy8+"),
            runtime: None,
            adapter: None,
            role: Some("researcher"),
            working_dir: Some("."),
            description: Some("Research and writing profile."),
            specialties: Some(vec!["research".to_string(), "writing".to_string()]),
            skill_set_ids: None,
            domain_ids: None,
            artifact_type_ids: None,
            mcp_connection_ids: None,
            prompt: None,
            output_contract: None,
            managed_by: None,
            model: None,
            external: None,
        },
    )
    .expect("profile should update");

    assert!(updated.path.ends_with(".nagare/agents/draft-agent.toml"));
    let profile = get_agent_profile(&root, "draft-agent").expect("profile should load");
    assert_eq!(profile.display_name, "Research Writer");
    assert_eq!(profile.role, "researcher");
    assert_eq!(profile.avatar, "data:image/svg+xml;base64,PHN2Zy8+");
    assert_eq!(profile.description, "Research and writing profile.");
    assert_eq!(profile.specialties, vec!["research", "writing"]);
    assert_eq!(profile.source, AgentProfileSource::ProjectAgentDirectory);
    assert_eq!(profile.prompt.version, "v1");

    let updated_prompt = update_agent_profile(
        &root,
        "draft-agent",
        UpdateAgentProfileInput {
            prompt: Some("Write from cited evidence."),
            ..UpdateAgentProfileInput::default()
        },
    )
    .expect("prompt should update");
    assert_eq!(updated_prompt.profile.prompt.version, "v2");

    let unchanged_prompt = update_agent_profile(
        &root,
        "draft-agent",
        UpdateAgentProfileInput {
            prompt: Some("Write from cited evidence."),
            ..UpdateAgentProfileInput::default()
        },
    )
    .expect("unchanged prompt should save");
    assert_eq!(unchanged_prompt.profile.prompt.version, "v2");
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_model_update_can_reset_to_runtime_default() {
    let root = test_root("agent-model-reset");
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "model-agent",
            display_name: "Model Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses a selected model.",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection {
                provider: "openai".to_string(),
                id: "gpt-5-codex".to_string(),
                base_url: String::new(),
                api_key_env: String::new(),
            },
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    update_agent_profile(
        &root,
        "model-agent",
        UpdateAgentProfileInput {
            model: Some(AgentModelSelection::default()),
            ..UpdateAgentProfileInput::default()
        },
    )
    .expect("profile should update");

    let profile = get_agent_profile(&root, "model-agent").expect("profile should load");
    assert_eq!(profile.model, AgentModelSelection::default());
    assert_eq!(profile.model.model_ref(), None);
    fs::remove_dir_all(root).ok();
}

#[test]
fn artifact_type_rubric_version_increments_only_when_rubric_changes() {
    let root = test_root("artifact-rubric-version");
    init_project(&root).expect("project should init");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "faq",
            domain_id: None,
            display_name: "FAQ",
            description: "Question and answer document.",
            artifact_types: Vec::new(),
            rubric: vec![
                "## 目的適合性 (100)".to_string(),
                "- 回答が質問に対応している。".to_string(),
            ],
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("artifact type should be added");
    let profile = get_artifact_type(&root, "faq").expect("artifact type should load");
    assert_eq!(profile.rubric_version, 1);
    assert_eq!(profile.definition_version, 1);

    update_artifact_type(
        &root,
        "faq",
        UpdateArtifactTypeInput {
            description: Some("Updated description."),
            ..UpdateArtifactTypeInput::default()
        },
    )
    .expect("non-rubric update should save");
    let profile = get_artifact_type(&root, "faq").expect("artifact type should load");
    assert_eq!(profile.rubric_version, 1);
    assert_eq!(profile.definition_version, 2);

    update_artifact_type(
        &root,
        "faq",
        UpdateArtifactTypeInput {
            rubric: Some(vec![
                "## 目的適合性 (60)".to_string(),
                "- 回答が質問に対応している。".to_string(),
                "## 再利用性 (40)".to_string(),
                "- 他の問い合わせにも参照しやすい。".to_string(),
            ]),
            ..UpdateArtifactTypeInput::default()
        },
    )
    .expect("rubric update should save");
    let profile = get_artifact_type(&root, "faq").expect("artifact type should load");
    assert_eq!(profile.rubric_version, 2);
    assert_eq!(profile.definition_version, 3);

    update_artifact_type(
        &root,
        "faq",
        UpdateArtifactTypeInput {
            workflow: Some(DomainWorkflowOverride {
                progress_mode: Some(WorkflowMode::FinishFirst),
                approval_policy: None,
            }),
            ..UpdateArtifactTypeInput::default()
        },
    )
    .expect("workflow-only update should save");
    let profile = get_artifact_type(&root, "faq").expect("artifact type should load");
    assert_eq!(profile.rubric_version, 2);
    assert_eq!(profile.definition_version, 3);
    fs::remove_dir_all(root).ok();
}

#[test]
fn domain_knowledge_version_increments_only_when_prompt_knowledge_changes() {
    let root = test_root("domain-knowledge-version");
    init_project(&root).expect("project should init");
    add_domain(
        &root,
        AddDomainInput {
            id: "knowledge-test",
            display_name: "Knowledge Test",
            description: "Initial description",
            shared_knowledge: vec!["Initial knowledge".to_string()],
            common_rubric: vec!["Initial rubric".to_string()],
            dispatch_hints: vec!["Initial hint".to_string()],
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("domain should be added");
    assert_eq!(
        get_domain(&root, "knowledge-test")
            .expect("domain should load")
            .knowledge_version,
        1
    );

    update_domain(
        &root,
        "knowledge-test",
        UpdateDomainInput {
            workflow: Some(DomainWorkflowOverride {
                progress_mode: Some(WorkflowMode::FinishFirst),
                approval_policy: None,
            }),
            ..UpdateDomainInput::default()
        },
    )
    .expect("workflow-only update should save");
    assert_eq!(
        get_domain(&root, "knowledge-test")
            .expect("domain should load")
            .knowledge_version,
        1
    );

    update_domain(
        &root,
        "knowledge-test",
        UpdateDomainInput {
            shared_knowledge: Some(vec!["Revised knowledge".to_string()]),
            ..UpdateDomainInput::default()
        },
    )
    .expect("knowledge update should save");
    assert_eq!(
        get_domain(&root, "knowledge-test")
            .expect("domain should load")
            .knowledge_version,
        2
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn workflow_settings_and_domain_override_resolve_for_new_items() {
    let root = test_root("workflow-settings-domain");
    init_project(&root).expect("project should init");
    set_workflow_settings(
        &root,
        WorkflowSettings {
            default_progress_mode: WorkflowMode::FinishFirst,
            approval_policy: ApprovalPolicy::AutoCompleteOnReviewPass,
        },
    )
    .expect("workflow settings should save");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "docs",
            domain_id: None,
            display_name: "Docs",
            description: "Documentation work",
            artifact_types: vec!["markdown".to_string()],
            rubric: vec!["clear for readers".to_string()],
            dispatch_hints: vec!["use for docs".to_string()],
            workflow: DomainWorkflowOverride {
                progress_mode: Some(WorkflowMode::ConfirmFirst),
                approval_policy: Some(ApprovalPolicy::ManualFinalApproval),
            },
        },
    )
    .expect("domain should be added");

    let project_default_item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Project default".to_string(),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    let domain_item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Domain override".to_string(),
            artifact_type_id: Some("docs".to_string()),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;

    assert_eq!(
        project_default_item.workflow_mode,
        WorkflowMode::FinishFirst
    );
    assert_eq!(
        project_default_item.approval_policy,
        ApprovalPolicy::AutoCompleteOnReviewPass
    );
    assert_eq!(domain_item.workflow_mode, WorkflowMode::ConfirmFirst);
    assert_eq!(
        domain_item.approval_policy,
        ApprovalPolicy::ManualFinalApproval
    );
    assert_eq!(domain_item.artifact_type_id.as_deref(), Some("docs"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn domain_defaults_domain_override_and_agent_scope_are_persisted() {
    let root = test_root("domain-group-scope");
    init_project(&root).expect("project should init");
    set_workflow_settings(
        &root,
        WorkflowSettings {
            default_progress_mode: WorkflowMode::ConfirmFirst,
            approval_policy: ApprovalPolicy::ManualFinalApproval,
        },
    )
    .expect("workflow settings should save");
    add_domain(
        &root,
        AddDomainInput {
            id: "software-development",
            display_name: "Software Development",
            description: "Software changes",
            shared_knowledge: vec!["small changes".to_string()],
            common_rubric: vec!["tests pass".to_string()],
            dispatch_hints: vec!["prefer code agents".to_string()],
            workflow: DomainWorkflowOverride {
                progress_mode: Some(WorkflowMode::FinishFirst),
                approval_policy: Some(ApprovalPolicy::AutoCompleteOnReviewPass),
            },
        },
    )
    .expect("Domain should be added");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "frontend-ui",
            domain_id: Some("software-development"),
            display_name: "Frontend UI",
            description: "UI work",
            artifact_types: vec!["html".to_string()],
            rubric: vec!["responsive".to_string()],
            dispatch_hints: vec!["use ui agents".to_string()],
            workflow: DomainWorkflowOverride {
                progress_mode: None,
                approval_policy: Some(ApprovalPolicy::ManualFinalApproval),
            },
        },
    )
    .expect("domain should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "ui-reviewer",
            display_name: "UI Reviewer",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "reviewer",
            working_dir: ".",
            description: "Reviews frontend UI.",
            specialties: vec!["ui".to_string(), "review".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: vec!["software-development".to_string()],
            artifact_type_ids: vec!["frontend-ui".to_string()],
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("agent should be added");

    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Polish settings UI".to_string(),
            artifact_type_id: Some("frontend-ui".to_string()),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    assert_eq!(item.domain_id.as_deref(), Some("software-development"));
    assert_eq!(item.artifact_type_id.as_deref(), Some("frontend-ui"));
    assert_eq!(item.workflow_mode, WorkflowMode::FinishFirst);
    assert_eq!(item.approval_policy, ApprovalPolicy::ManualFinalApproval);

    let profile = get_agent_profile(&root, "ui-reviewer").expect("profile should load");
    assert_eq!(profile.domain_ids, vec!["software-development"]);
    assert_eq!(profile.artifact_type_ids, vec!["frontend-ui"]);

    let mismatch = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Mismatched domain".to_string(),
            domain_id: Some("other".to_string()),
            artifact_type_id: Some("frontend-ui".to_string()),
            ..CreateWorkItemInput::default()
        },
    );
    assert!(mismatch.is_err());
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_profile_output_contracts_can_be_updated() {
    let root = test_root("output-contract");
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "review-agent",
            display_name: "Review Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "reviewer",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    update_agent_profile(
        &root,
        "review-agent",
        UpdateAgentProfileInput {
            output_contract: Some(AgentOutputContractUpdate {
                purpose: Some(AgentOutputContractPurpose::Review),
                contract: Some("nagare.review.custom.v1"),
                instruction_pack: Some("nagare-review-custom.v1"),
                required: Some(false),
                injection: Some(AgentOutputInjection::PromptSuffix),
            }),
            ..UpdateAgentProfileInput::default()
        },
    )
    .expect("profile should update");

    let profile = get_agent_profile(&root, "review-agent").expect("profile should load");
    assert_eq!(
        profile.output_contracts.review.contract,
        "nagare.review.custom.v1"
    );
    assert_eq!(
        profile.output_contracts.review.instruction_pack,
        "nagare-review-custom.v1"
    );
    assert!(!profile.output_contracts.review.required);
    fs::remove_dir_all(root).ok();
}

#[test]
fn nagare_agent_settings_can_select_default_work_agent() {
    let root = test_root("agent-settings");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let settings = set_nagare_agent_settings(
        &root,
        SetNagareAgentSettingsInput {
            work_agent: Some("codex-work"),
            review_agent: None,
            organizer_agent: Some("codex-work"),
            dispatch_agent: Some("codex-work"),
            supervisor_agent: Some("codex-work"),
        },
    )
    .expect("settings should update");
    assert_eq!(settings.work_agent, "codex-work");
    assert_eq!(settings.review_agent, "reviewer");
    assert_eq!(settings.organizer_agent.as_deref(), Some("codex-work"));
    assert_eq!(settings.dispatch_agent, "codex-work");
    assert_eq!(settings.supervisor_agent, "codex-work");

    let loaded = get_nagare_agent_settings(&root).expect("settings should load");
    assert_eq!(loaded.work_agent, "codex-work");
    assert_eq!(loaded.organizer_agent.as_deref(), Some("codex-work"));
    assert_eq!(loaded.supervisor_agent, "codex-work");
    fs::remove_dir_all(root).ok();
}

#[test]
fn workflow_dispatch_uses_project_organizer_with_dispatch_fallback() {
    let root = test_root("organizer-agent-fallback");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Route with organizer", "")
        .expect("item should create")
        .item;

    let fallback_decision =
        create_workflow_decision(&root, &item.id).expect("fallback decision should create");
    assert_eq!(
        fallback_decision.decision.action,
        WorkflowDecisionAction::Dispatch
    );
    assert_eq!(
        fallback_decision
            .decision
            .target_agent_profile_id
            .as_deref(),
        Some("organizer")
    );

    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "project-organizer",
            display_name: "Project Organizer",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "organizer",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("organizer profile should be added");

    set_nagare_agent_settings(
        &root,
        SetNagareAgentSettingsInput {
            work_agent: None,
            review_agent: None,
            organizer_agent: Some("project-organizer"),
            dispatch_agent: None,
            supervisor_agent: None,
        },
    )
    .expect("organizer should update");

    let organizer_decision =
        create_workflow_decision(&root, &item.id).expect("organizer decision should create");
    assert_eq!(
        organizer_decision.decision.action,
        WorkflowDecisionAction::Dispatch
    );
    assert_eq!(
        organizer_decision
            .decision
            .target_agent_profile_id
            .as_deref(),
        Some("project-organizer")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn project_organizer_setting_can_be_cleared_to_builtin_fallback() {
    let root = test_root("organizer-agent-clear");
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "project-organizer",
            display_name: "Project Organizer",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "organizer",
            working_dir: ".",
            description: "",
            specialties: Vec::new(),
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("organizer profile should be added");

    let settings = set_project_organizer_agent(&root, Some("project-organizer"))
        .expect("organizer should update");
    assert_eq!(
        settings.organizer_agent.as_deref(),
        Some("project-organizer")
    );

    let settings =
        set_project_organizer_agent(&root, None).expect("organizer should clear to fallback");
    assert_eq!(settings.organizer_agent, None);

    let loaded = get_nagare_agent_settings(&root).expect("settings should load");
    assert_eq!(loaded.organizer_agent, None);
    assert_eq!(loaded.dispatch_agent, "organizer");
    fs::remove_dir_all(root).ok();
}

#[test]
fn project_metadata_can_be_saved_and_loaded() {
    let root = test_root("project-metadata");
    init_project(&root).expect("project should init");

    let metadata = set_project_metadata(
        &root,
        SetProjectMetadataInput {
            name: Some("Docs Site"),
            icon: Some("📘"),
            default_domain_id: Some("general"),
            default_artifact_type_id: Some("general"),
        },
    )
    .expect("metadata should save");
    assert_eq!(metadata.name, "Docs Site");
    assert_eq!(metadata.icon, "📘");
    assert_eq!(metadata.default_domain_id, "general");
    assert_eq!(metadata.default_artifact_type_id, "general");

    let loaded = get_project_metadata(&root).expect("metadata should load");
    assert_eq!(loaded.name, "Docs Site");
    assert_eq!(loaded.icon, "📘");
    assert_eq!(loaded.default_domain_id, "general");
    assert_eq!(loaded.default_artifact_type_id, "general");
    fs::remove_dir_all(root).ok();
}

#[test]
fn improvement_history_can_be_recorded_and_replaced() {
    let root = test_root("improvement-history");
    init_project(&root).expect("project should init");

    let first = record_improvement_applied(
        &root,
        RecordImprovementInput {
            proposal_id: "proposal-prompt-reviewer",
            kind: "プロンプト",
            title: "Reviewer のプロンプト改善",
            target_label: "Reviewer / 形式の準拠",
            summary: "形式基準を先に確認する手順を追加します",
            evidence: "形式の準拠が直近2件で60%",
        },
    )
    .expect("improvement should record");
    assert_eq!(first.proposal_id, "proposal-prompt-reviewer");
    assert_eq!(first.status, "applied");
    assert_eq!(first.effect_label, "効果測定中");

    let dismissed = record_improvement_dismissed(
        &root,
        RecordImprovementInput {
            proposal_id: "proposal-prompt-reviewer",
            kind: "プロンプト",
            title: "Reviewer のプロンプト改善を見送り",
            target_label: "Reviewer / 形式の準拠",
            summary: "今回は見送ります",
            evidence: "手動判断",
        },
    )
    .expect("same proposal should be dismissed");
    assert_eq!(dismissed.status, "dismissed");
    assert_eq!(dismissed.effect_label, "見送り済み");

    let history = list_improvement_history(&root).expect("history should load");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].proposal_id, "proposal-prompt-reviewer");
    assert_eq!(history[0].status, "dismissed");
    assert_eq!(history[0].title, "Reviewer のプロンプト改善を見送り");
    assert_eq!(history[0].target_label, "Reviewer / 形式の準拠");
    fs::remove_dir_all(root).ok();
}

#[test]
fn delete_project_state_removes_only_nagare_directory() {
    let root = test_root("delete-project-state");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let ordinary_file = root.join("kept.txt");
    fs::write(&ordinary_file, "keep").expect("ordinary file should write");
    assert!(layout.nagare_dir.exists());

    let removed = delete_project_state(&root).expect("project state should delete");
    assert!(removed);
    assert!(!layout.nagare_dir.exists());
    assert_eq!(
        fs::read_to_string(&ordinary_file).expect("ordinary file should remain"),
        "keep"
    );
    assert!(!delete_project_state(&root).expect("second delete is a no-op"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_and_review_runs_do_not_advance_item_status() {
    let root = test_root("purpose");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Route and review", "")
        .expect("item should create")
        .item;
    let command = scenario_command("agent purpose run", true);

    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
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
            agent_profile_id: "reviewer",
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
    let root = test_root("handoff-dispatch");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");
    let item = create_work_item(&root, "Handoff dispatch", "")
        .expect("item should create")
        .item;
    create_handoff(
        &root,
        &item.id,
        "worker",
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
            agent_profile_id: "worker",
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
    let root = test_root("dispatch-accept");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
            agent_profile_id: "worker",
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
    let root = test_root("dispatch-json");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
            agent_profile_id: "worker",
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
    let root = test_root("dispatch-contract");
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
            agent_profile_id: "worker",
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
    assert_eq!(plan.target_agent_profile_id, "worker");
    assert_eq!(plan.summary, "No target provided.");
    assert!(plan.selection_warnings.iter().any(|warning| {
        warning.contains("missing required target_agent_profile_id")
            && warning.contains("fallback target `worker`")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_requires_human_confirmation_when_domain_agent_is_missing() {
    let root = test_root("dispatch-missing-domain-agent");
    init_project(&root).expect("project should init");
    add_domain(
        &root,
        AddDomainInput {
            id: "software-development",
            display_name: "Software Development",
            description: "Software changes",
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("Domain should be added");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "frontend-ui",
            domain_id: Some("software-development"),
            display_name: "Frontend UI",
            description: "UI work",
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("domain should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process-codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Handles research work.",
            specialties: vec!["research".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("unrelated agent should be added");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Frontend dispatch".to_string(),
            artifact_type_id: Some("frontend-ui".to_string()),
            domain_agent_policy: DomainAgentPolicy::ConfirmGeneralFallback,
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"research-agent","summary":"Use the research agent.","risks":[],"missing_information":[]}"#,
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
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should record confirmation need");

    assert_eq!(preview.item_status, WorkItemStatus::NeedsInput);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(snapshot.item.status, WorkItemStatus::NeedsInput);
    assert_eq!(plan.target_agent_profile_id, "worker");
    assert!(plan.missing_information.iter().any(|missing| {
        missing.contains("No candidate agent is scoped to domain `frontend-ui`")
            && missing.contains("confirm whether to proceed with general fallback agent `worker`")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_blocks_when_domain_agent_is_required_and_missing() {
    let root = test_root("dispatch-require-domain-agent");
    init_project(&root).expect("project should init");
    add_domain(
        &root,
        AddDomainInput {
            id: "software-development",
            display_name: "Software Development",
            description: "Software changes",
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("Domain should be added");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "frontend-ui",
            domain_id: Some("software-development"),
            display_name: "Frontend UI",
            description: "UI work",
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("domain should be added");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Frontend dispatch".to_string(),
            artifact_type_id: Some("frontend-ui".to_string()),
            domain_agent_policy: DomainAgentPolicy::RequireDomainAgent,
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use the worker.","risks":[],"missing_information":[]}"#,
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
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should record required domain agent blocker");

    assert_eq!(preview.item_status, WorkItemStatus::NeedsInput);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(
        snapshot.item.domain_agent_policy,
        DomainAgentPolicy::RequireDomainAgent
    );
    assert!(plan.missing_information.iter().any(|missing| {
        missing.contains("Domain-scoped agent is required for domain `frontend-ui`")
            && missing.contains("add a matching agent or change the domain agent policy")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_uses_general_fallback_without_confirmation_by_default() {
    let root = test_root("dispatch-general-fallback");
    init_project(&root).expect("project should init");
    add_domain(
        &root,
        AddDomainInput {
            id: "software-development",
            display_name: "Software Development",
            description: "Software changes",
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("Domain should be added");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "frontend-ui",
            domain_id: Some("software-development"),
            display_name: "Frontend UI",
            description: "UI work",
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("domain should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process-codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Handles research work.",
            specialties: vec!["research".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("unrelated agent should be added");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Frontend dispatch".to_string(),
            artifact_type_id: Some("frontend-ui".to_string()),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"research-agent","summary":"Use the research agent.","risks":[],"missing_information":[]}"#,
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
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should use general fallback");

    assert_eq!(preview.item_status, WorkItemStatus::Ready);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(snapshot.item.status, WorkItemStatus::Ready);
    assert_eq!(
        snapshot.item.domain_agent_policy,
        DomainAgentPolicy::AutoGeneralFallback
    );
    assert_eq!(plan.target_agent_profile_id, "worker");
    assert!(plan.missing_information.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_does_not_block_when_domain_agent_exists() {
    let root = test_root("dispatch-domain-agent");
    init_project(&root).expect("project should init");
    add_domain(
        &root,
        AddDomainInput {
            id: "software-development",
            display_name: "Software Development",
            description: "Software changes",
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("Domain should be added");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "frontend-ui",
            domain_id: Some("software-development"),
            display_name: "Frontend UI",
            description: "UI work",
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("domain should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "ui-worker",
            display_name: "UI Worker",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Handles frontend UI work.",
            specialties: vec!["ui".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: vec!["software-development".to_string()],
            artifact_type_ids: vec!["frontend-ui".to_string()],
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("agent should be added");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Frontend dispatch".to_string(),
            artifact_type_id: Some("frontend-ui".to_string()),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"ui-worker","summary":"Use the UI worker.","risks":[],"missing_information":[]}"#,
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
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should run");

    assert_eq!(preview.item_status, WorkItemStatus::Ready);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(snapshot.item.status, WorkItemStatus::Ready);
    assert_eq!(plan.target_agent_profile_id, "ui-worker");
    assert!(plan.missing_information.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_contract_control_agent_target_falls_back_to_work_agent() {
    let root = test_root("dispatch-control-target");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Dispatch control target", "")
        .expect("item should create")
        .item;
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"organizer","summary":"Incorrectly selected organizer.","risks":[],"missing_information":[]}"#,
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
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should fallback");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(plan.dispatch_agent_profile_id, "organizer");
    assert_eq!(plan.target_agent_profile_id, "worker");
    assert!(
        plan.selection_warnings
            .iter()
            .any(|warning| warning.contains("not registered") && warning.contains("`worker`"))
    );
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
fn dispatch_plan_suggestion_accepts_single_string_diagnostics() {
    let output = r#"{"target_agent_profile_id":"software-worker","summary":"Use the software worker.","risks":"No blocker.","missing_information":"No additional input."}"#;
    let suggestion = parse_dispatch_plan_suggestion(output).expect("suggestion should parse");

    assert_eq!(
        suggestion.target_agent_profile_id.as_deref(),
        Some("software-worker")
    );
    assert_eq!(suggestion.risks, vec!["No blocker.".to_string()]);
    assert_eq!(
        suggestion.missing_information,
        vec!["No additional input.".to_string()]
    );
}

#[test]
fn project_rule_resolution_selects_most_specific_rule() {
    let root = test_root("rule");
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
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: None,
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
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
review_agent = "reviewer"
skill_sets = ["rust-core"]
permission_policy = "medium-code-task"
workspace_policy = "project-root"
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

    let default_resolution =
        resolve_rule_for_path(&root, Some("README.md"), None).expect("rule should resolve");
    assert_eq!(default_resolution.matched_rule_id.as_deref(), None);
    assert_eq!(default_resolution.agent_profile_id, "worker");
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_with_path_records_resolved_skill_context_and_run_packet() {
    let root = test_root("run-packet");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Resolve packet", "")
        .expect("item should create")
        .item;
    let command = scenario_command("resolved packet", true);
    let result = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
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
    assert_eq!(context.agent_profile_id, "worker");
    assert!(context.project_rule_ids.is_empty());
    assert!(context.applied_skill_set_ids.is_empty());
    assert_eq!(packet.resolved_skill_context_id, context.id);
    assert_eq!(packet.agent_profile_id, "worker");
    assert_eq!(packet.adapter_id, "process.codex-cli");
    assert_eq!(packet.purpose, AgentRunPurpose::DispatchPreview);
    assert_eq!(packet.goal, "Resolve packet");
    assert_eq!(packet.prompt_version, "v1");
    assert_eq!(packet.rubric_id, None);
    assert_eq!(packet.rubric_version, None);
    assert_eq!(packet.domain_knowledge_id, None);
    assert_eq!(packet.domain_knowledge_version, None);
    assert_eq!(packet.artifact_definition_id, None);
    assert_eq!(packet.artifact_definition_version, None);
    assert!(packet.working_dir.contains("nagare-run-packet"));
    assert_eq!(packet.path.as_deref(), Some("README.md"));
    assert!(packet.project_rule_ids.is_empty());
    let plan = &ledger.dispatch_plans[0];
    assert_eq!(plan.resolved_run_packet_id, packet.id);
    assert_eq!(plan.dispatch_agent_profile_id, "worker");
    assert_eq!(plan.target_agent_profile_id, "worker");
    assert_eq!(plan.path.as_deref(), Some("README.md"));
    assert!(
        layout
            .logs_dir
            .join(format!("{}.json", context.id))
            .exists()
    );
    assert!(layout.logs_dir.join(format!("{}.json", packet.id)).exists());
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_does_not_inherit_skill_sets_from_project_rules() {
    let root = test_root("skill-skip");
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
default_agent = "worker"
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
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: Some("secure/config.rs"),
            prompt: None,
            dev_command: Some(scenario_command("skill skip", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed without inheriting the rule skill");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let context = &snapshot.resolved_skill_contexts[0];
    assert!(context.declared_skill_set_ids.is_empty());
    assert!(context.applied_skill_set_ids.is_empty());
    assert!(context.skipped_skill_set_ids.is_empty());
    assert!(
        context
            .scope_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("network-only"))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn run_applies_only_skill_sets_assigned_to_the_selected_agent() {
    let root = test_root("agent-skill-merge");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_sets.rule-rust]
paths = ["skills/rule-rust"]
required_capabilities = ["repo_read"]
optional_capabilities = []

[skill_sets.agent-review]
paths = ["skills/agent-review"]
required_capabilities = ["repo_read"]
optional_capabilities = ["shell_command"]

[[project_rules]]
id = "rust-rule"
match = ["crates/**"]
default_agent = "skill-agent"
skill_sets = ["rule-rust"]
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "skill-agent",
            display_name: "Skill Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses an agent-specific review skill.",
            specialties: Vec::new(),
            skill_set_ids: vec!["agent-review".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let item = create_work_item(&root, "Scope skill sets", "")
        .expect("item should create")
        .item;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "skill-agent",
            dispatch_plan_id: None,
            path: Some("crates/nagare-core/src/lib.rs"),
            prompt: None,
            dev_command: Some(scenario_command("skill scope", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let context = &snapshot.resolved_skill_contexts[0];
    assert_eq!(
        context.declared_skill_set_ids,
        vec!["agent-review".to_string()]
    );
    assert_eq!(
        context.applied_skill_set_ids,
        vec!["agent-review".to_string()]
    );
    assert!(context.skipped_skill_set_ids.is_empty());
    assert!(
        context
            .scope_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("rule-rust"))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn separate_agents_receive_only_their_minimal_local_skill() {
    let root = test_root("minimal-agent-skills");
    init_project(&root).expect("project should init");
    let writer_skill = root.join(".agents/skills/minimal-writer");
    let designer_skill = root.join(".agents/skills/minimal-designer");
    fs::create_dir_all(&writer_skill).expect("writer skill directory should create");
    fs::create_dir_all(&designer_skill).expect("designer skill directory should create");
    fs::write(writer_skill.join("SKILL.md"), "# Minimal Writer\n")
        .expect("writer skill should write");
    fs::write(designer_skill.join("SKILL.md"), "# Minimal Designer\n")
        .expect("designer skill should write");

    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_sets.minimal-writer]
paths = [".agents/skills/minimal-writer"]
required_capabilities = []
optional_capabilities = []

[skill_sets.minimal-designer]
paths = [".agents/skills/minimal-designer"]
required_capabilities = []
optional_capabilities = []
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");

    for (id, skill_set_id) in [
        ("minimal-writer-agent", "minimal-writer"),
        ("minimal-designer-agent", "minimal-designer"),
    ] {
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id,
                display_name: id,
                runtime: "codex-local",
                adapter: "process.codex-cli",
                role: "worker",
                working_dir: ".",
                description: "minimal skill scope test",
                specialties: Vec::new(),
                skill_set_ids: vec![skill_set_id.to_string()],
                domain_ids: Vec::new(),
                artifact_type_ids: Vec::new(),
                mcp_connection_ids: Vec::new(),
                managed_by: Some("nagare"),
                model: AgentModelSelection::default(),
                external: ExternalAgentBinding::default(),
            },
        )
        .expect("profile should be added");
    }

    for (agent_id, expected_skill_id, expected_path) in [
        (
            "minimal-writer-agent",
            "minimal-writer",
            writer_skill.join("SKILL.md"),
        ),
        (
            "minimal-designer-agent",
            "minimal-designer",
            designer_skill.join("SKILL.md"),
        ),
    ] {
        let item = create_work_item(&root, agent_id, "")
            .expect("item should create")
            .item;
        run_work_item_with_input(
            &root,
            &item.id,
            RunWorkItemInput {
                agent_profile_id: agent_id,
                dispatch_plan_id: None,
                path: None,
                prompt: None,
                dev_command: Some(scenario_command("minimal skill", true).as_str()),
                purpose: AgentRunPurpose::Work,
            },
        )
        .expect("run should succeed");
        let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
        let context = &snapshot.resolved_skill_contexts[0];
        assert_eq!(context.applied_skill_set_ids, vec![expected_skill_id]);
        assert_eq!(
            context.effective_skill_paths,
            vec![expected_path.to_string_lossy().to_string()]
        );
        assert!(
            !context
                .applied_skill_set_ids
                .iter()
                .any(|skill| skill != expected_skill_id)
        );
    }

    fs::remove_dir_all(root).ok();
}

#[test]
fn run_skips_remote_skill_package_without_local_install() {
    let root = test_root("remote-skill-not-installed");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_packages."hachiware-labs/hachi-search"]
source_kind = "vercel"
source = "hachiware-labs/hachi-search"
provided_skill_sets = ["hachiware-labs/hachi-search"]

[skill_sets."hachiware-labs/hachi-search"]
paths = ["."]
required_capabilities = []
optional_capabilities = []
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "search-agent",
            display_name: "Search Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses search skills.",
            specialties: Vec::new(),
            skill_set_ids: vec!["hachiware-labs/hachi-search".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let item = create_work_item(&root, "Use hachi-search", "")
        .expect("item should create")
        .item;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "search-agent",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(scenario_command("remote skill missing", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed with remote skill skipped");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let context = &snapshot.resolved_skill_contexts[0];
    assert_eq!(
        context.declared_skill_set_ids,
        vec!["hachiware-labs/hachi-search".to_string()]
    );
    assert!(context.applied_skill_set_ids.is_empty());
    assert_eq!(
        context.skipped_skill_set_ids,
        vec!["hachiware-labs/hachi-search".to_string()]
    );
    assert!(
        snapshot.resolved_run_packets[0]
            .constraints
            .iter()
            .any(|constraint| constraint.contains("registered but not installed locally"))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn reference_skill_sources_register_and_skip_without_local_install() {
    let root = test_root("reference-skill-sources");
    init_project(&root).expect("project should init");

    for (id, source_kind) in [
        ("openai/code-review", "openai"),
        ("anthropic/research", "anthropic"),
        ("manual/custom-skill", "manual"),
    ] {
        let result = add_skill_package(
            &root,
            AddSkillPackageInput {
                id: Some(id),
                source_kind,
                source: Some(id),
                path: None,
                install: true,
                install_scope: Some("project"),
                install_targets: vec!["codex".to_string()],
                reference: None,
                checksum: None,
                skill_set_id: Some(id),
                skill_paths: Vec::new(),
                required_capabilities: Vec::new(),
                optional_capabilities: Vec::new(),
            },
        )
        .expect("reference skill should register without invoking a local installer");
        assert_eq!(result.package.source_kind, source_kind);
        assert!(result.package.installed_paths.is_empty());
        assert_eq!(result.package.provided_skill_sets, vec![id.to_string()]);
    }

    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "reference-skill-agent",
            display_name: "Reference Skill Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses reference skills.",
            specialties: Vec::new(),
            skill_set_ids: vec!["openai/code-review".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let item = create_work_item(&root, "Use reference skill", "")
        .expect("item should create")
        .item;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reference-skill-agent",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(scenario_command("reference skill missing", true).as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("run should succeed with reference skill skipped");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    let context = &snapshot.resolved_skill_contexts[0];
    assert_eq!(
        context.declared_skill_set_ids,
        vec!["openai/code-review".to_string()]
    );
    assert!(context.applied_skill_set_ids.is_empty());
    assert_eq!(
        context.skipped_skill_set_ids,
        vec!["openai/code-review".to_string()]
    );
    assert!(
        snapshot.resolved_run_packets[0]
            .constraints
            .iter()
            .any(|constraint| constraint
                .contains("package source `openai` is registered but not installed locally"))
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn uninstall_agent_skill_package_limits_body_removal_to_selected_tool() {
    let root = test_root("uninstall-skill-package");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let skill_dir = root.join(".agents").join("skills").join("search-tools");
    let openclaw_skill_dir = root.join("skills").join("search-tools");
    fs::create_dir_all(&skill_dir).expect("skill dir should create");
    fs::create_dir_all(&openclaw_skill_dir).expect("openclaw skill dir should create");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: search-tools
description: Project search helpers.
---

# Search Tools
"#,
    )
    .expect("skill should write");
    fs::write(
        openclaw_skill_dir.join("SKILL.md"),
        r#"---
name: search-tools
description: Project search helpers for OpenClaw.
---

# Search Tools
"#,
    )
    .expect("openclaw skill should write");
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_packages.search-tools]
source_kind = "local"
source = ".agents/skills/search-tools"
installed_path = ".agents/skills/search-tools"
installed_paths = [".agents/skills/search-tools", "skills/search-tools"]
install_scope = "project"
installed_targets = ["codex", "openclaw"]
provided_skill_sets = ["search-tools"]

[skill_sets.search-tools]
paths = [".agents/skills/search-tools", "skills/search-tools"]
required_capabilities = []
optional_capabilities = []
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    fs::write(
        root.join("skills-lock.json"),
        r#"{
  "version": 1,
  "skills": {
    "search-tools": {
      "source": ".agents/skills/search-tools"
    }
  }
}
"#,
    )
    .expect("lock file should write");
    for agent_id in ["search-a", "search-b"] {
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id: agent_id,
                display_name: agent_id,
                runtime: "codex-local",
                adapter: "process.codex-cli",
                role: "worker",
                working_dir: ".",
                description: "Uses search skills.",
                specialties: Vec::new(),
                skill_set_ids: vec!["search-tools".to_string()],
                domain_ids: Vec::new(),
                artifact_type_ids: Vec::new(),
                mcp_connection_ids: Vec::new(),
                managed_by: Some("nagare"),
                model: AgentModelSelection::default(),
                external: ExternalAgentBinding::default(),
            },
        )
        .expect("profile should be added");
    }

    let first = uninstall_agent_skill_package(
        &root,
        UninstallAgentSkillPackageInput {
            agent_profile_id: "search-a",
            skill_set_id: "search-tools",
            uninstall_package: true,
        },
    )
    .expect("first uninstall should succeed");
    assert!(first.removed_from_agent);
    assert!(!first.package_removed);
    assert!(skill_dir.exists());
    assert!(
        fs::read_to_string(&layout.config_path)
            .expect("config should read")
            .contains("[skill_packages.search-tools]")
    );

    let second = uninstall_agent_skill_package(
        &root,
        UninstallAgentSkillPackageInput {
            agent_profile_id: "search-b",
            skill_set_id: "search-tools",
            uninstall_package: true,
        },
    )
    .expect("second uninstall should succeed");
    assert!(second.removed_from_agent);
    assert!(!second.package_removed);
    assert!(second.installed_path_removed);
    assert!(!skill_dir.exists());
    assert!(openclaw_skill_dir.exists());
    let config = fs::read_to_string(&layout.config_path).expect("config should read");
    assert!(config.contains("[skill_packages.search-tools]"));
    assert!(config.contains("installed_targets = [\"openclaw\"]"));
    assert!(config.contains("installed_path = \"skills/search-tools\""));
    assert!(config.contains("installed_paths = [\"skills/search-tools\"]"));
    assert!(config.contains("paths = [\"skills/search-tools\"]"));
    let lock = fs::read_to_string(root.join("skills-lock.json")).expect("lock should read");
    assert!(lock.contains("search-tools"));
    let agent = get_agent_profile(&root, "search-b").expect("agent should load");
    assert!(agent.skill_set_ids.is_empty());
    fs::remove_dir_all(root).ok();
}

#[test]
fn delete_skill_package_detaches_agents_and_removes_library_entry() {
    let root = test_root("delete-skill-package");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let skill_dir = root.join(".agents").join("skills").join("delete-me");
    fs::create_dir_all(&skill_dir).expect("skill dir should create");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: delete-me
description: Temporary skill.
---

# Delete Me
"#,
    )
    .expect("skill should write");
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_packages.delete-me]
source_kind = "local"
source = ".agents/skills/delete-me"
installed_path = ".agents/skills/delete-me"
installed_paths = [".agents/skills/delete-me"]
install_scope = "project"
installed_targets = ["codex"]
provided_skill_sets = ["delete-me"]

[skill_sets.delete-me]
paths = [".agents/skills/delete-me"]
required_capabilities = []
optional_capabilities = []
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    fs::write(
        root.join("skills-lock.json"),
        r#"{
  "version": 1,
  "skills": {
    "delete-me": {
      "source": ".agents/skills/delete-me"
    }
  }
}
"#,
    )
    .expect("lock file should write");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "uses-delete-me",
            display_name: "Uses Delete Me",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses a temporary skill.",
            specialties: Vec::new(),
            skill_set_ids: vec!["delete-me".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let result = delete_skill_package(
        &root,
        DeleteSkillPackageInput {
            package_id: "delete-me",
            remove_installed_body: true,
        },
    )
    .expect("package should delete");
    assert_eq!(result.detached_agents, vec!["uses-delete-me".to_string()]);
    assert_eq!(result.removed_skill_sets, vec!["delete-me".to_string()]);
    assert!(result.installed_body_removed);
    assert!(result.warnings.is_empty());
    assert!(!skill_dir.exists());

    let config = fs::read_to_string(&layout.config_path).expect("config should read");
    assert!(!config.contains("[skill_packages.delete-me]"));
    assert!(!config.contains("[skill_sets.delete-me]"));
    let agent = get_agent_profile(&root, "uses-delete-me").expect("agent should load");
    assert!(agent.skill_set_ids.is_empty());
    let lock = fs::read_to_string(root.join("skills-lock.json")).expect("lock should read");
    assert!(!lock.contains("delete-me"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn failed_skill_package_delete_restores_agent_assignments_before_body_removal() {
    let root = test_root("delete-skill-package-rollback");
    init_project(&root).expect("project should init");
    let layout = ProjectLayout::new(&root);
    let skill_dir = root.join(".agents").join("skills").join("delete-me");
    fs::create_dir_all(&skill_dir).expect("skill dir should create");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: delete-me
description: Temporary skill.
---

# Delete Me
"#,
    )
    .expect("skill should write");
    let mut config = fs::read_to_string(&layout.config_path).expect("config should read");
    config.push_str(
        r#"

[skill_packages.delete-me]
source_kind = "local"
source = ".agents/skills/delete-me"
installed_path = ".agents/skills/delete-me"
installed_paths = [".agents/skills/delete-me"]
install_scope = "project"
installed_targets = ["codex"]
provided_skill_sets = ["delete-me"]

[skill_sets.delete-me]
paths = [".agents/skills/delete-me"]
required_capabilities = []
optional_capabilities = []
"#,
    );
    fs::write(&layout.config_path, config).expect("config should write");
    fs::write(
        root.join("skills-lock.json"),
        r#"{
  "version": 1,
  "skills": {
    "delete-me": {
      "source": ".agents/skills/delete-me"
    }
  }
}
"#,
    )
    .expect("lock file should write");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "uses-delete-me",
            display_name: "Uses Delete Me",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "worker",
            working_dir: ".",
            description: "Uses a temporary skill.",
            specialties: Vec::new(),
            skill_set_ids: vec!["delete-me".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding::default(),
        },
    )
    .expect("profile should be added");

    let mut permissions = fs::metadata(&layout.config_path)
        .expect("config metadata")
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&layout.config_path, permissions).expect("config should become readonly");

    let result = delete_skill_package(
        &root,
        DeleteSkillPackageInput {
            package_id: "delete-me",
            remove_installed_body: true,
        },
    );

    let mut permissions = fs::metadata(&layout.config_path)
        .expect("config metadata")
        .permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&layout.config_path, permissions).expect("config should become writable");

    assert!(
        result.is_err(),
        "readonly project config should make skill package deletion fail"
    );
    assert!(skill_dir.exists(), "skill body should not be removed");
    let config = fs::read_to_string(&layout.config_path).expect("config should read");
    assert!(config.contains("[skill_packages.delete-me]"));
    assert!(config.contains("[skill_sets.delete-me]"));
    let agent = get_agent_profile(&root, "uses-delete-me").expect("agent should load");
    assert_eq!(agent.skill_set_ids, vec!["delete-me".to_string()]);
    let lock = fs::read_to_string(root.join("skills-lock.json")).expect("lock should read");
    assert!(lock.contains("delete-me"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn locale_is_recorded_and_used_for_generated_evidence() {
    let root = test_root("locale");
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
        "worker",
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
