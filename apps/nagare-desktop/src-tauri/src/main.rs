use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use nagare_core::{
    AcceptRecoveryPlanResult, AddAgentProfileInput, AddArtifactTypeInput, AddDomainInput,
    AddMcpConnectionInput, AddSkillPackageInput, AdvanceUntilBlockedInput, AdvanceWorkItemInput,
    AgentModelSelection, AgentOutputRecord, AgentProfile, AgentProfileSource, AgentRunPurpose,
    AgentToolKind, ApplyRecoveryPlanInput, ApprovalPolicy, ArtifactType, CreateRecoveryPlanResult,
    CreateWorkItemInput, DeleteSkillPackageInput, DeleteSkillPackageResult, Domain,
    DomainWorkflowOverride, ExternalAgentBinding, ImprovementHistoryEntry,
    McpConnectionCatalogEntry, McpConnectionTestResult, NagareAgentSettings, ProjectLayout,
    ProjectMetadata, RUNTIME_MCP_CAPABILITIES, RecordImprovementInput, RecoveryPlan,
    RecoveryPlanStatus, ReviewResult, ReviewVerdict, SetNagareAgentSettingsInput,
    SetProjectMetadataInput, SkillPackageCatalogEntry, SkillSetCatalogEntry, TraceRecord,
    UpdateAgentProfileInput, UpdateArtifactTypeInput, UpdateDomainInput, UpdateMcpConnectionInput,
    WorkItem, WorkItemSnapshot, WorkItemStatus, WorkflowMode, WorkflowSettings, add_agent_profile,
    add_artifact_type, add_domain, add_mcp_connection, add_skill_package,
    advance_work_item_until_blocked, answer_work_item, apply_recovery_plan, approve_work_item,
    create_recovery_plan, create_work_item_with_input, delete_agent_profile, delete_artifact_type,
    delete_domain, delete_mcp_connection, delete_project_state, delete_skill_package,
    delete_work_item, get_agent_profile, get_artifact_type, get_nagare_agent_settings,
    get_project_metadata, get_work_item_snapshot, get_workflow_settings, init_project,
    list_agent_profiles, list_artifact_types, list_domains, list_improvement_history,
    list_mcp_connections, list_skill_packages, list_skill_set_catalog, list_work_items,
    list_work_trace, record_improvement_applied, record_improvement_dismissed, reject_work_item,
    set_nagare_agent_settings, set_project_metadata, set_project_organizer_agent,
    set_workflow_settings, test_mcp_connection, update_agent_profile, update_artifact_type,
    update_domain, update_mcp_connection,
};
use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    ui_source: &'static str,
}

#[derive(Serialize)]
struct DesktopState {
    app: AppInfo,
    root: String,
    initialized: bool,
    project: Option<ProjectView>,
    work_items: Vec<WorkListItem>,
    agents: Vec<AgentView>,
    domains: Vec<DomainView>,
    artifact_types: Vec<ArtifactTypeView>,
    skill_sets: Vec<SkillSetView>,
    skill_packages: Vec<SkillPackageView>,
    mcp_connections: Vec<McpConnectionView>,
    mcp_capabilities: Vec<McpCapabilityView>,
    runtimes: Vec<RuntimeView>,
    insights: InsightsView,
}

#[derive(Serialize)]
struct ProjectView {
    name: String,
    icon: String,
    root: String,
    default_domain_id: String,
    default_artifact_type_id: String,
    workflow_mode: String,
    approval_policy: String,
    organizer_agent_id: String,
    organizer_label: String,
    work_agent: String,
    review_agent: String,
    agent_count: usize,
    domain_count: usize,
    artifact_type_count: usize,
    work_count: usize,
    status_counts: Vec<StatusCountView>,
}

#[derive(Serialize)]
struct StatusCountView {
    label: String,
    kind: String,
    count: usize,
}

#[derive(Serialize)]
struct AgentView {
    id: String,
    name: String,
    avatar: String,
    role: String,
    description: String,
    runtime: String,
    adapter: String,
    tool_kind: String,
    model: String,
    model_provider: String,
    model_base_url: String,
    prompt: String,
    specialties: Vec<String>,
    domain_ids: Vec<String>,
    artifact_type_ids: Vec<String>,
    skill_set_ids: Vec<String>,
    mcp_connection_ids: Vec<String>,
    source: String,
    builtin: bool,
    mcp_assignable: bool,
    mcp_note: String,
}

#[derive(Serialize)]
struct DomainView {
    id: String,
    name: String,
    description: String,
    shared_knowledge: Vec<String>,
    common_rubric: Vec<String>,
    dispatch_hints: Vec<String>,
    shared_knowledge_count: usize,
    common_rubric_count: usize,
    artifact_type_count: usize,
}

#[derive(Serialize)]
struct ArtifactTypeView {
    id: String,
    domain_id: String,
    name: String,
    description: String,
    knowledge: Vec<String>,
    rubric: Vec<String>,
    dispatch_hints: Vec<String>,
    knowledge_count: usize,
    rubric_count: usize,
    rubric_score_total: u32,
    rubric_version: u32,
}

#[derive(Serialize)]
struct SkillSetView {
    id: String,
    paths: Vec<String>,
    required_capabilities: Vec<String>,
    optional_capabilities: Vec<String>,
}

#[derive(Serialize)]
struct SkillPackageView {
    id: String,
    source_kind: String,
    source: String,
    install_scope: String,
    installed_targets: Vec<String>,
    provided_skill_sets: Vec<String>,
}

#[derive(Serialize)]
struct McpCapabilityView {
    tool_kind: String,
    runtime_label: String,
    scope: String,
    agent_assignable: bool,
    note: String,
}

#[derive(Serialize)]
struct McpConnectionView {
    id: String,
    name: String,
    tool_kind: String,
    runtime_label: String,
    scope: String,
    agent_assignable: bool,
    command: String,
    args: Vec<String>,
    env: Vec<String>,
    env_count: usize,
    test_args: Vec<String>,
    test_status: String,
    test_detail: String,
    tested_at: String,
}

#[derive(Serialize)]
struct WorkListItem {
    id: String,
    title: String,
    description: String,
    project: String,
    status_label: String,
    status_kind: String,
    next_action: String,
    result_summary: String,
    updated_at: String,
    workflow_mode: String,
    approval_policy: String,
}

#[derive(Serialize)]
struct WorkDetailView {
    root: String,
    item: WorkListItem,
    domain_id: String,
    artifact_type_id: String,
    next_action_kind: String,
    approval_ready: bool,
    question: Option<String>,
    question_source: String,
    recovery: Option<RecoveryView>,
    request: String,
    answer: String,
    artifacts: Vec<ArtifactView>,
    review: Option<ReviewView>,
    steps: Vec<StepView>,
}

#[derive(Serialize)]
struct ArtifactView {
    title: String,
    uri: String,
}

#[derive(Serialize)]
struct ReviewView {
    verdict: String,
    summary: String,
    score_label: String,
    concerns: Vec<String>,
    items: Vec<ReviewItemView>,
}

#[derive(Serialize)]
struct ReviewItemView {
    item: String,
    verdict: String,
    evidence: String,
    score_label: String,
    concern_note: String,
}

#[derive(Serialize)]
struct StepView {
    kind: String,
    title: String,
    state: String,
    actor: String,
    summary: String,
    rationale: String,
    input: String,
    output: String,
    knowledge_refs: Vec<String>,
    diagnostics: String,
    review_items: Vec<ReviewItemView>,
}

#[derive(Serialize)]
struct RecoveryView {
    id: String,
    status: String,
    action: String,
    failure_class: String,
    reason: String,
    summary: String,
    impact: String,
    handoff_completed: Vec<String>,
    handoff_pending: Vec<String>,
    target_agent: String,
    command_hint: String,
    warnings: Vec<String>,
    prompt_hint: Option<String>,
}

#[derive(Clone, Serialize)]
struct RuntimeView {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    available: bool,
    detail: String,
    model_note: &'static str,
    model_mode: &'static str,
    model_choices: Vec<&'static str>,
    configured_model: String,
    configured_provider: String,
    configured_base_url: String,
    configured_agent_count: usize,
}

#[derive(Serialize)]
struct InsightsView {
    review_count: usize,
    average_score_label: String,
    concern_count: usize,
    proposal_count: usize,
    agent_scores: Vec<AgentInsightView>,
    issue_matrix: Vec<InsightIssueView>,
    proposals: Vec<ImprovementProposalView>,
    applied_improvements: Vec<AppliedImprovementView>,
    recent_reviews: Vec<InsightReviewView>,
}

#[derive(Serialize)]
struct AgentInsightView {
    agent_id: String,
    agent_name: String,
    role: String,
    review_count: usize,
    average_score: u8,
    average_score_label: String,
    status_label: String,
    top_issue: String,
}

#[derive(Serialize)]
struct InsightIssueView {
    agent_name: String,
    item: String,
    rate: u8,
    rate_label: String,
    occurrences: usize,
    suggestion_kind: String,
}

#[derive(Serialize)]
struct ImprovementProposalView {
    id: String,
    kind: String,
    title: String,
    target_label: String,
    summary: String,
    evidence: String,
    current_text: String,
    suggested_text: String,
    diff_lines: Vec<String>,
    next_step: String,
    action_label: String,
}

#[derive(Serialize)]
struct AppliedImprovementView {
    id: String,
    proposal_id: String,
    kind: String,
    title: String,
    target_label: String,
    summary: String,
    applied_at: String,
    effect_label: String,
}

#[derive(Serialize)]
struct InsightReviewView {
    work_id: String,
    title: String,
    agent_name: String,
    verdict: String,
    score_label: String,
    concerns: Vec<String>,
}

#[derive(Deserialize)]
struct CreateWorkRequest {
    root: Option<String>,
    description: String,
    project: Option<String>,
    domain_id: Option<String>,
    artifact_type_id: Option<String>,
    workflow_mode: Option<String>,
    approval_policy: Option<String>,
}

#[derive(Clone, Deserialize)]
struct WorkActionRequest {
    root: Option<String>,
    id: String,
    prompt: Option<String>,
    dev_command: Option<String>,
    dispatch_dev_command: Option<String>,
    review_dev_command: Option<String>,
    synthesis_dev_command: Option<String>,
    max_steps: Option<usize>,
    auto_recover: Option<bool>,
}

#[derive(Deserialize)]
struct HumanDecisionRequest {
    root: Option<String>,
    id: String,
    rationale: Option<String>,
}

#[derive(Deserialize)]
struct AnswerRequest {
    root: Option<String>,
    id: String,
    question: Option<String>,
    answer: String,
}

#[derive(Deserialize)]
struct RecoveryRequest {
    root: Option<String>,
    id: String,
    recovery_plan_id: Option<String>,
    prompt: Option<String>,
    dev_command: Option<String>,
}

#[derive(Deserialize)]
struct ProjectSettingsRequest {
    root: Option<String>,
    display_name: Option<String>,
    icon: Option<String>,
    default_domain_id: Option<String>,
    default_artifact_type_id: Option<String>,
    organizer_agent_id: Option<String>,
    work_agent_id: Option<String>,
    review_agent_id: Option<String>,
    workflow_mode: Option<String>,
    approval_policy: Option<String>,
    improvement_proposal_id: Option<String>,
    improvement_kind: Option<String>,
    improvement_title: Option<String>,
    improvement_target_label: Option<String>,
    improvement_summary: Option<String>,
    improvement_evidence: Option<String>,
}

#[derive(Deserialize)]
struct ProjectDeleteRequest {
    root: Option<String>,
}

#[derive(Deserialize)]
struct SaveDomainRequest {
    root: Option<String>,
    id: String,
    display_name: String,
    description: Option<String>,
    shared_knowledge: Option<String>,
    common_rubric: Option<String>,
    dispatch_hints: Option<String>,
}

#[derive(Deserialize)]
struct DeleteDomainRequest {
    root: Option<String>,
    id: String,
}

#[derive(Deserialize)]
struct SaveArtifactTypeRequest {
    root: Option<String>,
    id: String,
    domain_id: Option<String>,
    display_name: String,
    description: Option<String>,
    knowledge: Option<String>,
    rubric: Option<String>,
    dispatch_hints: Option<String>,
    improvement_proposal_id: Option<String>,
    improvement_kind: Option<String>,
    improvement_title: Option<String>,
    improvement_target_label: Option<String>,
    improvement_summary: Option<String>,
    improvement_evidence: Option<String>,
}

#[derive(Deserialize)]
struct DeleteArtifactTypeRequest {
    root: Option<String>,
    id: String,
}

#[derive(Deserialize)]
struct SaveAgentRequest {
    root: Option<String>,
    id: String,
    display_name: String,
    avatar: Option<String>,
    role: String,
    tool_kind: String,
    model: Option<String>,
    model_provider: Option<String>,
    model_base_url: Option<String>,
    description: Option<String>,
    prompt: Option<String>,
    specialties: Vec<String>,
    domain_ids: Vec<String>,
    artifact_type_ids: Vec<String>,
    skill_set_ids: Vec<String>,
    mcp_connection_ids: Vec<String>,
    improvement_proposal_id: Option<String>,
    improvement_kind: Option<String>,
    improvement_title: Option<String>,
    improvement_target_label: Option<String>,
    improvement_summary: Option<String>,
    improvement_evidence: Option<String>,
}

#[derive(Deserialize)]
struct DeleteAgentRequest {
    root: Option<String>,
    id: String,
}

#[derive(Deserialize)]
struct AgentPromptDraftRequest {
    root: Option<String>,
    display_name: String,
    role: String,
    description: Option<String>,
    specialties: Vec<String>,
    domain_ids: Vec<String>,
    artifact_type_ids: Vec<String>,
}

#[derive(Deserialize)]
struct RubricDraftRequest {
    root: Option<String>,
    domain_id: Option<String>,
    display_name: String,
    description: Option<String>,
    knowledge: Vec<String>,
}

#[derive(Serialize)]
struct DraftTextResponse {
    text: String,
}

struct ImprovementRequestFields {
    proposal_id: String,
    kind: String,
    title: String,
    target_label: String,
    summary: String,
    evidence: String,
}

#[derive(Deserialize)]
struct DismissImprovementRequest {
    root: Option<String>,
    proposal_id: String,
    kind: Option<String>,
    title: Option<String>,
    target_label: Option<String>,
    summary: Option<String>,
    evidence: Option<String>,
}

#[derive(Deserialize)]
struct AddSkillRequest {
    root: Option<String>,
    package_id: Option<String>,
    source_kind: String,
    source: Option<String>,
    path: Option<String>,
    install: Option<bool>,
    install_scope: Option<String>,
    install_targets: Vec<String>,
    skill_set_id: Option<String>,
    skill_paths: Option<String>,
    required_capabilities: Option<String>,
    optional_capabilities: Option<String>,
}

#[derive(Deserialize)]
struct DeleteSkillPackageRequest {
    root: Option<String>,
    package_id: String,
    remove_installed_body: Option<bool>,
}

#[derive(Serialize)]
struct SkillDeleteResponse {
    state: DesktopState,
    package_id: String,
    removed_skill_sets: Vec<String>,
    detached_agents: Vec<String>,
    installed_body_removed: bool,
    warnings: Vec<String>,
}

#[derive(Deserialize)]
struct SaveMcpConnectionRequest {
    root: Option<String>,
    id: String,
    display_name: String,
    tool_kind: String,
    command: String,
    args: Option<String>,
    env: Option<String>,
    test_args: Option<String>,
}

#[derive(Deserialize)]
struct McpConnectionRequest {
    root: Option<String>,
    id: String,
}

#[derive(Serialize)]
struct McpTestResponse {
    state: DesktopState,
    success: bool,
    detail: String,
}

#[derive(Deserialize)]
struct InitializeProjectRequest {
    root: Option<String>,
    runtime_id: Option<String>,
    display_name: Option<String>,
    icon: Option<String>,
}

#[derive(Deserialize)]
struct RuntimeStatusRequest {
    root: Option<String>,
    runtime_id: String,
}

#[derive(Deserialize)]
struct SaveRuntimeModelDefaultsRequest {
    root: Option<String>,
    runtime_id: String,
    model_provider: Option<String>,
    model: Option<String>,
    model_base_url: Option<String>,
}

#[derive(Serialize)]
struct RuntimeStatusResponse {
    runtime: RuntimeView,
    state: DesktopState,
}

#[derive(Deserialize)]
struct ReadArtifactContentRequest {
    root: Option<String>,
    uri: String,
}

#[derive(Serialize)]
struct ArtifactContentResponse {
    display_path: String,
    content: String,
    size_bytes: u64,
    truncated: bool,
}

#[tauri::command]
fn app_info() -> AppInfo {
    app_info_value()
}

#[tauri::command]
fn app_state(root: Option<String>) -> Result<DesktopState, String> {
    if let Some(saved_root) = root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let saved_root = PathBuf::from(saved_root);
        if !saved_root.is_dir() {
            return Ok(empty_desktop_state());
        }
    }
    let root = match resolve_desktop_root(root) {
        Ok(root) => root,
        Err(error) if is_missing_path_error(&error) => return Ok(empty_desktop_state()),
        Err(error) => return Err(error),
    };
    match desktop_state(root) {
        Ok(state) => Ok(state),
        Err(error) if is_missing_path_error(&error) => Ok(empty_desktop_state()),
        Err(error) => Err(error),
    }
}

#[tauri::command]
fn initialize_project(root: Option<String>) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(root)?;
    ensure_initial_runtime_available("codex")?;
    let init = init_project(&root).map_err(|error| error.to_string())?;
    if init.created_config {
        let default_name = project_name_from_root(&root);
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some(&default_name),
                icon: Some(default_project_icon()),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .map_err(|error| error.to_string())?;
    }
    configure_initial_agent_runtime(&root, "codex")?;
    desktop_state(root)
}

#[tauri::command]
fn initialize_project_with_runtime(
    request: InitializeProjectRequest,
) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let runtime_id = request
        .runtime_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("codex");
    ensure_initial_runtime_available(runtime_id)?;
    let init = init_project(&root).map_err(|error| error.to_string())?;
    let requested_name = request
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let requested_icon = request
        .icon
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if init.created_config || requested_name.is_some() || requested_icon.is_some() {
        let default_name = project_name_from_root(&root);
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some(requested_name.unwrap_or(default_name.as_str())),
                icon: Some(requested_icon.unwrap_or(default_project_icon())),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .map_err(|error| error.to_string())?;
    }
    configure_initial_agent_runtime(&root, runtime_id)?;
    desktop_state(root)
}

#[tauri::command]
fn refresh_runtime_status(request: RuntimeStatusRequest) -> Result<RuntimeStatusResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let mut state = desktop_state(root)?;
    let runtime = runtime_view_by_id(request.runtime_id.trim(), &state.agents)?;
    state.runtimes = state
        .runtimes
        .into_iter()
        .map(|item| {
            if item.id == runtime.id {
                runtime.clone()
            } else {
                item
            }
        })
        .collect();
    Ok(RuntimeStatusResponse { runtime, state })
}

#[tauri::command]
fn save_runtime_model_defaults(
    request: SaveRuntimeModelDefaultsRequest,
) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let runtime_id = request.runtime_id.trim();
    let (runtime, adapter, _provider) = initial_runtime_mapping(runtime_id)?;
    let target_tool_kind = AgentToolKind::infer(runtime, adapter);
    let model_id = request.model.as_deref().map(str::trim).unwrap_or("");
    let model = AgentModelSelection {
        provider: request
            .model_provider
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        id: model_id.to_string(),
        base_url: request
            .model_base_url
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        api_key_env: String::new(),
    };
    let profiles = list_agent_profiles(&root).map_err(|error| error.to_string())?;
    let mut updated = 0usize;
    let mut previous_models = Vec::new();
    for profile in profiles {
        if profile.tool_kind == target_tool_kind || profile.runtime == runtime {
            let previous_model = profile.model.clone();
            if let Err(error) = update_agent_profile(
                &root,
                &profile.id,
                UpdateAgentProfileInput {
                    model: Some(model.clone()),
                    ..UpdateAgentProfileInput::default()
                },
            ) {
                restore_runtime_model_defaults(&root, previous_models)?;
                return Err(error.to_string());
            }
            previous_models.push((profile.id.clone(), previous_model));
            updated += 1;
        }
    }
    if updated == 0 {
        return Err("この実行環境を使うエージェントがありません。".to_string());
    }
    desktop_state(root)
}

fn restore_runtime_model_defaults(
    root: &Path,
    previous_models: Vec<(String, AgentModelSelection)>,
) -> Result<(), String> {
    for (agent_id, model) in previous_models {
        update_agent_profile(
            root,
            &agent_id,
            UpdateAgentProfileInput {
                model: Some(model),
                ..UpdateAgentProfileInput::default()
            },
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn read_artifact_content(
    request: ReadArtifactContentRequest,
) -> Result<ArtifactContentResponse, String> {
    const MAX_PREVIEW_BYTES: u64 = 256 * 1024;
    let root = resolve_desktop_root(request.root)?;
    let artifact_path = resolve_artifact_uri(&root, request.uri.trim())?;
    let metadata = fs::metadata(&artifact_path)
        .map_err(|error| format!("failed to read artifact metadata: {error}"))?;
    if !metadata.is_file() {
        return Err(format!(
            "artifact path `{}` is not a file",
            artifact_path.display()
        ));
    }

    let mut file = fs::File::open(&artifact_path)
        .map_err(|error| format!("failed to open artifact: {error}"))?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read artifact: {error}"))?;
    let truncated = bytes.len() as u64 > MAX_PREVIEW_BYTES;
    if truncated {
        bytes.truncate(MAX_PREVIEW_BYTES as usize);
    }
    let content = String::from_utf8_lossy(&bytes).to_string();

    Ok(ArtifactContentResponse {
        display_path: root_to_string(&artifact_path),
        content,
        size_bytes: metadata.len(),
        truncated,
    })
}

fn configure_initial_agent_runtime(root: &Path, runtime_id: &str) -> Result<(), String> {
    let (runtime, adapter, provider) = initial_runtime_mapping(runtime_id)?;
    for agent_id in ["worker", "reviewer", "dispatcher", "supervisor"] {
        update_agent_profile(
            root,
            agent_id,
            UpdateAgentProfileInput {
                runtime: Some(runtime),
                adapter: Some(adapter),
                managed_by: Some("nagare"),
                external: Some(ExternalAgentBinding {
                    provider: provider.to_string(),
                    agent_id: agent_id.to_string(),
                    managed: true,
                    source: "created".to_string(),
                }),
                ..UpdateAgentProfileInput::default()
            },
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn initial_runtime_mapping(
    runtime_id: &str,
) -> Result<(&'static str, &'static str, &'static str), String> {
    match runtime_id
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "codex" | "codex_cli" => Ok(("codex-local", "process-codex-cli", "codex-cli")),
        "claude" | "claude_code" => Ok(("claude-local", "process-claude-code", "claude-code")),
        "opencode" | "open_code" => Ok(("opencode-local", "process-opencode", "opencode")),
        "openclaw" => Ok(("openclaw-local", "process-openclaw-agent", "openclaw")),
        other => Err(format!(
            "初期セットアップで `{other}` は既定エージェントの実行環境として使えません。Claude Code / Codex CLI / OpenCode / OpenClaw から選択してください。"
        )),
    }
}

#[tauri::command]
fn save_project_settings(request: ProjectSettingsRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root.clone())?;
    let improvement = improvement_request_from_project(&request);
    let requested_workflow_mode = request
        .workflow_mode
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(WorkflowMode::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    let requested_approval_policy = request
        .approval_policy
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ApprovalPolicy::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    let organizer_agent = request
        .organizer_agent_id
        .as_deref()
        .map(str::trim)
        .map(|value| {
            if value.is_empty() || value == "__builtin__" {
                None
            } else {
                Some(value)
            }
        })
        .unwrap_or(None);
    let work_agent = request
        .work_agent_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let review_agent = request
        .review_agent_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    validate_project_settings_agents(&root, [organizer_agent, work_agent, review_agent])?;
    let previous_metadata = get_project_metadata(&root).map_err(|error| error.to_string())?;
    let previous_workflow = get_workflow_settings(&root).map_err(|error| error.to_string())?;
    let previous_agent_settings =
        get_nagare_agent_settings(&root).map_err(|error| error.to_string())?;
    if request.display_name.is_some()
        || request.icon.is_some()
        || request.default_domain_id.is_some()
        || request.default_artifact_type_id.is_some()
    {
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: request.display_name.as_deref(),
                icon: request.icon.as_deref(),
                default_domain_id: request.default_domain_id.as_deref(),
                default_artifact_type_id: request.default_artifact_type_id.as_deref(),
            },
        )
        .map_err(|error| error.to_string())?;
    }
    let mut workflow = previous_workflow.clone();
    if let Some(mode) = requested_workflow_mode {
        workflow.default_progress_mode = mode;
    }
    if let Some(policy) = requested_approval_policy {
        workflow.approval_policy = policy;
    }
    set_workflow_settings(&root, workflow).map_err(|error| error.to_string())?;
    if request.organizer_agent_id.is_some() {
        set_project_organizer_agent(&root, organizer_agent).map_err(|error| error.to_string())?;
    }
    if work_agent.is_some() || review_agent.is_some() {
        set_nagare_agent_settings(
            &root,
            SetNagareAgentSettingsInput {
                work_agent,
                review_agent,
                organizer_agent: None,
                dispatch_agent: None,
                supervisor_agent: None,
            },
        )
        .map_err(|error| error.to_string())?;
    }
    if let Err(error) = record_improvement_request(&root, improvement) {
        restore_project_settings(
            &root,
            previous_metadata,
            previous_workflow,
            previous_agent_settings,
        )?;
        return Err(error);
    }
    desktop_state(root)
}

fn restore_project_settings(
    root: &Path,
    metadata: ProjectMetadata,
    workflow: WorkflowSettings,
    agents: NagareAgentSettings,
) -> Result<(), String> {
    set_project_metadata(
        root,
        SetProjectMetadataInput {
            name: Some(&metadata.name),
            icon: Some(&metadata.icon),
            default_domain_id: Some(&metadata.default_domain_id),
            default_artifact_type_id: Some(&metadata.default_artifact_type_id),
        },
    )
    .map_err(|error| error.to_string())?;
    set_workflow_settings(root, workflow).map_err(|error| error.to_string())?;
    set_project_organizer_agent(root, agents.organizer_agent.as_deref())
        .map_err(|error| error.to_string())?;
    set_nagare_agent_settings(
        root,
        SetNagareAgentSettingsInput {
            work_agent: Some(&agents.work_agent),
            review_agent: Some(&agents.review_agent),
            organizer_agent: None,
            dispatch_agent: Some(&agents.dispatch_agent),
            supervisor_agent: Some(&agents.supervisor_agent),
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn validate_project_settings_agents<const N: usize>(
    root: &Path,
    agent_ids: [Option<&str>; N],
) -> Result<(), String> {
    let requested = agent_ids
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .collect::<BTreeSet<_>>();
    if requested.is_empty() {
        return Ok(());
    }
    let known = list_agent_profiles(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|agent| agent.id)
        .collect::<BTreeSet<_>>();
    if let Some(missing) = requested.iter().find(|id| !known.contains(**id)) {
        return Err(format!("agent profile `{missing}` not found"));
    }
    Ok(())
}

#[tauri::command]
fn delete_project(request: ProjectDeleteRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    delete_project_state(&root).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn save_domain(request: SaveDomainRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let id = request.id.trim();
    let display_name = request.display_name.trim();
    if id.is_empty() {
        return Err("ドメインIDを入力してください。".to_string());
    }
    if display_name.is_empty() {
        return Err("ドメイン名を入力してください。".to_string());
    }
    let exists = list_domains(&root)
        .map_err(|error| error.to_string())?
        .iter()
        .any(|domain| domain.id == id);
    if exists {
        update_domain(
            &root,
            id,
            UpdateDomainInput {
                display_name: Some(display_name),
                description: Some(optional_text(request.description.as_deref())),
                shared_knowledge: Some(text_lines(request.shared_knowledge.as_deref())),
                common_rubric: Some(text_lines(request.common_rubric.as_deref())),
                dispatch_hints: Some(text_lines(request.dispatch_hints.as_deref())),
                workflow: None,
            },
        )
        .map_err(|error| error.to_string())?;
    } else {
        add_domain(
            &root,
            AddDomainInput {
                id,
                display_name,
                description: optional_text(request.description.as_deref()),
                shared_knowledge: text_lines(request.shared_knowledge.as_deref()),
                common_rubric: text_lines(request.common_rubric.as_deref()),
                dispatch_hints: text_lines(request.dispatch_hints.as_deref()),
                workflow: DomainWorkflowOverride::default(),
            },
        )
        .map_err(|error| error.to_string())?;
    }
    desktop_state(root)
}

#[tauri::command]
fn delete_domain_command(request: DeleteDomainRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let id = request.id.trim();
    if id.is_empty() {
        return Err("削除するドメインIDが空です。".to_string());
    }
    delete_domain(&root, id).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn save_artifact_type(request: SaveArtifactTypeRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root.clone())?;
    let improvement = improvement_request_from_artifact(&request);
    let id = request.id.trim();
    let display_name = request.display_name.trim();
    let rubric_raw = optional_text(request.rubric.as_deref());
    validate_rubric_markdown(rubric_raw)?;
    let rubric_lines = text_lines(request.rubric.as_deref());
    let domain_id = request
        .domain_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "ドメインを選択してください。".to_string())?;
    if id.is_empty() {
        return Err("成果物種別IDを入力してください。".to_string());
    }
    if display_name.is_empty() {
        return Err("成果物種別名を入力してください。".to_string());
    }
    let previous_artifact = get_artifact_type(&root, id).ok();
    if previous_artifact.is_some() {
        update_artifact_type(
            &root,
            id,
            UpdateArtifactTypeInput {
                domain_id: Some(Some(domain_id)),
                display_name: Some(display_name),
                description: Some(optional_text(request.description.as_deref())),
                artifact_types: Some(text_lines(request.knowledge.as_deref())),
                rubric: Some(rubric_lines),
                dispatch_hints: Some(text_lines(request.dispatch_hints.as_deref())),
                workflow: None,
            },
        )
        .map_err(|error| error.to_string())?;
    } else {
        add_artifact_type(
            &root,
            AddArtifactTypeInput {
                id,
                domain_id: Some(domain_id),
                display_name,
                description: optional_text(request.description.as_deref()),
                artifact_types: text_lines(request.knowledge.as_deref()),
                rubric: rubric_lines,
                dispatch_hints: text_lines(request.dispatch_hints.as_deref()),
                workflow: DomainWorkflowOverride::default(),
            },
        )
        .map_err(|error| error.to_string())?;
    }
    if let Err(error) = record_improvement_request(&root, improvement) {
        restore_artifact_type_save(&root, id, previous_artifact)?;
        return Err(error);
    }
    desktop_state(root)
}

fn restore_artifact_type_save(
    root: &Path,
    id: &str,
    previous: Option<ArtifactType>,
) -> Result<(), String> {
    match previous {
        Some(artifact) => {
            update_artifact_type(
                root,
                id,
                UpdateArtifactTypeInput {
                    domain_id: Some(artifact.domain_id.as_deref()),
                    display_name: Some(&artifact.display_name),
                    description: Some(&artifact.description),
                    artifact_types: Some(artifact.artifact_types.clone()),
                    rubric: Some(artifact.rubric.clone()),
                    dispatch_hints: Some(artifact.dispatch_hints.clone()),
                    workflow: Some(artifact.workflow),
                },
            )
            .map_err(|error| error.to_string())?;
        }
        None => {
            delete_artifact_type(root, id).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn delete_artifact_type_command(
    request: DeleteArtifactTypeRequest,
) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let id = request.id.trim();
    if id.is_empty() {
        return Err("削除する成果物種別IDが空です。".to_string());
    }
    delete_artifact_type(&root, id).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn save_agent(request: SaveAgentRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root.clone())?;
    let improvement = improvement_request_from_agent(&request);
    let id = request.id.trim();
    let display_name = request.display_name.trim();
    let role = request.role.trim();
    if id.is_empty() {
        return Err("エージェントIDを入力してください。".to_string());
    }
    if display_name.is_empty() {
        return Err("エージェント名を入力してください。".to_string());
    }
    if role.is_empty() {
        return Err("ロールを選択してください。".to_string());
    }
    let avatar = optional_text(request.avatar.as_deref());
    let (runtime, adapter) = agent_runtime_adapter(&request.tool_kind)?;
    let model = AgentModelSelection {
        provider: request
            .model_provider
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        id: request
            .model
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        base_url: request
            .model_base_url
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        api_key_env: String::new(),
    };
    let previous_agent = get_agent_profile(&root, id).ok();
    if previous_agent.is_some() {
        update_agent_profile(
            &root,
            id,
            UpdateAgentProfileInput {
                display_name: Some(display_name),
                avatar: Some(avatar),
                runtime: Some(runtime),
                adapter: Some(adapter),
                role: Some(role),
                working_dir: Some("."),
                description: Some(optional_text(request.description.as_deref())),
                specialties: Some(normalized_values(request.specialties)),
                skill_set_ids: Some(normalized_values(request.skill_set_ids)),
                domain_ids: Some(normalized_values(request.domain_ids)),
                artifact_type_ids: Some(normalized_values(request.artifact_type_ids)),
                mcp_connection_ids: Some(normalized_values(request.mcp_connection_ids)),
                prompt: request.prompt.as_deref(),
                managed_by: Some("nagare"),
                model: Some(model),
                external: None,
                output_contract: None,
            },
        )
        .map_err(|error| error.to_string())?;
    } else {
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id,
                display_name,
                runtime,
                adapter,
                role,
                working_dir: ".",
                description: optional_text(request.description.as_deref()),
                specialties: normalized_values(request.specialties),
                skill_set_ids: normalized_values(request.skill_set_ids),
                domain_ids: normalized_values(request.domain_ids),
                artifact_type_ids: normalized_values(request.artifact_type_ids),
                mcp_connection_ids: normalized_values(request.mcp_connection_ids),
                managed_by: Some("nagare"),
                model,
                external: ExternalAgentBinding::default(),
            },
        )
        .map_err(|error| error.to_string())?;
        if !avatar.is_empty()
            || request
                .prompt
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
        {
            update_agent_profile(
                &root,
                id,
                UpdateAgentProfileInput {
                    avatar: Some(avatar),
                    prompt: request.prompt.as_deref(),
                    ..UpdateAgentProfileInput::default()
                },
            )
            .map_err(|error| error.to_string())?;
        }
    }
    if let Err(error) = record_improvement_request(&root, improvement) {
        restore_agent_save(&root, id, previous_agent)?;
        return Err(error);
    }
    desktop_state(root)
}

fn restore_agent_save(root: &Path, id: &str, previous: Option<AgentProfile>) -> Result<(), String> {
    match previous {
        Some(profile) => {
            update_agent_profile(
                root,
                id,
                UpdateAgentProfileInput {
                    display_name: Some(&profile.display_name),
                    avatar: Some(&profile.avatar),
                    runtime: Some(&profile.runtime),
                    adapter: Some(&profile.adapter),
                    role: Some(&profile.role),
                    working_dir: Some(&profile.working_dir),
                    description: Some(&profile.description),
                    specialties: Some(profile.specialties.clone()),
                    skill_set_ids: Some(profile.skill_set_ids.clone()),
                    domain_ids: Some(profile.domain_ids.clone()),
                    artifact_type_ids: Some(profile.artifact_type_ids.clone()),
                    mcp_connection_ids: Some(profile.mcp_connection_ids.clone()),
                    prompt: Some(&profile.prompt.instructions),
                    managed_by: Some(&profile.managed_by),
                    model: Some(profile.model.clone()),
                    external: Some(profile.external.clone()),
                    output_contract: None,
                },
            )
            .map_err(|error| error.to_string())?;
        }
        None => {
            delete_agent_profile(root, id).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn delete_agent_command(request: DeleteAgentRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let id = request.id.trim();
    if id.is_empty() {
        return Err("削除するエージェントIDが空です。".to_string());
    }
    delete_agent_profile(&root, id).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn generate_agent_prompt_draft(
    request: AgentPromptDraftRequest,
) -> Result<DraftTextResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let domains = list_domains(&root).map_err(|error| error.to_string())?;
    let artifacts = list_artifact_types(&root).map_err(|error| error.to_string())?;
    let selected_domain_labels = request
        .domain_ids
        .iter()
        .filter_map(|id| {
            domains
                .iter()
                .find(|domain| domain.id == *id)
                .map(|domain| domain.display_name.clone())
        })
        .collect::<Vec<_>>();
    let selected_artifact_labels = request
        .artifact_type_ids
        .iter()
        .filter_map(|id| {
            artifacts
                .iter()
                .find(|artifact| artifact.id == *id)
                .map(|artifact| artifact.display_name.clone())
        })
        .collect::<Vec<_>>();
    Ok(DraftTextResponse {
        text: agent_prompt_draft_text(
            &request.display_name,
            &request.role,
            request.description.as_deref(),
            &request.specialties,
            &selected_domain_labels,
            &selected_artifact_labels,
        ),
    })
}

#[tauri::command]
fn generate_rubric_draft(request: RubricDraftRequest) -> Result<DraftTextResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let domain = request.domain_id.as_deref().and_then(|id| {
        list_domains(&root)
            .ok()?
            .into_iter()
            .find(|domain| domain.id == id)
    });
    Ok(DraftTextResponse {
        text: rubric_draft_text(
            &request.display_name,
            request.description.as_deref(),
            &request.knowledge,
            domain.as_ref(),
        ),
    })
}

fn improvement_request_from_project(
    request: &ProjectSettingsRequest,
) -> Option<ImprovementRequestFields> {
    improvement_request_fields(
        request.improvement_proposal_id.as_deref(),
        request.improvement_kind.as_deref(),
        request.improvement_title.as_deref(),
        request.improvement_target_label.as_deref(),
        request.improvement_summary.as_deref(),
        request.improvement_evidence.as_deref(),
    )
}

fn improvement_request_from_artifact(
    request: &SaveArtifactTypeRequest,
) -> Option<ImprovementRequestFields> {
    improvement_request_fields(
        request.improvement_proposal_id.as_deref(),
        request.improvement_kind.as_deref(),
        request.improvement_title.as_deref(),
        request.improvement_target_label.as_deref(),
        request.improvement_summary.as_deref(),
        request.improvement_evidence.as_deref(),
    )
}

fn improvement_request_from_agent(request: &SaveAgentRequest) -> Option<ImprovementRequestFields> {
    improvement_request_fields(
        request.improvement_proposal_id.as_deref(),
        request.improvement_kind.as_deref(),
        request.improvement_title.as_deref(),
        request.improvement_target_label.as_deref(),
        request.improvement_summary.as_deref(),
        request.improvement_evidence.as_deref(),
    )
}

fn improvement_request_fields(
    proposal_id: Option<&str>,
    kind: Option<&str>,
    title: Option<&str>,
    target_label: Option<&str>,
    summary: Option<&str>,
    evidence: Option<&str>,
) -> Option<ImprovementRequestFields> {
    let proposal_id = proposal_id?.trim();
    if proposal_id.is_empty() {
        return None;
    }
    Some(ImprovementRequestFields {
        proposal_id: proposal_id.to_string(),
        kind: kind.unwrap_or("改善").trim().to_string(),
        title: title.unwrap_or("改善を適用").trim().to_string(),
        target_label: target_label.unwrap_or("").trim().to_string(),
        summary: summary.unwrap_or("").trim().to_string(),
        evidence: evidence.unwrap_or("").trim().to_string(),
    })
}

fn record_improvement_request(
    root: &Path,
    improvement: Option<ImprovementRequestFields>,
) -> Result<(), String> {
    let Some(improvement) = improvement else {
        return Ok(());
    };
    #[cfg(test)]
    if improvement.proposal_id == "__fail_record_improvement__" {
        return Err("forced improvement history failure".to_string());
    }
    record_improvement_applied(
        root,
        RecordImprovementInput {
            proposal_id: &improvement.proposal_id,
            kind: &improvement.kind,
            title: &improvement.title,
            target_label: &improvement.target_label,
            summary: &improvement.summary,
            evidence: &improvement.evidence,
        },
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn dismiss_improvement(request: DismissImprovementRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let proposal_id = request.proposal_id.trim();
    if proposal_id.is_empty() {
        return Err("improvement proposal id must not be empty".to_string());
    }
    record_improvement_dismissed(
        &root,
        RecordImprovementInput {
            proposal_id,
            kind: request.kind.as_deref().unwrap_or(""),
            title: request.title.as_deref().unwrap_or("改善提案を見送り"),
            target_label: request.target_label.as_deref().unwrap_or(""),
            summary: request.summary.as_deref().unwrap_or(""),
            evidence: request.evidence.as_deref().unwrap_or(""),
        },
    )
    .map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn add_skill(request: AddSkillRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let source_kind = request.source_kind.trim();
    if source_kind.is_empty() {
        return Err("追加元を選択してください。".to_string());
    }
    let install = request.install.unwrap_or(false);
    add_skill_package(
        &root,
        AddSkillPackageInput {
            id: request
                .package_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            source_kind,
            source: request
                .source
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            path: request
                .path
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            install,
            install_scope: request
                .install_scope
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            install_targets: normalized_values(request.install_targets),
            reference: None,
            checksum: None,
            skill_set_id: request
                .skill_set_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            skill_paths: text_lines(request.skill_paths.as_deref()),
            required_capabilities: text_lines(request.required_capabilities.as_deref()),
            optional_capabilities: text_lines(request.optional_capabilities.as_deref()),
        },
    )
    .map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn delete_skill_package_command(
    request: DeleteSkillPackageRequest,
) -> Result<SkillDeleteResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let result: DeleteSkillPackageResult = delete_skill_package(
        &root,
        DeleteSkillPackageInput {
            package_id: request.package_id.trim(),
            remove_installed_body: request.remove_installed_body.unwrap_or(true),
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(SkillDeleteResponse {
        state: desktop_state(root)?,
        package_id: result.package_id,
        removed_skill_sets: result.removed_skill_sets,
        detached_agents: result.detached_agents,
        installed_body_removed: result.installed_body_removed,
        warnings: result.warnings,
    })
}

#[tauri::command]
fn save_mcp_connection(request: SaveMcpConnectionRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    let id = request.id.trim();
    let display_name = request.display_name.trim();
    let command = request.command.trim();
    if id.is_empty() {
        return Err("MCP接続IDを入力してください。".to_string());
    }
    if display_name.is_empty() {
        return Err("MCP接続名を入力してください。".to_string());
    }
    if command.is_empty() {
        return Err("MCPサーバーのコマンドを入力してください。".to_string());
    }
    let tool_kind = AgentToolKind::parse(&request.tool_kind).map_err(|error| error.to_string())?;
    let exists = list_mcp_connections(&root)
        .map_err(|error| error.to_string())?
        .iter()
        .any(|connection| connection.id == id);
    if exists {
        update_mcp_connection(
            &root,
            id,
            UpdateMcpConnectionInput {
                display_name: Some(display_name),
                tool_kind: Some(tool_kind),
                command: Some(command),
                args: Some(text_lines(request.args.as_deref())),
                env: Some(env_lines(request.env.as_deref())?),
                test_args: Some(text_lines(request.test_args.as_deref())),
            },
        )
        .map_err(|error| error.to_string())?;
    } else {
        add_mcp_connection(
            &root,
            AddMcpConnectionInput {
                id,
                display_name,
                tool_kind,
                command,
                args: text_lines(request.args.as_deref()),
                env: env_lines(request.env.as_deref())?,
                test_args: text_lines(request.test_args.as_deref()),
            },
        )
        .map_err(|error| error.to_string())?;
    }
    desktop_state(root)
}

#[tauri::command]
fn test_mcp_connection_command(request: McpConnectionRequest) -> Result<McpTestResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let result: McpConnectionTestResult =
        test_mcp_connection(&root, request.id.trim()).map_err(|error| error.to_string())?;
    Ok(McpTestResponse {
        state: desktop_state(root)?,
        success: result.success,
        detail: result.detail,
    })
}

#[tauri::command]
fn delete_mcp_connection_command(request: McpConnectionRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    delete_mcp_connection(&root, request.id.trim()).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn create_work(request: CreateWorkRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    let description = request.description.trim();
    if description.is_empty() {
        return Err("依頼内容を入力してください。".to_string());
    }
    let result = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: derive_title(description),
            description: description.to_string(),
            work_folder: request
                .project
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty() && *value != "nagare")
                .map(ToOwned::to_owned),
            domain_id: request
                .domain_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            artifact_type_id: request
                .artifact_type_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned),
            workflow_mode: request
                .workflow_mode
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(WorkflowMode::parse)
                .transpose()
                .map_err(|error| error.to_string())?,
            approval_policy: request
                .approval_policy
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ApprovalPolicy::parse)
                .transpose()
                .map_err(|error| error.to_string())?,
            ..CreateWorkItemInput::default()
        },
    )
    .map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), result.item.id)
}

#[tauri::command]
fn get_work_detail(root: Option<String>, id: String) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(root)?;
    let snapshot = get_work_item_snapshot(&root, &id).map_err(|error| error.to_string())?;
    Ok(work_detail_view(&root, snapshot))
}

#[tauri::command]
fn delete_work(request: WorkActionRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root)?;
    delete_work_item(&root, &request.id).map_err(|error| error.to_string())?;
    desktop_state(root)
}

#[tauri::command]
fn advance_work(request: WorkActionRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root.clone())?;
    let id = request.id.clone();
    advance_work_for_request(&root, &request)?;
    get_work_detail(Some(root_to_string(&root)), id)
}

#[tauri::command]
fn start_work_background(request: WorkActionRequest) -> Result<DesktopState, String> {
    let root = resolve_desktop_root(request.root.clone())?;
    let thread_root = root.clone();
    std::thread::spawn(move || {
        if let Err(error) = advance_work_for_request(&thread_root, &request) {
            eprintln!("background work advance failed: {error}");
        }
    });
    desktop_state(root)
}

fn advance_work_for_request(root: &Path, request: &WorkActionRequest) -> Result<(), String> {
    let result = advance_work_item_until_blocked(
        root,
        &request.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                path: None,
                prompt: request
                    .prompt
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                dev_command: request
                    .dev_command
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                dispatch_dev_command: request
                    .dispatch_dev_command
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                review_dev_command: request
                    .review_dev_command
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                synthesis_dev_command: request
                    .synthesis_dev_command
                    .as_deref()
                    .filter(|value| !value.trim().is_empty()),
                use_supervisor: false,
                supervisor_dev_command: None,
                auto_recover: request.auto_recover.unwrap_or(false),
                workflow_mode: None,
            },
            max_steps: request.max_steps.unwrap_or(8),
        },
    )
    .map_err(advance_error)?;
    if result.steps.is_empty() {
        return Err(result.stopped_reason);
    }
    Ok(())
}

#[tauri::command]
fn approve_work(request: HumanDecisionRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    approve_work_item(
        &root,
        &request.id,
        request.rationale.as_deref().unwrap_or("desktop approval"),
    )
    .map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn reject_work(request: HumanDecisionRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    let rationale = request
        .rationale
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "差し戻しコメントを入力してください。".to_string())?;
    reject_work_item(&root, &request.id, rationale).map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn answer_work(request: AnswerRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    answer_work_item(
        &root,
        &request.id,
        nagare_core::AnswerWorkItemInput {
            question: request.question.as_deref(),
            answer: request.answer.trim(),
        },
    )
    .map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn create_work_recovery(request: RecoveryRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    let _: CreateRecoveryPlanResult =
        create_recovery_plan(&root, &request.id).map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn accept_work_recovery(request: RecoveryRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    let _: AcceptRecoveryPlanResult =
        nagare_core::accept_recovery_plan(&root, &request.id, request.recovery_plan_id.as_deref())
            .map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn apply_work_recovery(request: RecoveryRequest) -> Result<WorkDetailView, String> {
    let root = resolve_desktop_root(request.root)?;
    apply_recovery_plan(
        &root,
        &request.id,
        ApplyRecoveryPlanInput {
            recovery_plan_id: request.recovery_plan_id.as_deref(),
            prompt: request
                .prompt
                .as_deref()
                .filter(|value| !value.trim().is_empty()),
            dev_command: request
                .dev_command
                .as_deref()
                .filter(|value| !value.trim().is_empty()),
        },
    )
    .map_err(|error| error.to_string())?;
    get_work_detail(Some(root_to_string(&root)), request.id)
}

#[tauri::command]
fn choose_project_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string());
    Ok(folder)
}

#[tauri::command]
fn choose_agent_avatar_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("画像", &["png", "jpg", "jpeg", "svg"])
        .blocking_pick_file()
        .map(|path| path.to_string());
    Ok(file)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            app_info,
            app_state,
            choose_project_folder,
            choose_agent_avatar_file,
            initialize_project,
            initialize_project_with_runtime,
            refresh_runtime_status,
            save_runtime_model_defaults,
            save_project_settings,
            dismiss_improvement,
            delete_project,
            save_domain,
            delete_domain_command,
            save_artifact_type,
            delete_artifact_type_command,
            save_agent,
            delete_agent_command,
            generate_agent_prompt_draft,
            generate_rubric_draft,
            add_skill,
            delete_skill_package_command,
            save_mcp_connection,
            test_mcp_connection_command,
            delete_mcp_connection_command,
            create_work,
            get_work_detail,
            delete_work,
            read_artifact_content,
            advance_work,
            start_work_background,
            approve_work,
            reject_work,
            answer_work,
            create_work_recovery,
            accept_work_recovery,
            apply_work_recovery
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Nagare desktop");
}

fn app_info_value() -> AppInfo {
    AppInfo {
        name: "Nagare",
        version: env!("CARGO_PKG_VERSION"),
        ui_source: "apps/nagare-desktop/src",
    }
}

fn desktop_state(root: PathBuf) -> Result<DesktopState, String> {
    let initialized = is_project_initialized(&root);
    let (
        work_items,
        agents,
        domains,
        artifact_types,
        skill_sets,
        skill_packages,
        mcp_connections,
        project,
        insights,
    ) = if initialized {
        let agent_profiles = sorted(
            list_agent_profiles(&root).map_err(|error| error.to_string())?,
            |agent| agent.id.clone(),
        );
        let domains = sorted(
            list_domains(&root).map_err(|error| error.to_string())?,
            |domain| domain.id.clone(),
        );
        let artifact_types = sorted(
            list_artifact_types(&root).map_err(|error| error.to_string())?,
            |artifact_type| artifact_type.id.clone(),
        );
        let skill_sets = sorted(
            list_skill_set_catalog(&root).map_err(|error| error.to_string())?,
            |skill_set| skill_set.id.clone(),
        );
        let skill_packages = sorted(
            list_skill_packages(&root).map_err(|error| error.to_string())?,
            |package| package.id.clone(),
        );
        let mcp_connections = sorted(
            list_mcp_connections(&root).map_err(|error| error.to_string())?,
            |connection| connection.id.clone(),
        );
        let mut snapshots = Vec::new();
        let work_items = list_work_items(&root)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|item| {
                let snapshot = get_work_item_snapshot(&root, &item.id).ok();
                let trace_records = snapshot
                    .as_ref()
                    .map(|snapshot| list_work_trace(&root, &snapshot.item.id).unwrap_or_default())
                    .unwrap_or_default();
                if let Some(snapshot) = snapshot.as_ref() {
                    snapshots.push(snapshot.clone());
                }
                work_list_item_with_trace(&item, snapshot.as_ref(), &trace_records)
            })
            .collect::<Vec<_>>();
        let insights = insights_view(&root, &snapshots, &agent_profiles);
        let project = project_view(
            &root,
            &work_items,
            &agent_profiles,
            &domains,
            &artifact_types,
        )?;
        (
            work_items,
            agent_profiles.iter().map(agent_view).collect(),
            domains
                .iter()
                .map(|domain| domain_view(domain, &artifact_types))
                .collect(),
            artifact_types.iter().map(artifact_type_view).collect(),
            skill_sets.iter().map(skill_set_view).collect(),
            skill_packages.iter().map(skill_package_view).collect(),
            mcp_connections.iter().map(mcp_connection_view).collect(),
            Some(project),
            insights,
        )
    } else {
        (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
            empty_insights_view(),
        )
    };
    let runtimes = runtime_views(&agents);
    Ok(DesktopState {
        app: app_info_value(),
        root: root_to_string(&root),
        initialized,
        project,
        work_items,
        agents,
        domains,
        artifact_types,
        skill_sets,
        skill_packages,
        mcp_connections,
        mcp_capabilities: mcp_capability_views(),
        runtimes,
        insights,
    })
}

fn empty_desktop_state() -> DesktopState {
    DesktopState {
        app: app_info_value(),
        root: String::new(),
        initialized: false,
        project: None,
        work_items: Vec::new(),
        agents: Vec::new(),
        domains: Vec::new(),
        artifact_types: Vec::new(),
        skill_sets: Vec::new(),
        skill_packages: Vec::new(),
        mcp_connections: Vec::new(),
        mcp_capabilities: mcp_capability_views(),
        runtimes: runtime_views(&[]),
        insights: empty_insights_view(),
    }
}

fn is_missing_path_error(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("os error 3")
        || error.contains("path not found")
        || error.contains("cannot find path")
        || error.contains("指定されたパスが見つかりません")
}

fn sorted<T, F, K>(mut values: Vec<T>, key: F) -> Vec<T>
where
    F: FnMut(&T) -> K,
    K: Ord,
{
    values.sort_by_key(key);
    values
}

fn project_view(
    root: &Path,
    work_items: &[WorkListItem],
    agents: &[AgentProfile],
    domains: &[Domain],
    artifact_types: &[ArtifactType],
) -> Result<ProjectView, String> {
    let workflow = get_workflow_settings(root.to_path_buf()).map_err(|error| error.to_string())?;
    let settings =
        get_nagare_agent_settings(root.to_path_buf()).map_err(|error| error.to_string())?;
    let metadata = get_project_metadata(root.to_path_buf()).map_err(|error| error.to_string())?;
    let root_name = project_name_from_root(root);
    let metadata_name = metadata.name.trim();
    let project_name = if !metadata_name.is_empty()
        && !(metadata_name == "nagare-local" && root_name != "nagare-local")
    {
        metadata_name.to_string()
    } else {
        root_name
    };
    let project_icon = if metadata.icon.trim().is_empty() {
        default_project_icon().to_string()
    } else {
        metadata.icon.trim().to_string()
    };
    let organizer_agent_id = settings.organizer_agent.clone().unwrap_or_default();
    let organizer_label = settings
        .organizer_agent
        .as_deref()
        .and_then(|id| agent_name(agents, id))
        .unwrap_or_else(|| "標準（内蔵オーガナイザー）".to_string());
    Ok(ProjectView {
        name: project_name,
        icon: project_icon,
        root: root_to_string(root),
        default_domain_id: metadata.default_domain_id,
        default_artifact_type_id: metadata.default_artifact_type_id,
        workflow_mode: workflow.default_progress_mode.to_string(),
        approval_policy: workflow.approval_policy.to_string(),
        organizer_agent_id,
        organizer_label,
        work_agent: agent_name(agents, &settings.work_agent).unwrap_or(settings.work_agent),
        review_agent: agent_name(agents, &settings.review_agent).unwrap_or(settings.review_agent),
        agent_count: agents.len(),
        domain_count: domains.len(),
        artifact_type_count: artifact_types.len(),
        work_count: work_items.len(),
        status_counts: status_counts(work_items),
    })
}

fn agent_name(agents: &[AgentProfile], id: &str) -> Option<String> {
    agents
        .iter()
        .find(|agent| agent.id == id)
        .map(|agent| agent.display_name.clone())
}

fn status_counts(work_items: &[WorkListItem]) -> Vec<StatusCountView> {
    [
        ("要対応・質問", "question"),
        ("要対応・確認", "review"),
        ("要対応・回復", "recover"),
        ("処理中", "running"),
        ("完了", "done"),
    ]
    .into_iter()
    .map(|(label, kind)| StatusCountView {
        label: label.to_string(),
        kind: kind.to_string(),
        count: work_items
            .iter()
            .filter(|item| item.status_kind == kind)
            .count(),
    })
    .collect()
}

#[derive(Default)]
struct AgentInsightAccum {
    agent_id: String,
    agent_name: String,
    role: String,
    score_sum: u32,
    review_count: usize,
    issue_counts: BTreeMap<String, usize>,
}

struct IssueAccum {
    agent_name: String,
    item: String,
    score_sum: u32,
    occurrences: usize,
    concern_count: usize,
}

fn empty_insights_view() -> InsightsView {
    InsightsView {
        review_count: 0,
        average_score_label: "-".to_string(),
        concern_count: 0,
        proposal_count: 0,
        agent_scores: Vec::new(),
        issue_matrix: Vec::new(),
        proposals: Vec::new(),
        applied_improvements: Vec::new(),
        recent_reviews: Vec::new(),
    }
}

fn insights_view(
    root: &Path,
    snapshots: &[WorkItemSnapshot],
    agents: &[AgentProfile],
) -> InsightsView {
    let mut agent_accums: BTreeMap<String, AgentInsightAccum> = BTreeMap::new();
    let mut issue_accums: BTreeMap<(String, String), IssueAccum> = BTreeMap::new();
    let mut recent_reviews = Vec::new();
    let mut total_score = 0u32;
    let mut review_count = 0usize;
    let mut concern_count = 0usize;

    for snapshot in snapshots {
        let Some(review) = snapshot.review_results.iter().rev().next() else {
            continue;
        };
        let trace_records = list_work_trace(root, &snapshot.item.id).unwrap_or_default();
        let agent_id = reviewed_agent_id(snapshot, &trace_records)
            .unwrap_or_else(|| review.agent_profile_id.clone());
        let (agent_name, role) = agent_label(agents, &agent_id);
        let score = review_score_percent(review, &trace_records);
        let concerns = review_concerns(review, &trace_records);
        let review_items = insight_review_items(review, &trace_records, score);

        review_count += 1;
        total_score += u32::from(score);
        concern_count += concerns.len();

        let agent = agent_accums
            .entry(agent_id.clone())
            .or_insert_with(|| AgentInsightAccum {
                agent_id: agent_id.clone(),
                agent_name: agent_name.clone(),
                role: role.clone(),
                score_sum: 0,
                review_count: 0,
                issue_counts: BTreeMap::new(),
            });
        agent.score_sum += u32::from(score);
        agent.review_count += 1;

        for (item, item_score, is_concern) in review_items {
            if is_concern {
                *agent.issue_counts.entry(item.clone()).or_insert(0) += 1;
            }
            let issue = issue_accums
                .entry((agent_id.clone(), item.clone()))
                .or_insert_with(|| IssueAccum {
                    agent_name: agent_name.clone(),
                    item: item.clone(),
                    score_sum: 0,
                    occurrences: 0,
                    concern_count: 0,
                });
            issue.score_sum += u32::from(item_score);
            issue.occurrences += 1;
            if is_concern {
                issue.concern_count += 1;
            }
        }

        recent_reviews.push(InsightReviewView {
            work_id: snapshot.item.id.clone(),
            title: snapshot.item.title.clone(),
            agent_name,
            verdict: review.verdict.to_string(),
            score_label: format!("{score} / 100"),
            concerns,
        });
    }

    recent_reviews.sort_by(|a, b| b.work_id.cmp(&a.work_id));
    recent_reviews.truncate(5);

    let mut agent_scores = agent_accums
        .into_values()
        .map(|agent| {
            let average = average_u8(agent.score_sum, agent.review_count);
            let top_issue = agent
                .issue_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(item, count)| format!("{item} ({count}件)"))
                .unwrap_or_else(|| "目立つ失点なし".to_string());
            AgentInsightView {
                agent_id: agent.agent_id,
                agent_name: agent.agent_name,
                role: agent.role,
                review_count: agent.review_count,
                average_score: average,
                average_score_label: format!("{average} / 100"),
                status_label: if average < 75 {
                    "要改善".to_string()
                } else if average >= 90 && agent.review_count >= 3 {
                    "安定".to_string()
                } else {
                    "観察中".to_string()
                },
                top_issue,
            }
        })
        .collect::<Vec<_>>();
    agent_scores.sort_by_key(|agent| agent.average_score);

    let mut issue_matrix = issue_accums
        .into_values()
        .map(|issue| {
            let rate = average_u8(issue.score_sum, issue.occurrences);
            InsightIssueView {
                agent_name: issue.agent_name,
                item: issue.item.clone(),
                rate,
                rate_label: format!("{rate}%"),
                occurrences: issue.occurrences,
                suggestion_kind: suggestion_kind_for_issue(&issue.item, rate).to_string(),
            }
        })
        .collect::<Vec<_>>();
    issue_matrix.sort_by_key(|issue| (issue.rate, std::cmp::Reverse(issue.occurrences)));
    issue_matrix.truncate(8);

    let mut proposals = issue_matrix
        .iter()
        .filter(|issue| issue.rate < 75)
        .take(4)
        .map(|issue| {
            let kind = issue.suggestion_kind.clone();
            let current_text = current_improvement_text(&kind, &issue.agent_name, &issue.item);
            let suggested_text = suggested_improvement_text(&kind, &issue.item);
            ImprovementProposalView {
                id: proposal_id(&issue.agent_name, &issue.item, &kind),
                kind: kind.clone(),
                title: format!("{} の{}改善", issue.agent_name, kind),
                target_label: format!("{} / {}", issue.agent_name, issue.item),
                summary: format!(
                    "「{}」の獲得率が {} です。該当エージェントの指示、知識、または判定基準の見直し候補として扱います。",
                    issue.item, issue.rate_label
                ),
                evidence: format!("レビュー履歴 {}件から算出。75%未満を改善候補として表示しています。", issue.occurrences),
                diff_lines: vec![
                    format!("- {current_text}"),
                    format!("+ {suggested_text}"),
                ],
                current_text,
                suggested_text,
                next_step: next_step_for_improvement_kind(&kind).to_string(),
                action_label: "プレビューで確認".to_string(),
            }
        })
        .collect::<Vec<_>>();

    if review_count >= 10 && concern_count == 0 && average_score(total_score, review_count) >= 90.0
    {
        let current_text = "確認ポリシー: 最後に人が確認する".to_string();
        let suggested_text = "確認ポリシー: レビュー懸念がある時だけ人が確認する".to_string();
        proposals.push(ImprovementProposalView {
            id: "operation-approval-policy".to_string(),
            kind: "運用".to_string(),
            title: "確認ポリシー緩和の提案".to_string(),
            target_label: "プロジェクト設定 / 確認ポリシー".to_string(),
            summary: "直近レビューが安定しているため、「最後に確認する」から「重要時のみ確認」への切り替え候補です。".to_string(),
            evidence: format!("レビュー {}件、平均 {:.0}点、懸念0件。", review_count, average_score(total_score, review_count)),
            diff_lines: vec![
                format!("- {current_text}"),
                format!("+ {suggested_text}"),
            ],
            current_text,
            suggested_text,
            next_step: "プロジェクト設定で確認ポリシーを確認し、人が必要と判断した場合だけ保存します。".to_string(),
            action_label: "設定で確認".to_string(),
        });
    }

    let applied_history = list_improvement_history(root).unwrap_or_default();
    let applied_proposal_ids = applied_history
        .iter()
        .map(|entry| entry.proposal_id.as_str())
        .collect::<BTreeSet<_>>();
    proposals.retain(|proposal| !applied_proposal_ids.contains(proposal.id.as_str()));
    let proposal_count = proposals.len();
    let current_average = average_score(total_score, review_count);
    let applied_improvements = applied_history
        .into_iter()
        .filter(|entry| entry.status != "dismissed")
        .map(|entry| {
            applied_improvement_view(
                entry,
                &issue_matrix,
                review_count,
                concern_count,
                current_average,
            )
        })
        .collect();
    InsightsView {
        review_count,
        average_score_label: if review_count == 0 {
            "-".to_string()
        } else {
            format!("{current_average:.0} / 100")
        },
        concern_count,
        proposal_count,
        agent_scores,
        issue_matrix,
        proposals,
        applied_improvements,
        recent_reviews,
    }
}

fn applied_improvement_view(
    entry: ImprovementHistoryEntry,
    issue_matrix: &[InsightIssueView],
    review_count: usize,
    concern_count: usize,
    average_score: f64,
) -> AppliedImprovementView {
    let effect_label = improvement_effect_label(
        &entry,
        issue_matrix,
        review_count,
        concern_count,
        average_score,
    );
    AppliedImprovementView {
        id: entry.id,
        proposal_id: entry.proposal_id,
        kind: entry.kind.clone(),
        title: if entry.title.is_empty() {
            entry.kind.clone()
        } else {
            entry.title
        },
        target_label: entry.target_label,
        summary: entry.summary,
        applied_at: entry.applied_at,
        effect_label,
    }
}

fn improvement_effect_label(
    entry: &ImprovementHistoryEntry,
    issue_matrix: &[InsightIssueView],
    review_count: usize,
    concern_count: usize,
    average_score: f64,
) -> String {
    if !entry.effect_label.trim().is_empty() && entry.effect_label != "効果測定中" {
        return entry.effect_label.clone();
    }
    if entry.kind.contains("運用") || entry.target_label.contains("確認ポリシー") {
        return if review_count == 0 {
            "効果測定中".to_string()
        } else {
            format!("測定中: 平均 {average_score:.0}点 / 懸念{concern_count}件")
        };
    }
    let target_parts = entry
        .target_label
        .split('/')
        .map(str::trim)
        .collect::<Vec<_>>();
    let target_agent = target_parts.first().copied().unwrap_or_default();
    let target_item = target_parts.get(1).copied().unwrap_or_default();
    if let Some(issue) = issue_matrix.iter().find(|issue| {
        !target_agent.is_empty()
            && !target_item.is_empty()
            && issue.agent_name == target_agent
            && issue.item == target_item
    }) {
        return format!(
            "測定中: {} {}（{}件）",
            issue.item, issue.rate_label, issue.occurrences
        );
    }
    if review_count > 0 {
        "効果: 最近の同種懸念なし".to_string()
    } else {
        "効果測定中".to_string()
    }
}

fn reviewed_agent_id(snapshot: &WorkItemSnapshot, trace_records: &[TraceRecord]) -> Option<String> {
    trace_records
        .iter()
        .rev()
        .find(|record| record.record == "worker_output")
        .and_then(|record| {
            value_path_str(&record.payload, &["agent", "id"])
                .or_else(|| value_path_str(&record.payload, &["agent", "name"]))
        })
        .or_else(|| {
            snapshot
                .agent_outputs
                .iter()
                .rev()
                .find(|output| output.purpose == AgentRunPurpose::Work)
                .map(|output| output.agent_profile_id.clone())
        })
        .or_else(|| {
            snapshot
                .dispatch_plans
                .iter()
                .rev()
                .find(|plan| !plan.target_agent_profile_id.trim().is_empty())
                .map(|plan| plan.target_agent_profile_id.clone())
        })
}

fn agent_label(agents: &[AgentProfile], agent_id: &str) -> (String, String) {
    agents
        .iter()
        .find(|agent| agent.id == agent_id)
        .map(|agent| (agent.display_name.clone(), agent.role.clone()))
        .unwrap_or_else(|| (agent_id.to_string(), "worker".to_string()))
}

fn review_score_percent(review: &ReviewResult, trace_records: &[TraceRecord]) -> u8 {
    if let Some((total, max)) = latest_trace_score(trace_records) {
        return percent_u8(total, max);
    }
    if !review.criteria_results.is_empty() {
        let passed = review
            .criteria_results
            .iter()
            .filter(|result| result.status.to_string() == "passed")
            .count() as u64;
        return percent_u8(passed, review.criteria_results.len() as u64);
    }
    match review.verdict {
        ReviewVerdict::Pass => 100,
        ReviewVerdict::RequestChanges => 60,
        ReviewVerdict::Blocked => 40,
        ReviewVerdict::Unknown => 0,
    }
}

fn latest_trace_score(trace_records: &[TraceRecord]) -> Option<(u64, u64)> {
    trace_records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .and_then(|record| {
            let total = value_u64(&record.payload, "total_score")?;
            let max = value_u64(&record.payload, "max_score")?;
            (max > 0).then_some((total, max))
        })
}

fn review_concerns(review: &ReviewResult, trace_records: &[TraceRecord]) -> Vec<String> {
    let mut concerns = Vec::new();
    concerns.extend(meaningful_items(&review.findings));
    concerns.extend(meaningful_items(&review.requested_changes));
    concerns.extend(meaningful_items(&review.questions));
    for result in &review.criteria_results {
        if result.status.to_string() != "passed" {
            let note = if meaningful_text(&result.note) {
                result.note.clone()
            } else {
                result.criterion.clone()
            };
            concerns.push(note);
        }
    }
    for item in latest_trace_review_items(trace_records).unwrap_or_default() {
        if !verdict_is_pass(&item.verdict) {
            concerns.push(if meaningful_text(&item.concern_note) {
                format!("{}: {}", item.item, item.concern_note)
            } else {
                item.item
            });
        }
    }
    concerns.sort();
    concerns.dedup();
    concerns
}

fn insight_review_items(
    review: &ReviewResult,
    trace_records: &[TraceRecord],
    fallback_score: u8,
) -> Vec<(String, u8, bool)> {
    let trace_items = latest_trace_review_items(trace_records).unwrap_or_default();
    if !trace_items.is_empty() {
        return trace_items
            .into_iter()
            .map(|item| {
                let score = parse_score_label(&item.score_label).unwrap_or_else(|| {
                    if verdict_is_pass(&item.verdict) {
                        100
                    } else {
                        50
                    }
                });
                let is_concern = !verdict_is_pass(&item.verdict);
                (item.item, score, is_concern)
            })
            .collect();
    }
    if !review.criteria_results.is_empty() {
        return review
            .criteria_results
            .iter()
            .map(|result| {
                let status = result.status.to_string();
                let score = match status.as_str() {
                    "passed" => 100,
                    "unknown" => 50,
                    _ => 0,
                };
                (result.criterion.clone(), score, status != "passed")
            })
            .collect();
    }
    vec![(
        "総合判定".to_string(),
        fallback_score,
        review.verdict != ReviewVerdict::Pass,
    )]
}

fn meaningful_items(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter(|value| meaningful_text(value))
        .cloned()
        .collect()
}

fn meaningful_text(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    !matches!(
        normalized.as_str(),
        "" | "-" | "none" | "no" | "n/a" | "なし" | "無し" | "ありません" | "問題なし"
    )
}

fn verdict_is_pass(verdict: &str) -> bool {
    matches!(
        verdict
            .trim()
            .to_ascii_lowercase()
            .replace('-', "_")
            .as_str(),
        "pass" | "passed" | "ok" | "approved"
    )
}

fn parse_score_label(label: &str) -> Option<u8> {
    let (left, right) = label.split_once('/')?;
    let total = left.trim().parse::<u64>().ok()?;
    let max = right.trim().parse::<u64>().ok()?;
    Some(percent_u8(total, max))
}

fn percent_u8(total: u64, max: u64) -> u8 {
    if max == 0 {
        return 0;
    }
    ((total.saturating_mul(100) + (max / 2)) / max).min(100) as u8
}

fn average_score(total: u32, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        f64::from(total) / count as f64
    }
}

fn average_u8(total: u32, count: usize) -> u8 {
    average_score(total, count).round().clamp(0.0, 100.0) as u8
}

fn suggestion_kind_for_issue(item: &str, rate: u8) -> &'static str {
    let normalized = item.to_ascii_lowercase();
    if normalized.contains("用語") || normalized.contains("知識") || normalized.contains("term")
    {
        "知識"
    } else if normalized.contains("基準")
        || normalized.contains("rubric")
        || normalized.contains("判定")
    {
        "ルーブリック"
    } else if rate < 75 {
        "プロンプト"
    } else {
        "観察"
    }
}

fn proposal_id(agent_name: &str, item: &str, kind: &str) -> String {
    let raw = format!("{kind}-{agent_name}-{item}");
    let mut id = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch.is_whitespace() || matches!(ch, '/' | '_' | '-') {
                '-'
            } else {
                ch
            }
        })
        .collect::<String>();
    while id.contains("--") {
        id = id.replace("--", "-");
    }
    id.trim_matches('-').to_string()
}

fn current_improvement_text(kind: &str, agent_name: &str, item: &str) -> String {
    match kind {
        "ルーブリック" => format!(
            "成果物種別のルーブリックでは「{item}」の判定観点が弱く、レビューごとの判断にぶれが残っています。"
        ),
        "知識" => {
            format!("ドメイン知識には「{item}」の前提・用語・参照先が十分に集約されていません。")
        }
        _ => format!(
            "{agent_name} のプロンプトには、出力前に「{item}」を自己確認する明示的な手順がありません。"
        ),
    }
}

fn suggested_improvement_text(kind: &str, item: &str) -> String {
    match kind {
        "ルーブリック" => format!(
            "ルーブリックへ「{item}」の配点、合格条件、差し戻し条件を追加し、レビュー時に根拠を残すようにします。"
        ),
        "知識" => format!(
            "ドメイン知識へ「{item}」に関する定義、使う場面、参照例を追加し、担当エージェントが実行時に参照できるようにします。"
        ),
        _ => format!(
            "エージェントのプロンプトへ「最終出力前に {item} を確認し、不足があれば完了扱いにしない」というセルフチェックを追加します。"
        ),
    }
}

fn next_step_for_improvement_kind(kind: &str) -> &'static str {
    match kind {
        "ルーブリック" | "知識" => {
            "ナレッジ設定を開き、人が差分を確認してから保存します。自動適用はしません。"
        }
        "運用" => {
            "プロジェクト設定を開き、人が確認ポリシーを確認してから保存します。自動適用はしません。"
        }
        _ => "エージェント設定を開き、人がプロンプトを確認してから保存します。自動適用はしません。",
    }
}

fn agent_view(profile: &AgentProfile) -> AgentView {
    let capability = nagare_core::runtime_mcp_capability(profile.tool_kind);
    AgentView {
        id: profile.id.clone(),
        name: profile.display_name.clone(),
        avatar: profile.avatar.clone(),
        role: profile.role.clone(),
        description: profile.description.clone(),
        runtime: profile.runtime.clone(),
        adapter: profile.adapter.clone(),
        tool_kind: profile.tool_kind.to_string(),
        model: profile
            .model
            .model_ref()
            .unwrap_or_else(|| "実行環境既定".to_string()),
        model_provider: profile.model.provider.clone(),
        model_base_url: profile.model.base_url.clone(),
        prompt: profile.prompt.instructions.clone(),
        specialties: profile.specialties.clone(),
        domain_ids: profile.domain_ids.clone(),
        artifact_type_ids: profile.artifact_type_ids.clone(),
        skill_set_ids: profile.skill_set_ids.clone(),
        mcp_connection_ids: profile.mcp_connection_ids.clone(),
        source: profile.source.to_string(),
        builtin: profile.source == AgentProfileSource::ProjectConfig,
        mcp_assignable: capability.agent_assignable,
        mcp_note: capability.note.to_string(),
    }
}

fn domain_view(domain: &Domain, artifact_types: &[ArtifactType]) -> DomainView {
    DomainView {
        id: domain.id.clone(),
        name: domain.display_name.clone(),
        description: domain.description.clone(),
        shared_knowledge: domain.shared_knowledge.clone(),
        common_rubric: domain.common_rubric.clone(),
        dispatch_hints: domain.dispatch_hints.clone(),
        shared_knowledge_count: domain.shared_knowledge.len(),
        common_rubric_count: domain.common_rubric.len(),
        artifact_type_count: artifact_types
            .iter()
            .filter(|artifact_type| artifact_type.domain_id.as_deref() == Some(domain.id.as_str()))
            .count(),
    }
}

fn artifact_type_view(artifact_type: &ArtifactType) -> ArtifactTypeView {
    let rubric_summary = rubric_markdown_summary(&artifact_type.rubric);
    ArtifactTypeView {
        id: artifact_type.id.clone(),
        domain_id: artifact_type
            .domain_id
            .clone()
            .unwrap_or_else(|| "general".to_string()),
        name: artifact_type.display_name.clone(),
        description: artifact_type.description.clone(),
        knowledge: artifact_type.artifact_types.clone(),
        rubric: artifact_type.rubric.clone(),
        dispatch_hints: artifact_type.dispatch_hints.clone(),
        knowledge_count: artifact_type.artifact_types.len(),
        rubric_count: rubric_summary
            .as_ref()
            .map(|summary| summary.item_count)
            .unwrap_or(artifact_type.rubric.len()),
        rubric_score_total: rubric_summary
            .as_ref()
            .map(|summary| summary.total_score)
            .unwrap_or(0),
        rubric_version: artifact_type.rubric_version.max(1),
    }
}

fn skill_set_view(skill_set: &SkillSetCatalogEntry) -> SkillSetView {
    SkillSetView {
        id: skill_set.id.clone(),
        paths: skill_set.paths.clone(),
        required_capabilities: skill_set.required_capabilities.clone(),
        optional_capabilities: skill_set.optional_capabilities.clone(),
    }
}

fn skill_package_view(package: &SkillPackageCatalogEntry) -> SkillPackageView {
    SkillPackageView {
        id: package.id.clone(),
        source_kind: package.source_kind.clone(),
        source: package.source.clone(),
        install_scope: package.install_scope.clone(),
        installed_targets: package.installed_targets.clone(),
        provided_skill_sets: package.provided_skill_sets.clone(),
    }
}

fn mcp_connection_view(connection: &McpConnectionCatalogEntry) -> McpConnectionView {
    McpConnectionView {
        id: connection.id.clone(),
        name: connection.display_name.clone(),
        tool_kind: connection.tool_kind.to_string(),
        runtime_label: connection.runtime_label.clone(),
        scope: connection.scope.to_string(),
        agent_assignable: connection.agent_assignable,
        command: connection.command.clone(),
        args: connection.args.clone(),
        env: connection
            .env
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect(),
        env_count: connection.env.len(),
        test_args: connection.test_args.clone(),
        test_status: if connection.last_test_status.is_empty() {
            "untested".to_string()
        } else {
            connection.last_test_status.clone()
        },
        test_detail: connection.last_test_detail.clone(),
        tested_at: connection.last_tested_at.clone(),
    }
}

fn mcp_capability_views() -> Vec<McpCapabilityView> {
    RUNTIME_MCP_CAPABILITIES
        .iter()
        .map(|capability| McpCapabilityView {
            tool_kind: capability.tool_kind.to_string(),
            runtime_label: capability.runtime_label.to_string(),
            scope: capability.scope.to_string(),
            agent_assignable: capability.agent_assignable,
            note: capability.note.to_string(),
        })
        .collect()
}

fn work_detail_view(root: &Path, snapshot: WorkItemSnapshot) -> WorkDetailView {
    let trace_records = list_work_trace(root, &snapshot.item.id).unwrap_or_default();
    let item = work_list_item_with_trace(&snapshot.item, Some(&snapshot), &trace_records);
    let answer = latest_answer(&snapshot).unwrap_or_else(|| item.result_summary.clone());
    let review = snapshot
        .review_results
        .iter()
        .rev()
        .next()
        .map(|review| review_view(review, &trace_records));
    let artifacts = snapshot
        .artifacts
        .iter()
        .map(|artifact| ArtifactView {
            title: artifact.title.clone(),
            uri: artifact.uri.clone(),
        })
        .collect();
    let trace_steps = trace_step_views(&trace_records);
    let steps = if trace_steps.is_empty() {
        history_step_views(&snapshot)
    } else {
        trace_steps
    };
    let request = if snapshot.item.description.trim().is_empty() {
        snapshot.item.title.clone()
    } else {
        snapshot.item.description.clone()
    };
    WorkDetailView {
        root: root_to_string(root),
        item,
        domain_id: snapshot.item.domain_id.clone().unwrap_or_default(),
        artifact_type_id: snapshot.item.artifact_type_id.clone().unwrap_or_default(),
        next_action_kind: snapshot.completion.next_action.clone(),
        approval_ready: snapshot.approval_gate.ready,
        question: latest_question(&snapshot),
        question_source: latest_question_source(&snapshot),
        recovery: latest_recovery(&snapshot).map(recovery_view),
        request,
        answer,
        artifacts,
        review,
        steps,
    }
}

fn latest_question(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| output.questions.first().cloned())
}

fn latest_question_source(snapshot: &WorkItemSnapshot) -> String {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| !output.questions.is_empty())
        .map(|output| {
            let step = question_purpose_label(output.purpose);
            if output.agent_profile_id.trim().is_empty() {
                step.to_string()
            } else {
                format!("{step} / {}", output.agent_profile_id)
            }
        })
        .unwrap_or_default()
}

fn question_purpose_label(purpose: AgentRunPurpose) -> &'static str {
    match purpose {
        AgentRunPurpose::DispatchPreview => "整理工程",
        AgentRunPurpose::Work => "作業工程",
        AgentRunPurpose::Review => "レビュー工程",
        AgentRunPurpose::Synthesis => "統合工程",
        AgentRunPurpose::WorkflowSupervision => "進行管理",
    }
}

fn latest_recovery(snapshot: &WorkItemSnapshot) -> Option<&RecoveryPlan> {
    snapshot.recovery_plans.iter().rev().find(|plan| {
        matches!(
            plan.status,
            RecoveryPlanStatus::Draft | RecoveryPlanStatus::Accepted
        )
    })
}

fn recovery_view(plan: &RecoveryPlan) -> RecoveryView {
    RecoveryView {
        id: plan.id.clone(),
        status: plan.status.to_string(),
        action: plan.action.to_string(),
        failure_class: plan.failure_class.clone(),
        reason: plan.reason.clone(),
        summary: plan.summary.clone(),
        impact: recovery_impact(plan),
        handoff_completed: recovery_completed(plan),
        handoff_pending: recovery_pending(plan),
        target_agent: plan.target_agent_profile_id.clone().unwrap_or_default(),
        command_hint: plan.command_hint.clone().unwrap_or_default(),
        warnings: plan.warnings.clone(),
        prompt_hint: plan.prompt_hint.clone(),
    }
}

fn recovery_impact(plan: &RecoveryPlan) -> String {
    match plan.action {
        nagare_core::RecoveryAction::RerunSameAgent
        | nagare_core::RecoveryAction::RerunWithContractReminder => {
            "完了済みの記録は残し、対象エージェントだけを続きから再実行します。".to_string()
        }
        nagare_core::RecoveryAction::Handoff => {
            "完了済みの記録は残し、別のエージェントへ引き継ぎます。".to_string()
        }
        nagare_core::RecoveryAction::AskHuman => {
            "ユーザーの判断が必要です。回答後、このワークだけを再開します。".to_string()
        }
        nagare_core::RecoveryAction::RequestChanges => {
            "レビュー懸念を次の作業指示として渡し、修正作業へ戻します。".to_string()
        }
        nagare_core::RecoveryAction::Redispatch => {
            "整理工程に戻り、担当や進め方を選び直します。".to_string()
        }
    }
}

fn recovery_completed(plan: &RecoveryPlan) -> Vec<String> {
    let mut completed = Vec::new();
    if meaningful_text(&plan.summary) {
        completed.push(plan.summary.clone());
    }
    if let Some(source) = &plan.source_event_id {
        if meaningful_text(source) {
            completed.push(format!("直前の記録: {source}"));
        }
    }
    if completed.is_empty() {
        completed.push("これまでの実行記録と成果物候補は保持されています。".to_string());
    }
    completed
}

fn recovery_pending(plan: &RecoveryPlan) -> Vec<String> {
    let mut pending = Vec::new();
    if meaningful_text(&plan.reason) {
        pending.push(plan.reason.clone());
    }
    if let Some(command) = &plan.command_hint {
        if meaningful_text(command) {
            pending.push(format!("次の実行: {command}"));
        }
    }
    pending.extend(
        plan.warnings
            .iter()
            .filter(|warning| meaningful_text(warning))
            .cloned(),
    );
    if pending.is_empty() {
        pending.push("回復案を選ぶと、次の工程へ引き継ぎます。".to_string());
    }
    pending
}

fn history_step_views(snapshot: &WorkItemSnapshot) -> Vec<StepView> {
    snapshot
        .history_steps
        .iter()
        .map(|step| StepView {
            kind: step.kind.clone(),
            title: step.title.clone(),
            state: step.state.clone(),
            actor: step.actor.clone().unwrap_or_else(|| "-".to_string()),
            summary: step.summary.clone(),
            rationale: String::new(),
            input: step
                .facts
                .iter()
                .find(|fact| fact.label == "completed")
                .map(|fact| fact.value.clone())
                .unwrap_or_default(),
            output: step
                .facts
                .iter()
                .find(|fact| fact.label == "artifact detail" || fact.label == "evidence detail")
                .map(|fact| fact.value.clone())
                .unwrap_or_default(),
            knowledge_refs: Vec::new(),
            diagnostics: step
                .links
                .iter()
                .find(|link| link.record_type == "execution_record")
                .map(|link| link.record_id.clone())
                .unwrap_or_default(),
            review_items: Vec::new(),
        })
        .collect()
}

fn trace_step_views(records: &[TraceRecord]) -> Vec<StepView> {
    records
        .iter()
        .filter(|record| {
            matches!(
                record.record.as_str(),
                "organizer_decision" | "worker_output" | "reviewer_verdict" | "organizer_summary"
            )
        })
        .map(trace_step_view)
        .collect()
}

fn trace_step_view(record: &TraceRecord) -> StepView {
    let payload = &record.payload;
    let actor = value_path_str(payload, &["agent", "name"])
        .or_else(|| value_path_str(payload, &["agent", "id"]))
        .unwrap_or_else(|| "-".to_string());
    let state = value_str(payload, "status").unwrap_or_else(|| "completed".to_string());
    let kind = value_str(payload, "step_kind").unwrap_or_else(|| record.record.clone());
    let knowledge_refs = payload
        .get("knowledge_refs")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| value_str(item, "id"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let diagnostics = value_path_str(payload, &["diagnostics", "session_ref"]).unwrap_or_default();
    match record.record.as_str() {
        "organizer_decision" => StepView {
            kind,
            title: "整理・担当決定".to_string(),
            state,
            actor,
            summary: value_str(payload, "interpreted_request")
                .unwrap_or_else(|| "担当を決定しました。".to_string()),
            rationale: first_assignment_rationale(payload),
            input: format!(
                "ドメイン: {} / 成果物種別: {}",
                value_str(payload, "domain_id").unwrap_or_else(|| "general".to_string()),
                value_str(payload, "artifact_type_id").unwrap_or_else(|| "general".to_string())
            ),
            output: first_plan_target(payload)
                .map(|target| format!("担当: {target}"))
                .unwrap_or_default(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "worker_output" => StepView {
            kind,
            title: "作業実行".to_string(),
            state,
            actor,
            summary: value_str(payload, "actions_summary")
                .unwrap_or_else(|| "作業を実行しました。".to_string()),
            rationale: "割り当て済みエージェントが依頼を処理しました。".to_string(),
            input: value_path_str(payload, &["inputs", "summary"]).unwrap_or_default(),
            output: worker_output_summary(payload),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "organizer_summary" => StepView {
            kind,
            title: "オーガナイザーまとめ".to_string(),
            state,
            actor,
            summary: value_str(payload, "actions_summary")
                .or_else(|| value_str(payload, "answer"))
                .unwrap_or_else(|| "複数の結果を統合しました。".to_string()),
            rationale: "複数の作業結果とレビュー結果を依頼者向けにまとめました。".to_string(),
            input: value_path_str(payload, &["inputs", "summary"]).unwrap_or_default(),
            output: value_str(payload, "answer")
                .or_else(|| value_str(payload, "actions_summary"))
                .unwrap_or_default(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "reviewer_verdict" => StepView {
            kind,
            title: "レビュー".to_string(),
            state,
            actor,
            summary: value_str(payload, "summary")
                .unwrap_or_else(|| "レビューを実行しました。".to_string()),
            rationale: review_rationale_summary(payload),
            input: array_strings(payload, "target_artifacts").join(", "),
            output: review_output_summary(payload),
            knowledge_refs,
            diagnostics,
            review_items: trace_review_item_views(payload),
        },
        _ => StepView {
            kind,
            title: record.record.clone(),
            state,
            actor,
            summary: String::new(),
            rationale: String::new(),
            input: String::new(),
            output: String::new(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
    }
}

fn work_list_item_with_trace(
    item: &WorkItem,
    snapshot: Option<&WorkItemSnapshot>,
    trace_records: &[TraceRecord],
) -> WorkListItem {
    let (status_label, status_kind) = status_view(item.status, snapshot);
    WorkListItem {
        id: item.id.clone(),
        title: item.title.clone(),
        description: item.description.clone(),
        project: project_label(item.work_folder.as_deref()),
        status_label,
        status_kind,
        next_action: snapshot
            .map(|snapshot| next_action_label(&snapshot.completion.next_action))
            .unwrap_or_else(|| "作成済み".to_string()),
        result_summary: snapshot
            .map(|snapshot| work_list_result_summary(snapshot, trace_records))
            .unwrap_or_else(|| "まだ結果はありません。".to_string()),
        updated_at: item.updated_at.clone(),
        workflow_mode: item.workflow_mode.to_string(),
        approval_policy: item.approval_policy.to_string(),
    }
}

fn work_list_result_summary(snapshot: &WorkItemSnapshot, trace_records: &[TraceRecord]) -> String {
    let answer = latest_answer(snapshot);
    let review = snapshot.review_results.iter().rev().next();
    match (answer, review) {
        (Some(answer), Some(review)) => {
            let score = review_score_label(review, trace_records);
            let review_summary =
                first_text(&review.summary).or_else(|| first_text(&review.findings));
            [
                answer,
                format!("評価 {score}"),
                review_summary.unwrap_or_default(),
            ]
            .into_iter()
            .filter(|part| meaningful_text(part))
            .collect::<Vec<_>>()
            .join(" · ")
        }
        (Some(answer), None) => answer,
        (None, Some(review)) => {
            let score = review_score_label(review, trace_records);
            let summary = first_text(&review.summary)
                .or_else(|| first_text(&review.findings))
                .unwrap_or_else(|| review.verdict.to_string());
            format!("評価 {score} · {summary}")
        }
        (None, None) => {
            latest_summary(snapshot).unwrap_or_else(|| "まだ結果はありません。".to_string())
        }
    }
}

fn review_score_label(review: &ReviewResult, trace_records: &[TraceRecord]) -> String {
    if let Some((total, max)) = latest_trace_score(trace_records) {
        return format!("{total} / {max}");
    }
    if !review.criteria_results.is_empty() {
        let passed = review
            .criteria_results
            .iter()
            .filter(|result| result.status.to_string() == "passed")
            .count();
        return format!("{passed} / {}", review.criteria_results.len());
    }
    format!("{} / 100", review_score_percent(review, trace_records))
}

fn latest_trace_review_items(records: &[TraceRecord]) -> Option<Vec<ReviewItemView>> {
    records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .map(|record| trace_review_item_views(&record.payload))
        .filter(|items| !items.is_empty())
}

fn trace_review_item_views(payload: &serde_json::Value) -> Vec<ReviewItemView> {
    payload
        .get("item_verdicts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let points = value_u64(item, "points").unwrap_or_default();
                    let max_points = value_u64(item, "max_points").unwrap_or(1);
                    let score_label = if max_points == 0 {
                        String::new()
                    } else {
                        format!("{points}/{max_points}")
                    };
                    ReviewItemView {
                        item: value_str(item, "item").unwrap_or_else(|| "評価項目".to_string()),
                        verdict: value_str(item, "verdict").unwrap_or_else(|| "-".to_string()),
                        evidence: value_str(item, "evidence").unwrap_or_default(),
                        score_label,
                        concern_note: value_str(item, "concern_note").unwrap_or_default(),
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn first_assignment_rationale(payload: &serde_json::Value) -> String {
    payload
        .get("assignments")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
        .and_then(|item| value_str(item, "rationale"))
        .unwrap_or_default()
}

fn first_plan_target(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("plan")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
        .and_then(|item| value_str(item, "agent_id"))
}

fn worker_output_summary(payload: &serde_json::Value) -> String {
    let answer = value_str(payload, "answer").unwrap_or_default();
    let artifacts = payload
        .get("artifacts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| value_str(item, "path"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if artifacts.is_empty() {
        answer
    } else if answer.is_empty() {
        format!("できたもの: {}", artifacts.join(", "))
    } else {
        format!("{answer}\nできたもの: {}", artifacts.join(", "))
    }
}

fn review_output_summary(payload: &serde_json::Value) -> String {
    review_score_summary(payload)
        .or_else(|| value_str(payload, "recommendation").map(|value| recommendation_label(&value)))
        .unwrap_or_default()
}

fn review_score_summary(payload: &serde_json::Value) -> Option<String> {
    match (
        value_u64(payload, "total_score"),
        value_u64(payload, "max_score"),
    ) {
        (Some(total), Some(max)) if max > 0 => Some(format!("{total}/{max}")),
        _ => None,
    }
}

fn review_rationale_summary(payload: &serde_json::Value) -> String {
    let item_evidence = trace_review_item_views(payload)
        .into_iter()
        .filter(|item| meaningful_text(&item.evidence))
        .map(|item| format!("{}: {}", item.item, item.evidence))
        .collect::<Vec<_>>();
    if !item_evidence.is_empty() {
        return item_evidence
            .into_iter()
            .take(2)
            .collect::<Vec<_>>()
            .join(" / ");
    }
    value_str(payload, "rationale")
        .filter(|value| meaningful_text(value))
        .unwrap_or_default()
}

fn recommendation_label(value: &str) -> String {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "approve" | "approved" | "pass" | "passed" | "ok" => "採用を推奨".to_string(),
        "revise" | "request_changes" | "changes_requested" | "reject" => {
            "差し戻しを推奨".to_string()
        }
        "blocked" | "stop" => "停止を推奨".to_string(),
        other if other.is_empty() => String::new(),
        _ => value.to_string(),
    }
}

fn array_strings(payload: &serde_json::Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn value_path_str(payload: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = payload;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

fn value_str(payload: &serde_json::Value, key: &str) -> Option<String> {
    payload.get(key).and_then(|value| match value {
        serde_json::Value::String(value) if !value.trim().is_empty() => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn value_u64(payload: &serde_json::Value, key: &str) -> Option<u64> {
    payload.get(key).and_then(|value| value.as_u64())
}

fn status_view(status: WorkItemStatus, snapshot: Option<&WorkItemSnapshot>) -> (String, String) {
    let next_action = snapshot
        .map(|snapshot| snapshot.completion.next_action.as_str())
        .unwrap_or("");
    match status {
        WorkItemStatus::AgentRunning => ("処理中".to_string(), "running".to_string()),
        WorkItemStatus::NeedsInput => ("要対応・質問".to_string(), "question".to_string()),
        WorkItemStatus::ReadyForReview => ("要対応・確認".to_string(), "review".to_string()),
        WorkItemStatus::ChangesRequested => ("要対応・回復".to_string(), "recover".to_string()),
        WorkItemStatus::Done => ("完了".to_string(), "done".to_string()),
        WorkItemStatus::NeedsHandoff => ("要対応・回復".to_string(), "recover".to_string()),
        WorkItemStatus::Ready if matches!(next_action, "approve") => {
            ("要対応・確認".to_string(), "review".to_string())
        }
        WorkItemStatus::Ready => ("処理中".to_string(), "running".to_string()),
    }
}

fn next_action_label(next_action: &str) -> String {
    match next_action {
        "answer_question" => "質問に回答".to_string(),
        "approve" => "結果を確認".to_string(),
        "recover" | "apply_recovery" => "回復方法を選択".to_string(),
        "review" => "レビューを実行".to_string(),
        "run_agent" => "エージェント実行".to_string(),
        "dispatch" | "accept_dispatch" => "担当を整理".to_string(),
        "done" | "none" => "操作不要".to_string(),
        other if other.is_empty() => "操作不要".to_string(),
        other => other.to_string(),
    }
}

fn latest_summary(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .review_results
        .iter()
        .rev()
        .find_map(|review| first_text(&review.summary).or_else(|| first_text(&review.findings)))
        .or_else(|| snapshot.agent_outputs.iter().rev().find_map(output_summary))
}

fn latest_answer(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .filter(|output| {
            matches!(
                output.purpose,
                AgentRunPurpose::Work | AgentRunPurpose::Synthesis
            )
        })
        .find_map(output_summary)
}

fn output_summary(output: &AgentOutputRecord) -> Option<String> {
    output
        .fields
        .get("summary")
        .and_then(|values| values.first())
        .cloned()
        .or_else(|| {
            output
                .fields
                .get("completed")
                .and_then(|values| values.first())
                .cloned()
        })
        .or_else(|| output.questions.first().cloned())
}

fn review_view(review: &ReviewResult, trace_records: &[TraceRecord]) -> ReviewView {
    let items = latest_trace_review_items(trace_records).unwrap_or_else(|| {
        review
            .criteria_results
            .iter()
            .map(|result| ReviewItemView {
                item: result.criterion.clone(),
                verdict: result.status.to_string(),
                evidence: result.note.clone(),
                score_label: if result.status.to_string() == "passed" {
                    "1/1".to_string()
                } else {
                    "0/1".to_string()
                },
                concern_note: if result.status.to_string() == "passed" {
                    String::new()
                } else {
                    result.note.clone()
                },
            })
            .collect()
    });
    ReviewView {
        verdict: review.verdict.to_string(),
        summary: first_text(&review.summary)
            .unwrap_or_else(|| "レビュー要約はありません。".to_string()),
        score_label: review_score_label(review, trace_records),
        concerns: if review.requested_changes.is_empty() {
            review.findings.clone()
        } else {
            review.requested_changes.clone()
        },
        items,
    }
}

fn first_text(values: &[String]) -> Option<String> {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn derive_title(description: &str) -> String {
    let line = description
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("新しいワーク");
    let mut title = line.chars().take(42).collect::<String>();
    if line.chars().count() > 42 {
        title.push('…');
    }
    title
}

fn optional_text(value: Option<&str>) -> &str {
    value.map(str::trim).unwrap_or("")
}

fn text_lines(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn agent_prompt_draft_text(
    display_name: &str,
    role: &str,
    description: Option<&str>,
    specialties: &[String],
    domain_labels: &[String],
    artifact_labels: &[String],
) -> String {
    let name = display_name.trim();
    let name = if name.is_empty() {
        "このエージェント"
    } else {
        name
    };
    let description = optional_text(description);
    let description_line = if description.is_empty() {
        String::new()
    } else {
        format!("説明: {description}")
    };
    let role_action = match role.trim() {
        "organizer" => "依頼を整理し、担当エージェントとレビュー条件を選ぶ",
        "reviewer" => "成果物をルーブリックで評価し、点数・根拠・懸念を分けて返す",
        "worker" => "依頼に対して成果物を作成し、根拠と変更内容を明確にする",
        _ => "割り当てられた作業を実行する",
    };
    [
        format!("あなたは Nagare の「{name}」です。"),
        format!("主な役割: {role_action}。"),
        description_line,
        (!specialties.is_empty())
            .then(|| format!("得意分野: {}", specialties.join(" / ")))
            .unwrap_or_default(),
        (!domain_labels.is_empty())
            .then(|| format!("担当ドメイン: {}", domain_labels.join(" / ")))
            .unwrap_or_default(),
        (!artifact_labels.is_empty())
            .then(|| format!("担当成果物: {}", artifact_labels.join(" / ")))
            .unwrap_or_default(),
        String::new(),
        "守ること:".to_string(),
        "- 不明点を推測で埋めず、必要なら質問として返す".to_string(),
        "- 成果物の種類に応じた知識とルーブリックを参照する".to_string(),
        "- 出力には、作業結果、根拠、残る懸念、次に必要な操作を分けて書く".to_string(),
    ]
    .into_iter()
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

fn rubric_draft_text(
    display_name: &str,
    description: Option<&str>,
    knowledge: &[String],
    domain: Option<&Domain>,
) -> String {
    let name = display_name.trim();
    let name = if name.is_empty() { "成果物" } else { name };
    let description = optional_text(description);
    let domain_knowledge = domain
        .map(|domain| domain.shared_knowledge.join(" / "))
        .unwrap_or_default();
    let artifact_knowledge = knowledge.join(" / ");
    let knowledge_line = [
        (!domain_knowledge.is_empty()).then(|| format!("共通知識: {domain_knowledge}")),
        (!artifact_knowledge.is_empty()).then(|| format!("成果物知識: {artifact_knowledge}")),
        Some("必要な知識を反映し、不要な内部用語を避けている。".to_string()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    [
        "## 目的への適合 (30)".to_string(),
        format!(
            "{name}が依頼の目的に合っている。{}",
            if description.is_empty() {
                String::new()
            } else {
                format!("対象: {description}")
            }
        ),
        String::new(),
        "## 正確性と根拠 (30)".to_string(),
        "事実、手順、判断の根拠が確認でき、誤りや未検証の断定がない。".to_string(),
        String::new(),
        "## 読みやすさ (20)".to_string(),
        "対象読者が迷わず読める構成と表現になっている。".to_string(),
        String::new(),
        "## ドメイン知識の反映 (20)".to_string(),
        knowledge_line,
    ]
    .join("\n")
}

#[derive(Debug, Clone)]
struct RubricMarkdownSummary {
    item_count: usize,
    total_score: u32,
}

fn validate_rubric_markdown(raw: &str) -> Result<RubricMarkdownSummary, String> {
    let has_content = raw.lines().any(|line| !line.trim().is_empty());
    if !has_content {
        return Ok(RubricMarkdownSummary {
            item_count: 0,
            total_score: 0,
        });
    }

    let mut seen_titles = BTreeSet::new();
    let mut current_title: Option<String> = None;
    let mut current_has_body = false;
    let mut total_score = 0_u32;
    let mut item_count = 0_usize;

    for (index, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("## ") && !trimmed.starts_with("###") {
            if let Some(title) = current_title.as_deref() {
                if !current_has_body {
                    return Err(format!(
                        "ルーブリック項目「{title}」に判定基準本文を入力してください。"
                    ));
                }
            }
            let (title, score) = parse_rubric_heading(trimmed, index + 1)?;
            if !seen_titles.insert(title.clone()) {
                return Err(format!("ルーブリック項目「{title}」が重複しています。"));
            }
            current_title = Some(title);
            current_has_body = false;
            total_score = total_score.saturating_add(score);
            item_count += 1;
            continue;
        }
        if trimmed.starts_with('#') {
            return Err(format!(
                "ルーブリック{}行目は `## 項目名 (配点)` の見出し形式にしてください。",
                index + 1
            ));
        }
        if current_title.is_none() {
            return Err("ルーブリックは `## 項目名 (配点)` から開始してください。".to_string());
        }
        current_has_body = true;
    }

    let Some(title) = current_title else {
        return Err(
            "ルーブリックは `## 項目名 (配点)` の項目を1つ以上入力してください。".to_string(),
        );
    };
    if !current_has_body {
        return Err(format!(
            "ルーブリック項目「{title}」に判定基準本文を入力してください。"
        ));
    }
    if total_score != 100 {
        return Err(format!(
            "ルーブリックの配点合計は100点にしてください。現在は{total_score}点です。"
        ));
    }
    Ok(RubricMarkdownSummary {
        item_count,
        total_score,
    })
}

fn parse_rubric_heading(line: &str, line_number: usize) -> Result<(String, u32), String> {
    let body = line.trim_start_matches("##").trim();
    let Some(score_start) = body.rfind('(') else {
        return Err(format!(
            "ルーブリック{}行目は `## 項目名 (配点)` の形式にしてください。",
            line_number
        ));
    };
    if !body.ends_with(')') {
        return Err(format!(
            "ルーブリック{}行目は配点を括弧で閉じてください。",
            line_number
        ));
    }
    let title = body[..score_start].trim();
    if title.is_empty() {
        return Err(format!("ルーブリック{}行目の項目名が空です。", line_number));
    }
    let score_text = body[score_start + 1..body.len() - 1]
        .trim()
        .trim_end_matches('点')
        .trim();
    let score = score_text.parse::<u32>().map_err(|_| {
        format!(
            "ルーブリック{}行目の配点は数値で入力してください。",
            line_number
        )
    })?;
    if score == 0 {
        return Err(format!(
            "ルーブリック{}行目の配点は1点以上にしてください。",
            line_number
        ));
    }
    Ok((title.to_string(), score))
}

fn rubric_markdown_summary(lines: &[String]) -> Option<RubricMarkdownSummary> {
    validate_rubric_markdown(&lines.join("\n")).ok()
}

fn env_lines(value: Option<&str>) -> Result<BTreeMap<String, String>, String> {
    let mut env = BTreeMap::new();
    for line in value.unwrap_or("").lines().map(str::trim) {
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "環境変数 `{line}` は KEY=VALUE 形式で入力してください。"
            ));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err("環境変数のキーが空です。".to_string());
        }
        env.insert(key.to_string(), value.trim().to_string());
    }
    Ok(env)
}

fn normalized_values(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn agent_runtime_adapter(tool_kind: &str) -> Result<(&'static str, &'static str), String> {
    match tool_kind.trim() {
        "codex_cli" | "codex-cli" => Ok(("codex-local", "process-codex-cli")),
        "codex" | "codex_app" | "codex-app" => Ok(("codex-app-local", "stdio-codex-app-server")),
        "claude" | "claude_code" | "claude-code" => Ok(("claude-local", "process-claude-code")),
        "opencode" | "open_code" | "open-code" => Ok(("opencode-local", "process-opencode")),
        "openclaw" => Ok(("openclaw-local", "process-openclaw-agent")),
        other => Err(format!(
            "未対応のエージェントツールです: {other}。Claude Code / Codex CLI / Codex App Server / OpenCode / OpenClaw から選択してください。"
        )),
    }
}

fn project_label(work_folder: Option<&str>) -> String {
    let value = work_folder.unwrap_or(".").trim();
    if value.is_empty() || value == "." {
        "nagare".to_string()
    } else {
        value
            .trim_matches('/')
            .trim_matches('\\')
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(value)
            .to_string()
    }
}

fn is_project_initialized(root: &Path) -> bool {
    let layout = ProjectLayout::new(root);
    layout.config_path.exists() && layout.ledger_path.exists()
}

fn default_project_icon() -> &'static str {
    "🌊"
}

fn project_name_from_root(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("nagare")
        .to_string()
}

fn resolve_desktop_root(root: Option<String>) -> Result<PathBuf, String> {
    if let Some(root) = root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(root));
    }
    if let Some(root) = env::var_os("NAGARE_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let current = env::current_dir().map_err(|error| error.to_string())?;
    if let Some(root) = find_repo_root(&current) {
        return Ok(root);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(root) = find_repo_root(parent) {
                return Ok(root);
            }
        }
    }
    Ok(current)
}

fn resolve_artifact_uri(root: &Path, uri: &str) -> Result<PathBuf, String> {
    if uri.trim().is_empty() {
        return Err("artifact uri is required".to_string());
    }
    let root_canonical = root
        .canonicalize()
        .map_err(|error| format!("failed to resolve project root: {error}"))?;
    let candidate = artifact_uri_to_path(uri)?;
    let path = if candidate.is_absolute() {
        candidate
    } else {
        root_canonical.join(candidate)
    };
    let canonical = path.canonicalize().map_err(|error| {
        format!(
            "failed to resolve artifact path `{}`: {error}",
            path.display()
        )
    })?;
    if !canonical.starts_with(&root_canonical) {
        return Err(format!(
            "refusing to read artifact outside project root: `{}`",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn artifact_uri_to_path(uri: &str) -> Result<PathBuf, String> {
    let trimmed = uri.trim();
    if let Some(rest) = trimmed.strip_prefix("file://") {
        let decoded = percent_decode_path(rest)?;
        let normalized = if cfg!(windows)
            && decoded.starts_with('/')
            && decoded
                .as_bytes()
                .get(2)
                .is_some_and(|value| *value == b':')
        {
            &decoded[1..]
        } else {
            decoded.as_str()
        };
        return Ok(PathBuf::from(normalized));
    }
    Ok(PathBuf::from(trimmed))
}

fn percent_decode_path(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(format!("invalid percent-encoded artifact uri `{value}`"));
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .map_err(|error| error.to_string())?;
            let decoded = u8::from_str_radix(hex, 16)
                .map_err(|_| format!("invalid percent-encoded artifact uri `{value}`"))?;
            output.push(decoded);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|error| error.to_string())
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    for path in start.ancestors() {
        if path
            .join("docs/design-assets/prototype/index.html")
            .exists()
            && path.join("crates/nagare-core").exists()
        {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn root_to_string(root: &Path) -> String {
    root.to_string_lossy().replace('\\', "/")
}

struct RuntimeCatalogEntry {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    args: &'static [&'static str],
    model_note: &'static str,
    model_mode: &'static str,
    model_choices: &'static [&'static str],
}

fn runtime_views(agents: &[AgentView]) -> Vec<RuntimeView> {
    runtime_catalog()
        .into_iter()
        .map(|entry| runtime_view_from_catalog(entry, agents))
        .collect()
}

fn runtime_view_by_id(runtime_id: &str, agents: &[AgentView]) -> Result<RuntimeView, String> {
    let requested = runtime_id.trim();
    let runtime_id = runtime_catalog_id(requested);
    runtime_catalog()
        .into_iter()
        .find(|entry| entry.id == runtime_id)
        .map(|entry| runtime_view_from_catalog(entry, agents))
        .ok_or_else(|| format!("unknown runtime `{requested}`"))
}

fn ensure_initial_runtime_available(runtime_id: &str) -> Result<(), String> {
    let runtime = runtime_view_by_id(runtime_id, &[])?;
    if runtime.available {
        return Ok(());
    }
    Err(format!(
        "{} が見つかりません。`{}` をインストールし、PATHから実行できる状態にしてください。詳細: {}",
        runtime.label, runtime.command, runtime.detail
    ))
}

fn runtime_view_from_catalog(entry: RuntimeCatalogEntry, agents: &[AgentView]) -> RuntimeView {
    let (available, detail) = command_status(entry.command, entry.args);
    let (configured_model, configured_provider, configured_base_url, configured_agent_count) =
        runtime_model_configuration(entry.id, agents);
    RuntimeView {
        id: entry.id,
        label: entry.label,
        command: entry.command,
        available,
        detail,
        model_note: entry.model_note,
        model_mode: entry.model_mode,
        model_choices: entry.model_choices.to_vec(),
        configured_model,
        configured_provider,
        configured_base_url,
        configured_agent_count,
    }
}

fn runtime_model_configuration(
    runtime_id: &str,
    agents: &[AgentView],
) -> (String, String, String, usize) {
    let tool_kind = runtime_tool_kind(runtime_id);
    let target_agents = agents
        .iter()
        .filter(|agent| agent.tool_kind == tool_kind || agent.runtime == runtime_id)
        .collect::<Vec<_>>();
    let configured_agent_count = target_agents.len();
    let mut models = BTreeSet::new();
    let mut providers = BTreeSet::new();
    let mut base_urls = BTreeSet::new();
    for agent in target_agents {
        if !agent.model.trim().is_empty() && agent.model != "実行環境既定" {
            models.insert(agent.model.clone());
        }
        if !agent.model_provider.trim().is_empty() {
            providers.insert(agent.model_provider.clone());
        }
        if !agent.model_base_url.trim().is_empty() {
            base_urls.insert(agent.model_base_url.clone());
        }
    }
    let single_or_mixed = |values: BTreeSet<String>| -> String {
        match values.len() {
            0 => String::new(),
            1 => values.into_iter().next().unwrap_or_default(),
            _ => "個別設定".to_string(),
        }
    };
    (
        single_or_mixed(models),
        single_or_mixed(providers),
        single_or_mixed(base_urls),
        configured_agent_count,
    )
}

fn runtime_tool_kind(runtime_id: &str) -> &'static str {
    match runtime_catalog_id(runtime_id) {
        "claude" => "claude_code",
        "codex" => "codex_cli",
        "opencode" => "opencode",
        "openclaw" => "openclaw",
        _ => "",
    }
}

fn runtime_catalog_id(runtime_id: &str) -> &'static str {
    match runtime_id
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
        .as_str()
    {
        "claude" | "claude_code" => "claude",
        "codex" | "codex_cli" => "codex",
        "opencode" | "open_code" => "opencode",
        "openclaw" | "open_claw" => "openclaw",
        _ => "",
    }
}

fn runtime_catalog() -> Vec<RuntimeCatalogEntry> {
    vec![
        RuntimeCatalogEntry {
            id: "claude",
            label: "Claude Code",
            command: "claude",
            args: &["--version"],
            model_note: "既定値または手入力",
            model_mode: "既定値 / 手入力",
            model_choices: &["実行環境既定", "手入力"],
        },
        RuntimeCatalogEntry {
            id: "codex",
            label: "Codex CLI",
            command: "codex",
            args: &["--version"],
            model_note: "OpenAIモデルを選択または手入力",
            model_mode: "OpenAIモデル",
            model_choices: &["実行環境既定", "gpt-5-codex", "手入力"],
        },
        RuntimeCatalogEntry {
            id: "opencode",
            label: "OpenCode",
            command: "opencode",
            args: &["--version"],
            model_note: "Provider別モデル",
            model_mode: "Provider / Model",
            model_choices: &["実行環境既定", "Provider指定", "手入力"],
        },
        RuntimeCatalogEntry {
            id: "openclaw",
            label: "OpenClaw",
            command: "openclaw",
            args: &["--version"],
            model_note: "OpenAI / Ollama / LMStudio",
            model_mode: "Provider / Base URL / Model",
            model_choices: &["OpenAI", "Ollama", "LMStudio", "手入力"],
        },
    ]
}

fn command_status(command: &str, args: &[&str]) -> (bool, String) {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stdout
                .lines()
                .chain(stderr.lines())
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or("available")
                .to_string();
            (true, detail)
        }
        Ok(output) => (false, format!("exit status {}", output.status)),
        Err(error) if cfg!(windows) && error.kind() == std::io::ErrorKind::NotFound => {
            match Command::new(format!("{command}.cmd")).args(args).output() {
                Ok(output) if output.status.success() => (true, "available".to_string()),
                Ok(output) => (false, format!("exit status {}", output.status)),
                Err(error) => (false, error.to_string()),
            }
        }
        Err(error) => (false, error.to_string()),
    }
}

fn advance_error(error: nagare_core::NagareError) -> String {
    let message = error.to_string();
    if message.contains("program not found") || message.contains("No such file") {
        format!(
            "{message}\n実行環境が見つかりません。実行環境画面でCLIのインストールと接続状態を確認してください。"
        )
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nagare_core::{
        AgentOutputParseStatus, Artifact, CriteriaReviewResult, CriteriaReviewStatus,
        DomainAgentPolicy, WorkItemApprovalGate, WorkItemCompletion,
    };
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn initial_runtime_mapping_supports_default_agent_runtimes() {
        assert_eq!(
            initial_runtime_mapping("codex").expect("codex should map"),
            ("codex-local", "process-codex-cli", "codex-cli")
        );
        assert_eq!(
            initial_runtime_mapping("claude").expect("claude should map"),
            ("claude-local", "process-claude-code", "claude-code")
        );
        assert_eq!(
            initial_runtime_mapping("opencode").expect("opencode should map"),
            ("opencode-local", "process-opencode", "opencode")
        );
        assert_eq!(
            initial_runtime_mapping("openclaw").expect("openclaw should map"),
            ("openclaw-local", "process-openclaw-agent", "openclaw")
        );
        assert!(initial_runtime_mapping("unknown").is_err());
    }

    #[test]
    fn runtime_catalog_id_accepts_setup_aliases() {
        assert_eq!(runtime_catalog_id("codex-cli"), "codex");
        assert_eq!(runtime_catalog_id("codex_cli"), "codex");
        assert_eq!(runtime_catalog_id("claude-code"), "claude");
        assert_eq!(runtime_catalog_id("open_code"), "opencode");
        assert_eq!(runtime_tool_kind("openclaw"), "openclaw");
        assert_eq!(runtime_tool_kind("unknown"), "");
    }

    #[test]
    fn save_runtime_model_defaults_rolls_back_prior_agent_updates_on_failure() {
        let root = temp_test_dir("runtime-model-rollback");
        init_project(&root).expect("init project");
        let layout = ProjectLayout::new(&root);
        for agent_id in ["aa-runtime-agent", "zz-blocked-runtime-agent"] {
            add_agent_profile(
                &root,
                AddAgentProfileInput {
                    id: agent_id,
                    display_name: agent_id,
                    runtime: "codex-local",
                    adapter: "process-codex-cli",
                    role: "worker",
                    working_dir: ".",
                    description: "Runtime model rollback test agent.",
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
            .expect("test agent should be added");
        }
        let blocked_profile_path = layout.agents_dir.join("zz-blocked-runtime-agent.toml");
        assert!(
            blocked_profile_path.is_file(),
            "blocked profile should exist"
        );
        let mut permissions = fs::metadata(&blocked_profile_path)
            .expect("profile metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&blocked_profile_path, permissions)
            .expect("profile should become readonly");

        let result = save_runtime_model_defaults(SaveRuntimeModelDefaultsRequest {
            root: Some(root_to_string(&root)),
            runtime_id: "codex".to_string(),
            model_provider: Some("openai".to_string()),
            model: Some("gpt-5.1-codex".to_string()),
            model_base_url: None,
        });

        let mut permissions = fs::metadata(&blocked_profile_path)
            .expect("profile metadata")
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&blocked_profile_path, permissions)
            .expect("profile should become writable");

        assert!(
            result.is_err(),
            "readonly profile should reject runtime model save"
        );
        let profiles = list_agent_profiles(&root).expect("profiles should list");
        for profile in profiles.iter().filter(|profile| {
            profile.tool_kind == AgentToolKind::CodexCli
                && (profile.id == "aa-runtime-agent" || profile.id == "zz-blocked-runtime-agent")
        }) {
            assert_eq!(
                profile.model,
                AgentModelSelection::default(),
                "{} should keep its original model",
                profile.id
            );
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_artifact_uri_accepts_project_file_uri() {
        let root = temp_test_dir("artifact-uri-ok");
        let artifact = root.join("README.md");
        fs::write(&artifact, "# README").expect("write artifact");
        let uri = format!("file:///{}", root_to_string(&artifact));

        let resolved = resolve_artifact_uri(&root, &uri).expect("artifact should resolve");

        assert_eq!(
            resolved,
            artifact.canonicalize().expect("canonical artifact")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_artifact_uri_rejects_files_outside_project() {
        let root = temp_test_dir("artifact-uri-root");
        let outside_root = temp_test_dir("artifact-uri-outside");
        let outside = outside_root.join("secret.md");
        fs::write(&outside, "secret").expect("write outside artifact");

        let error = resolve_artifact_uri(&root, outside.to_string_lossy().as_ref())
            .expect_err("outside artifact should be rejected");

        assert!(error.contains("outside project root"));
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside_root);
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "nagare-desktop-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temp test dir");
        path
    }

    #[test]
    fn app_state_treats_missing_saved_root_as_uninitialized() {
        let root = temp_test_dir("missing-saved-root");
        fs::remove_dir_all(&root).expect("remove temp root");

        let state = app_state(Some(root_to_string(&root))).expect("app state should recover");

        assert!(!state.initialized);
        assert_eq!(state.root, "");
        assert!(state.project.is_none());
        assert!(state.work_items.is_empty());
        assert!(state.runtimes.iter().any(|runtime| runtime.id == "codex"));
    }

    #[test]
    fn desktop_state_after_project_initialization_matches_home_contract() {
        let root = temp_test_dir("desktop-state");
        init_project(&root).expect("init project");
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some("Nagare UI 刷新"),
                icon: Some("流"),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .expect("set project metadata");
        configure_initial_agent_runtime(&root, "codex").expect("configure initial runtime");

        let state = desktop_state(root.clone()).expect("desktop state");
        let project = state.project.as_ref().expect("project view");

        assert!(state.initialized);
        assert_eq!(project.name, "Nagare UI 刷新");
        assert_eq!(project.icon, "流");
        assert_eq!(project.organizer_agent_id, "");
        assert_eq!(project.organizer_label, "標準（内蔵オーガナイザー）");
        assert!(
            state
                .agents
                .iter()
                .any(|agent| agent.role == "worker" && agent.tool_kind == "codex_cli")
        );
        assert!(state.agents.iter().any(|agent| agent.role == "reviewer"));
        assert!(
            !state.domains.is_empty(),
            "default domains should be visible"
        );
        assert!(
            !state.artifact_types.is_empty(),
            "default artifact types should be visible"
        );
        assert!(state.runtimes.iter().any(|runtime| runtime.id == "codex"));
        assert!(
            state
                .mcp_capabilities
                .iter()
                .any(|capability| capability.tool_kind == "codex_cli"),
            "MCP capability table should be returned for settings screens"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_mcp_connection_rejects_invalid_env_without_adding_connection() {
        let root = temp_test_dir("mcp-invalid-env");
        init_project(&root).expect("init project");

        let result = save_mcp_connection(SaveMcpConnectionRequest {
            root: Some(root_to_string(&root)),
            id: "filesystem".to_string(),
            display_name: "Filesystem MCP".to_string(),
            tool_kind: "codex_cli".to_string(),
            command: "npx".to_string(),
            args: Some("-y\n@modelcontextprotocol/server-filesystem".to_string()),
            env: Some("BROKEN_ENV_LINE".to_string()),
            test_args: None,
        });
        let error = match result {
            Ok(_) => panic!("invalid env should reject MCP save"),
            Err(error) => error,
        };

        assert!(error.contains("KEY=VALUE"));
        let state = desktop_state(root.clone()).expect("desktop state");
        assert!(
            state
                .mcp_connections
                .iter()
                .all(|connection| connection.id != "filesystem"),
            "invalid MCP connection should not be saved"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_artifact_type_rejects_invalid_rubric_without_side_effects() {
        let root = temp_test_dir("artifact-invalid-rubric");
        init_project(&root).expect("init project");

        let result = save_artifact_type(SaveArtifactTypeRequest {
            root: Some(root_to_string(&root)),
            id: "faq".to_string(),
            domain_id: Some("general".to_string()),
            display_name: "FAQ".to_string(),
            description: Some("よくある質問".to_string()),
            knowledge: Some("FAQテンプレート".to_string()),
            rubric: Some("## 正確性 (60)\n具体的である\n\n## 正確性 (40)\n根拠がある".to_string()),
            dispatch_hints: Some("faq".to_string()),
            improvement_proposal_id: Some("proposal-rubric-faq".to_string()),
            improvement_kind: Some("ルーブリック".to_string()),
            improvement_title: Some("FAQ ルーブリック改善".to_string()),
            improvement_target_label: Some("FAQ / 正確性".to_string()),
            improvement_summary: Some("重複項目のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => panic!("invalid rubric should reject artifact save"),
            Err(error) => error,
        };

        assert!(error.contains("重複"));
        let state = desktop_state(root.clone()).expect("desktop state");
        assert!(
            state
                .artifact_types
                .iter()
                .all(|artifact| artifact.id != "faq"),
            "invalid artifact type should not be saved"
        );
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "invalid artifact save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_artifact_type_rolls_back_new_artifact_when_improvement_history_fails() {
        let root = temp_test_dir("artifact-improvement-rollback");
        init_project(&root).expect("init project");
        let layout = ProjectLayout::new(&root);
        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become readonly");

        let result = save_artifact_type(SaveArtifactTypeRequest {
            root: Some(root_to_string(&root)),
            id: "faq".to_string(),
            domain_id: Some("general".to_string()),
            display_name: "FAQ".to_string(),
            description: Some("よくある質問".to_string()),
            knowledge: Some("FAQテンプレート".to_string()),
            rubric: Some("## 正確性 (100)\n根拠がある".to_string()),
            dispatch_hints: Some("faq".to_string()),
            improvement_proposal_id: Some("proposal-rubric-faq".to_string()),
            improvement_kind: Some("ルーブリック".to_string()),
            improvement_title: Some("FAQ ルーブリック改善".to_string()),
            improvement_target_label: Some("FAQ / 正確性".to_string()),
            improvement_summary: Some("改善履歴失敗のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });

        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become writable");

        assert!(
            result.is_err(),
            "readonly project config should reject improvement history write"
        );
        let state = desktop_state(root.clone()).expect("desktop state");
        assert!(
            state
                .artifact_types
                .iter()
                .all(|artifact| artifact.id != "faq"),
            "failed improvement save should remove newly saved artifact type"
        );
        assert!(
            !layout.artifact_types_dir.join("faq.toml").exists(),
            "new artifact file should be removed during rollback"
        );
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_artifact_type_restores_existing_artifact_when_improvement_history_fails() {
        let root = temp_test_dir("artifact-update-improvement-rollback");
        init_project(&root).expect("init project");
        add_artifact_type(
            &root,
            AddArtifactTypeInput {
                id: "faq",
                domain_id: Some("general"),
                display_name: "FAQ",
                description: "Original FAQ",
                artifact_types: vec!["Original knowledge".to_string()],
                rubric: vec!["## 正確性 (100)".to_string(), "根拠がある".to_string()],
                dispatch_hints: vec!["faq".to_string()],
                workflow: DomainWorkflowOverride::default(),
            },
        )
        .expect("artifact should be added");
        let layout = ProjectLayout::new(&root);
        let before = get_artifact_type(&root, "faq").expect("artifact should load");
        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become readonly");

        let result = save_artifact_type(SaveArtifactTypeRequest {
            root: Some(root_to_string(&root)),
            id: "faq".to_string(),
            domain_id: Some("general".to_string()),
            display_name: "FAQ Updated".to_string(),
            description: Some("Changed FAQ".to_string()),
            knowledge: Some("Changed knowledge".to_string()),
            rubric: Some(
                "## 正確性 (60)\n根拠がある\n\n## 読みやすさ (40)\n読みやすい".to_string(),
            ),
            dispatch_hints: Some("changed".to_string()),
            improvement_proposal_id: Some("proposal-rubric-faq".to_string()),
            improvement_kind: Some("ルーブリック".to_string()),
            improvement_title: Some("FAQ ルーブリック改善".to_string()),
            improvement_target_label: Some("FAQ / 正確性".to_string()),
            improvement_summary: Some("改善履歴失敗のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });

        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become writable");

        assert!(
            result.is_err(),
            "readonly project config should reject improvement history write"
        );
        let after = get_artifact_type(&root, "faq").expect("artifact should remain");
        assert_eq!(after.display_name, before.display_name);
        assert_eq!(after.description, before.description);
        assert_eq!(after.artifact_types, before.artifact_types);
        assert_eq!(after.rubric, before.rubric);
        assert_eq!(after.rubric_version, before.rubric_version);
        assert_eq!(after.dispatch_hints, before.dispatch_hints);
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_exclude_applied_improvement_proposals() {
        let root = temp_test_dir("insights-applied-proposals");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.review_results[0].criteria_results[0].status = CriteriaReviewStatus::Failed;
        snapshot.review_results[0].criteria_results[0].note =
            "目的への適合が不足しています。".to_string();
        let snapshots = vec![snapshot];

        let initial = insights_view(&root, &snapshots, &[]);
        assert_eq!(initial.proposal_count, 1);
        let proposal = initial.proposals.first().expect("proposal should exist");
        let proposal_id = proposal.id.clone();

        record_improvement_applied(
            &root,
            RecordImprovementInput {
                proposal_id: &proposal_id,
                kind: &proposal.kind,
                title: &proposal.title,
                target_label: &proposal.target_label,
                summary: &proposal.summary,
                evidence: &proposal.evidence,
            },
        )
        .expect("improvement should record");

        let refreshed = insights_view(&root, &snapshots, &[]);
        assert_eq!(refreshed.proposal_count, 0);
        assert!(
            refreshed
                .proposals
                .iter()
                .all(|item| item.id != proposal_id),
            "applied proposal should not return to the pending list"
        );
        assert!(
            refreshed
                .applied_improvements
                .iter()
                .any(|item| item.proposal_id == proposal_id
                    && item.title == proposal.title
                    && item.kind == proposal.kind
                    && item.target_label == proposal.target_label),
            "applied history should still show the improvement"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_project_settings_updates_default_worker_and_reviewer() {
        let root = temp_test_dir("project-default-agents");
        init_project(&root).expect("init project");
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id: "project-worker",
                display_name: "Project Worker",
                runtime: "codex-local",
                adapter: "process-codex-cli",
                role: "worker",
                working_dir: ".",
                description: "Project-specific worker",
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
        .expect("worker agent should be added");
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id: "project-reviewer",
                display_name: "Project Reviewer",
                runtime: "codex-local",
                adapter: "process-codex-cli",
                role: "reviewer",
                working_dir: ".",
                description: "Project-specific reviewer",
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
        .expect("reviewer agent should be added");

        let state = save_project_settings(ProjectSettingsRequest {
            root: Some(root_to_string(&root)),
            display_name: Some("Default Agent Project".to_string()),
            icon: Some("担".to_string()),
            default_domain_id: Some("general".to_string()),
            default_artifact_type_id: Some("general".to_string()),
            organizer_agent_id: Some("__builtin__".to_string()),
            work_agent_id: Some("project-worker".to_string()),
            review_agent_id: Some("project-reviewer".to_string()),
            workflow_mode: Some("finish_first".to_string()),
            approval_policy: Some("manual_on_review_concern".to_string()),
            improvement_proposal_id: None,
            improvement_kind: None,
            improvement_title: None,
            improvement_target_label: None,
            improvement_summary: None,
            improvement_evidence: None,
        })
        .expect("project settings should save");

        let project = state.project.expect("project view");
        assert_eq!(project.name, "Default Agent Project");
        assert_eq!(project.icon, "担");
        assert_eq!(project.default_domain_id, "general");
        assert_eq!(project.default_artifact_type_id, "general");
        assert_eq!(project.work_agent, "Project Worker");
        assert_eq!(project.review_agent, "Project Reviewer");
        assert_eq!(project.workflow_mode, "finish_first");
        assert_eq!(project.approval_policy, "manual_on_review_concern");
        let settings = get_nagare_agent_settings(&root).expect("settings");
        assert_eq!(settings.work_agent, "project-worker");
        assert_eq!(settings.review_agent, "project-reviewer");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_project_settings_rejects_default_artifact_outside_selected_domain() {
        let root = temp_test_dir("project-default-artifact-mismatch");
        init_project(&root).expect("init project");
        add_domain(
            &root,
            AddDomainInput {
                id: "support",
                display_name: "Support",
                description: "Support knowledge",
                shared_knowledge: Vec::new(),
                common_rubric: Vec::new(),
                dispatch_hints: Vec::new(),
                workflow: DomainWorkflowOverride::default(),
            },
        )
        .expect("support domain should be added");
        add_artifact_type(
            &root,
            AddArtifactTypeInput {
                id: "faq",
                domain_id: Some("support"),
                display_name: "FAQ",
                description: "FAQ answer",
                artifact_types: Vec::new(),
                rubric: vec!["## 正確性 (100)".to_string(), "根拠がある".to_string()],
                dispatch_hints: Vec::new(),
                workflow: DomainWorkflowOverride::default(),
            },
        )
        .expect("faq artifact should be added");

        let result = save_project_settings(ProjectSettingsRequest {
            root: Some(root_to_string(&root)),
            display_name: Some("Should Not Apply".to_string()),
            icon: Some("不".to_string()),
            default_domain_id: Some("general".to_string()),
            default_artifact_type_id: Some("faq".to_string()),
            organizer_agent_id: None,
            work_agent_id: None,
            review_agent_id: None,
            workflow_mode: Some("finish_first".to_string()),
            approval_policy: Some("manual_on_review_concern".to_string()),
            improvement_proposal_id: Some("operation-approval-policy".to_string()),
            improvement_kind: Some("運用".to_string()),
            improvement_title: Some("確認ポリシー緩和の提案".to_string()),
            improvement_target_label: Some("プロジェクト設定 / 確認ポリシー".to_string()),
            improvement_summary: Some("不整合保存のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => panic!("mismatched default artifact should reject project settings save"),
            Err(error) => error,
        };

        assert!(error.contains("does not belong to Domain `general`"));
        let metadata = get_project_metadata(root.clone()).expect("project metadata");
        assert_ne!(metadata.name, "Should Not Apply");
        assert_ne!(metadata.icon, "不");
        assert_ne!(metadata.default_domain_id, "general");
        assert_ne!(metadata.default_artifact_type_id, "faq");
        let workflow = get_workflow_settings(&root).expect("workflow settings");
        assert_eq!(workflow.default_progress_mode, WorkflowMode::ConfirmFirst);
        assert_eq!(
            workflow.approval_policy,
            ApprovalPolicy::ManualFinalApproval
        );
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed project settings save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_project_settings_validates_workflow_before_writing_metadata() {
        let root = temp_test_dir("project-invalid-workflow");
        init_project(&root).expect("init project");
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some("Original Project"),
                icon: Some("原"),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .expect("set original metadata");

        let result = save_project_settings(ProjectSettingsRequest {
            root: Some(root_to_string(&root)),
            display_name: Some("Should Not Apply".to_string()),
            icon: Some("不".to_string()),
            default_domain_id: None,
            default_artifact_type_id: None,
            organizer_agent_id: None,
            work_agent_id: None,
            review_agent_id: None,
            workflow_mode: Some("invalid-mode".to_string()),
            approval_policy: Some("manual_on_review_concern".to_string()),
            improvement_proposal_id: Some("operation-approval-policy".to_string()),
            improvement_kind: Some("運用".to_string()),
            improvement_title: Some("確認ポリシー緩和の提案".to_string()),
            improvement_target_label: Some("プロジェクト設定 / 確認ポリシー".to_string()),
            improvement_summary: Some("不正ワークフローのテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => panic!("invalid workflow should reject project settings save"),
            Err(error) => error,
        };

        assert!(error.contains("invalid_mode"));
        let metadata = get_project_metadata(root.clone()).expect("project metadata");
        assert_eq!(metadata.name, "Original Project");
        assert_eq!(metadata.icon, "原");
        let workflow = get_workflow_settings(&root).expect("workflow settings");
        assert_eq!(workflow.default_progress_mode, WorkflowMode::ConfirmFirst);
        assert_eq!(
            workflow.approval_policy,
            ApprovalPolicy::ManualFinalApproval
        );
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed project settings save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_project_settings_validates_agents_before_writing_metadata() {
        let root = temp_test_dir("project-invalid-default-agent");
        init_project(&root).expect("init project");
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some("Original Project"),
                icon: Some("原"),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .expect("set original metadata");

        let result = save_project_settings(ProjectSettingsRequest {
            root: Some(root_to_string(&root)),
            display_name: Some("Should Not Apply".to_string()),
            icon: Some("不".to_string()),
            default_domain_id: None,
            default_artifact_type_id: None,
            organizer_agent_id: Some("__builtin__".to_string()),
            work_agent_id: Some("missing-worker".to_string()),
            review_agent_id: Some("reviewer".to_string()),
            workflow_mode: Some("finish_first".to_string()),
            approval_policy: Some("manual_on_review_concern".to_string()),
            improvement_proposal_id: Some("operation-approval-policy".to_string()),
            improvement_kind: Some("運用".to_string()),
            improvement_title: Some("確認ポリシー緩和の提案".to_string()),
            improvement_target_label: Some("プロジェクト設定 / 確認ポリシー".to_string()),
            improvement_summary: Some("不正エージェントのテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => panic!("missing default agent should reject project settings save"),
            Err(error) => error,
        };

        assert!(error.contains("agent profile `missing-worker` not found"));
        let metadata = get_project_metadata(root.clone()).expect("project metadata");
        assert_eq!(metadata.name, "Original Project");
        assert_eq!(metadata.icon, "原");
        let workflow = get_workflow_settings(&root).expect("workflow settings");
        assert_eq!(workflow.default_progress_mode, WorkflowMode::ConfirmFirst);
        assert_eq!(
            workflow.approval_policy,
            ApprovalPolicy::ManualFinalApproval
        );
        let settings = get_nagare_agent_settings(&root).expect("settings");
        assert_eq!(settings.work_agent, "worker");
        assert_eq!(settings.review_agent, "reviewer");
        assert_eq!(settings.organizer_agent, None);
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed project settings save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_project_settings_restores_settings_when_improvement_history_fails() {
        let root = temp_test_dir("project-improvement-rollback");
        init_project(&root).expect("init project");
        set_project_metadata(
            &root,
            SetProjectMetadataInput {
                name: Some("Original Project"),
                icon: Some("原"),
                default_domain_id: None,
                default_artifact_type_id: None,
            },
        )
        .expect("set original metadata");
        for (id, name, role) in [
            ("project-worker", "Project Worker", "worker"),
            ("project-reviewer", "Project Reviewer", "reviewer"),
        ] {
            add_agent_profile(
                &root,
                AddAgentProfileInput {
                    id,
                    display_name: name,
                    runtime: "codex-local",
                    adapter: "process-codex-cli",
                    role,
                    working_dir: ".",
                    description: "Project agent",
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
            .expect("agent should be added");
        }

        let result = save_project_settings(ProjectSettingsRequest {
            root: Some(root_to_string(&root)),
            display_name: Some("Should Not Apply".to_string()),
            icon: Some("不".to_string()),
            default_domain_id: Some("general".to_string()),
            default_artifact_type_id: Some("general".to_string()),
            organizer_agent_id: Some("__builtin__".to_string()),
            work_agent_id: Some("project-worker".to_string()),
            review_agent_id: Some("project-reviewer".to_string()),
            workflow_mode: Some("finish_first".to_string()),
            approval_policy: Some("manual_on_review_concern".to_string()),
            improvement_proposal_id: Some("__fail_record_improvement__".to_string()),
            improvement_kind: Some("運用".to_string()),
            improvement_title: Some("確認ポリシー緩和の提案".to_string()),
            improvement_target_label: Some("プロジェクト設定 / 確認ポリシー".to_string()),
            improvement_summary: Some("改善履歴失敗のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => {
                panic!("forced improvement history failure should reject project settings save")
            }
            Err(error) => error,
        };

        assert!(error.contains("forced improvement history failure"));
        let metadata = get_project_metadata(root.clone()).expect("project metadata");
        assert_eq!(metadata.name, "Original Project");
        assert_eq!(metadata.icon, "原");
        assert_eq!(metadata.default_domain_id, "");
        assert_eq!(metadata.default_artifact_type_id, "");
        let workflow = get_workflow_settings(&root).expect("workflow settings");
        assert_eq!(workflow.default_progress_mode, WorkflowMode::ConfirmFirst);
        assert_eq!(
            workflow.approval_policy,
            ApprovalPolicy::ManualFinalApproval
        );
        let settings = get_nagare_agent_settings(&root).expect("settings");
        assert_eq!(settings.work_agent, "worker");
        assert_eq!(settings.review_agent, "reviewer");
        assert_eq!(settings.organizer_agent, None);
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed project improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_agent_rejects_invalid_capabilities_without_writing_profile() {
        let root = temp_test_dir("agent-invalid-capability");
        init_project(&root).expect("init project");
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id: "ui-worker",
                display_name: "UI Worker",
                runtime: "codex-local",
                adapter: "process-codex-cli",
                role: "worker",
                working_dir: ".",
                description: "Original description",
                specialties: vec!["ui".to_string()],
                skill_set_ids: Vec::new(),
                domain_ids: Vec::new(),
                artifact_type_ids: Vec::new(),
                mcp_connection_ids: Vec::new(),
                managed_by: None,
                model: AgentModelSelection::default(),
                external: ExternalAgentBinding::default(),
            },
        )
        .expect("agent should be added");

        let result = save_agent(SaveAgentRequest {
            root: Some(root_to_string(&root)),
            id: "ui-worker".to_string(),
            display_name: "Should Not Apply".to_string(),
            avatar: Some("changed.svg".to_string()),
            role: "reviewer".to_string(),
            tool_kind: "codex_cli".to_string(),
            model: Some("gpt-5-codex".to_string()),
            model_provider: Some("openai".to_string()),
            model_base_url: None,
            description: Some("Changed description".to_string()),
            prompt: Some("Changed prompt".to_string()),
            specialties: vec!["changed".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            skill_set_ids: vec!["missing-skill".to_string()],
            mcp_connection_ids: Vec::new(),
            improvement_proposal_id: Some("agent-prompt-ui-worker".to_string()),
            improvement_kind: Some("エージェント".to_string()),
            improvement_title: Some("UI Worker の改善".to_string()),
            improvement_target_label: Some("エージェント / UI Worker".to_string()),
            improvement_summary: Some("不正能力保存のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });
        let error = match result {
            Ok(_) => panic!("invalid skill capability should reject agent save"),
            Err(error) => error,
        };

        assert!(error.contains("skill set `missing-skill` is not declared"));
        let profile = list_agent_profiles(&root)
            .expect("agents should list")
            .into_iter()
            .find(|profile| profile.id == "ui-worker")
            .expect("agent should remain");
        assert_eq!(profile.display_name, "UI Worker");
        assert_eq!(profile.avatar, "");
        assert_eq!(profile.role, "worker");
        assert_eq!(profile.description, "Original description");
        assert_eq!(profile.specialties, vec!["ui".to_string()]);
        assert!(profile.skill_set_ids.is_empty());
        assert_eq!(profile.model, AgentModelSelection::default());
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed agent save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_agent_restores_existing_agent_when_improvement_history_fails() {
        let root = temp_test_dir("agent-improvement-rollback");
        init_project(&root).expect("init project");
        add_agent_profile(
            &root,
            AddAgentProfileInput {
                id: "ui-worker",
                display_name: "UI Worker",
                runtime: "codex-local",
                adapter: "process-codex-cli",
                role: "worker",
                working_dir: ".",
                description: "Original description",
                specialties: vec!["ui".to_string()],
                skill_set_ids: Vec::new(),
                domain_ids: Vec::new(),
                artifact_type_ids: Vec::new(),
                mcp_connection_ids: Vec::new(),
                managed_by: None,
                model: AgentModelSelection::default(),
                external: ExternalAgentBinding::default(),
            },
        )
        .expect("agent should be added");
        let layout = ProjectLayout::new(&root);
        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become readonly");

        let result = save_agent(SaveAgentRequest {
            root: Some(root_to_string(&root)),
            id: "ui-worker".to_string(),
            display_name: "Should Not Apply".to_string(),
            avatar: Some("changed.svg".to_string()),
            role: "reviewer".to_string(),
            tool_kind: "codex_cli".to_string(),
            model: Some("gpt-5-codex".to_string()),
            model_provider: Some("openai".to_string()),
            model_base_url: None,
            description: Some("Changed description".to_string()),
            prompt: Some("Changed prompt".to_string()),
            specialties: vec!["changed".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            skill_set_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            improvement_proposal_id: Some("agent-prompt-ui-worker".to_string()),
            improvement_kind: Some("エージェント".to_string()),
            improvement_title: Some("UI Worker の改善".to_string()),
            improvement_target_label: Some("エージェント / UI Worker".to_string()),
            improvement_summary: Some("改善履歴失敗のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });

        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become writable");

        assert!(
            result.is_err(),
            "readonly project config should reject improvement history write"
        );
        let profile = get_agent_profile(&root, "ui-worker").expect("agent should remain");
        assert_eq!(profile.display_name, "UI Worker");
        assert_eq!(profile.avatar, "");
        assert_eq!(profile.role, "worker");
        assert_eq!(profile.description, "Original description");
        assert_eq!(profile.specialties, vec!["ui".to_string()]);
        assert_eq!(profile.model, AgentModelSelection::default());
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed agent improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn save_agent_removes_new_agent_when_improvement_history_fails() {
        let root = temp_test_dir("agent-new-improvement-rollback");
        init_project(&root).expect("init project");
        let layout = ProjectLayout::new(&root);
        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become readonly");

        let result = save_agent(SaveAgentRequest {
            root: Some(root_to_string(&root)),
            id: "ui-worker".to_string(),
            display_name: "UI Worker".to_string(),
            avatar: Some("worker.svg".to_string()),
            role: "worker".to_string(),
            tool_kind: "codex_cli".to_string(),
            model: Some("gpt-5-codex".to_string()),
            model_provider: Some("openai".to_string()),
            model_base_url: None,
            description: Some("UI work".to_string()),
            prompt: Some("Do UI work".to_string()),
            specialties: vec!["ui".to_string()],
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            skill_set_ids: Vec::new(),
            mcp_connection_ids: Vec::new(),
            improvement_proposal_id: Some("agent-prompt-ui-worker".to_string()),
            improvement_kind: Some("エージェント".to_string()),
            improvement_title: Some("UI Worker の改善".to_string()),
            improvement_target_label: Some("エージェント / UI Worker".to_string()),
            improvement_summary: Some("改善履歴失敗のテスト".to_string()),
            improvement_evidence: Some("テスト".to_string()),
        });

        let mut permissions = fs::metadata(&layout.config_path)
            .expect("config metadata")
            .permissions();
        permissions.set_readonly(false);
        fs::set_permissions(&layout.config_path, permissions)
            .expect("config should become writable");

        assert!(
            result.is_err(),
            "readonly project config should reject improvement history write"
        );
        assert!(
            get_agent_profile(&root, "ui-worker").is_err(),
            "new agent should be removed during rollback"
        );
        assert!(
            !layout.agents_dir.join("ui-worker.toml").exists(),
            "new agent file should be removed during rollback"
        );
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed agent improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn create_work_preserves_confirmed_domain_and_artifact_type() {
        let root = temp_test_dir("create-work-domain-artifact");
        init_project(&root).expect("init project");

        let detail = create_work(CreateWorkRequest {
            root: Some(root_to_string(&root)),
            description: "README のセットアップ手順を更新して".to_string(),
            project: Some("nagare".to_string()),
            domain_id: Some("general".to_string()),
            artifact_type_id: Some("general".to_string()),
            workflow_mode: Some("confirm_first".to_string()),
            approval_policy: Some("manual_final_approval".to_string()),
        })
        .expect("work should be created");

        assert_eq!(detail.domain_id, "general");
        assert_eq!(detail.artifact_type_id, "general");
        let snapshot = get_work_item_snapshot(&root, &detail.item.id).expect("work snapshot");
        assert_eq!(snapshot.item.domain_id.as_deref(), Some("general"));
        assert_eq!(snapshot.item.artifact_type_id.as_deref(), Some("general"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_work_removes_work_item_from_desktop_state() {
        let root = temp_test_dir("delete-work-item");
        init_project(&root).expect("init project");
        let detail = create_work(CreateWorkRequest {
            root: Some(root_to_string(&root)),
            description: "README のセットアップ手順を更新して".to_string(),
            project: Some("nagare".to_string()),
            domain_id: Some("general".to_string()),
            artifact_type_id: Some("general".to_string()),
            workflow_mode: Some("confirm_first".to_string()),
            approval_policy: Some("manual_final_approval".to_string()),
        })
        .expect("work should be created");

        let state = delete_work(WorkActionRequest {
            root: Some(root_to_string(&root)),
            id: detail.item.id,
            prompt: None,
            dev_command: None,
            dispatch_dev_command: None,
            review_dev_command: None,
            synthesis_dev_command: None,
            max_steps: None,
            auto_recover: None,
        })
        .expect("work should be deleted");

        assert!(state.work_items.is_empty());
        assert_eq!(state.project.expect("project view").work_count, 0);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn work_detail_view_exposes_result_review_and_trace_contract() {
        let root = temp_test_dir("work-detail-contract");
        init_project(&root).expect("init project");
        let snapshot = sample_completed_snapshot();
        write_trace_records(&root, &snapshot.item.id);

        let detail = work_detail_view(&root, snapshot);

        assert_eq!(detail.item.status_kind, "done");
        assert_eq!(detail.item.status_label, "完了");
        assert_eq!(detail.item.next_action, "操作不要");
        assert!(
            detail
                .item
                .result_summary
                .contains("README のセットアップ手順を整理しました。")
        );
        assert!(detail.item.result_summary.contains("評価 92 / 100"));
        assert_eq!(detail.answer, "README のセットアップ手順を整理しました。");
        assert_eq!(detail.artifacts.len(), 1);
        assert_eq!(detail.artifacts[0].title, "README.md");
        let review = detail.review.as_ref().expect("review");
        assert_eq!(review.score_label, "92 / 100");
        assert!(review.summary.contains("手順と確認方法"));
        assert_eq!(review.items.len(), 2);
        assert!(review.items.iter().any(|item| item.item == "目的への適合"
            && item.score_label == "46/50"
            && item.evidence.contains("依頼範囲")));
        assert_eq!(detail.steps.len(), 3);
        assert!(
            detail
                .steps
                .iter()
                .any(|step| step.title == "整理・担当決定" && step.actor == "オーガナイザー")
        );
        assert!(
            detail
                .steps
                .iter()
                .any(|step| step.title == "作業実行" && step.actor == "Writer")
        );
        assert!(detail.steps.iter().any(|step| {
            step.title == "レビュー" && step.actor == "Reviewer" && step.output == "92/100"
        }));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn zero_max_review_trace_does_not_surface_zero_score() {
        let payload = serde_json::json!({
            "summary": "100/100。質問に対して、二番目に高い山名・標高・所在地を簡潔に回答した。",
            "recommendation": "approve",
            "total_score": 0,
            "max_score": 0,
            "item_verdicts": []
        });

        assert_eq!(review_score_summary(&payload), None);
        assert_eq!(review_output_summary(&payload), "採用を推奨");
        assert_eq!(review_rationale_summary(&payload), "");
    }

    #[test]
    fn organizer_summary_trace_renders_as_final_summary_step() {
        let record = TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "organizer_summary".to_string(),
            work_id: "work_1".to_string(),
            seq: 4,
            at: "2026-07-05T00:03:00+09:00".to_string(),
            payload: serde_json::json!({
                "step_kind": "synthesis",
                "agent": { "id": "supervisor", "name": "Organizer", "role": "organizer" },
                "status": "completed",
                "inputs": { "summary": "複数ワーカーの結果とレビュー結果" },
                "actions_summary": "依頼者向けに最終回答を統合しました。",
                "answer": "最終回答です。",
                "knowledge_refs": [{ "id": "general" }]
            }),
        };

        let step = trace_step_view(&record);

        assert_eq!(step.title, "オーガナイザーまとめ");
        assert_eq!(step.actor, "Organizer");
        assert_eq!(step.summary, "依頼者向けに最終回答を統合しました。");
        assert_eq!(step.input, "複数ワーカーの結果とレビュー結果");
        assert_eq!(step.output, "最終回答です。");
        assert_eq!(step.knowledge_refs, vec!["general".to_string()]);
    }

    fn write_trace_records(root: &Path, work_id: &str) {
        let trace_path = root
            .join(".nagare")
            .join("works")
            .join(work_id)
            .join("trace.jsonl");
        fs::create_dir_all(trace_path.parent().expect("trace parent")).expect("trace dir");
        let records = [
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "organizer_decision".to_string(),
                work_id: work_id.to_string(),
                seq: 1,
                at: "2026-07-05T00:00:30+09:00".to_string(),
                payload: serde_json::json!({
                    "step_no": 1,
                    "step_kind": "intake",
                    "agent": { "id": "dispatcher", "name": "オーガナイザー", "role": "organizer" },
                    "status": "completed",
                    "knowledge_refs": [{ "id": "general" }],
                    "interpreted_request": "README のセットアップ手順更新として整理しました。",
                    "domain_id": "general",
                    "artifact_type_id": "docs",
                    "plan": [{ "step_no": 2, "step_kind": "create", "agent_id": "writer" }],
                    "assignments": [{
                        "step_no": 2,
                        "agent_id": "writer",
                        "rationale": "ドキュメント作成が得意な Writer を選びました。"
                    }]
                }),
            },
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "worker_output".to_string(),
                work_id: work_id.to_string(),
                seq: 2,
                at: "2026-07-05T00:01:30+09:00".to_string(),
                payload: serde_json::json!({
                    "step_no": 2,
                    "step_kind": "create",
                    "agent": { "id": "writer", "name": "Writer", "role": "worker" },
                    "status": "completed",
                    "knowledge_refs": [{ "id": "docs-style" }],
                    "inputs": { "summary": "README のセットアップ節を更新する" },
                    "actions_summary": "README のセットアップ手順を整理しました。",
                    "artifacts": [{ "path": "README.md" }],
                    "answer": "README のセットアップ手順を整理しました。"
                }),
            },
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "reviewer_verdict".to_string(),
                work_id: work_id.to_string(),
                seq: 3,
                at: "2026-07-05T00:02:00+09:00".to_string(),
                payload: serde_json::json!({
                    "step_no": 3,
                    "step_kind": "review",
                    "agent": { "id": "reviewer", "name": "Reviewer", "role": "reviewer" },
                    "status": "completed",
                    "target_artifacts": ["README.md"],
                    "item_verdicts": [
                        {
                            "item": "目的への適合",
                            "verdict": "passed",
                            "evidence": "依頼範囲のセットアップ手順に集中しています。",
                            "points": 46,
                            "max_points": 50
                        },
                        {
                            "item": "読みやすさ",
                            "verdict": "passed",
                            "evidence": "手順が短く整理されています。",
                            "points": 46,
                            "max_points": 50
                        }
                    ],
                    "total_score": 92,
                    "max_score": 100,
                    "recommendation": "採用できます",
                    "summary": "手順と確認方法が揃っています。"
                }),
            },
        ];
        let raw = records
            .iter()
            .map(|record| serde_json::to_string(record).expect("serialize trace"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(trace_path, format!("{raw}\n")).expect("write trace");
    }

    fn sample_completed_snapshot() -> WorkItemSnapshot {
        let mut fields = BTreeMap::new();
        fields.insert(
            "summary".to_string(),
            vec!["README のセットアップ手順を整理しました。".to_string()],
        );
        WorkItemSnapshot {
            item: WorkItem {
                id: "work_1".to_string(),
                title: "README更新".to_string(),
                description: "READMEを更新して".to_string(),
                acceptance_criteria: Vec::new(),
                expected_artifacts: Vec::new(),
                work_folder: Some("nagare".to_string()),
                constraints: Vec::new(),
                domain_id: Some("general".to_string()),
                artifact_type_id: Some("docs".to_string()),
                domain_agent_policy: DomainAgentPolicy::AutoGeneralFallback,
                require_domain_agent: false,
                workflow_mode: WorkflowMode::ConfirmFirst,
                approval_policy: ApprovalPolicy::ManualFinalApproval,
                locale: "ja".to_string(),
                status: WorkItemStatus::Done,
                created_at: "2026-07-05T00:00:00+09:00".to_string(),
                updated_at: "2026-07-05T00:01:00+09:00".to_string(),
            },
            completion: WorkItemCompletion {
                state: "done".to_string(),
                blocking_reason: None,
                next_action: "done".to_string(),
                next_command_hint: None,
            },
            approval_gate: WorkItemApprovalGate {
                state: "done".to_string(),
                ready: false,
                latest_review_id: Some("review_1".to_string()),
                criteria_passed: 0,
                criteria_total: 0,
                artifact_count: 1,
                recovery_count: 0,
                blockers: Vec::new(),
                command_hint: None,
            },
            runs: Vec::new(),
            artifacts: vec![Artifact {
                id: "artifact_1".to_string(),
                work_item_id: "work_1".to_string(),
                agent_run_id: Some("run_worker".to_string()),
                artifact_type: "docs".to_string(),
                uri: "file:///README.md".to_string(),
                title: "README.md".to_string(),
                locale: "ja".to_string(),
                created_at: "2026-07-05T00:01:30+09:00".to_string(),
            }],
            execution_records: Vec::new(),
            evidence: Vec::new(),
            review_results: vec![ReviewResult {
                id: "review_1".to_string(),
                work_item_id: "work_1".to_string(),
                agent_run_id: "run_review".to_string(),
                agent_profile_id: "reviewer".to_string(),
                verdict: ReviewVerdict::Pass,
                summary: vec!["手順と確認方法が揃っています。".to_string()],
                findings: Vec::new(),
                requested_changes: Vec::new(),
                referenced_artifacts: vec!["artifact_1".to_string()],
                criteria_results: vec![
                    CriteriaReviewResult {
                        criterion: "目的への適合".to_string(),
                        status: CriteriaReviewStatus::Passed,
                        note: "依頼範囲を満たしています。".to_string(),
                    },
                    CriteriaReviewResult {
                        criterion: "読みやすさ".to_string(),
                        status: CriteriaReviewStatus::Passed,
                        note: "手順が短く整理されています。".to_string(),
                    },
                ],
                questions: Vec::new(),
                next_action: Some("approve".to_string()),
                execution_record_id: "exec_review".to_string(),
                locale: "ja".to_string(),
                created_at: "2026-07-05T00:02:00+09:00".to_string(),
            }],
            handoffs: Vec::new(),
            decisions: Vec::new(),
            human_feedback: Vec::new(),
            dispatch_plans: Vec::new(),
            recovery_plans: Vec::new(),
            workflow_decisions: Vec::new(),
            resolved_skill_contexts: Vec::new(),
            resolved_run_packets: Vec::new(),
            agent_outputs: vec![AgentOutputRecord {
                id: "output_1".to_string(),
                work_item_id: "work_1".to_string(),
                agent_run_id: "run_worker".to_string(),
                agent_profile_id: "worker".to_string(),
                purpose: AgentRunPurpose::Work,
                contract: "work".to_string(),
                instruction_pack: String::new(),
                parse_status: AgentOutputParseStatus::Parsed,
                fields,
                questions: Vec::new(),
                next_action: Some("review".to_string()),
                warnings: Vec::new(),
                execution_record_id: "exec_work".to_string(),
                locale: "ja".to_string(),
                created_at: "2026-07-05T00:01:30+09:00".to_string(),
            }],
            timeline: Vec::new(),
            history_steps: Vec::new(),
        }
    }

    #[test]
    fn work_list_result_summary_includes_answer_and_review_score() {
        let snapshot = sample_completed_snapshot();
        let trace_records = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: "work_1".to_string(),
            seq: 4,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "total_score": 92,
                "max_score": 100
            }),
        }];

        let summary = work_list_result_summary(&snapshot, &trace_records);

        assert!(summary.contains("README のセットアップ手順を整理しました。"));
        assert!(summary.contains("評価 92 / 100"));
        assert!(summary.contains("手順と確認方法が揃っています。"));
    }
}
