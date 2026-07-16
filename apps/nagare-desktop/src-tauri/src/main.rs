#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    AgentRunStatus, AgentToolKind, ApplyRecoveryPlanInput, ApprovalPolicy, Artifact, ArtifactType,
    CreateRecoveryPlanResult,
    CreateWorkItemInput, CriteriaReviewStatus, DeleteSkillPackageInput, DeleteSkillPackageResult, Domain,
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
    run_codex_cli_prompt,
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
    usage_count: usize,
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
    effective_capabilities: Vec<EffectiveCapabilityView>,
    prohibited_task_gate: Option<ProhibitedTaskGateView>,
    review: Option<ReviewView>,
    steps: Vec<StepView>,
}

#[derive(Serialize)]
struct EffectiveCapabilityView {
    purpose: String,
    agent_id: String,
    agent_label: String,
    skills: Vec<String>,
    mcp_connections: Vec<String>,
    allowed_skill_count: usize,
    disabled_skill_count: usize,
    scope_diagnostics: Vec<String>,
    skill_paths: Vec<String>,
}

#[derive(Serialize)]
struct ProhibitedTaskGateView {
    rules: Vec<String>,
    status: String,
    summary: String,
    evidence: Vec<String>,
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
    outcome: String,
    actor: String,
    summary: String,
    rationale: String,
    input: String,
    output: String,
    score_label: String,
    criteria_label: String,
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
    model_choices: Vec<String>,
}

#[derive(Serialize)]
struct InsightsView {
    review_count: usize,
    average_score_label: String,
    concern_count: usize,
    proposal_count: usize,
    signals: Vec<InsightSignalView>,
    agent_scores: Vec<AgentInsightView>,
    issue_matrix: Vec<InsightIssueView>,
    proposals: Vec<ImprovementProposalView>,
    applied_improvements: Vec<AppliedImprovementView>,
    recent_reviews: Vec<InsightReviewView>,
    prompt_comparisons: Vec<PromptComparisonView>,
}

#[derive(Clone, Serialize)]
struct InsightSignalView {
    id: String,
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    title: String,
    item_label: String,
    observation: String,
    history_assessment: String,
    scope: String,
    scope_detail: String,
    primary_cause_kind: String,
    primary_cause_label: String,
    confidence_label: String,
    proposal_ready: bool,
    proposal_status_label: String,
    proposal_kind: String,
    proposal_target_id: String,
    proposal_target_label: String,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    prompt_version_label: String,
    evidence: Vec<InsightSignalEvidenceView>,
    competing_causes: Vec<String>,
}

#[derive(Clone, Serialize)]
struct InsightSignalEvidenceView {
    work_id: String,
    stage: String,
    summary: String,
}

#[derive(Serialize)]
struct AgentInsightView {
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    activity_count: usize,
    recent_activity_label: String,
    review_count: usize,
    average_score: u8,
    average_score_label: String,
    status_label: String,
    top_issue: String,
}

#[derive(Clone, Serialize)]
struct InsightIssueView {
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    item: String,
    rate: u8,
    rate_label: String,
    occurrences: usize,
    suggestion_kind: String,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    prompt_version_label: String,
    assignment_mode: String,
    assignment_label: String,
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
    project_name: String,
    verdict: String,
    score_label: String,
    concerns: Vec<String>,
    items: Vec<InsightReviewItemView>,
}

#[derive(Serialize)]
struct InsightReviewItemView {
    item: String,
    score_label: String,
    concern: bool,
}

#[derive(Serialize)]
struct PromptComparisonView {
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    assignment_label: String,
    variants: Vec<PromptComparisonVariantView>,
}

#[derive(Serialize)]
struct PromptComparisonVariantView {
    prompt_version_label: String,
    review_count: usize,
    average_score: u8,
    average_score_label: String,
    work_refs: Vec<PromptComparisonWorkView>,
    items: Vec<PromptComparisonItemView>,
}

#[derive(Serialize)]
struct PromptComparisonWorkView {
    work_id: String,
    title: String,
    score_label: String,
}

#[derive(Serialize)]
struct PromptComparisonItemView {
    item: String,
    average_score: u8,
    score_label: String,
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
    constraints: Option<String>,
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

#[derive(Deserialize)]
struct ArtifactDefinitionDraftRequest {
    root: Option<String>,
    domain_id: String,
    artifact_name: String,
}

#[derive(Deserialize)]
struct ArtifactDefinitionRefineRequest {
    root: Option<String>,
    domain_id: String,
    artifact_name: String,
    description: String,
    knowledge: Vec<String>,
    rubric: String,
    #[serde(default)]
    dispatch_hints: Vec<String>,
    feedback: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ArtifactDefinitionDraftResponse {
    description: String,
    knowledge: Vec<String>,
    rubric: String,
    #[serde(default)]
    dispatch_hints: Vec<String>,
    #[serde(default)]
    coverage: Vec<ArtifactDefinitionCoverage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ArtifactDefinitionCoverage {
    dimension: String,
    applicability: String,
    knowledge_ids: Vec<String>,
    rubric_sections: Vec<String>,
    reason: String,
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
fn launch_root() -> Option<String> {
    env::var_os("NAGARE_ROOT")
        .map(PathBuf::from)
        .filter(|root| root.is_dir())
        .map(|root| root_to_string(&root))
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
    for agent_id in ["organizer", "worker", "reviewer"] {
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
                knowledge_version: None,
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
        return Err("成果物IDを入力してください。".to_string());
    }
    if display_name.is_empty() {
        return Err("成果物名を入力してください。".to_string());
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
                rubric_version: None,
                definition_version: None,
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
                    rubric_version: Some(artifact.rubric_version),
                    definition_version: Some(artifact.definition_version),
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
        return Err("削除する成果物IDが空です。".to_string());
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

#[tauri::command]
fn generate_artifact_definition(
    request: ArtifactDefinitionDraftRequest,
) -> Result<ArtifactDefinitionDraftResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let domain = artifact_definition_domain(&root, &request.domain_id)?;
    let artifact_name = request.artifact_name.trim();
    if artifact_name.is_empty() {
        return Err("成果物名を入力してください。".to_string());
    }
    let prompt = artifact_definition_prompt(artifact_name, &domain, None);
    generate_precise_artifact_definition(&root, &prompt)
}

#[tauri::command]
fn refine_artifact_definition(
    request: ArtifactDefinitionRefineRequest,
) -> Result<ArtifactDefinitionDraftResponse, String> {
    let root = resolve_desktop_root(request.root)?;
    let domain = artifact_definition_domain(&root, &request.domain_id)?;
    if request.feedback.trim().is_empty() {
        return Err("AIへの改善コメントを入力してください。".to_string());
    }
    let current = ArtifactDefinitionDraftResponse {
        description: request.description,
        knowledge: request.knowledge,
        rubric: request.rubric,
        dispatch_hints: request.dispatch_hints,
        coverage: Vec::new(),
    };
    let refinement = format!(
        "改善対象: 成果物定義全体\nユーザーコメント: {}\n現在の定義:\n{}\n説明・作成指示・評価基準の対応関係を崩さず、コメントに必要な変更だけを行ってください。作成指示を変えた場合は対応する評価基準も見直し、評価基準を変えた場合は対応する作成指示が判定可能か確認してください。変更不要な内容は維持してください。",
        request.feedback.trim(),
        serde_json::to_string_pretty(&current).map_err(|error| error.to_string())?
    );
    let prompt = artifact_definition_prompt(
        request.artifact_name.trim(),
        &domain,
        Some(refinement.as_str()),
    );
    generate_precise_artifact_definition(&root, &prompt)
}

fn generate_precise_artifact_definition(
    root: &Path,
    prompt: &str,
) -> Result<ArtifactDefinitionDraftResponse, String> {
    let model = artifact_definition_model(root);
    let raw = run_codex_cli_prompt(root, prompt, model.as_deref())
        .map_err(|error| error.to_string())?;
    match parse_artifact_definition_response(&raw) {
        Ok(definition) => Ok(definition),
        Err(validation_error) => {
            let repair_prompt = format!(
                r###"次の成果物定義はNagareの精密度検証に不合格でした。
元の要件と検証エラーを満たすように、情報を削らず、原子的な作成指示へ分解して修正してください。
JSON objectだけを返してください。

検証エラー:
{validation_error}

元の要件:
{prompt}

不合格だった出力:
{raw}"###
            );
            let repaired = run_codex_cli_prompt(root, &repair_prompt, model.as_deref())
                .map_err(|error| error.to_string())?;
            parse_artifact_definition_response(&repaired).map_err(|repair_error| {
                format!(
                    "AIが精密な成果物定義を生成できませんでした。初回: {validation_error} / 再生成: {repair_error}"
                )
            })
        }
    }
}

fn artifact_definition_domain(root: &Path, domain_id: &str) -> Result<Domain, String> {
    list_domains(root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|domain| domain.id == domain_id.trim())
        .ok_or_else(|| format!("ドメイン `{}` が見つかりません。", domain_id.trim()))
}

fn artifact_definition_model(root: &Path) -> Option<String> {
    let settings = get_nagare_agent_settings(root).ok()?;
    let organizer_id = settings
        .organizer_agent
        .as_deref()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or(settings.dispatch_agent.as_str());
    get_agent_profile(root, organizer_id).ok()?.model.model_ref()
}

fn artifact_definition_prompt(
    artifact_name: &str,
    domain: &Domain,
    refinement: Option<&str>,
) -> String {
    let shared_knowledge = domain.shared_knowledge.join("\n- ");
    let common_rubric = domain.common_rubric.join("\n- ");
    let dispatch_hints = domain.dispatch_hints.join("\n- ");
    format!(
        r###"あなたは成果物定義を設計する上級品質アーキテクトです。
ドメインの知識と成果物名から、作成エージェントとレビューエージェントが追加説明なしで使える、精密な成果物定義を作成してください。

成果物名: {artifact_name}
ドメイン: {} ({})
ドメイン説明: {}
共通知識:
- {shared_knowledge}
共通評価基準:
- {common_rubric}
振り分けヒント:
- {dispatch_hints}

網羅性の検査対象（coverageにこの13項目をすべて出力する）:
- 目的と利用者
- 範囲と前提
- 構成と責任
- 入力と出力
- 正確性と根拠
- 整合性と追跡可能性
- 正常系と異常系
- 境界条件と例外
- セキュリティとプライバシー
- アクセシビリティ
- 性能と信頼性
- 運用と保守性
- 検証と証跡
該当しない観点も省略せず、applicabilityを `非該当` として、非該当と判断できる具体的理由を書くこと。

出力条件:
- descriptionは成果物の目的、利用者、利用場面、完成状態を2〜4文で定義する。
- knowledgeは必要な情報単位を省略せず16〜40項目で出力する。項目数を減らすために独立した要求を結合しない。
- knowledgeの各項目は `[K01][カテゴリ] 対象: ... | 条件: ... | 要求: ... | 証跡: ... | 例外: ...` の1行形式とする。
- 1項目には1つの判定可能な要求だけを書く。対象、適用条件、要求結果、確認可能な証跡、例外または非該当条件を具体化する。
- 「適切に」「十分に」「必要に応じて」「考慮する」「など」だけで判定を利用者へ委ねない。使用する場合は直後に判定条件を列挙する。
- rubricは8〜12項目、合計100点のMarkdownとする。
- rubricの各項目は `## 項目名 (配点)` で始め、本文に `満点条件:`、`部分点条件:`、`重大な不足:`、`確認する証跡:` を必ず含める。
- 重大な不足は、その項目を0点または即時差し戻しにする観測可能な条件を書く。
- dispatch_hintsは、この成果物を担当できる専門性を2〜6項目で表す。
- coverageは上記13観点を1件ずつ出力し、applicabilityは `必須`、`条件付き`、`非該当` のいずれかとする。
- coverageのknowledge_idsには対応するK番号、rubric_sectionsには対応する評価項目名を入れる。必須・条件付きなら両方を空にしない。
- 出力前に、全knowledgeと全rubric項目がcoverageから少なくとも1回参照されていることを自己検査する。
{}

次のJSON objectだけを返してください。Markdownのコードフェンスや前後の説明は不要です。
{{
  "description": "...",
  "knowledge": ["[K01][目的] 対象: ... | 条件: ... | 要求: ... | 証跡: ... | 例外: ..."],
  "rubric": "## ... (配点)\n満点条件: ...\n部分点条件: ...\n重大な不足: ...\n確認する証跡: ...",
  "dispatch_hints": ["..."],
  "coverage": [
    {{
      "dimension": "目的と利用者",
      "applicability": "必須",
      "knowledge_ids": ["K01"],
      "rubric_sections": ["目的への適合"],
      "reason": "この成果物で該当する理由"
    }}
  ]
}}"###,
        domain.display_name,
        domain.id,
        domain.description,
        refinement.unwrap_or("")
    )
}

fn parse_artifact_definition_response(
    raw: &str,
) -> Result<ArtifactDefinitionDraftResponse, String> {
    let trimmed = raw.trim();
    let json = serde_json::from_str::<ArtifactDefinitionDraftResponse>(trimmed).or_else(|_| {
        let start = trimmed.find('{').ok_or_else(|| {
            serde_json::Error::io(std::io::Error::other("JSON objectが見つかりません"))
        })?;
        let end = trimmed.rfind('}').ok_or_else(|| {
            serde_json::Error::io(std::io::Error::other("JSON objectが閉じられていません"))
        })?;
        serde_json::from_str::<ArtifactDefinitionDraftResponse>(&trimmed[start..=end])
    })
    .map_err(|error| format!("AIの生成結果を読み取れませんでした: {error}"))?;
    if json.description.trim().is_empty() {
        return Err("AIの生成結果に成果物の説明がありません。".to_string());
    }
    validate_artifact_knowledge(&json.knowledge)?;
    let summary = validate_rubric_markdown(&json.rubric)?;
    if summary.item_count < 8 {
        return Err("AIの評価基準が十分な粒度ではありません。8項目以上必要です。".to_string());
    }
    for required in [
        "満点条件:",
        "部分点条件:",
        "重大な不足:",
        "確認する証跡:",
    ] {
        if json.rubric.matches(required).count() < summary.item_count {
            return Err(format!(
                "AIの評価基準では、各項目に `{required}` を記述する必要があります。"
            ));
        }
    }
    validate_artifact_coverage(&json)?;
    Ok(json)
}

const ARTIFACT_COVERAGE_DIMENSIONS: [&str; 13] = [
    "目的と利用者",
    "範囲と前提",
    "構成と責任",
    "入力と出力",
    "正確性と根拠",
    "整合性と追跡可能性",
    "正常系と異常系",
    "境界条件と例外",
    "セキュリティとプライバシー",
    "アクセシビリティ",
    "性能と信頼性",
    "運用と保守性",
    "検証と証跡",
];

fn validate_artifact_knowledge(knowledge: &[String]) -> Result<(), String> {
    if !(16..=40).contains(&knowledge.len()) {
        return Err(
            "AIの作成指示は、原子的な情報単位へ分解した16〜40項目である必要があります。"
                .to_string(),
        );
    }
    for (index, item) in knowledge.iter().enumerate() {
        let expected_id = format!("K{:02}", index + 1);
        if !item.starts_with(&format!("[{expected_id}][")) {
            return Err(format!(
                "作成指示{}は `{expected_id}` から始まり、順番に採番する必要があります。",
                index + 1
            ));
        }
        for required in ["対象:", "条件:", "要求:", "証跡:", "例外:"] {
            if !item.contains(required) {
                return Err(format!(
                    "作成指示 `{expected_id}` に `{required}` がありません。"
                ));
            }
        }
        if item.chars().count() < 60 {
            return Err(format!(
                "作成指示 `{expected_id}` が短すぎます。対象・条件・要求・証跡・例外を具体化してください。"
            ));
        }
    }
    Ok(())
}

fn validate_artifact_coverage(
    definition: &ArtifactDefinitionDraftResponse,
) -> Result<(), String> {
    let knowledge_ids = definition
        .knowledge
        .iter()
        .filter_map(|item| item.strip_prefix('[')?.split(']').next())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let rubric_sections = definition
        .rubric
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .filter_map(|heading| {
            heading
                .rsplit_once(" (")
                .map(|(name, _)| name.trim().to_string())
        })
        .collect::<BTreeSet<_>>();
    let dimensions = definition
        .coverage
        .iter()
        .map(|entry| entry.dimension.as_str())
        .collect::<BTreeSet<_>>();
    for required in ARTIFACT_COVERAGE_DIMENSIONS {
        if !dimensions.contains(required) {
            return Err(format!("AIの網羅表に `{required}` がありません。"));
        }
    }
    if dimensions.len() != ARTIFACT_COVERAGE_DIMENSIONS.len() {
        return Err("AIの網羅表に重複または未定義の観点があります。".to_string());
    }
    let mut covered_knowledge = BTreeSet::new();
    let mut covered_rubrics = BTreeSet::new();
    for entry in &definition.coverage {
        if !["必須", "条件付き", "非該当"].contains(&entry.applicability.as_str()) {
            return Err(format!(
                "網羅表 `{}` の該当性は、必須・条件付き・非該当のいずれかにしてください。",
                entry.dimension
            ));
        }
        if entry.reason.trim().chars().count() < 10 {
            return Err(format!(
                "網羅表 `{}` の該当性理由を具体化してください。",
                entry.dimension
            ));
        }
        if entry.applicability != "非該当"
            && (entry.knowledge_ids.is_empty() || entry.rubric_sections.is_empty())
        {
            return Err(format!(
                "網羅表 `{}` には作成指示と評価項目の両方を対応付けてください。",
                entry.dimension
            ));
        }
        for id in &entry.knowledge_ids {
            if !knowledge_ids.contains(id) {
                return Err(format!(
                    "網羅表 `{}` が存在しない作成指示 `{id}` を参照しています。",
                    entry.dimension
                ));
            }
            covered_knowledge.insert(id.clone());
        }
        for section in &entry.rubric_sections {
            if !rubric_sections.contains(section) {
                return Err(format!(
                    "網羅表 `{}` が存在しない評価項目 `{section}` を参照しています。",
                    entry.dimension
                ));
            }
            covered_rubrics.insert(section.clone());
        }
    }
    if covered_knowledge != knowledge_ids {
        return Err("網羅表から参照されていない作成指示があります。".to_string());
    }
    if covered_rubrics != rubric_sections {
        return Err("網羅表から参照されていない評価項目があります。".to_string());
    }
    Ok(())
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
            constraints: text_lines(request.constraints.as_deref()),
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
            launch_root,
            app_state,
            choose_project_folder,
            choose_agent_avatar_file,
            initialize_project,
            initialize_project_with_runtime,
            refresh_runtime_status,
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
            generate_artifact_definition,
            refine_artifact_definition,
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
        let mut agent_usage_counts = BTreeMap::<String, usize>::new();
        let work_items = list_work_items(&root)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|item| {
                let snapshot = get_work_item_snapshot(&root, &item.id).ok();
                let trace_records = snapshot
                    .as_ref()
                    .map(|snapshot| list_work_trace(&root, &snapshot.item.id).unwrap_or_default())
                    .unwrap_or_default();
                accumulate_agent_usage_counts(&mut agent_usage_counts, &trace_records);
                if let Some(snapshot) = snapshot.as_ref() {
                    snapshots.push(snapshot.clone());
                }
                work_list_item_with_trace(&item, snapshot.as_ref(), &trace_records)
            })
            .collect::<Vec<_>>();
        let insights = insights_view(
            &root,
            &snapshots,
            &agent_profiles,
            &domains,
            &artifact_types,
        );
        let project = project_view(
            &root,
            &work_items,
            &agent_profiles,
            &domains,
            &artifact_types,
        )?;
        (
            work_items,
            agent_profiles
                .iter()
                .map(|profile| {
                    agent_view(
                        profile,
                        agent_usage_counts.get(&profile.id).copied().unwrap_or(0),
                    )
                })
                .collect(),
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

fn accumulate_agent_usage_counts(
    counts: &mut BTreeMap<String, usize>,
    trace_records: &[TraceRecord],
) {
    for record in trace_records.iter().filter(|record| {
        matches!(
            record.record.as_str(),
            "organizer_decision" | "worker_output" | "reviewer_verdict" | "organizer_summary"
        )
    }) {
        if let Some(agent_id) = value_path_str(&record.payload, &["agent", "id"])
            .filter(|agent_id| !agent_id.trim().is_empty())
        {
            *counts.entry(agent_id).or_default() += 1;
        }
    }
}

fn status_counts(work_items: &[WorkListItem]) -> Vec<StatusCountView> {
    [
        ("要対応・質問", "question"),
        ("要対応・確認", "review"),
        ("要対応", "recover"),
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
    project_name: String,
    activity_count: usize,
    dispatch_count: usize,
    work_count: usize,
    review_activity_count: usize,
    synthesis_count: usize,
    supervision_count: usize,
    failed_count: usize,
    score_events: Vec<(String, u8)>,
    score_sum: u32,
    review_count: usize,
    scored_review_count: usize,
    issue_counts: BTreeMap<String, usize>,
}

struct IssueAccum {
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    item: String,
    score_sum: u32,
    occurrences: usize,
    concern_count: usize,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    prompt_version_label: String,
    direct_assignment_count: usize,
    organizer_assignment_count: usize,
    unknown_assignment_count: usize,
}

#[derive(Clone)]
struct InsightEpisode {
    work_id: String,
    title: String,
    project_name: String,
    worker_id: String,
    worker_name: String,
    reviewer_id: String,
    reviewer_name: String,
    organizer_id: String,
    organizer_name: String,
    assignment_mode: String,
    assignment_label: String,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    prompt_version_label: String,
    reviewer_prompt_version_label: String,
    organizer_prompt_version_label: String,
    review_verdict: String,
    review_items: BTreeMap<String, u8>,
    human_decision_type: String,
    human_decision_rationale: String,
    worker_questions: Vec<String>,
    handoff_summaries: Vec<String>,
    recovery_summaries: Vec<String>,
    organizer_summary_count: usize,
}

struct InsightScope {
    domain_id: String,
    domain_name: String,
    artifact_type_id: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    prompt_version_label: String,
}

fn agent_activity_entry<'a>(
    agent_accums: &'a mut BTreeMap<(String, String), AgentInsightAccum>,
    agents: &[AgentProfile],
    agent_id: &str,
    project_name: &str,
    fallback_name: Option<&str>,
    fallback_role: &str,
) -> &'a mut AgentInsightAccum {
    let profile = agents.iter().find(|agent| agent.id == agent_id);
    let agent_name = profile
        .map(|agent| agent.display_name.clone())
        .or_else(|| fallback_name.filter(|name| !name.trim().is_empty()).map(str::to_string))
        .unwrap_or_else(|| agent_id.to_string());
    let role = profile
        .map(|agent| agent.role.clone())
        .filter(|role| !role.trim().is_empty())
        .unwrap_or_else(|| fallback_role.to_string());
    agent_accums
        .entry((agent_id.to_string(), project_name.to_string()))
        .or_insert_with(|| AgentInsightAccum {
            agent_id: agent_id.to_string(),
            agent_name,
            role,
            project_name: project_name.to_string(),
            ..AgentInsightAccum::default()
        })
}

fn record_agent_activity(agent: &mut AgentInsightAccum, kind: &str, failed: bool) {
    agent.activity_count += 1;
    match kind {
        "organizer_decision" => agent.dispatch_count += 1,
        "worker_output" => agent.work_count += 1,
        "reviewer_verdict" => agent.review_activity_count += 1,
        "organizer_summary" => agent.synthesis_count += 1,
        "workflow_supervision" => agent.supervision_count += 1,
        _ => {}
    }
    if failed {
        agent.failed_count += 1;
    }
}

fn accumulate_agent_activities(
    agent_accums: &mut BTreeMap<(String, String), AgentInsightAccum>,
    snapshot: &WorkItemSnapshot,
    trace_records: &[TraceRecord],
    agents: &[AgentProfile],
    project_name: &str,
) {
    let mut traced_kinds = BTreeSet::new();
    for record in trace_records.iter().filter(|record| {
        matches!(
            record.record.as_str(),
            "organizer_decision" | "worker_output" | "reviewer_verdict" | "organizer_summary"
        )
    }) {
        let Some(agent_id) = value_path_str(&record.payload, &["agent", "id"])
            .filter(|agent_id| !agent_id.trim().is_empty())
        else {
            continue;
        };
        let fallback_role = match record.record.as_str() {
            "organizer_decision" | "organizer_summary" => "organizer",
            "reviewer_verdict" => "reviewer",
            _ => "worker",
        };
        let agent = agent_activity_entry(
            agent_accums,
            agents,
            &agent_id,
            project_name,
            value_path_str(&record.payload, &["agent", "name"]).as_deref(),
            value_path_str(&record.payload, &["agent", "role"])
                .as_deref()
                .unwrap_or(fallback_role),
        );
        record_agent_activity(
            agent,
            &record.record,
            value_path_str(&record.payload, &["status"])
                .is_some_and(|status| status == "failed"),
        );
        traced_kinds.insert(record.record.clone());
    }

    if !traced_kinds.contains("organizer_decision") {
        for plan in &snapshot.dispatch_plans {
            let agent = agent_activity_entry(
                agent_accums,
                agents,
                &plan.dispatch_agent_profile_id,
                project_name,
                None,
                "organizer",
            );
            record_agent_activity(agent, "organizer_decision", false);
        }
    }
    if !traced_kinds.contains("worker_output") {
        for output in snapshot
            .agent_outputs
            .iter()
            .filter(|output| output.purpose == AgentRunPurpose::Work)
        {
            let agent = agent_activity_entry(
                agent_accums,
                agents,
                &output.agent_profile_id,
                project_name,
                None,
                "worker",
            );
            record_agent_activity(agent, "worker_output", false);
        }
    }
    if !traced_kinds.contains("reviewer_verdict") {
        for review in &snapshot.review_results {
            let agent = agent_activity_entry(
                agent_accums,
                agents,
                &review.agent_profile_id,
                project_name,
                None,
                "reviewer",
            );
            record_agent_activity(agent, "reviewer_verdict", false);
        }
    }
    if !traced_kinds.contains("organizer_summary") {
        for output in snapshot
            .agent_outputs
            .iter()
            .filter(|output| output.purpose == AgentRunPurpose::Synthesis)
        {
            let agent = agent_activity_entry(
                agent_accums,
                agents,
                &output.agent_profile_id,
                project_name,
                None,
                "organizer",
            );
            record_agent_activity(agent, "organizer_summary", false);
        }
    }

    for run in &snapshot.runs {
        let kind = match run.purpose {
            AgentRunPurpose::DispatchPreview => "organizer_decision",
            AgentRunPurpose::Work => "worker_output",
            AgentRunPurpose::Review => "reviewer_verdict",
            AgentRunPurpose::Synthesis => "organizer_summary",
            AgentRunPurpose::WorkflowSupervision => "workflow_supervision",
        };
        let represented = match run.purpose {
            AgentRunPurpose::DispatchPreview => snapshot
                .dispatch_plans
                .iter()
                .any(|plan| plan.agent_run_id == run.id),
            AgentRunPurpose::Review => snapshot
                .review_results
                .iter()
                .any(|review| review.agent_run_id == run.id),
            AgentRunPurpose::Work | AgentRunPurpose::Synthesis => snapshot
                .agent_outputs
                .iter()
                .any(|output| output.agent_run_id == run.id),
            AgentRunPurpose::WorkflowSupervision => false,
        } || traced_kinds.contains(kind);
        if run.status == AgentRunStatus::Failed || !represented {
            let fallback_role = match run.purpose {
                AgentRunPurpose::Review => "reviewer",
                AgentRunPurpose::Work => "worker",
                _ => "organizer",
            };
            let agent = agent_activity_entry(
                agent_accums,
                agents,
                &run.agent_profile_id,
                project_name,
                None,
                fallback_role,
            );
            if represented {
                if run.status == AgentRunStatus::Failed && !traced_kinds.contains(kind) {
                    agent.failed_count += 1;
                }
            } else {
                record_agent_activity(agent, kind, run.status == AgentRunStatus::Failed);
            }
        }
    }
}

fn insight_role_kind(role: &str) -> &'static str {
    let role = role.to_ascii_lowercase();
    if role.contains("organizer") || role.contains("dispatch") || role.contains("supervisor") {
        "organizer"
    } else if role.contains("review") {
        "reviewer"
    } else {
        "worker"
    }
}

fn recent_agent_activity_label(agent: &mut AgentInsightAccum) -> String {
    agent.score_events.sort_by(|left, right| left.0.cmp(&right.0));
    let score_flow = agent
        .score_events
        .iter()
        .rev()
        .take(3)
        .map(|(_, score)| format!("{score}点"))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" → ");
    let mut label = match insight_role_kind(&agent.role) {
        "reviewer" if !score_flow.is_empty() => format!("付与した点 {score_flow}"),
        "reviewer" if agent.review_activity_count > 0 => {
            format!("レビュー {}件 · 得点未記録", agent.review_activity_count)
        }
        "organizer" => {
            let mut parts = Vec::new();
            if agent.dispatch_count > 0 {
                parts.push(format!("割り当て {}件", agent.dispatch_count));
            }
            if agent.synthesis_count > 0 {
                parts.push(format!("まとめ {}件", agent.synthesis_count));
            }
            if agent.supervision_count > 0 {
                parts.push(format!("進行判断 {}件", agent.supervision_count));
            }
            if parts.is_empty() {
                "記録不足".to_string()
            } else {
                parts.join(" · ")
            }
        }
        _ if !score_flow.is_empty() => format!("評価された点 {score_flow}"),
        _ if agent.work_count > 0 => format!("作業 {}件 · 得点未記録", agent.work_count),
        _ => "記録不足".to_string(),
    };
    if agent.failed_count > 0 {
        label.push_str(&format!(" · 失敗 {}件", agent.failed_count));
    }
    label
}

#[allow(clippy::too_many_arguments)]
fn insight_episode(
    snapshot: &WorkItemSnapshot,
    trace_records: &[TraceRecord],
    agents: &[AgentProfile],
    project_name: &str,
    worker_id: &str,
    worker_name: &str,
    review: &ReviewResult,
    scope: &InsightScope,
    assignment_mode: &str,
    review_items: &[(String, u8, bool)],
) -> InsightEpisode {
    let organizer_trace = trace_records
        .iter()
        .rev()
        .find(|record| {
            record.record == "organizer_decision"
                && record.at.as_str() <= review.created_at.as_str()
        });
    let organizer_id = organizer_trace
        .and_then(|record| value_path_str(&record.payload, &["agent", "id"]))
        .or_else(|| {
            snapshot
                .dispatch_plans
                .iter()
                .rev()
                .filter(|plan| plan.created_at.as_str() <= review.created_at.as_str())
                .map(|plan| plan.dispatch_agent_profile_id.clone())
                .find(|id| !id.trim().is_empty())
        })
        .unwrap_or_default();
    let organizer_name = organizer_trace
        .and_then(|record| value_path_str(&record.payload, &["agent", "name"]))
        .or_else(|| agent_name(agents, &organizer_id))
        .unwrap_or_else(|| {
            if organizer_id.is_empty() {
                "オーガナイザー未記録".to_string()
            } else {
                organizer_id.clone()
            }
        });
    let reviewer_name = agent_name(agents, &review.agent_profile_id)
        .unwrap_or_else(|| review.agent_profile_id.clone());
    let work_output = snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| {
            output.purpose == AgentRunPurpose::Work
                && output.agent_profile_id == worker_id
                && output.created_at.as_str() <= review.created_at.as_str()
        });
    let episode_started_at = work_output
        .map(|output| output.created_at.as_str())
        .or_else(|| {
            snapshot
                .dispatch_plans
                .iter()
                .rev()
                .filter(|plan| plan.created_at.as_str() <= review.created_at.as_str())
                .map(|plan| plan.created_at.as_str())
                .next()
        })
        .unwrap_or(snapshot.item.created_at.as_str());
    let human_decision = snapshot
        .decisions
        .iter()
        .rev()
        .find(|decision| decision.created_at.as_str() >= review.created_at.as_str());
    let worker_questions = snapshot
        .agent_outputs
        .iter()
        .filter(|output| {
            output.purpose == AgentRunPurpose::Work
                && output.agent_profile_id == worker_id
                && output.created_at.as_str() >= episode_started_at
                && output.created_at.as_str() <= review.created_at.as_str()
        })
        .flat_map(|output| output.questions.clone())
        .filter(|question| meaningful_text(question))
        .collect::<Vec<_>>();
    let handoff_summaries = snapshot
        .handoffs
        .iter()
        .filter(|handoff| {
            handoff.created_at.as_str() >= episode_started_at
                && (handoff.from_agent_profile == worker_id
                    || handoff.to_agent_profile == worker_id)
        })
        .map(|handoff| {
            if meaningful_text(&handoff.summary) {
                handoff.summary.clone()
            } else {
                handoff.reason.clone()
            }
        })
        .collect::<Vec<_>>();
    let recovery_summaries = snapshot
        .recovery_plans
        .iter()
        .filter(|plan| {
            plan.created_at.as_str() >= episode_started_at
                && plan.status != RecoveryPlanStatus::Superseded
                && plan
                    .target_agent_profile_id
                    .as_deref()
                    .is_none_or(|target| target == worker_id)
        })
        .map(|plan| format!("{}: {}", plan.action, plan.reason))
        .collect::<Vec<_>>();
    let organizer_summary_count = trace_records
        .iter()
        .filter(|record| {
            record.record == "organizer_summary"
                && record.at.as_str() >= review.created_at.as_str()
        })
        .count()
        .max(
            snapshot
                .agent_outputs
                .iter()
                .filter(|output| {
                    output.purpose == AgentRunPurpose::Synthesis
                        && output.created_at.as_str() >= review.created_at.as_str()
                })
                .count(),
        );
    let prompt_version_for = |agent_id: &str, purpose: AgentRunPurpose| {
        snapshot
            .resolved_run_packets
            .iter()
            .rev()
            .find(|packet| packet.purpose == purpose && packet.agent_profile_id == agent_id)
            .map(|packet| packet.prompt_version.trim())
            .filter(|version| !version.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "実行時版未記録".to_string())
    };
    let reviewer_prompt_version_label =
        prompt_version_for(&review.agent_profile_id, AgentRunPurpose::Review);
    let organizer_prompt_version_label = if organizer_id.is_empty() {
        "実行時版未記録".to_string()
    } else {
        prompt_version_for(&organizer_id, AgentRunPurpose::DispatchPreview)
    };

    InsightEpisode {
        work_id: snapshot.item.id.clone(),
        title: snapshot.item.title.clone(),
        project_name: project_name.to_string(),
        worker_id: worker_id.to_string(),
        worker_name: worker_name.to_string(),
        reviewer_id: review.agent_profile_id.clone(),
        reviewer_name,
        organizer_id,
        organizer_name,
        assignment_mode: assignment_mode.to_string(),
        assignment_label: match assignment_mode {
            "direct" => "担当を明示指定".to_string(),
            "organizer" => "オーガナイザーが割り当て".to_string(),
            _ => "割り当て元は記録不足".to_string(),
        },
        domain_name: scope.domain_name.clone(),
        artifact_type_name: scope.artifact_type_name.clone(),
        rubric_version_label: scope.rubric_version_label.clone(),
        knowledge_version_label: scope.knowledge_version_label.clone(),
        prompt_version_label: scope.prompt_version_label.clone(),
        reviewer_prompt_version_label,
        organizer_prompt_version_label,
        review_verdict: review.verdict.to_string(),
        review_items: review_items
            .iter()
            .map(|(item, score, _)| (item.clone(), *score))
            .collect(),
        human_decision_type: human_decision
            .map(|decision| decision.decision_type.clone())
            .unwrap_or_default(),
        human_decision_rationale: human_decision
            .map(|decision| decision.rationale.clone())
            .unwrap_or_default(),
        worker_questions,
        handoff_summaries,
        recovery_summaries,
        organizer_summary_count,
    }
}

struct PromptComparisonAccum {
    agent_id: String,
    agent_name: String,
    role: String,
    project_name: String,
    domain_name: String,
    artifact_type_name: String,
    rubric_version_label: String,
    knowledge_version_label: String,
    assignment_label: String,
    item_order: Vec<String>,
    variants: BTreeMap<String, PromptComparisonVariantAccum>,
}

#[derive(Default)]
struct PromptComparisonVariantAccum {
    score_sum: u32,
    review_count: usize,
    scored_review_count: usize,
    work_refs: Vec<(String, String, Option<u8>)>,
    item_scores: BTreeMap<String, (u32, usize)>,
}

fn empty_insights_view() -> InsightsView {
    InsightsView {
        review_count: 0,
        average_score_label: "-".to_string(),
        concern_count: 0,
        proposal_count: 0,
        signals: Vec::new(),
        agent_scores: Vec::new(),
        issue_matrix: Vec::new(),
        proposals: Vec::new(),
        applied_improvements: Vec::new(),
        recent_reviews: Vec::new(),
        prompt_comparisons: Vec::new(),
    }
}

fn insights_view(
    root: &Path,
    snapshots: &[WorkItemSnapshot],
    agents: &[AgentProfile],
    domains: &[Domain],
    artifact_types: &[ArtifactType],
) -> InsightsView {
    let mut agent_accums: BTreeMap<(String, String), AgentInsightAccum> = BTreeMap::new();
    let mut issue_accums: BTreeMap<
        (String, String, String, String, String, String, String, String),
        IssueAccum,
    > = BTreeMap::new();
    let mut recent_reviews = Vec::new();
    let mut episodes = Vec::new();
    let mut prompt_comparison_accums: BTreeMap<
        (String, String, String, String, String, String, String),
        PromptComparisonAccum,
    > = BTreeMap::new();
    let mut total_score = 0u32;
    let mut review_count = 0usize;
    let mut scored_review_count = 0usize;
    let mut concern_count = 0usize;
    let root_project_name = analysis_root_project_name(root);

    for snapshot in snapshots {
        let trace_records = list_work_trace(root, &snapshot.item.id).unwrap_or_default();
        let project_name = analysis_project_name(
            &root_project_name,
            snapshot.item.work_folder.as_deref(),
        );
        accumulate_agent_activities(
            &mut agent_accums,
            snapshot,
            &trace_records,
            agents,
            &project_name,
        );
        let Some(review) = snapshot.review_results.iter().rev().next() else {
            continue;
        };
        let agent_id = reviewed_agent_id(snapshot, &trace_records)
            .unwrap_or_else(|| review.agent_profile_id.clone());
        let (agent_name, role) = agent_label(agents, &agent_id);
        let scope = insight_scope(snapshot, &trace_records, agents, domains, artifact_types, &agent_id);
        let assignment_mode = insight_assignment_mode(snapshot, &agent_id);
        let score = review_score_percent(review, &trace_records);
        let concerns = review_concerns(review, &trace_records);
        let review_items = insight_review_items(review, &trace_records, score.unwrap_or_default());
        let review_item_views = review_items
            .iter()
            .map(|(item, item_score, is_concern)| InsightReviewItemView {
                item: item.clone(),
                score_label: format!("{item_score}%"),
                concern: *is_concern,
            })
            .collect::<Vec<_>>();
        episodes.push(insight_episode(
            snapshot,
            &trace_records,
            agents,
            &project_name,
            &agent_id,
            &agent_name,
            review,
            &scope,
            assignment_mode,
            &review_items,
        ));

        if recorded_prompt_version(&scope.prompt_version_label) {
            let assignment_label = match assignment_mode {
                "direct" => "担当を明示指定",
                "organizer" => "オーガナイザーが割り当て",
                _ => "割り当て元は記録不足",
            };
            let comparison = prompt_comparison_accums
                .entry((
                    agent_id.clone(),
                    project_name.clone(),
                    scope.domain_id.clone(),
                    scope.artifact_type_id.clone(),
                    scope.rubric_version_label.clone(),
                    scope.knowledge_version_label.clone(),
                    assignment_mode.to_string(),
                ))
                .or_insert_with(|| PromptComparisonAccum {
                    agent_id: agent_id.clone(),
                    agent_name: agent_name.clone(),
                    role: role.clone(),
                    project_name: project_name.clone(),
                    domain_name: scope.domain_name.clone(),
                    artifact_type_name: scope.artifact_type_name.clone(),
                    rubric_version_label: scope.rubric_version_label.clone(),
                    knowledge_version_label: scope.knowledge_version_label.clone(),
                    assignment_label: assignment_label.to_string(),
                    item_order: Vec::new(),
                    variants: BTreeMap::new(),
                });
            for (item, _, _) in &review_items {
                if !comparison.item_order.contains(item) {
                    comparison.item_order.push(item.clone());
                }
            }
            let variant = comparison
                .variants
                .entry(scope.prompt_version_label.clone())
                .or_default();
            variant.review_count += 1;
            if let Some(score) = score {
                variant.score_sum += u32::from(score);
                variant.scored_review_count += 1;
            }
            variant
                .work_refs
                .push((snapshot.item.id.clone(), snapshot.item.title.clone(), score));
            for (item, item_score, _) in &review_items {
                let item_accum = variant.item_scores.entry(item.clone()).or_default();
                item_accum.0 += u32::from(*item_score);
                item_accum.1 += 1;
            }
        }

        review_count += 1;
        concern_count += concerns.len();

        let agent = agent_activity_entry(
            &mut agent_accums,
            agents,
            &agent_id,
            &project_name,
            Some(&agent_name),
            &role,
        );
        agent.review_count += 1;
        if let Some(score) = score {
            total_score += u32::from(score);
            scored_review_count += 1;
            agent.score_sum += u32::from(score);
            agent.scored_review_count += 1;
            agent.score_events.push((review.created_at.clone(), score));
        }

        for (item, item_score, is_concern) in review_items {
            if is_concern {
                *agent.issue_counts.entry(item.clone()).or_insert(0) += 1;
            }
            let issue = issue_accums
                .entry((
                    agent_id.clone(),
                    project_name.clone(),
                    item.clone(),
                    scope.domain_id.clone(),
                    scope.artifact_type_id.clone(),
                    scope.rubric_version_label.clone(),
                    scope.knowledge_version_label.clone(),
                    scope.prompt_version_label.clone(),
                ))
                .or_insert_with(|| IssueAccum {
                    agent_id: agent_id.clone(),
                    agent_name: agent_name.clone(),
                    role: role.clone(),
                    project_name: project_name.clone(),
                    item: item.clone(),
                    score_sum: 0,
                    occurrences: 0,
                    concern_count: 0,
                    domain_name: scope.domain_name.clone(),
                    artifact_type_name: scope.artifact_type_name.clone(),
                    rubric_version_label: scope.rubric_version_label.clone(),
                    knowledge_version_label: scope.knowledge_version_label.clone(),
                    prompt_version_label: scope.prompt_version_label.clone(),
                    direct_assignment_count: 0,
                    organizer_assignment_count: 0,
                    unknown_assignment_count: 0,
                });
            issue.score_sum += u32::from(item_score);
            issue.occurrences += 1;
            if is_concern {
                issue.concern_count += 1;
            }
            match assignment_mode {
                "direct" => issue.direct_assignment_count += 1,
                "organizer" => issue.organizer_assignment_count += 1,
                _ => issue.unknown_assignment_count += 1,
            }
        }

        if let Some(score) = score {
            let reviewer = agent_activity_entry(
                &mut agent_accums,
                agents,
                &review.agent_profile_id,
                &project_name,
                None,
                "reviewer",
            );
            reviewer
                .score_events
                .push((review.created_at.clone(), score));
        }

        recent_reviews.push(InsightReviewView {
            work_id: snapshot.item.id.clone(),
            title: snapshot.item.title.clone(),
            agent_name,
            project_name,
            verdict: review.verdict.to_string(),
            score_label: score
                .map(|score| format!("{score} / 100"))
                .unwrap_or_else(|| "得点未記録".to_string()),
            concerns,
            items: review_item_views,
        });
    }

    recent_reviews.sort_by(|a, b| b.work_id.cmp(&a.work_id));
    recent_reviews.truncate(5);

    let mut prompt_comparisons = prompt_comparison_accums
        .into_values()
        .filter(|comparison| comparison.variants.len() >= 2)
        .map(|comparison| {
            let PromptComparisonAccum {
                agent_id,
                agent_name,
                role,
                project_name,
                domain_name,
                artifact_type_name,
                rubric_version_label,
                knowledge_version_label,
                assignment_label,
                item_order,
                variants,
            } = comparison;
            let variants = variants
                .into_iter()
                .map(|(prompt_version_label, variant)| {
                    let average_score =
                        average_u8(variant.score_sum, variant.scored_review_count);
                    let items = item_order
                        .iter()
                        .filter_map(|item| {
                            variant.item_scores.get(item).map(|(score_sum, count)| {
                                let item_average = average_u8(*score_sum, *count);
                                PromptComparisonItemView {
                                    item: item.clone(),
                                    average_score: item_average,
                                    score_label: format!("{item_average}%"),
                                }
                            })
                        })
                        .collect();
                    let mut work_refs = variant.work_refs;
                    work_refs.sort_by(|a, b| b.0.cmp(&a.0));
                    PromptComparisonVariantView {
                        prompt_version_label,
                        review_count: variant.review_count,
                        average_score,
                        average_score_label: if variant.scored_review_count == 0 {
                            "得点未記録".to_string()
                        } else {
                            format!("{average_score} / 100")
                        },
                        work_refs: work_refs
                            .into_iter()
                            .take(5)
                            .map(|(work_id, title, score)| PromptComparisonWorkView {
                                work_id,
                                title,
                                score_label: score
                                    .map(|score| format!("{score}点"))
                                    .unwrap_or_else(|| "得点未記録".to_string()),
                            })
                            .collect(),
                        items,
                    }
                })
                .collect();
            PromptComparisonView {
                agent_id,
                agent_name,
                role,
                project_name,
                domain_name,
                artifact_type_name,
                rubric_version_label,
                knowledge_version_label,
                assignment_label,
                variants,
            }
        })
        .collect::<Vec<_>>();
    prompt_comparisons.sort_by(|a, b| {
        (
            &a.project_name,
            &a.agent_name,
            &a.domain_name,
            &a.artifact_type_name,
        )
            .cmp(&(
                &b.project_name,
                &b.agent_name,
                &b.domain_name,
                &b.artifact_type_name,
            ))
    });

    let mut agent_scores = agent_accums
        .into_values()
        .map(|mut agent| {
            let role_kind = insight_role_kind(&agent.role);
            let average = average_u8(agent.score_sum, agent.scored_review_count);
            let recent_activity_label = recent_agent_activity_label(&mut agent);
            let top_issue = if role_kind == "worker" {
                agent
                    .issue_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(item, count)| format!("{item} ({count}件)"))
                    .unwrap_or_else(|| "目立つ失点なし".to_string())
            } else if agent.failed_count > 0 {
                format!("実行失敗 {}件", agent.failed_count)
            } else {
                "目立つ実行異常なし".to_string()
            };
            AgentInsightView {
                agent_id: agent.agent_id,
                agent_name: agent.agent_name,
                role: agent.role,
                project_name: agent.project_name,
                activity_count: agent.activity_count,
                recent_activity_label,
                review_count: agent.review_count,
                average_score: if role_kind == "worker" { average } else { 0 },
                average_score_label: if role_kind != "worker" {
                    "品質評価未対応".to_string()
                } else if agent.scored_review_count == 0 {
                    "得点未記録".to_string()
                } else {
                    format!("{average} / 100")
                },
                status_label: if agent.failed_count > 0 {
                    "要確認".to_string()
                } else if role_kind != "worker" && agent.activity_count > 0 {
                    "動作記録あり".to_string()
                } else if agent.scored_review_count == 0 {
                    "記録不足".to_string()
                } else if average < 75 {
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
    agent_scores.sort_by(|left, right| {
        let role_order = |role: &str| match insight_role_kind(role) {
            "organizer" => 0,
            "worker" => 1,
            _ => 2,
        };
        (
            &left.project_name,
            role_order(&left.role),
            &left.agent_name,
        )
            .cmp(&(
                &right.project_name,
                role_order(&right.role),
                &right.agent_name,
            ))
    });

    let mut issue_matrix = issue_accums
        .into_values()
        .map(|issue| {
            let rate = average_u8(issue.score_sum, issue.occurrences);
            let (assignment_mode, assignment_label) = if issue.direct_assignment_count == issue.occurrences {
                (
                    "direct".to_string(),
                    format!("担当を明示指定（{}件）", issue.direct_assignment_count),
                )
            } else if issue.organizer_assignment_count == issue.occurrences {
                (
                    "organizer".to_string(),
                    format!("オーガナイザーが割り当て（{}件）", issue.organizer_assignment_count),
                )
            } else if issue.unknown_assignment_count == issue.occurrences {
                ("unknown".to_string(), "割り当て元は記録不足".to_string())
            } else {
                (
                    "mixed".to_string(),
                    format!(
                        "割り当て元が混在（明示{} / オーガナイザー{} / 不明{}）",
                        issue.direct_assignment_count,
                        issue.organizer_assignment_count,
                        issue.unknown_assignment_count
                    ),
                )
            };
            InsightIssueView {
                agent_id: issue.agent_id,
                agent_name: issue.agent_name,
                role: issue.role,
                project_name: issue.project_name,
                item: issue.item.clone(),
                rate,
                rate_label: format!("{rate}%"),
                occurrences: issue.occurrences,
                suggestion_kind: suggestion_kind_for_issue(&issue.item, rate).to_string(),
                domain_name: issue.domain_name,
                artifact_type_name: issue.artifact_type_name,
                rubric_version_label: issue.rubric_version_label,
                knowledge_version_label: issue.knowledge_version_label,
                prompt_version_label: issue.prompt_version_label,
                assignment_mode,
                assignment_label,
            }
        })
        .collect::<Vec<_>>();
    issue_matrix.sort_by_key(|issue| (issue.rate, std::cmp::Reverse(issue.occurrences)));

    let signals = build_insight_signals(&issue_matrix, &episodes);
    let mut proposals = improvement_proposals_from_signals(&signals);

    if scored_review_count >= 10
        && concern_count == 0
        && average_score(total_score, scored_review_count) >= 90.0
    {
        let current_text = "確認ポリシー: 最後に人が確認する".to_string();
        let suggested_text = "確認ポリシー: レビュー懸念がある時だけ人が確認する".to_string();
        proposals.push(ImprovementProposalView {
            id: "operation-approval-policy".to_string(),
            kind: "運用".to_string(),
            title: "確認ポリシー緩和の提案".to_string(),
            target_label: "プロジェクト設定 / 確認ポリシー".to_string(),
            summary: "直近レビューが安定しているため、「最後に確認する」から「重要時のみ確認」への切り替え候補です。".to_string(),
            evidence: format!("得点記録のあるレビュー {}件、平均 {:.0}点、懸念0件。", scored_review_count, average_score(total_score, scored_review_count)),
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
    let current_average = average_score(total_score, scored_review_count);
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
        average_score_label: if scored_review_count == 0 {
            "得点未記録".to_string()
        } else {
            format!("{current_average:.0} / 100（得点記録 {scored_review_count}/{review_count}件）")
        },
        concern_count,
        proposal_count,
        signals,
        agent_scores,
        issue_matrix,
        proposals,
        applied_improvements,
        recent_reviews,
        prompt_comparisons,
    }
}

fn build_insight_signals(
    issue_matrix: &[InsightIssueView],
    episodes: &[InsightEpisode],
) -> Vec<InsightSignalView> {
    let mut signals = issue_matrix
        .iter()
        .filter(|issue| issue.rate < 75)
        .map(|issue| review_issue_signal(issue, issue_matrix, episodes))
        .map(|signal| (signal.id.clone(), signal))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect::<Vec<_>>();
    signals.extend(reviewer_override_signals(episodes));
    signals.extend(organizer_assignment_signals(episodes));
    signals.sort_by(|left, right| {
        (
            !left.proposal_ready,
            &left.project_name,
            &left.primary_cause_label,
            &left.title,
        )
            .cmp(&(
                !right.proposal_ready,
                &right.project_name,
                &right.primary_cause_label,
                &right.title,
            ))
    });
    signals
}

fn review_issue_signal(
    issue: &InsightIssueView,
    issue_matrix: &[InsightIssueView],
    episodes: &[InsightEpisode],
) -> InsightSignalView {
    let matching = episodes
        .iter()
        .filter(|episode| {
            episode.project_name == issue.project_name
                && episode.worker_id == issue.agent_id
                && episode.domain_name == issue.domain_name
                && episode.artifact_type_name == issue.artifact_type_name
                && episode.rubric_version_label == issue.rubric_version_label
                && episode.knowledge_version_label == issue.knowledge_version_label
                && episode.prompt_version_label == issue.prompt_version_label
                && episode.review_items.contains_key(&issue.item)
        })
        .collect::<Vec<_>>();
    let peer_agents = issue_matrix
        .iter()
        .filter(|candidate| {
            candidate.rate < 75
                && candidate.project_name == issue.project_name
                && candidate.domain_name == issue.domain_name
                && candidate.artifact_type_name == issue.artifact_type_name
                && candidate.rubric_version_label == issue.rubric_version_label
                && candidate.knowledge_version_label == issue.knowledge_version_label
                && candidate.item == issue.item
        })
        .map(|candidate| candidate.agent_id.as_str())
        .collect::<BTreeSet<_>>();
    let peer_matching = episodes
        .iter()
        .filter(|episode| {
            episode.project_name == issue.project_name
                && peer_agents.contains(episode.worker_id.as_str())
                && episode.domain_name == issue.domain_name
                && episode.artifact_type_name == issue.artifact_type_name
                && episode.rubric_version_label == issue.rubric_version_label
                && episode.knowledge_version_label == issue.knowledge_version_label
                && episode.review_items.contains_key(&issue.item)
        })
        .collect::<Vec<_>>();
    let reviewer_override_count = matching
        .iter()
        .filter(|episode| {
            verdict_is_pass(&episode.review_verdict)
                && episode.human_decision_type == "reject"
        })
        .count();
    let friction_count = matching
        .iter()
        .filter(|episode| {
            !episode.worker_questions.is_empty()
                || !episode.handoff_summaries.is_empty()
                || !episode.recovery_summaries.is_empty()
        })
        .count();
    let shared_cause = peer_agents.len() >= 2;
    let shared_conflict_count = peer_matching
        .iter()
        .filter(|episode| {
            !episode.worker_questions.is_empty()
                || !episode.handoff_summaries.is_empty()
                || !episode.recovery_summaries.is_empty()
                || (verdict_is_pass(&episode.review_verdict)
                    && episode.human_decision_type == "reject")
        })
        .count();
    let repeated = issue.occurrences >= 2;
    let (primary_cause_kind, primary_cause_label, confidence_label, proposal_ready, proposal_status_label, proposal_kind, proposal_target_id, proposal_target_label, history_assessment, competing_causes) =
        if shared_cause && shared_conflict_count == 0 {
            (
                "shared".to_string(),
                "共有知識・成果物定義".to_string(),
                "有力".to_string(),
                true,
                "改善候補にできます".to_string(),
                "知識".to_string(),
                format!("shared:{}:{}", issue.domain_name, issue.artifact_type_name),
                issue.artifact_type_name.clone(),
                format!(
                    "履歴全体を比較すると、同じ知識・ルーブリックを使う{}人のワーカーで「{}」の失点が確認されました。個別ワーカーより、共有知識または成果物指示の不足が有力です。",
                    peer_agents.len(), issue.item
                ),
                vec![
                    "各ワーカー固有のプロンプト".to_string(),
                    "ルーブリックの判定条件".to_string(),
                ],
            )
        } else if shared_cause {
            (
                "undetermined".to_string(),
                "判断保留".to_string(),
                "競合あり".to_string(),
                false,
                "複数ワーカーの競合履歴を先に確認します".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format!(
                    "同じ知識・ルーブリックを使う{}人のワーカーで「{}」の失点がありますが、質問、handoff、回復、または人の判定覆しが{}件含まれます。共有定義の問題とはまだ断定しません。",
                    peer_agents.len(), issue.item, shared_conflict_count
                ),
                vec![
                    "共有知識・成果物定義".to_string(),
                    "オーガナイザーの入力・割り当て".to_string(),
                    "各ワーカー固有の方策".to_string(),
                ],
            )
        } else if reviewer_override_count > 0 {
            (
                "undetermined".to_string(),
                "判断保留".to_string(),
                "競合あり".to_string(),
                false,
                "人の差し戻し理由を先に確認します".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format!(
                    "「{}」の失点に加え、レビュー合格後の人の差し戻しが{}件あります。ワーカー品質とレビュアー較正が競合するため、この失点だけでは改善対象を確定できません。",
                    issue.item, reviewer_override_count
                ),
                vec![
                    "ワーカーの方策".to_string(),
                    "レビュアーの判定較正".to_string(),
                    "依頼目的の変更".to_string(),
                ],
            )
        } else if friction_count > 0 {
            (
                "undetermined".to_string(),
                "判断保留".to_string(),
                "競合あり".to_string(),
                false,
                "質問・handoff・回復理由の確認が必要です".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format!(
                    "「{}」の失点がある実行のうち{}件で、質問、handoff、または回復が発生しています。入力不足、割り当て、実行環境が交絡するため、ワーカー改善へ直結させません。",
                    issue.item, friction_count
                ),
                vec![
                    "オーガナイザーのRun Packet".to_string(),
                    "担当範囲・実行環境".to_string(),
                    "ワーカーの方策".to_string(),
                ],
            )
        } else if repeated {
            (
                "worker".to_string(),
                issue.agent_name.clone(),
                "有力".to_string(),
                true,
                "改善候補にできます".to_string(),
                "プロンプト".to_string(),
                issue.agent_id.clone(),
                issue.agent_name.clone(),
                format!(
                    "依頼からレビュー後の判断まで確認した範囲では、担当変更、回復、人による判定覆しは記録されず、{}が同じ条件で「{}」を{}件反復して失点しています。ワーカー方策が有力ですが、共有指示は競合候補として残します。",
                    issue.agent_name, issue.item, issue.occurrences
                ),
                vec![
                    "共有知識・成果物指示".to_string(),
                    "依頼ごとの難度差".to_string(),
                ],
            )
        } else {
            (
                "undetermined".to_string(),
                "判断保留".to_string(),
                "記録不足".to_string(),
                false,
                "同条件の履歴を追加して確認します".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format!(
                    "「{}」の失点は{}件だけです。履歴は確認できましたが、単発事例からワーカー、割り当て、共有指示のいずれかへ帰属する証拠は不足しています。",
                    issue.item, issue.occurrences
                ),
                vec![
                    "ワーカーの方策".to_string(),
                    "オーガナイザーの入力・割り当て".to_string(),
                    "共有知識・成果物定義".to_string(),
                ],
            )
        };
    let evidence_source = if shared_cause {
        &peer_matching
    } else {
        &matching
    };
    let evidence = evidence_source
        .iter()
        .flat_map(|episode| issue_episode_evidence(episode, &issue.item))
        .take(12)
        .collect::<Vec<_>>();
    let signal_id_target = if shared_cause {
        format!("shared:{}:{}", issue.domain_name, issue.artifact_type_name)
    } else {
        issue.agent_id.clone()
    };
    let observation = if shared_cause {
        format!(
            "レビュー項目「{}」の基準未達が{}人のワーカーで確認されました。",
            issue.item,
            peer_agents.len()
        )
    } else {
        format!(
            "レビュー項目「{}」の獲得率は{}（{}件）です。",
            issue.item, issue.rate_label, issue.occurrences
        )
    };
    let prompt_version_label = if shared_cause {
        "複数ワーカー".to_string()
    } else {
        issue.prompt_version_label.clone()
    };
    InsightSignalView {
        id: proposal_id(
            &signal_id_target,
            &format!(
                "{}-{}-{}-{}",
                issue.project_name, issue.domain_name, issue.artifact_type_name, issue.item
            ),
            "signal",
        ),
        agent_id: if primary_cause_kind == "shared" {
            String::new()
        } else {
            issue.agent_id.clone()
        },
        agent_name: if primary_cause_kind == "shared" {
            "共有知識・成果物定義".to_string()
        } else {
            issue.agent_name.clone()
        },
        role: if primary_cause_kind == "shared" {
            "shared".to_string()
        } else {
            issue.role.clone()
        },
        project_name: issue.project_name.clone(),
        title: format!("{}が基準未達", issue.item),
        item_label: issue.item.clone(),
        observation,
        history_assessment,
        scope: format!(
            "{} / {} / {}",
            issue.project_name, issue.domain_name, issue.artifact_type_name
        ),
        scope_detail: format!(
            "ルーブリック {} · 知識 {} · プロンプト {} · {}",
            issue.rubric_version_label,
            issue.knowledge_version_label,
            prompt_version_label,
            issue.assignment_label
        ),
        primary_cause_kind,
        primary_cause_label,
        confidence_label,
        proposal_ready,
        proposal_status_label,
        proposal_kind,
        proposal_target_id,
        proposal_target_label,
        domain_name: issue.domain_name.clone(),
        artifact_type_name: issue.artifact_type_name.clone(),
        rubric_version_label: issue.rubric_version_label.clone(),
        knowledge_version_label: issue.knowledge_version_label.clone(),
        prompt_version_label,
        evidence,
        competing_causes,
    }
}

fn issue_episode_evidence(
    episode: &InsightEpisode,
    item: &str,
) -> Vec<InsightSignalEvidenceView> {
    let mut evidence = vec![InsightSignalEvidenceView {
        work_id: episode.work_id.clone(),
        stage: "レビュー".to_string(),
        summary: format!(
            "{} · {}が「{}」を{}点と評価",
            episode.title,
            episode.reviewer_name,
            item,
            episode.review_items.get(item).copied().unwrap_or_default()
        ),
    }];
    evidence.push(InsightSignalEvidenceView {
        work_id: episode.work_id.clone(),
        stage: "割り当て".to_string(),
        summary: format!("{} · 担当 {}", episode.assignment_label, episode.worker_name),
    });
    if !episode.worker_questions.is_empty() {
        evidence.push(InsightSignalEvidenceView {
            work_id: episode.work_id.clone(),
            stage: "質問".to_string(),
            summary: episode.worker_questions.join(" / "),
        });
    }
    if !episode.handoff_summaries.is_empty() {
        evidence.push(InsightSignalEvidenceView {
            work_id: episode.work_id.clone(),
            stage: "handoff".to_string(),
            summary: episode.handoff_summaries.join(" / "),
        });
    }
    if !episode.recovery_summaries.is_empty() {
        evidence.push(InsightSignalEvidenceView {
            work_id: episode.work_id.clone(),
            stage: "要対応".to_string(),
            summary: episode.recovery_summaries.join(" / "),
        });
    }
    if !episode.human_decision_type.is_empty() {
        evidence.push(InsightSignalEvidenceView {
            work_id: episode.work_id.clone(),
            stage: "人の判断".to_string(),
            summary: format!(
                "{}: {}",
                episode.human_decision_type, episode.human_decision_rationale
            ),
        });
    }
    if episode.organizer_summary_count > 0 {
        evidence.push(InsightSignalEvidenceView {
            work_id: episode.work_id.clone(),
            stage: "最終まとめ".to_string(),
            summary: format!(
                "オーガナイザーのまとめ {}件を確認",
                episode.organizer_summary_count
            ),
        });
    }
    evidence
}

fn reviewer_override_signals(episodes: &[InsightEpisode]) -> Vec<InsightSignalView> {
    let mut groups = BTreeMap::<(String, String, String, String, String), Vec<&InsightEpisode>>::new();
    for episode in episodes.iter().filter(|episode| {
        verdict_is_pass(&episode.review_verdict) && episode.human_decision_type == "reject"
    }) {
        groups
            .entry((
                episode.reviewer_id.clone(),
                episode.project_name.clone(),
                episode.domain_name.clone(),
                episode.artifact_type_name.clone(),
                episode.reviewer_prompt_version_label.clone(),
            ))
            .or_default()
            .push(episode);
    }
    groups
        .into_values()
        .map(|group| {
            let first = group[0];
            let evidence = group
                .iter()
                .map(|episode| InsightSignalEvidenceView {
                    work_id: episode.work_id.clone(),
                    stage: "レビュー → 人の判断".to_string(),
                    summary: format!(
                        "{}は合格、人は差し戻し: {}",
                        episode.reviewer_name, episode.human_decision_rationale
                    ),
                })
                .collect::<Vec<_>>();
            InsightSignalView {
                id: proposal_id(
                    &first.reviewer_id,
                    &format!("{}-{}-review-override", first.project_name, first.artifact_type_name),
                    "signal",
                ),
                agent_id: first.reviewer_id.clone(),
                agent_name: first.reviewer_name.clone(),
                role: "reviewer".to_string(),
                project_name: first.project_name.clone(),
                title: "レビュー合格後に人が差し戻し".to_string(),
                item_label: "判定較正".to_string(),
                observation: format!(
                    "レビュー合格後の人の差し戻しが{}件あります。",
                    group.len()
                ),
                history_assessment: "レビューから人の判断までを照合すると、レビュアーの合格判定と最終判断が一致していません。レビュアーの判定手順が有力ですが、依頼目的の変更とルーブリック不足も確認対象です。".to_string(),
                scope: format!(
                    "{} / {} / {}",
                    first.project_name, first.domain_name, first.artifact_type_name
                ),
                scope_detail: format!(
                    "レビュアー {} · プロンプト {}",
                    first.reviewer_name, first.reviewer_prompt_version_label
                ),
                primary_cause_kind: "reviewer".to_string(),
                primary_cause_label: first.reviewer_name.clone(),
                confidence_label: "有力".to_string(),
                proposal_ready: true,
                proposal_status_label: "改善候補にできます".to_string(),
                proposal_kind: "プロンプト".to_string(),
                proposal_target_id: first.reviewer_id.clone(),
                proposal_target_label: first.reviewer_name.clone(),
                domain_name: first.domain_name.clone(),
                artifact_type_name: first.artifact_type_name.clone(),
                rubric_version_label: first.rubric_version_label.clone(),
                knowledge_version_label: first.knowledge_version_label.clone(),
                prompt_version_label: first.reviewer_prompt_version_label.clone(),
                evidence,
                competing_causes: vec![
                    "依頼目的の変更".to_string(),
                    "ルーブリック・成果物定義の不足".to_string(),
                ],
            }
        })
        .collect()
}

fn organizer_assignment_signals(episodes: &[InsightEpisode]) -> Vec<InsightSignalView> {
    let mut groups = BTreeMap::<(String, String, String, String, String), Vec<&InsightEpisode>>::new();
    for episode in episodes.iter().filter(|episode| {
        episode.assignment_mode == "organizer"
            && !episode.organizer_id.is_empty()
            && (!episode.handoff_summaries.is_empty()
                || episode.recovery_summaries.iter().any(|summary| {
                    summary.starts_with("handoff") || summary.starts_with("redispatch")
                }))
    }) {
        groups
            .entry((
                episode.organizer_id.clone(),
                episode.project_name.clone(),
                episode.domain_name.clone(),
                episode.artifact_type_name.clone(),
                episode.organizer_prompt_version_label.clone(),
            ))
            .or_default()
            .push(episode);
    }
    groups
        .into_values()
        .map(|group| {
            let first = group[0];
            let ready = group.len() >= 2;
            let evidence = group
                .iter()
                .flat_map(|episode| {
                    episode
                        .handoff_summaries
                        .iter()
                        .chain(episode.recovery_summaries.iter())
                        .map(|summary| InsightSignalEvidenceView {
                            work_id: episode.work_id.clone(),
                            stage: "割り当て後の見直し".to_string(),
                            summary: summary.clone(),
                        })
                })
                .collect::<Vec<_>>();
            InsightSignalView {
                id: proposal_id(
                    &first.organizer_id,
                    &format!("{}-{}-assignment", first.project_name, first.artifact_type_name),
                    "signal",
                ),
                agent_id: first.organizer_id.clone(),
                agent_name: first.organizer_name.clone(),
                role: "organizer".to_string(),
                project_name: first.project_name.clone(),
                title: "割り当て後に担当見直しが発生".to_string(),
                item_label: "担当選定とRun Packet".to_string(),
                observation: format!(
                    "オーガナイザーの割り当て後にhandoffまたは再割り当てが{}件あります。",
                    group.len()
                ),
                history_assessment: format!(
                    "割り当てから後続工程までを確認すると、担当見直しが{}件発生しています。反復している場合は担当選定またはRun Packetが有力ですが、専門分業として意図されたhandoffかも確認が必要です。",
                    group.len()
                ),
                scope: format!(
                    "{} / {} / {}",
                    first.project_name, first.domain_name, first.artifact_type_name
                ),
                scope_detail: format!(
                    "オーガナイザー {} · プロンプト {}",
                    first.organizer_name, first.organizer_prompt_version_label
                ),
                primary_cause_kind: "organizer".to_string(),
                primary_cause_label: first.organizer_name.clone(),
                confidence_label: if ready { "有力" } else { "競合あり" }.to_string(),
                proposal_ready: ready,
                proposal_status_label: if ready {
                    "改善候補にできます"
                } else {
                    "同条件での再発を確認します"
                }
                .to_string(),
                proposal_kind: if ready { "プロンプト" } else { "" }.to_string(),
                proposal_target_id: if ready {
                    first.organizer_id.clone()
                } else {
                    String::new()
                },
                proposal_target_label: if ready {
                    first.organizer_name.clone()
                } else {
                    String::new()
                },
                domain_name: first.domain_name.clone(),
                artifact_type_name: first.artifact_type_name.clone(),
                rubric_version_label: first.rubric_version_label.clone(),
                knowledge_version_label: first.knowledge_version_label.clone(),
                prompt_version_label: first.organizer_prompt_version_label.clone(),
                evidence,
                competing_causes: vec![
                    "意図された専門分業のhandoff".to_string(),
                    "ワーカーの実行能力".to_string(),
                    "依頼情報の不足".to_string(),
                ],
            }
        })
        .collect()
}

fn improvement_proposals_from_signals(
    signals: &[InsightSignalView],
) -> Vec<ImprovementProposalView> {
    let mut groups = BTreeMap::<
        (String, String, String, String, String, String, String, String),
        Vec<&InsightSignalView>,
    >::new();
    for signal in signals.iter().filter(|signal| {
        signal.proposal_ready
            && !signal.proposal_kind.is_empty()
            && !signal.proposal_target_id.is_empty()
    }) {
        groups
            .entry((
                signal.proposal_target_id.clone(),
                signal.proposal_kind.clone(),
                signal.project_name.clone(),
                signal.domain_name.clone(),
                signal.artifact_type_name.clone(),
                signal.rubric_version_label.clone(),
                signal.knowledge_version_label.clone(),
                signal.prompt_version_label.clone(),
            ))
            .or_default()
            .push(signal);
    }
    groups
        .into_values()
        .take(4)
        .map(|group| {
            let first = group[0];
            let item_label = group
                .iter()
                .map(|signal| signal.item_label.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("、");
            let work_ids = group
                .iter()
                .flat_map(|signal| signal.evidence.iter().map(|evidence| evidence.work_id.as_str()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("、");
            let current_text = current_improvement_text(
                &first.proposal_kind,
                &first.proposal_target_label,
                &item_label,
            );
            let suggested_text = suggested_improvement_text(&first.proposal_kind, &item_label);
            ImprovementProposalView {
                id: proposal_id(
                    &first.proposal_target_id,
                    &format!(
                        "{}-{}-{}-{}",
                        first.project_name,
                        first.domain_name,
                        first.artifact_type_name,
                        item_label
                    ),
                    &first.proposal_kind,
                ),
                kind: first.proposal_kind.clone(),
                title: format!(
                    "{} の{}改善",
                    first.proposal_target_label, first.proposal_kind
                ),
                target_label: format!("{} / {}", first.proposal_target_label, item_label),
                summary: format!(
                    "履歴全体を照合した結果、{}が有力です。一つの方策変更として「{}」だけを限定的に検証します。",
                    first.primary_cause_label, item_label
                ),
                evidence: format!(
                    "根拠ワーク: {}。{} 競合候補: {}。",
                    work_ids,
                    group
                        .iter()
                        .map(|signal| signal.history_assessment.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                    first.competing_causes.join("、")
                ),
                diff_lines: vec![
                    format!("- {current_text}"),
                    format!("+ {suggested_text}"),
                ],
                current_text,
                suggested_text,
                next_step: next_step_for_improvement_kind(&first.proposal_kind).to_string(),
                action_label: "根拠と差分を確認".to_string(),
            }
        })
        .collect()
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
    let target_items = target_parts
        .get(1)
        .copied()
        .unwrap_or_default()
        .split('、')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<BTreeSet<_>>();
    let matching_issues = issue_matrix
        .iter()
        .filter(|issue| {
            !target_agent.is_empty()
                && issue.agent_name == target_agent
                && target_items.contains(issue.item.as_str())
        })
        .collect::<Vec<_>>();
    if let [issue] = matching_issues.as_slice() {
        return format!(
            "測定中: {} {}（{}件）",
            issue.item, issue.rate_label, issue.occurrences
        );
    }
    if !matching_issues.is_empty() {
        let minimum_rate = matching_issues
            .iter()
            .map(|issue| issue.rate)
            .min()
            .unwrap_or_default();
        let minimum_occurrences = matching_issues
            .iter()
            .map(|issue| issue.occurrences)
            .min()
            .unwrap_or_default();
        return format!(
            "測定中: {}項目の最低獲得率 {}%（各{}件以上）",
            matching_issues.len(),
            minimum_rate,
            minimum_occurrences
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

fn insight_scope(
    snapshot: &WorkItemSnapshot,
    trace_records: &[TraceRecord],
    agents: &[AgentProfile],
    domains: &[Domain],
    artifact_types: &[ArtifactType],
    agent_id: &str,
) -> InsightScope {
    let domain_id = snapshot.item.domain_id.clone().unwrap_or_default();
    let artifact_type_id = snapshot.item.artifact_type_id.clone().unwrap_or_default();
    let domain_name = domains
        .iter()
        .find(|domain| domain.id == domain_id)
        .map(|domain| domain.display_name.clone())
        .unwrap_or_else(|| {
            if domain_id.is_empty() {
                "ドメイン未指定".to_string()
            } else {
                domain_id.clone()
            }
        });
    let artifact_type_name = artifact_types
        .iter()
        .find(|artifact_type| artifact_type.id == artifact_type_id)
        .map(|artifact_type| artifact_type.display_name.clone())
        .unwrap_or_else(|| {
            if artifact_type_id.is_empty() {
                "成果物未指定".to_string()
            } else {
                artifact_type_id.clone()
            }
        });
    let reviewer_payload = trace_records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .map(|record| &record.payload);
    let rubric_version_label = reviewer_payload
        .and_then(|payload| value_path_u64(payload, &["rubric_ref", "version"]))
        .map(|version| format!("v{version}"))
        .unwrap_or_else(|| "版未記録".to_string());
    let knowledge_version_label = reviewer_payload
        .and_then(|payload| payload.get("knowledge_refs"))
        .and_then(|value| value.as_array())
        .map(|references| {
            references
                .iter()
                .filter_map(|reference| {
                    let id = reference.get("id")?.as_str()?.trim();
                    if id.is_empty() {
                        return None;
                    }
                    let name = domains
                        .iter()
                        .find(|domain| domain.id == id)
                        .map(|domain| domain.display_name.as_str())
                        .or_else(|| {
                            artifact_types
                                .iter()
                                .find(|artifact_type| artifact_type.id == id)
                                .map(|artifact_type| artifact_type.display_name.as_str())
                        })
                        .unwrap_or(id);
                    let version = reference
                        .get("version")
                        .and_then(|value| value.as_u64())
                        .map(|version| format!(" v{version}"))
                        .unwrap_or_else(|| " 版未記録".to_string());
                    Some(format!("{name}{version}"))
                })
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| "版未記録".to_string());
    let prompt_version_label = snapshot
        .resolved_run_packets
        .iter()
        .rev()
        .find(|packet| {
            packet.purpose == AgentRunPurpose::Work && packet.agent_profile_id == agent_id
        })
        .map(|packet| packet.prompt_version.trim())
        .filter(|version| !version.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            agents
                .iter()
                .find(|agent| agent.id == agent_id)
                .map(|agent| agent.prompt.version.trim())
                .filter(|version| !version.is_empty())
                .map(|version| format!("実行時版未記録（現在 {version}）"))
        })
        .unwrap_or_else(|| "実行時版未記録".to_string());

    InsightScope {
        domain_id,
        domain_name,
        artifact_type_id,
        artifact_type_name,
        rubric_version_label,
        knowledge_version_label,
        prompt_version_label,
    }
}

fn recorded_prompt_version(label: &str) -> bool {
    let label = label.trim();
    !label.is_empty() && !label.starts_with("実行時版未記録")
}

fn insight_assignment_mode(snapshot: &WorkItemSnapshot, agent_id: &str) -> &'static str {
    if let Some(packet) = snapshot.resolved_run_packets.iter().rev().find(|packet| {
        packet.purpose == AgentRunPurpose::Work && packet.agent_profile_id == agent_id
    }) {
        return if packet.dispatch_plan_id.is_some() {
            "organizer"
        } else {
            "direct"
        };
    }
    if snapshot
        .dispatch_plans
        .iter()
        .rev()
        .any(|plan| plan.target_agent_profile_id == agent_id)
    {
        "organizer"
    } else {
        "unknown"
    }
}

fn review_score_percent(_review: &ReviewResult, trace_records: &[TraceRecord]) -> Option<u8> {
    latest_trace_score(trace_records).map(|(total, max)| percent_u8(total, max))
}

fn latest_trace_score(trace_records: &[TraceRecord]) -> Option<(u64, u64)> {
    trace_records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .and_then(|record| {
            if let Some(score) = explicit_review_score(&record.payload) {
                return Some((u64::from(score), 100));
            }
            None
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
    let rubric_status = latest_trace_rubric_complete(trace_records);
    if rubric_status == Some(false) {
        return Vec::new();
    }
    let rubric_items = latest_trace_rubric_items(trace_records).unwrap_or_default();
    if rubric_status == Some(true) {
        return rubric_items
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

fn suggestion_kind_for_issue(_item: &str, rate: u8) -> &'static str {
    if rate < 75 {
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
            "成果物のルーブリックでは「{item}」の判定観点が弱く、レビューごとの判断にぶれが残っています。"
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

fn agent_view(profile: &AgentProfile, usage_count: usize) -> AgentView {
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
        usage_count,
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
    let artifacts = artifact_views(&snapshot.artifacts);
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
        question: unanswered_question(&snapshot),
        question_source: unanswered_question_source(&snapshot),
        recovery: current_recovery(&snapshot).map(recovery_view),
        request,
        answer,
        artifacts,
        effective_capabilities: effective_capability_views(root, &snapshot),
        prohibited_task_gate: prohibited_task_gate_view(&snapshot),
        review,
        steps,
    }
}

fn effective_capability_views(
    root: &Path,
    snapshot: &WorkItemSnapshot,
) -> Vec<EffectiveCapabilityView> {
    snapshot
        .resolved_skill_contexts
        .iter()
        .rev()
        .map(|context| {
            let packet = snapshot
                .resolved_run_packets
                .iter()
                .rev()
                .find(|packet| packet.resolved_skill_context_id == context.id);
            let profile = get_agent_profile(root, &context.agent_profile_id).ok();
            let agent_label = profile
                .as_ref()
                .map(|profile| profile.display_name.trim())
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| context.agent_profile_id.clone());
            let allowed_skill_count = context
                .codex_skill_config
                .iter()
                .filter(|entry| entry.enabled)
                .count();
            let disabled_skill_count = context
                .codex_skill_config
                .len()
                .saturating_sub(allowed_skill_count);
            EffectiveCapabilityView {
                purpose: packet
                    .map(|packet| question_purpose_label(packet.purpose).to_string())
                    .unwrap_or_else(|| "実行工程".to_string()),
                agent_id: context.agent_profile_id.clone(),
                agent_label,
                skills: context.applied_skill_set_ids.clone(),
                mcp_connections: profile
                    .map(|profile| profile.mcp_connection_ids)
                    .unwrap_or_default(),
                allowed_skill_count,
                disabled_skill_count,
                scope_diagnostics: context.scope_diagnostics.clone(),
                skill_paths: context.effective_skill_paths.clone(),
            }
        })
        .collect()
}

fn prohibited_task_gate_view(snapshot: &WorkItemSnapshot) -> Option<ProhibitedTaskGateView> {
    let rules = nagare_core::prohibited_task_constraints(&snapshot.item.constraints);
    if rules.is_empty() {
        return None;
    }
    let Some(review) = snapshot.review_results.last() else {
        return Some(ProhibitedTaskGateView {
            summary: "禁止タスクの Reviewer 確認待ちです。".to_string(),
            status: "pending".to_string(),
            rules,
            evidence: Vec::new(),
        });
    };
    let policy_items = review
        .criteria_results
        .iter()
        .filter(|result| nagare_core::is_prohibited_task_criterion(&result.criterion))
        .collect::<Vec<_>>();
    let passed = policy_items.len() == rules.len()
        && policy_items
            .iter()
            .all(|result| result.status == CriteriaReviewStatus::Passed);
    let evidence = policy_items
        .iter()
        .map(|result| format!("{} — {}", result.criterion, result.note))
        .collect::<Vec<_>>();
    Some(ProhibitedTaskGateView {
        status: if passed { "passed" } else { "failed" }.to_string(),
        summary: if passed {
            "Reviewer がすべての禁止タスクルールを確認しました。".to_string()
        } else {
            "禁止タスクの確認が未記録、または違反のためレビューを差し戻します。".to_string()
        },
        rules,
        evidence,
    })
}

fn artifact_views(artifacts: &[Artifact]) -> Vec<ArtifactView> {
    let mut seen = BTreeSet::new();
    artifacts
        .iter()
        .filter_map(|artifact| {
            let raw_key = if artifact.uri.trim().is_empty() {
                artifact.title.trim()
            } else {
                artifact.uri.trim()
            };
            let normalized_key = raw_key.replace('\\', "/");
            let key = if cfg!(windows) {
                normalized_key.to_lowercase()
            } else {
                normalized_key
            };
            if !seen.insert(key) {
                return None;
            }
            Some(ArtifactView {
                title: artifact.title.clone(),
                uri: artifact.uri.clone(),
            })
        })
        .collect()
}

fn latest_question(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| output.questions.first().cloned())
}

fn unanswered_question(snapshot: &WorkItemSnapshot) -> Option<String> {
    (snapshot.item.status == WorkItemStatus::NeedsInput)
        .then(|| latest_question(snapshot))
        .flatten()
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

fn unanswered_question_source(snapshot: &WorkItemSnapshot) -> String {
    if snapshot.item.status == WorkItemStatus::NeedsInput {
        latest_question_source(snapshot)
    } else {
        String::new()
    }
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

fn current_recovery(snapshot: &WorkItemSnapshot) -> Option<&RecoveryPlan> {
    if !matches!(
        snapshot.completion.next_action.as_str(),
        "recover" | "apply_recovery"
    ) {
        return None;
    }
    latest_recovery(snapshot)
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
        pending.push("対応内容を確認すると、次の工程へ進めます。".to_string());
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
            outcome: String::new(),
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
            score_label: String::new(),
            criteria_label: String::new(),
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
            outcome: String::new(),
            actor,
            summary: value_str(payload, "interpreted_request")
                .unwrap_or_else(|| "担当を決定しました。".to_string()),
            rationale: first_assignment_rationale(payload),
            input: format!(
                "ドメイン: {} / 成果物: {}",
                value_str(payload, "domain_id").unwrap_or_else(|| "general".to_string()),
                value_str(payload, "artifact_type_id").unwrap_or_else(|| "general".to_string())
            ),
            output: first_plan_target(payload)
                .map(|target| format!("担当: {target}"))
                .unwrap_or_default(),
            score_label: String::new(),
            criteria_label: String::new(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "worker_output" => StepView {
            kind,
            title: "作業実行".to_string(),
            state,
            outcome: String::new(),
            actor,
            summary: value_str(payload, "actions_summary")
                .unwrap_or_else(|| "作業を実行しました。".to_string()),
            rationale: "割り当て済みエージェントが依頼を処理しました。".to_string(),
            input: value_path_str(payload, &["inputs", "summary"]).unwrap_or_default(),
            output: worker_output_summary(payload),
            score_label: String::new(),
            criteria_label: String::new(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "organizer_summary" => StepView {
            kind,
            title: "オーガナイザーまとめ".to_string(),
            state,
            outcome: String::new(),
            actor,
            summary: value_str(payload, "actions_summary")
                .or_else(|| value_str(payload, "answer"))
                .unwrap_or_else(|| "複数の結果を統合しました。".to_string()),
            rationale: "複数の作業結果とレビュー結果を依頼者向けにまとめました。".to_string(),
            input: value_path_str(payload, &["inputs", "summary"]).unwrap_or_default(),
            output: value_str(payload, "answer")
                .or_else(|| value_str(payload, "actions_summary"))
                .unwrap_or_default(),
            score_label: String::new(),
            criteria_label: String::new(),
            knowledge_refs,
            diagnostics,
            review_items: Vec::new(),
        },
        "reviewer_verdict" => StepView {
            kind,
            title: "レビュー".to_string(),
            state,
            outcome: value_str(payload, "recommendation").unwrap_or_default(),
            actor,
            summary: value_str(payload, "summary")
                .unwrap_or_else(|| "レビューを実行しました。".to_string()),
            rationale: review_rationale_summary(payload),
            input: array_strings(payload, "target_artifacts").join(", "),
            output: review_output_summary(payload),
            score_label: review_score_summary(payload).unwrap_or_default(),
            criteria_label: review_criteria_summary(payload).unwrap_or_default(),
            knowledge_refs,
            diagnostics,
            review_items: trace_review_item_views(payload),
        },
        _ => StepView {
            kind,
            title: record.record.clone(),
            state,
            outcome: String::new(),
            actor,
            summary: String::new(),
            rationale: String::new(),
            input: String::new(),
            output: String::new(),
            score_label: String::new(),
            criteria_label: String::new(),
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
    review_score_percent(review, trace_records)
        .map(|score| format!("{score} / 100"))
        .unwrap_or_else(|| "得点未記録".to_string())
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
    trace_review_item_views_for_key(payload, "item_verdicts", false)
}

fn latest_trace_rubric_items(records: &[TraceRecord]) -> Option<Vec<ReviewItemView>> {
    records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .map(|record| {
            trace_review_item_views_for_key(&record.payload, "rubric_item_verdicts", true)
        })
        .filter(|items| !items.is_empty())
}

fn latest_trace_rubric_complete(records: &[TraceRecord]) -> Option<bool> {
    records
        .iter()
        .rev()
        .find(|record| record.record == "reviewer_verdict")
        .and_then(|record| {
            (value_u64(&record.payload, "rubric_items_expected").unwrap_or_default() > 0)
                .then(|| value_bool(&record.payload, "rubric_complete").unwrap_or(false))
        })
}

fn trace_review_item_views_for_key(
    payload: &serde_json::Value,
    key: &str,
    require_recorded: bool,
) -> Vec<ReviewItemView> {
    payload
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    !require_recorded
                        || (value_bool(item, "recorded").unwrap_or(false)
                            && value_u64(item, "points").is_some()
                            && value_u64(item, "max_points").unwrap_or_default() > 0)
                })
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
    explicit_review_score(payload).map(|score| format!("{score} / 100"))
}

fn review_criteria_summary(payload: &serde_json::Value) -> Option<String> {
    match (
        value_u64(payload, "total_score"),
        value_u64(payload, "max_score"),
    ) {
        (Some(total), Some(max)) if max > 0 => Some(format!("評価項目 {total} / {max}")),
        _ => None,
    }
}

fn explicit_review_score(payload: &serde_json::Value) -> Option<u8> {
    if let (Some(total), Some(max)) = (
        value_u64(payload, "overall_score"),
        value_u64(payload, "overall_max_score"),
    ) {
        return (max > 0).then(|| percent_u8(total, max));
    }
    if let Some(summary) = value_str(payload, "summary") {
        if let Some(score) = score_out_of_one_hundred(&summary) {
            return Some(score);
        }
    }
    match (
        value_u64(payload, "total_score"),
        value_u64(payload, "max_score"),
    ) {
        (Some(total), Some(100)) => Some(total.min(100) as u8),
        _ => None,
    }
}

fn score_out_of_one_hundred(value: &str) -> Option<u8> {
    let compact = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let marker_index = compact.rfind("/100")?;
    let digits = compact[..marker_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    digits.parse::<u8>().ok().filter(|score| *score <= 100)
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

fn value_path_u64(payload: &serde_json::Value, path: &[&str]) -> Option<u64> {
    let mut current = payload;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_u64()
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

fn value_bool(payload: &serde_json::Value, key: &str) -> Option<bool> {
    payload.get(key).and_then(|value| value.as_bool())
}

fn status_view(status: WorkItemStatus, snapshot: Option<&WorkItemSnapshot>) -> (String, String) {
    let next_action = snapshot
        .map(|snapshot| snapshot.completion.next_action.as_str())
        .unwrap_or("");
    if status != WorkItemStatus::Done && matches!(next_action, "recover" | "apply_recovery") {
        return ("要対応".to_string(), "recover".to_string());
    }
    match status {
        WorkItemStatus::AgentRunning => ("処理中".to_string(), "running".to_string()),
        WorkItemStatus::NeedsInput => ("要対応・質問".to_string(), "question".to_string()),
        WorkItemStatus::ReadyForReview => ("要対応・確認".to_string(), "review".to_string()),
        WorkItemStatus::ChangesRequested => ("要対応".to_string(), "recover".to_string()),
        WorkItemStatus::Done => ("完了".to_string(), "done".to_string()),
        WorkItemStatus::NeedsHandoff => ("要対応".to_string(), "recover".to_string()),
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
        "recover" | "apply_recovery" => "対応内容を確認".to_string(),
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

fn analysis_root_project_name(root: &Path) -> String {
    let root_name = project_name_from_root(root);
    let metadata_name = get_project_metadata(root.to_path_buf())
        .ok()
        .map(|metadata| metadata.name)
        .unwrap_or_default();
    let metadata_name = metadata_name.trim();
    if !metadata_name.is_empty()
        && !(metadata_name == "nagare-local" && root_name != "nagare-local")
    {
        metadata_name.to_string()
    } else {
        root_name
    }
}

fn analysis_project_name(root_project_name: &str, work_folder: Option<&str>) -> String {
    let work_folder = work_folder.unwrap_or(".").trim();
    if work_folder.is_empty() || work_folder == "." || work_folder == "nagare" {
        root_project_name.to_string()
    } else {
        project_label(Some(work_folder))
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

fn runtime_view_from_catalog(entry: RuntimeCatalogEntry, _agents: &[AgentView]) -> RuntimeView {
    let (available, detail) = command_status(entry.command, entry.args);
    RuntimeView {
        id: entry.id,
        label: entry.label,
        command: entry.command,
        available,
        detail,
        model_note: entry.model_note,
        model_mode: entry.model_mode,
        model_choices: runtime_model_choices(&entry),
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
            model_note: "GPT-5.6: sol（高性能）/ terra（バランス）/ luna（高速）",
            model_mode: "OpenAIモデル",
            model_choices: &[
                "実行環境既定",
                "gpt-5.6",
                "gpt-5.6-sol",
                "gpt-5.6-terra",
                "gpt-5.6-luna",
                "gpt-5.3-codex",
                "gpt-5.2-codex",
                "gpt-5.1-codex",
                "gpt-5-codex",
                "手入力",
            ],
        },
        RuntimeCatalogEntry {
            id: "opencode",
            label: "OpenCode",
            command: "opencode",
            args: &["--version"],
            model_note: "ローカル設定のモデルを選択または手入力",
            model_mode: "Provider / Model",
            model_choices: &["実行環境既定", "手入力"],
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

fn runtime_model_choices(entry: &RuntimeCatalogEntry) -> Vec<String> {
    if entry.id != "opencode" {
        return entry
            .model_choices
            .iter()
            .map(|choice| (*choice).to_string())
            .collect();
    }

    let mut choices = vec!["実行環境既定".to_string()];
    choices.extend(opencode_model_choices());
    choices.push("手入力".to_string());
    choices
}

fn opencode_model_choices() -> Vec<String> {
    opencode_model_choices_from_paths(&opencode_config_paths())
}

fn opencode_config_paths() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    if let Some(home) = user_home_dir() {
        directories.push(home.join(".config").join("opencode"));
    }
    if let Some(app_data) = env::var_os("APPDATA") {
        directories.push(PathBuf::from(app_data).join("opencode"));
    }

    let mut paths = Vec::new();
    for directory in directories {
        for filename in ["config.json", "opencode.json", "opencode.jsonc"] {
            let path = directory.join(filename);
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

fn user_home_dir() -> Option<PathBuf> {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
}

fn opencode_model_choices_from_paths(paths: &[PathBuf]) -> Vec<String> {
    let mut models = BTreeSet::new();
    for path in paths {
        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        let Some(config) = parse_jsonc(&contents) else {
            continue;
        };
        collect_opencode_models(&config, &mut models);
    }
    models.into_iter().collect()
}

fn collect_opencode_models(config: &serde_json::Value, models: &mut BTreeSet<String>) {
    add_opencode_model(config.get("model"), models);

    for agent in config
        .get("agent")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|agents| agents.values())
    {
        add_opencode_model(agent.get("model"), models);
    }

    for (provider, options) in config
        .get("provider")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|providers| providers.iter())
    {
        let Some(model_definitions) = options.get("models") else {
            continue;
        };
        match model_definitions {
            serde_json::Value::Object(definitions) => {
                for model in definitions.keys() {
                    add_opencode_provider_model(provider, model, models);
                }
            }
            serde_json::Value::Array(definitions) => {
                for model in definitions {
                    add_opencode_provider_model(
                        provider,
                        model.as_str().unwrap_or_default(),
                        models,
                    );
                }
            }
            _ => {}
        }
    }
}

fn add_opencode_model(value: Option<&serde_json::Value>, models: &mut BTreeSet<String>) {
    let Some(model) = value.and_then(serde_json::Value::as_str) else {
        return;
    };
    let model = model.trim();
    if !model.is_empty() {
        models.insert(model.to_string());
    }
}

fn add_opencode_provider_model(provider: &str, model: &str, models: &mut BTreeSet<String>) {
    let provider = provider.trim();
    let model = model.trim();
    if provider.is_empty() || model.is_empty() {
        return;
    }
    let model_ref = if model.contains('/') {
        model.to_string()
    } else {
        format!("{provider}/{model}")
    };
    models.insert(model_ref);
}

fn parse_jsonc(input: &str) -> Option<serde_json::Value> {
    serde_json::from_str(input)
        .ok()
        .or_else(|| serde_json::from_str(&remove_jsonc_comments(input)).ok())
}

fn remove_jsonc_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(character) = chars.next() {
        if in_string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
            output.push(character);
            continue;
        }
        if character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for line_character in chars.by_ref() {
                if line_character == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }
        if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut previous = '\0';
            for block_character in chars.by_ref() {
                if previous == '*' && block_character == '/' {
                    break;
                }
                previous = block_character;
            }
            continue;
        }
        output.push(character);
    }

    output
}

fn command_status(command: &str, args: &[&str]) -> (bool, String) {
    let mut last_error = None;
    for candidate in command_candidates(command) {
        match Command::new(&candidate).args(args).output() {
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
                return (true, detail);
            }
            Ok(output) => return (false, format!("exit status {}", output.status)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                last_error = Some(error);
            }
            Err(error) => return (false, error.to_string()),
        }
    }
    (
        false,
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "command not found".to_string()),
    )
}

fn command_candidates(command: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        if Path::new(command).extension().is_none() {
            return vec![format!("{command}.cmd"), command.to_string()];
        }
    }
    vec![command.to_string()]
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
        DomainAgentPolicy, DomainSource, RecoveryAction, WorkItemApprovalGate,
        WorkItemCompletion,
    };
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn precise_artifact_definition_json(rubric_detail: &str) -> String {
        let rubric = (1..=8)
            .map(|index| {
                format!(
                    "## 観点{index} ({}点)\n満点条件: 判定可能な完成条件をすべて満たす。\n部分点条件: 一部の条件のみ満たす場合は確認できた割合で採点する。\n重大な不足: 必須条件が欠ける。\n確認する証跡: 成果物内の対応箇所。{rubric_detail}",
                    if index <= 4 { 13 } else { 12 }
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        let knowledge = (1..=16)
            .map(|index| {
                format!(
                    "[K{index:02}][内容] 対象: 成果物内の情報単位{index} | 条件: 成果物を作成または更新するとき | 要求: 判定可能な完成条件を一つ明記する | 証跡: 対応する本文と検証記録 | 例外: 非該当の場合は理由を記録する"
                )
            })
            .collect::<Vec<_>>();
        let coverage = ARTIFACT_COVERAGE_DIMENSIONS
            .iter()
            .enumerate()
            .map(|(index, dimension)| {
                let mut knowledge_ids = vec![format!("K{:02}", index + 1)];
                if index == ARTIFACT_COVERAGE_DIMENSIONS.len() - 1 {
                    knowledge_ids.extend(["K14".to_string(), "K15".to_string(), "K16".to_string()]);
                }
                serde_json::json!({
                    "dimension": dimension,
                    "applicability": "必須",
                    "knowledge_ids": knowledge_ids,
                    "rubric_sections": [format!("観点{}", index.min(7) + 1)],
                    "reason": format!("成果物の完成性を判定するために{dimension}を確認する必要がある。")
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "description": "利用者が完成状態を判断できる成果物。目的と利用場面を明示する。",
            "knowledge": knowledge,
            "rubric": rubric,
            "dispatch_hints": ["専門知識", "品質保証"],
            "coverage": coverage
        })
        .to_string()
    }

    #[test]
    fn artifact_definition_parser_accepts_a_precise_definition() {
        let raw = precise_artifact_definition_json("");

        let definition =
            parse_artifact_definition_response(&raw).expect("precise definition should parse");

        assert_eq!(definition.knowledge.len(), 16);
        assert_eq!(validate_rubric_markdown(&definition.rubric).unwrap().item_count, 8);
        assert_eq!(definition.coverage.len(), 13);
    }

    #[test]
    fn artifact_definition_parser_rejects_shallow_creation_instructions() {
        let mut value: serde_json::Value =
            serde_json::from_str(&precise_artifact_definition_json("")).unwrap();
        value["knowledge"] = serde_json::json!(["指示1", "指示2", "指示3"]);

        let error = parse_artifact_definition_response(&value.to_string())
            .expect_err("shallow instructions must be rejected");

        assert!(error.contains("16〜40項目"));
    }

    #[test]
    fn artifact_definition_parser_rejects_rubric_without_evidence_per_section() {
        let raw = precise_artifact_definition_json("");
        let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        value["rubric"] = serde_json::Value::String(
            value["rubric"]
                .as_str()
                .unwrap()
                .replacen("確認する証跡:", "証跡:", 1),
        );

        let error = parse_artifact_definition_response(&value.to_string())
            .expect_err("every rubric section must identify evidence");

        assert!(error.contains("確認する証跡:"));
    }

    #[test]
    fn artifact_definition_parser_rejects_uncovered_atomic_instruction() {
        let mut value: serde_json::Value =
            serde_json::from_str(&precise_artifact_definition_json("")).unwrap();
        value["coverage"].as_array_mut().unwrap().iter_mut().for_each(|entry| {
            entry["knowledge_ids"]
                .as_array_mut()
                .unwrap()
                .retain(|id| id.as_str() != Some("K16"));
        });

        let error = parse_artifact_definition_response(&value.to_string())
            .expect_err("every instruction must be represented in coverage");

        assert!(error.contains("参照されていない作成指示"));
    }

    #[test]
    fn artifact_definition_prompt_contains_domain_context_and_quality_dimensions() {
        let domain = Domain {
            id: "software-development".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: "継続的に保守するソフトウェアを開発する。".to_string(),
            shared_knowledge: vec!["変更理由を追跡可能にする。".to_string()],
            common_rubric: vec!["検証結果を証跡として残す。".to_string()],
            knowledge_version: 1,
            dispatch_hints: vec!["ソフトウェア設計".to_string()],
            workflow: DomainWorkflowOverride::default(),
            source: DomainSource::ProjectDomainDirectory,
        };

        let prompt = artifact_definition_prompt("設計書", &domain, None);

        for expected in [
            "成果物名: 設計書",
            "変更理由を追跡可能にする。",
            "検証結果を証跡として残す。",
            "網羅性の検査対象（coverageにこの13項目をすべて出力する）",
            "正常系と異常系",
            "セキュリティとプライバシー",
            "アクセシビリティ",
            "性能と信頼性",
            "16〜40項目",
            "部分点条件:",
        ] {
            assert!(prompt.contains(expected), "prompt should contain `{expected}`");
        }
    }

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
    }

    #[test]
    fn opencode_model_choices_read_global_json_and_jsonc_settings() {
        let root = temp_test_dir("opencode-model-choices");
        let json = root.join("opencode.json");
        let jsonc = root.join("opencode.jsonc");
        fs::write(
            &json,
            r#"{
  "model": "openai/gpt-5.6",
  "provider": {
    "anthropic": {
      "models": {
        "claude-opus-4-6": {}
      }
    }
  }
}"#,
        )
        .expect("write OpenCode JSON settings");
        fs::write(
            &jsonc,
            r#"{
  // An agent may override the global model.
  "agent": {
    "review": { "model": "openai/gpt-5.6-mini" }
  },
  "provider": {
    "ollama": {
      /* Local models may be configured here. */
      "models": { "qwen3-coder": {} }
    }
  }
}"#,
        )
        .expect("write OpenCode JSONC settings");

        let choices = opencode_model_choices_from_paths(&[json, jsonc]);

        assert!(choices.contains(&"openai/gpt-5.6".to_string()));
        assert!(choices.contains(&"openai/gpt-5.6-mini".to_string()));
        assert!(choices.contains(&"anthropic/claude-opus-4-6".to_string()));
        assert!(choices.contains(&"ollama/qwen3-coder".to_string()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    #[cfg(windows)]
    fn windows_runtime_detection_prefers_cmd_wrappers() {
        assert_eq!(
            command_candidates("codex"),
            vec!["codex.cmd".to_string(), "codex".to_string()]
        );
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
        assert_eq!(after.definition_version, before.definition_version);
        assert_eq!(after.dispatch_hints, before.dispatch_hints);
        assert!(
            list_improvement_history(&root)
                .expect("improvement history")
                .is_empty(),
            "failed improvement save should not record improvement history"
        );

        let _ = fs::remove_dir_all(root);
    }

    fn sample_insight_issue(agent_id: &str, occurrences: usize) -> InsightIssueView {
        InsightIssueView {
            agent_id: agent_id.to_string(),
            agent_name: format!("{agent_id} name"),
            role: "worker".to_string(),
            project_name: "比較プロジェクト".to_string(),
            item: "目的への適合".to_string(),
            rate: 50,
            rate_label: "50 / 100".to_string(),
            occurrences,
            suggestion_kind: "プロンプト".to_string(),
            domain_name: "ソフトウェア開発".to_string(),
            artifact_type_name: "ソースコード".to_string(),
            rubric_version_label: "rubric-v1".to_string(),
            knowledge_version_label: "knowledge-v1".to_string(),
            prompt_version_label: "prompt-v1".to_string(),
            assignment_mode: "organizer".to_string(),
            assignment_label: "オーガナイザーが割り当て".to_string(),
        }
    }

    fn sample_insight_episode(work_id: &str, worker_id: &str) -> InsightEpisode {
        InsightEpisode {
            work_id: work_id.to_string(),
            title: format!("検証 {work_id}"),
            project_name: "比較プロジェクト".to_string(),
            worker_id: worker_id.to_string(),
            worker_name: format!("{worker_id} name"),
            reviewer_id: "reviewer-a".to_string(),
            reviewer_name: "レビュアーA".to_string(),
            organizer_id: "organizer-a".to_string(),
            organizer_name: "オーガナイザーA".to_string(),
            assignment_mode: "organizer".to_string(),
            assignment_label: "オーガナイザーが割り当て".to_string(),
            domain_name: "ソフトウェア開発".to_string(),
            artifact_type_name: "ソースコード".to_string(),
            rubric_version_label: "rubric-v1".to_string(),
            knowledge_version_label: "knowledge-v1".to_string(),
            prompt_version_label: "prompt-v1".to_string(),
            reviewer_prompt_version_label: "reviewer-prompt-v1".to_string(),
            organizer_prompt_version_label: "organizer-prompt-v1".to_string(),
            review_verdict: "pass".to_string(),
            review_items: BTreeMap::from([("目的への適合".to_string(), 50)]),
            human_decision_type: String::new(),
            human_decision_rationale: String::new(),
            worker_questions: Vec::new(),
            handoff_summaries: Vec::new(),
            recovery_summaries: Vec::new(),
            organizer_summary_count: 1,
        }
    }

    #[test]
    fn insight_episode_does_not_mix_later_retry_with_reviewed_cycle() {
        let mut snapshot = sample_completed_snapshot();
        let mut later_output = snapshot.agent_outputs[0].clone();
        later_output.id = "output_retry".to_string();
        later_output.created_at = "2026-07-05T00:03:00+09:00".to_string();
        later_output.questions = vec!["次サイクルで確認中の質問".to_string()];
        snapshot.agent_outputs.push(later_output);
        let review = snapshot.review_results[0].clone();
        let scope = InsightScope {
            domain_id: "general".to_string(),
            domain_name: "ソフトウェア開発".to_string(),
            artifact_type_id: "docs".to_string(),
            artifact_type_name: "ソースコード".to_string(),
            rubric_version_label: "rubric-v1".to_string(),
            knowledge_version_label: "knowledge-v1".to_string(),
            prompt_version_label: "prompt-v1".to_string(),
        };

        let episode = insight_episode(
            &snapshot,
            &[],
            &[],
            "比較プロジェクト",
            "worker",
            "Worker",
            &review,
            &scope,
            "direct",
            &[("目的への適合".to_string(), 100, false)],
        );

        assert!(episode.worker_questions.is_empty());
    }

    #[test]
    fn insight_signals_attribute_reviewer_override_from_full_history() {
        let mut episode = sample_insight_episode("work-review-override", "worker-a");
        episode.human_decision_type = "reject".to_string();
        episode.human_decision_rationale = "必須要件が成果物にありません".to_string();

        let signals = build_insight_signals(&[], &[episode]);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].role, "reviewer");
        assert_eq!(signals[0].primary_cause_kind, "reviewer");
        assert!(signals[0].proposal_ready);
        assert!(signals[0].history_assessment.contains("最終判断が一致していません"));
        let proposals = improvement_proposals_from_signals(&signals);
        assert_eq!(proposals.len(), 1);
        assert!(proposals[0].target_label.contains("レビュアーA"));
    }

    #[test]
    fn insight_signals_hold_worker_change_when_history_contains_friction() {
        let issue = sample_insight_issue("worker-a", 2);
        let first = sample_insight_episode("work-friction-1", "worker-a");
        let mut second = sample_insight_episode("work-friction-2", "worker-a");
        second.worker_questions = vec!["対象範囲を確認してください".to_string()];

        let signals = build_insight_signals(&[issue], &[first, second]);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].primary_cause_kind, "undetermined");
        assert!(!signals[0].proposal_ready);
        assert!(signals[0].history_assessment.contains("交絡"));
        assert!(improvement_proposals_from_signals(&signals).is_empty());
    }

    #[test]
    fn insight_signals_merge_shared_definition_issue_across_workers() {
        let issues = vec![
            sample_insight_issue("worker-a", 1),
            sample_insight_issue("worker-b", 1),
        ];
        let episodes = vec![
            sample_insight_episode("work-shared-1", "worker-a"),
            sample_insight_episode("work-shared-2", "worker-b"),
        ];

        let signals = build_insight_signals(&issues, &episodes);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].primary_cause_kind, "shared");
        assert_eq!(signals[0].role, "shared");
        assert!(signals[0].proposal_ready);
        assert_eq!(signals[0].evidence.len(), 6);
        let proposals = improvement_proposals_from_signals(&signals);
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].kind, "知識");
    }

    #[test]
    fn insight_signals_require_repeated_organizer_assignment_friction() {
        let mut first = sample_insight_episode("work-organizer-1", "worker-a");
        first.handoff_summaries = vec!["worker-a から worker-b へ担当変更".to_string()];
        let first_signals = build_insight_signals(&[], &[first.clone()]);
        assert_eq!(first_signals.len(), 1);
        assert_eq!(first_signals[0].role, "organizer");
        assert!(!first_signals[0].proposal_ready);

        let mut second = sample_insight_episode("work-organizer-2", "worker-a");
        second.handoff_summaries = vec!["worker-a から worker-b へ担当変更".to_string()];
        let repeated_signals = build_insight_signals(&[], &[first, second]);
        assert_eq!(repeated_signals.len(), 1);
        assert!(repeated_signals[0].proposal_ready);
        assert_eq!(improvement_proposals_from_signals(&repeated_signals).len(), 1);
    }

    #[test]
    fn insights_exclude_applied_improvement_proposals() {
        let root = temp_test_dir("insights-applied-proposals");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.review_results[0].criteria_results[0].status = CriteriaReviewStatus::Failed;
        snapshot.review_results[0].criteria_results[0].note =
            "目的への適合が不足しています。".to_string();
        let mut repeated_snapshot = snapshot.clone();
        repeated_snapshot.item.id = "work_2".to_string();
        repeated_snapshot.item.title = "README更新の再検証".to_string();
        let snapshots = vec![snapshot, repeated_snapshot];

        let initial = insights_view(&root, &snapshots, &[], &[], &[]);
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

        let refreshed = insights_view(&root, &snapshots, &[], &[], &[]);
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
    fn insights_hold_single_low_score_for_observation() {
        let root = temp_test_dir("insights-single-signal");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.review_results[0].criteria_results[0].status = CriteriaReviewStatus::Failed;
        snapshot.review_results[0].criteria_results[0].note =
            "目的への適合が不足しています。".to_string();

        let insights = insights_view(&root, &[snapshot], &[], &[], &[]);

        let signal = insights
            .issue_matrix
            .iter()
            .find(|issue| issue.item == "目的への適合")
            .expect("failed criterion should remain observable");
        assert_eq!(signal.occurrences, 1);
        assert!(signal.rate < 75);
        assert_eq!(insights.proposal_count, 0);
        assert!(insights.proposals.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_include_role_specific_activity_for_all_agent_roles() {
        let root = temp_test_dir("insights-all-agent-roles");
        init_project(&root).expect("init project");
        let snapshot = sample_completed_snapshot();
        write_trace_records(&root, &snapshot.item.id);
        let agents = list_agent_profiles(&root).expect("agents");
        let domains = list_domains(&root).expect("domains");
        let artifact_types = list_artifact_types(&root).expect("artifact types");

        let insights = insights_view(
            &root,
            &[snapshot],
            &agents,
            &domains,
            &artifact_types,
        );

        let organizer = insights
            .agent_scores
            .iter()
            .find(|agent| agent.agent_id == "organizer")
            .expect("organizer activity");
        assert_eq!(organizer.status_label, "動作記録あり");
        assert_eq!(organizer.recent_activity_label, "割り当て 1件");

        let worker = insights
            .agent_scores
            .iter()
            .find(|agent| agent.agent_id == "writer")
            .expect("worker activity");
        assert_eq!(worker.recent_activity_label, "評価された点 92点");

        let reviewer = insights
            .agent_scores
            .iter()
            .find(|agent| agent.agent_id == "reviewer")
            .expect("reviewer activity");
        assert_eq!(reviewer.average_score_label, "品質評価未対応");
        assert_eq!(reviewer.recent_activity_label, "付与した点 92点");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn analysis_prefers_recorded_rubric_scores_over_acceptance_counts() {
        let snapshot = sample_completed_snapshot();
        let trace = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: snapshot.item.id.clone(),
            seq: 1,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "item_verdicts": [
                    { "item": "acceptance", "points": 1, "max_points": 1, "verdict": "pass" }
                ],
                "rubric_item_verdicts": [
                    {
                        "item": "Correctness",
                        "points": 30,
                        "max_points": 60,
                        "verdict": "partial",
                        "recorded": true,
                        "evidence": "An edge case is missing."
                    },
                    {
                        "item": "Clarity",
                        "points": null,
                        "max_points": 40,
                        "verdict": "unknown",
                        "recorded": false,
                        "evidence": "Not recorded."
                    }
                ],
                "rubric_items_expected": 1,
                "rubric_complete": true
            }),
        }];

        let items = insight_review_items(&snapshot.review_results[0], &trace, 85);

        assert_eq!(items, vec![("Correctness".to_string(), 50, true)]);
    }

    #[test]
    fn analysis_does_not_replace_incomplete_rubric_scores_with_acceptance_counts() {
        let snapshot = sample_completed_snapshot();
        let trace = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: snapshot.item.id.clone(),
            seq: 1,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "item_verdicts": [
                    { "item": "acceptance", "points": 1, "max_points": 1, "verdict": "pass" }
                ],
                "rubric_item_verdicts": [],
                "rubric_items_expected": 8,
                "rubric_items_recorded": 0,
                "rubric_complete": false
            }),
        }];

        assert!(insight_review_items(&snapshot.review_results[0], &trace, 85).is_empty());
    }

    #[test]
    fn insight_scope_uses_recorded_knowledge_versions_and_preserves_legacy_labels() {
        let snapshot = sample_completed_snapshot();
        let domains = vec![Domain {
            id: "general".to_string(),
            display_name: "汎用".to_string(),
            description: String::new(),
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            knowledge_version: 3,
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
            source: DomainSource::ProjectDomainDirectory,
        }];
        let artifact_types = vec![ArtifactType {
            id: "docs".to_string(),
            domain_id: Some("general".to_string()),
            display_name: "文書".to_string(),
            description: String::new(),
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            rubric_version: 2,
            definition_version: 4,
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
            source: nagare_core::ArtifactTypeSource::ProjectArtifactTypeDirectory,
        }];
        let recorded = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: snapshot.item.id.clone(),
            seq: 1,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "rubric_ref": { "version": 2 },
                "knowledge_refs": [
                    { "id": "general", "kind": "domain_knowledge", "version": 3 },
                    { "id": "docs", "kind": "artifact_definition", "version": 4 }
                ]
            }),
        }];

        let recorded_scope = insight_scope(
            &snapshot,
            &recorded,
            &[],
            &domains,
            &artifact_types,
            "worker",
        );
        assert_eq!(recorded_scope.knowledge_version_label, "汎用 v3 / 文書 v4");

        let legacy = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: snapshot.item.id.clone(),
            seq: 1,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "knowledge_refs": [
                    { "id": "general", "kind": "domain_knowledge" },
                    { "id": "docs", "kind": "artifact_definition" }
                ]
            }),
        }];
        let legacy_scope = insight_scope(
            &snapshot,
            &legacy,
            &[],
            &domains,
            &artifact_types,
            "worker",
        );
        assert_eq!(
            legacy_scope.knowledge_version_label,
            "汎用 版未記録 / 文書 版未記録"
        );
    }

    #[test]
    fn insights_compare_prompt_versions_without_mixing_fixed_conditions() {
        fn run_packet(work_id: &str, prompt_version: &str) -> nagare_core::ResolvedRunPacket {
            nagare_core::ResolvedRunPacket {
                id: format!("packet-{work_id}"),
                work_item_id: work_id.to_string(),
                agent_profile_id: "worker".to_string(),
                adapter_id: "process.codex-cli".to_string(),
                purpose: AgentRunPurpose::Work,
                working_dir: String::new(),
                goal: String::new(),
                prompt_version: prompt_version.to_string(),
                rubric_id: Some("docs".to_string()),
                rubric_version: Some(2),
                domain_knowledge_id: Some("general".to_string()),
                domain_knowledge_version: Some(3),
                artifact_definition_id: Some("docs".to_string()),
                artifact_definition_version: Some(4),
                path: None,
                work_folder: None,
                dispatch_plan_id: None,
                permission_policy_id: None,
                workspace_policy_id: None,
                resolved_skill_context_id: String::new(),
                output_contract: Default::default(),
                model: Default::default(),
                external: Default::default(),
                project_rule_ids: Vec::new(),
                constraints: Vec::new(),
                execution_record_uri: String::new(),
                content_hash: format!("hash-{work_id}"),
                locale: "ja".to_string(),
                created_at: "2026-07-05T00:01:00+09:00".to_string(),
            }
        }

        fn write_comparison_trace(
            root: &Path,
            work_id: &str,
            overall_score: u8,
            correctness_points: u8,
            clarity_points: u8,
        ) {
            let trace_path = root
                .join(".nagare")
                .join("works")
                .join(work_id)
                .join("trace.jsonl");
            fs::create_dir_all(trace_path.parent().expect("trace parent")).expect("trace dir");
            let records = [
                TraceRecord {
                    schema: "nagare.trace/1.0".to_string(),
                    record: "worker_output".to_string(),
                    work_id: work_id.to_string(),
                    seq: 1,
                    at: "2026-07-05T00:01:00+09:00".to_string(),
                    payload: serde_json::json!({
                        "agent": { "id": "worker", "name": "Worker", "role": "worker" },
                        "status": "completed"
                    }),
                },
                TraceRecord {
                    schema: "nagare.trace/1.0".to_string(),
                    record: "reviewer_verdict".to_string(),
                    work_id: work_id.to_string(),
                    seq: 2,
                    at: "2026-07-05T00:02:00+09:00".to_string(),
                    payload: serde_json::json!({
                        "agent": { "id": "reviewer", "name": "Reviewer", "role": "reviewer" },
                        "overall_score": overall_score,
                        "overall_max_score": 100,
                        "rubric_ref": { "id": "docs", "version": 2 },
                        "knowledge_refs": [
                            { "id": "general", "kind": "domain_knowledge", "version": 3 },
                            { "id": "docs", "kind": "artifact_definition", "version": 4 }
                        ],
                        "rubric_items_expected": 2,
                        "rubric_items_recorded": 2,
                        "rubric_complete": true,
                        "rubric_item_verdicts": [
                            {
                                "item": "正確性",
                                "points": correctness_points,
                                "max_points": 50,
                                "recorded": true,
                                "verdict": if correctness_points >= 40 { "pass" } else { "fail" },
                                "evidence": "正確性の根拠"
                            },
                            {
                                "item": "明瞭性",
                                "points": clarity_points,
                                "max_points": 50,
                                "recorded": true,
                                "verdict": if clarity_points >= 40 { "pass" } else { "fail" },
                                "evidence": "明瞭性の根拠"
                            }
                        ]
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

        let root = temp_test_dir("insights-prompt-comparison");
        init_project(&root).expect("init project");
        let mut v1 = sample_completed_snapshot();
        v1.item.id = "work_v1".to_string();
        v1.item.title = "Prompt v1".to_string();
        v1.resolved_run_packets.push(run_packet("work_v1", "v1"));
        let mut v2 = sample_completed_snapshot();
        v2.item.id = "work_v2".to_string();
        v2.item.title = "Prompt v2".to_string();
        v2.resolved_run_packets.push(run_packet("work_v2", "v2"));
        write_comparison_trace(&root, "work_v1", 15, 5, 10);
        write_comparison_trace(&root, "work_v2", 85, 45, 40);

        let domains = vec![Domain {
            id: "general".to_string(),
            display_name: "汎用".to_string(),
            description: String::new(),
            shared_knowledge: Vec::new(),
            common_rubric: Vec::new(),
            knowledge_version: 3,
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
            source: DomainSource::ProjectDomainDirectory,
        }];
        let artifact_types = vec![ArtifactType {
            id: "docs".to_string(),
            domain_id: Some("general".to_string()),
            display_name: "文書".to_string(),
            description: String::new(),
            artifact_types: Vec::new(),
            rubric: Vec::new(),
            rubric_version: 2,
            definition_version: 4,
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
            source: nagare_core::ArtifactTypeSource::ProjectArtifactTypeDirectory,
        }];

        let mut v2_other_project = v2.clone();
        v2_other_project.item.work_folder = Some("other-project".to_string());
        let separated = insights_view(
            &root,
            &[v1.clone(), v2_other_project],
            &[],
            &domains,
            &artifact_types,
        );
        assert!(separated.prompt_comparisons.is_empty());

        let insights = insights_view(&root, &[v1, v2], &[], &domains, &artifact_types);

        assert_eq!(insights.prompt_comparisons.len(), 1);
        let comparison = &insights.prompt_comparisons[0];
        assert_eq!(comparison.project_name, analysis_root_project_name(&root));
        assert_eq!(comparison.rubric_version_label, "v2");
        assert_eq!(comparison.knowledge_version_label, "汎用 v3 / 文書 v4");
        assert_eq!(comparison.variants.len(), 2);
        assert_eq!(comparison.variants[0].prompt_version_label, "v1");
        assert_eq!(comparison.variants[0].average_score, 15);
        assert_eq!(comparison.variants[0].work_refs[0].work_id, "work_v1");
        assert_eq!(comparison.variants[0].work_refs[0].score_label, "15点");
        assert_eq!(comparison.variants[0].items[0].score_label, "10%");
        assert_eq!(comparison.variants[1].prompt_version_label, "v2");
        assert_eq!(comparison.variants[1].average_score, 85);
        assert_eq!(comparison.variants[1].work_refs[0].work_id, "work_v2");
        assert_eq!(comparison.variants[1].items[0].score_label, "90%");
        assert!(insights
            .issue_matrix
            .iter()
            .any(|issue| issue.prompt_version_label == "v1"));
        assert!(insights
            .issue_matrix
            .iter()
            .any(|issue| issue.prompt_version_label == "v2"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_keep_every_recorded_rubric_dimension() {
        let root = temp_test_dir("insights-all-rubric-dimensions");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.review_results[0].criteria_results = (1..=9)
            .map(|index| CriteriaReviewResult {
                criterion: format!("Rubric dimension {index}"),
                status: CriteriaReviewStatus::Failed,
                note: format!("Dimension {index} is incomplete."),
            })
            .collect();

        let insights = insights_view(&root, &[snapshot], &[], &[], &[]);

        assert_eq!(insights.issue_matrix.len(), 9);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_group_same_target_issues_into_one_experiment() {
        let root = temp_test_dir("insights-group-target-issues");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        for criterion in &mut snapshot.review_results[0].criteria_results {
            criterion.status = CriteriaReviewStatus::Failed;
            criterion.note = format!("{} が不足しています。", criterion.criterion);
        }
        let mut repeated_snapshot = snapshot.clone();
        repeated_snapshot.item.id = "work_2".to_string();
        repeated_snapshot.item.title = "README更新の再検証".to_string();

        let insights = insights_view(
            &root,
            &[snapshot, repeated_snapshot],
            &[],
            &[],
            &[],
        );

        assert_eq!(insights.issue_matrix.len(), 2);
        assert_eq!(insights.proposal_count, 1);
        let proposal = insights.proposals.first().expect("proposal should exist");
        assert!(proposal.target_label.contains("目的への適合"));
        assert!(proposal.target_label.contains("読みやすさ"));
        assert!(proposal.summary.contains("一つの方策変更"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_do_not_mix_different_artifact_scopes() {
        let root = temp_test_dir("insights-artifact-scope");
        init_project(&root).expect("init project");
        let mut docs_snapshot = sample_completed_snapshot();
        docs_snapshot.review_results[0].criteria_results.truncate(1);
        docs_snapshot.review_results[0].criteria_results[0].status =
            CriteriaReviewStatus::Failed;
        let mut docs_repeat = docs_snapshot.clone();
        docs_repeat.item.id = "work_docs_2".to_string();
        let mut faq_snapshot = docs_snapshot.clone();
        faq_snapshot.item.id = "work_faq_1".to_string();
        faq_snapshot.item.artifact_type_id = Some("faq".to_string());
        let mut faq_repeat = faq_snapshot.clone();
        faq_repeat.item.id = "work_faq_2".to_string();

        let insights = insights_view(
            &root,
            &[docs_snapshot, docs_repeat, faq_snapshot, faq_repeat],
            &[],
            &[],
            &[],
        );

        assert_eq!(insights.issue_matrix.len(), 2);
        assert!(
            insights
                .issue_matrix
                .iter()
                .all(|issue| issue.occurrences == 2)
        );
        assert_eq!(insights.proposal_count, 2);
        assert_ne!(insights.proposals[0].id, insights.proposals[1].id);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn insights_start_with_agent_policy_even_when_issue_mentions_rubric() {
        assert_eq!(
            suggestion_kind_for_issue("判定基準の明確さ", 60),
            "プロンプト"
        );
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
            constraints: Some("禁止: 本番環境へ書き込まない\n通常の制約: README を対象にする".to_string()),
        })
        .expect("work should be created");

        assert_eq!(detail.domain_id, "general");
        assert_eq!(detail.artifact_type_id, "general");
        let snapshot = get_work_item_snapshot(&root, &detail.item.id).expect("work snapshot");
        assert_eq!(snapshot.item.domain_id.as_deref(), Some("general"));
        assert_eq!(snapshot.item.artifact_type_id.as_deref(), Some("general"));
        assert_eq!(snapshot.item.constraints.len(), 2);
        let gate = detail
            .prohibited_task_gate
            .as_ref()
            .expect("prohibited task gate");
        assert_eq!(gate.status, "pending");
        assert_eq!(gate.rules, vec!["禁止: 本番環境へ書き込まない"]);

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
            constraints: None,
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
            step.title == "レビュー"
                && step.actor == "Reviewer"
                && step.output == "92 / 100"
                && step.score_label == "92 / 100"
        }));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn work_detail_exposes_effective_agent_skill_and_mcp_scope() {
        let root = temp_test_dir("work-detail-effective-capabilities");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.resolved_skill_contexts.push(nagare_core::ResolvedSkillContext {
            id: "skillctx_1".to_string(),
            work_item_id: snapshot.item.id.clone(),
            agent_profile_id: "writer".to_string(),
            capability_probe_id: None,
            project_rule_ids: Vec::new(),
            declared_skill_set_ids: vec!["hachi-readable-writing".to_string()],
            applied_skill_set_ids: vec!["hachi-readable-writing".to_string()],
            skipped_skill_set_ids: Vec::new(),
            capabilities_in_force: Vec::new(),
            instruction_sources: Vec::new(),
            effective_skill_paths: vec!["C:/skills/hachi-readable-writing/SKILL.md".to_string()],
            codex_skill_config: vec![
                nagare_core::CodexSkillConfigEntry {
                    path: "C:/skills/hachi-readable-writing/SKILL.md".to_string(),
                    enabled: true,
                },
                nagare_core::CodexSkillConfigEntry {
                    path: "C:/skills/hachi-ui/SKILL.md".to_string(),
                    enabled: false,
                },
            ],
            scope_diagnostics: vec!["Codex strict Skill allowlist materialized: 1 allowed, 1 disabled".to_string()],
            execution_record_uri: String::new(),
            content_hash: String::new(),
            locale: "ja".to_string(),
            resolved_at: "1".to_string(),
        });

        let detail = work_detail_view(&root, snapshot);
        assert_eq!(detail.effective_capabilities.len(), 1);
        let capability = &detail.effective_capabilities[0];
        assert_eq!(capability.agent_id, "writer");
        assert_eq!(capability.skills, vec!["hachi-readable-writing"]);
        assert_eq!(capability.allowed_skill_count, 1);
        assert_eq!(capability.disabled_skill_count, 1);
        assert!(capability.mcp_connections.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn artifact_views_deduplicate_repeated_run_records() {
        let snapshot = sample_completed_snapshot();
        let artifact = snapshot.artifacts[0].clone();
        let mut another = artifact.clone();
        another.id = "artifact_2".to_string();
        another.title = "CHANGELOG.md".to_string();
        another.uri = "file:///CHANGELOG.md".to_string();

        let artifacts = artifact_views(&[artifact.clone(), artifact, another]);

        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].title, "README.md");
        assert_eq!(artifacts[1].title, "CHANGELOG.md");
    }

    #[test]
    fn review_score_label_does_not_infer_score_from_criteria_count() {
        let snapshot = sample_completed_snapshot();
        let records = vec![TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: snapshot.item.id.clone(),
            seq: 4,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "total_score": 6,
                "max_score": 6
            }),
        }];

        assert_eq!(
            review_score_label(&snapshot.review_results[0], &records),
            "得点未記録"
        );
    }

    #[test]
    fn zero_max_review_trace_uses_explicit_summary_score() {
        let payload = serde_json::json!({
            "summary": "100/100。質問に対して、二番目に高い山名・標高・所在地を簡潔に回答した。",
            "recommendation": "approve",
            "total_score": 0,
            "max_score": 0,
            "item_verdicts": []
        });

        assert_eq!(review_score_summary(&payload), Some("100 / 100".to_string()));
        assert_eq!(review_output_summary(&payload), "100 / 100");
        assert_eq!(review_rationale_summary(&payload), "");
    }

    #[test]
    fn review_trace_keeps_criteria_count_separate_when_overall_score_is_missing() {
        let payload = serde_json::json!({
            "summary": "Nagare Review",
            "recommendation": "revise",
            "total_score": 6,
            "max_score": 6
        });

        assert_eq!(review_score_summary(&payload), None);
        assert_eq!(review_criteria_summary(&payload), Some("評価項目 6 / 6".to_string()));
        assert_eq!(review_output_summary(&payload), "差し戻しを推奨");
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
                "agent": { "id": "organizer", "name": "Organizer", "role": "organizer" },
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

    #[test]
    fn reviewer_trace_exposes_recommendation_as_step_outcome() {
        let record = TraceRecord {
            schema: "nagare.trace/1.0".to_string(),
            record: "reviewer_verdict".to_string(),
            work_id: "work_1".to_string(),
            seq: 3,
            at: "2026-07-05T00:02:00+09:00".to_string(),
            payload: serde_json::json!({
                "step_kind": "review",
                "agent": { "id": "reviewer", "name": "Reviewer", "role": "reviewer" },
                "status": "completed",
                "recommendation": "revise",
                "summary": "保存処理の修正が必要です。総合 50/100。",
                "total_score": 3,
                "max_score": 6
            }),
        };

        let step = trace_step_view(&record);

        assert_eq!(step.state, "completed");
        assert_eq!(step.outcome, "revise");
        assert_eq!(step.score_label, "50 / 100");
        assert_eq!(step.criteria_label, "評価項目 3 / 6");
    }

    #[test]
    fn agent_usage_counts_include_each_visible_agent_step() {
        let records = vec![
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "organizer_decision".to_string(),
                work_id: "work_1".to_string(),
                seq: 1,
                at: "2026-07-05T00:00:00+09:00".to_string(),
                payload: serde_json::json!({ "agent": { "id": "software-organizer" } }),
            },
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "worker_output".to_string(),
                work_id: "work_1".to_string(),
                seq: 2,
                at: "2026-07-05T00:01:00+09:00".to_string(),
                payload: serde_json::json!({ "agent": { "id": "software-worker" } }),
            },
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "reviewer_verdict".to_string(),
                work_id: "work_1".to_string(),
                seq: 3,
                at: "2026-07-05T00:02:00+09:00".to_string(),
                payload: serde_json::json!({ "agent": { "id": "software-reviewer" } }),
            },
            TraceRecord {
                schema: "nagare.trace/1.0".to_string(),
                record: "artifact_recorded".to_string(),
                work_id: "work_1".to_string(),
                seq: 4,
                at: "2026-07-05T00:03:00+09:00".to_string(),
                payload: serde_json::json!({ "agent": { "id": "software-worker" } }),
            },
        ];
        let mut counts = BTreeMap::new();

        accumulate_agent_usage_counts(&mut counts, &records);

        assert_eq!(counts.get("software-organizer"), Some(&1));
        assert_eq!(counts.get("software-worker"), Some(&1));
        assert_eq!(counts.get("software-reviewer"), Some(&1));
    }

    #[test]
    fn completed_work_does_not_expose_stale_recovery_as_current_action() {
        let mut snapshot = sample_completed_snapshot();
        snapshot.recovery_plans.push(RecoveryPlan {
            id: "recovery_1".to_string(),
            work_item_id: snapshot.item.id.clone(),
            status: RecoveryPlanStatus::Draft,
            action: RecoveryAction::RerunSameAgent,
            target_agent_profile_id: Some("worker".to_string()),
            failure_class: "no_diff".to_string(),
            reason: "no_diff_artifact".to_string(),
            summary: "過去の回復案".to_string(),
            source_event_id: None,
            command_hint: Some("nagare item run work_1".to_string()),
            prompt_hint: None,
            warnings: Vec::new(),
            locale: "ja".to_string(),
            created_at: "2026-07-05T00:02:30+09:00".to_string(),
        });

        assert_eq!(snapshot.completion.next_action, "done");
        assert!(current_recovery(&snapshot).is_none());

        snapshot.completion.next_action = "recover".to_string();
        assert_eq!(
            current_recovery(&snapshot).map(|plan| plan.id.as_str()),
            Some("recovery_1")
        );
    }

    #[test]
    fn completed_work_does_not_reopen_a_historical_question() {
        let root = temp_test_dir("completed-question");
        init_project(&root).expect("init project");
        let mut snapshot = sample_completed_snapshot();
        snapshot.agent_outputs[0].questions = vec!["公開範囲を確認してください。".to_string()];

        let completed = work_detail_view(&root, snapshot.clone());
        assert!(completed.question.is_none());
        assert!(completed.question_source.is_empty());

        snapshot.item.status = WorkItemStatus::NeedsInput;
        snapshot.completion.next_action = "answer_question".to_string();
        let waiting = work_detail_view(&root, snapshot);
        assert_eq!(
            waiting.question.as_deref(),
            Some("公開範囲を確認してください。")
        );
        assert!(!waiting.question_source.is_empty());

        let _ = fs::remove_dir_all(root);
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
                    "agent": { "id": "organizer", "name": "オーガナイザー", "role": "organizer" },
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
                rubric_results: Vec::new(),
                rubric_expected_count: 0,
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
