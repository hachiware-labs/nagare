use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub nagare_dir: PathBuf,
    pub config_path: PathBuf,
    pub agents_dir: PathBuf,
    pub state_dir: PathBuf,
    pub ledger_path: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl ProjectLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let nagare_dir = root.join(".nagare");
        let state_dir = nagare_dir.join("state");
        Self {
            config_path: nagare_dir.join("project.toml"),
            agents_dir: nagare_dir.join("agents"),
            ledger_path: state_dir.join("ledger.json"),
            state_dir,
            artifacts_dir: nagare_dir.join("artifacts"),
            logs_dir: nagare_dir.join("logs"),
            nagare_dir,
            root,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitResult {
    pub layout: ProjectLayout,
    pub created_config: bool,
    pub created_ledger: bool,
}

pub fn init_project(root: impl Into<PathBuf>) -> io::Result<InitResult> {
    let layout = ProjectLayout::new(root);
    fs::create_dir_all(&layout.state_dir)?;
    fs::create_dir_all(&layout.artifacts_dir)?;
    fs::create_dir_all(&layout.logs_dir)?;
    fs::create_dir_all(&layout.agents_dir)?;

    let created_config = if layout.config_path.exists() {
        false
    } else {
        fs::write(&layout.config_path, default_config())?;
        true
    };

    let created_ledger = if layout.ledger_path.exists() {
        false
    } else {
        save_ledger(&layout, &Ledger::default())?;
        true
    };

    Ok(InitResult {
        layout,
        created_config,
        created_ledger,
    })
}

fn default_config() -> &'static str {
    r#"# Nagare local project configuration.

[project]
name = "nagare-local"

[storage]
kind = "json-ledger"
path = ".nagare/state/ledger.json"
sqlite_future_path = ".nagare/state/nagare.db"

[locale]
language = "ja-JP"
timezone = "Asia/Tokyo"

[nagare_agents]
work_agent = "codex-cli"
review_agent = "codex-app-server"
dispatch_agent = "codex-cli"

[runtimes.codex-local]
kind = "process"
command = "codex"
args = ["exec"]
healthcheck = ["codex", "--version"]

[runtimes.codex-app-local]
kind = "stdio"
command = "codex"
args = ["app-server", "--listen", "stdio://"]
healthcheck = ["codex", "app-server", "--help"]

[adapters.process-codex-cli]
kind = "process.codex-cli"
runtime_kind = "process"
known_capabilities = ["repo_read", "file_edit", "shell_command", "stdin_prompt"]

[adapters.stdio-codex-app-server]
kind = "stdio.codex-app-server"
runtime_kind = "stdio"
known_capabilities = ["repo_read", "file_edit", "shell_command", "thread_state", "approval_flow", "event_stream"]

[agent_profiles.codex-cli]
display_name = "Codex CLI Implementer"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "implementer"
working_dir = "."

[agent_profiles.codex-app-server]
display_name = "Codex App Server Implementer"
runtime = "codex-app-local"
adapter = "stdio-codex-app-server"
role = "implementer"
working_dir = "."

[skill_sets.repo-default]
paths = ["AGENTS.md"]
required_capabilities = ["repo_read"]
optional_capabilities = []

[permission_policies.medium-code-task]
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]

[workspace_policies.project-root]
kind = "project_root"
isolate_per_work_item = false
cleanup = "keep"

[[project_rules]]
id = "default"
match = ["**"]
default_agent = "codex-cli"
review_agent = "codex-app-server"
skill_sets = ["repo-default"]
permission_policy = "medium-code-task"
workspace_policy = "project-root"
verification = []
"#
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub root: PathBuf,
    pub has_git: bool,
    pub has_config: bool,
    pub has_ledger: bool,
    pub tools: Vec<ToolStatus>,
}

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub detail: String,
}

impl fmt::Display for ToolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.available { "ok" } else { "missing" };
        write!(f, "{}: {} ({})", self.name, marker, self.detail)
    }
}

pub fn doctor(root: impl Into<PathBuf>) -> DoctorReport {
    let root = root.into();
    let layout = ProjectLayout::new(&root);
    DoctorReport {
        has_git: root.join(".git").exists(),
        has_config: layout.config_path.exists(),
        has_ledger: layout.ledger_path.exists(),
        tools: vec![
            check_tool("git", &["--version"]),
            check_tool("node", &["--version"]),
            check_tool("npm", &["--version"]),
            check_tool("codex", &["--version"]),
            check_tool("codex", &["app-server", "--help"]),
        ],
        root,
    }
}

fn check_tool(name: &str, args: &[&str]) -> ToolStatus {
    match run_tool(name, args) {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = first_nonempty_line(&stdout)
                .or_else(|| first_nonempty_line(&stderr))
                .unwrap_or_else(|| "available".to_string());
            ToolStatus {
                name: name.to_string(),
                available: true,
                detail,
            }
        }
        Ok(output) => ToolStatus {
            name: name.to_string(),
            available: false,
            detail: format!("exit status {}", output.status),
        },
        Err(error) => ToolStatus {
            name: name.to_string(),
            available: false,
            detail: error.to_string(),
        },
    }
}

fn check_command(command: &str, args: &[String]) -> ToolStatus {
    let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
    check_tool(command, &borrowed)
}

fn run_tool(name: &str, args: &[&str]) -> io::Result<std::process::Output> {
    match Command::new(name).args(args).output() {
        Ok(output) => Ok(output),
        Err(error) if cfg!(windows) && error.kind() == io::ErrorKind::NotFound => {
            Command::new(format!("{name}.cmd")).args(args).output()
        }
        Err(error) => Err(error),
    }
}

fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

pub fn resolve_root(root_arg: Option<&str>) -> io::Result<PathBuf> {
    match root_arg {
        Some(root) => Ok(Path::new(root).to_path_buf()),
        None if env::var_os("NAGARE_ROOT").is_some() => Ok(PathBuf::from(
            env::var_os("NAGARE_ROOT").expect("checked above"),
        )),
        None => env::current_dir(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub next_seq: u64,
    #[serde(default)]
    pub work_items: Vec<WorkItem>,
    #[serde(default)]
    pub runs: Vec<AgentRun>,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub verification_results: Vec<VerificationResult>,
    #[serde(default)]
    pub handoffs: Vec<HandoffPacket>,
    #[serde(default)]
    pub decisions: Vec<HumanDecision>,
    #[serde(default)]
    pub capability_probes: Vec<CapabilityProbe>,
    #[serde(default)]
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    #[serde(default)]
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            next_seq: 1,
            work_items: Vec::new(),
            runs: Vec::new(),
            artifacts: Vec::new(),
            evidence: Vec::new(),
            verification_results: Vec::new(),
            handoffs: Vec::new(),
            decisions: Vec::new(),
            capability_probes: Vec::new(),
            resolved_skill_contexts: Vec::new(),
            resolved_run_packets: Vec::new(),
        }
    }
}

impl Ledger {
    fn next_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}_{:04}", self.next_seq);
        self.next_seq += 1;
        id
    }

    fn work_item_mut(&mut self, id: &str) -> Result<&mut WorkItem, NagareError> {
        self.work_items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| NagareError::NotFound(format!("work item `{id}`")))
    }

    fn work_item(&self, id: &str) -> Result<&WorkItem, NagareError> {
        self.work_items
            .iter()
            .find(|item| item.id == id)
            .ok_or_else(|| NagareError::NotFound(format!("work item `{id}`")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub status: WorkItemStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemStatus {
    Ready,
    AgentRunning,
    FailedVerification,
    NeedsHandoff,
    ReadyForReview,
    Done,
}

impl fmt::Display for WorkItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ready => "ready",
            Self::AgentRunning => "agent_running",
            Self::FailedVerification => "failed_verification",
            Self::NeedsHandoff => "needs_handoff",
            Self::ReadyForReview => "ready_for_review",
            Self::Done => "done",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub work_item_id: String,
    pub agent_profile_id: String,
    pub adapter: String,
    #[serde(default = "default_agent_run_purpose")]
    pub purpose: AgentRunPurpose,
    pub command: String,
    pub status: AgentRunStatus,
    pub exit_code: Option<i32>,
    pub started_at: String,
    pub ended_at: String,
    pub artifact_id: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Succeeded,
    Failed,
}

impl fmt::Display for AgentRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Succeeded => f.write_str("succeeded"),
            Self::Failed => f.write_str("failed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunPurpose {
    Work,
    DispatchPreview,
    Review,
}

impl fmt::Display for AgentRunPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Work => f.write_str("work"),
            Self::DispatchPreview => f.write_str("dispatch_preview"),
            Self::Review => f.write_str("review"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub work_item_id: String,
    pub agent_run_id: Option<String>,
    pub artifact_type: String,
    pub uri: String,
    pub title: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub work_item_id: String,
    pub claim: String,
    pub basis: String,
    pub artifact_id: Option<String>,
    pub produced_by: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub id: String,
    pub work_item_id: String,
    pub command: String,
    pub result: VerificationStatus,
    pub evidence_id: String,
    pub artifact_id: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub verified_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Passed,
    Failed,
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Passed => f.write_str("passed"),
            Self::Failed => f.write_str("failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPacket {
    pub id: String,
    pub work_item_id: String,
    pub from_agent_profile: String,
    pub to_agent_profile: String,
    pub reason: String,
    pub summary: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDecision {
    pub id: String,
    pub work_item_id: String,
    pub decision_type: String,
    pub rationale: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProbe {
    pub id: String,
    pub agent_profile_id: String,
    pub runtime_id: String,
    pub adapter_id: String,
    pub runtime_version: String,
    pub available: bool,
    pub discovered_capabilities: Vec<String>,
    pub instruction_sources: Vec<String>,
    pub supported_skill_modes: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub probed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedSkillContext {
    pub id: String,
    pub work_item_id: String,
    pub agent_profile_id: String,
    pub capability_probe_id: Option<String>,
    #[serde(default)]
    pub project_rule_ids: Vec<String>,
    #[serde(default)]
    pub declared_skill_set_ids: Vec<String>,
    #[serde(default)]
    pub applied_skill_set_ids: Vec<String>,
    #[serde(default)]
    pub skipped_skill_set_ids: Vec<String>,
    #[serde(default)]
    pub capabilities_in_force: Vec<String>,
    #[serde(default)]
    pub instruction_sources: Vec<String>,
    pub artifact_uri: String,
    pub content_hash: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub resolved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRunPacket {
    pub id: String,
    pub work_item_id: String,
    pub agent_profile_id: String,
    pub adapter_id: String,
    pub path: Option<String>,
    pub permission_policy_id: Option<String>,
    pub workspace_policy_id: Option<String>,
    pub resolved_skill_context_id: String,
    #[serde(default)]
    pub project_rule_ids: Vec<String>,
    #[serde(default)]
    pub verification: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub artifact_uri: String,
    pub content_hash: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CommandRunOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub display_name: String,
    pub runtime: String,
    pub adapter: String,
    pub role: String,
    pub working_dir: String,
    #[serde(skip)]
    pub source: AgentProfileSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentProfileSource {
    #[default]
    ProjectConfig,
    ProjectAgentDirectory,
}

impl fmt::Display for AgentProfileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectConfig => f.write_str("project_config"),
            Self::ProjectAgentDirectory => f.write_str("project_agent_directory"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddAgentProfileInput<'a> {
    pub id: &'a str,
    pub display_name: &'a str,
    pub runtime: &'a str,
    pub adapter: &'a str,
    pub role: &'a str,
    pub working_dir: &'a str,
}

#[derive(Debug, Clone)]
pub struct AddAgentProfileResult {
    pub profile: AgentProfile,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NagareAgentSettings {
    #[serde(default = "default_work_agent_id")]
    pub work_agent: String,
    #[serde(default = "default_review_agent_id")]
    pub review_agent: String,
    #[serde(default = "default_dispatch_agent_id")]
    pub dispatch_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleSettings {
    #[serde(default = "default_locale_language")]
    pub language: String,
    #[serde(default = "default_locale_timezone")]
    pub timezone: String,
}

impl Default for LocaleSettings {
    fn default() -> Self {
        Self {
            language: default_locale_language(),
            timezone: default_locale_timezone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetLocaleInput<'a> {
    pub language: Option<&'a str>,
    pub timezone: Option<&'a str>,
}

impl Default for NagareAgentSettings {
    fn default() -> Self {
        Self {
            work_agent: default_work_agent_id(),
            review_agent: default_review_agent_id(),
            dispatch_agent: default_dispatch_agent_id(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SetNagareAgentSettingsInput<'a> {
    pub work_agent: Option<&'a str>,
    pub review_agent: Option<&'a str>,
    pub dispatch_agent: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct AgentDoctorReport {
    pub profile: AgentProfile,
    pub runtime: RuntimeDeclaration,
    pub health: ToolStatus,
}

#[derive(Debug, Clone)]
pub struct AgentProbeResult {
    pub probe: CapabilityProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDeclaration {
    pub kind: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub healthcheck: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSetDeclaration {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub optional_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicyDeclaration {
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub disallowed_actions: Vec<String>,
    #[serde(default)]
    pub approval_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePolicyDeclaration {
    #[serde(default = "default_workspace_policy_kind")]
    pub kind: String,
    #[serde(default)]
    pub isolate_per_work_item: bool,
    #[serde(default = "default_workspace_policy_cleanup")]
    pub cleanup: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRule {
    pub id: String,
    #[serde(default, rename = "match")]
    pub match_patterns: Vec<String>,
    #[serde(default)]
    pub default_agent: Option<String>,
    #[serde(default)]
    pub review_agent: Option<String>,
    #[serde(default)]
    pub skill_sets: Vec<String>,
    #[serde(default)]
    pub permission_policy: Option<String>,
    #[serde(default)]
    pub workspace_policy: Option<String>,
    #[serde(default)]
    pub verification: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuleResolution {
    pub path: Option<String>,
    pub matched_rule_id: Option<String>,
    pub agent_profile_id: String,
    pub review_agent_profile_id: String,
    pub skill_set_ids: Vec<String>,
    pub permission_policy_id: Option<String>,
    pub workspace_policy_id: Option<String>,
    pub verification: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RunWorkItemInput<'a> {
    pub agent_profile_id: &'a str,
    pub path: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub dev_command: Option<&'a str>,
    pub purpose: AgentRunPurpose,
}

#[derive(Debug, Clone)]
pub struct RunPacket {
    pub id: String,
    pub work_item_id: String,
    pub agent_profile_id: String,
    pub adapter_id: String,
    pub working_dir: String,
    pub goal: String,
}

#[derive(Debug, Clone)]
pub struct AdapterRunRequest<'a> {
    pub working_dir: &'a Path,
    pub run_packet: &'a RunPacket,
    pub prompt: &'a str,
    pub dev_command: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct AdapterRunOutput {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

trait AgentAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError>;
}

struct ProcessCodexCliAdapter;

impl AgentAdapter for ProcessCodexCliAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError> {
        if let Some(command) = request.dev_command {
            return run_dev_command(command, request.working_dir);
        }

        let output = run_tool(
            "codex",
            &[
                "exec",
                "--cd",
                &request.working_dir.display().to_string(),
                request.prompt,
            ],
        )?;
        Ok(AdapterRunOutput {
            command: format!("codex exec --cd {} <prompt>", request.working_dir.display()),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }
}

struct StdioCodexAppServerAdapter;

impl AgentAdapter for StdioCodexAppServerAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError> {
        if let Some(command) = request.dev_command {
            return run_dev_command(command, request.working_dir);
        }

        Err(NagareError::InvalidState(
            "`stdio.codex-app-server` adapter is registered but not implemented yet; use `codex-cli` or pass --command for the smoke path".to_string(),
        ))
    }
}

#[derive(Debug)]
pub enum NagareError {
    Io(io::Error),
    Json(serde_json::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    NotFound(String),
    InvalidState(String),
}

impl fmt::Display for NagareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::TomlDe(error) => write!(f, "{error}"),
            Self::TomlSer(error) => write!(f, "{error}"),
            Self::NotFound(value) => write!(f, "not found: {value}"),
            Self::InvalidState(value) => write!(f, "{value}"),
        }
    }
}

impl std::error::Error for NagareError {}

impl From<io::Error> for NagareError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for NagareError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<toml::de::Error> for NagareError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDe(value)
    }
}

impl From<toml::ser::Error> for NagareError {
    fn from(value: toml::ser::Error) -> Self {
        Self::TomlSer(value)
    }
}

#[derive(Debug, Clone)]
pub struct CreateItemResult {
    pub item: WorkItem,
}

#[derive(Debug, Clone)]
pub struct RunWorkItemResult {
    pub run: AgentRun,
    pub evidence_id: String,
    pub item_status: WorkItemStatus,
}

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub verification: VerificationResult,
    pub item_status: WorkItemStatus,
}

#[derive(Debug, Clone)]
pub struct HandoffResult {
    pub handoff: HandoffPacket,
}

#[derive(Debug, Clone)]
pub struct DecisionResult {
    pub decision: HumanDecision,
    pub item_status: WorkItemStatus,
}

#[derive(Debug, Clone)]
pub struct ScenarioResult {
    pub work_item_id: String,
    pub codex_run_id: String,
    pub handoff_id: String,
    pub codex_app_run_id: String,
    pub verification_id: String,
    pub decision_id: String,
    pub final_status: WorkItemStatus,
}

pub fn load_ledger(layout: &ProjectLayout) -> Result<Ledger, NagareError> {
    if !layout.ledger_path.exists() {
        return Ok(Ledger::default());
    }
    let raw = fs::read_to_string(&layout.ledger_path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_ledger(layout: &ProjectLayout, ledger: &Ledger) -> io::Result<()> {
    fs::create_dir_all(&layout.state_dir)?;
    let raw = serde_json::to_string_pretty(ledger).map_err(io::Error::other)?;
    fs::write(&layout.ledger_path, format!("{raw}\n"))
}

pub fn list_agent_profiles(root: impl Into<PathBuf>) -> Result<Vec<AgentProfile>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_agent_profiles(&layout)?.into_values().collect())
}

pub fn get_agent_profile(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentProfile, NagareError> {
    let layout = ensure_project(root)?;
    get_agent_profile_from_layout(&layout, agent_profile_id)
}

pub fn add_agent_profile(
    root: impl Into<PathBuf>,
    input: AddAgentProfileInput<'_>,
) -> Result<AddAgentProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(input.id)?;

    let mut existing = load_agent_profiles(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "agent profile `{}` already exists",
            input.id
        )));
    }

    let adapter = normalize_adapter_id(input.adapter)?;
    let profile = AgentProfile {
        id: input.id.to_string(),
        display_name: if input.display_name.trim().is_empty() {
            input.id.to_string()
        } else {
            input.display_name.to_string()
        },
        runtime: input.runtime.to_string(),
        adapter: adapter.to_string(),
        role: if input.role.trim().is_empty() {
            "implementer".to_string()
        } else {
            input.role.to_string()
        },
        working_dir: normalize_working_dir(input.working_dir)?,
        source: AgentProfileSource::ProjectAgentDirectory,
    };
    existing.insert(profile.id.clone(), profile.clone());

    fs::create_dir_all(&layout.agents_dir)?;
    let path = layout.agents_dir.join(format!("{}.toml", profile.id));
    let document = AgentProfileFile {
        agent_profile: Some(AgentProfileFileEntry {
            id: Some(profile.id.clone()),
            display_name: profile.display_name.clone(),
            runtime: profile.runtime.clone(),
            adapter: profile.adapter.clone(),
            role: profile.role.clone(),
            working_dir: profile.working_dir.clone(),
        }),
        agent_profiles: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;

    Ok(AddAgentProfileResult { profile, path })
}

pub fn get_nagare_agent_settings(
    root: impl Into<PathBuf>,
) -> Result<NagareAgentSettings, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_project_config(&layout)?.nagare_agents)
}

pub fn set_nagare_agent_settings(
    root: impl Into<PathBuf>,
    input: SetNagareAgentSettingsInput<'_>,
) -> Result<NagareAgentSettings, NagareError> {
    let layout = ensure_project(root)?;
    let mut settings = get_nagare_agent_settings(&layout.root)?;

    if let Some(agent) = input.work_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.work_agent = agent.to_string();
    }
    if let Some(agent) = input.review_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.review_agent = agent.to_string();
    }
    if let Some(agent) = input.dispatch_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.dispatch_agent = agent.to_string();
    }

    write_nagare_agent_settings(&layout, &settings)?;
    Ok(settings)
}

pub fn get_locale_settings(root: impl Into<PathBuf>) -> Result<LocaleSettings, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_project_config(&layout)?.locale)
}

pub fn resolve_rule_for_path(
    root: impl Into<PathBuf>,
    path: Option<&str>,
    agent_override: Option<&str>,
) -> Result<RuleResolution, NagareError> {
    let layout = ensure_project(root)?;
    resolve_rule_for_path_from_layout(&layout, path, agent_override)
}

pub fn set_locale_settings(
    root: impl Into<PathBuf>,
    input: SetLocaleInput<'_>,
) -> Result<LocaleSettings, NagareError> {
    let layout = ensure_project(root)?;
    let mut settings = get_locale_settings(&layout.root)?;
    if let Some(language) = input.language {
        validate_locale_language(language)?;
        settings.language = language.to_string();
    }
    if let Some(timezone) = input.timezone {
        validate_timezone(timezone)?;
        settings.timezone = timezone.to_string();
    }
    write_locale_settings(&layout, &settings)?;
    Ok(settings)
}

pub fn agent_doctor(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentDoctorReport, NagareError> {
    let layout = ensure_project(root)?;
    let profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    let runtime = get_runtime_declaration(&layout, &profile.runtime)?;
    let health = runtime_healthcheck(&runtime);
    Ok(AgentDoctorReport {
        profile,
        runtime,
        health,
    })
}

pub fn agent_probe(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentProbeResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    let runtime = get_runtime_declaration(&layout, &profile.runtime)?;
    let health = runtime_healthcheck(&runtime);
    let mut ledger = load_ledger(&layout)?;
    let probe = CapabilityProbe {
        id: ledger.next_id("probe"),
        agent_profile_id: profile.id,
        runtime_id: profile.runtime,
        adapter_id: normalize_adapter_id(&profile.adapter)?.to_string(),
        runtime_version: health.detail.clone(),
        available: health.available,
        discovered_capabilities: capabilities_for_adapter(&profile.adapter)?,
        instruction_sources: instruction_sources(&layout),
        supported_skill_modes: skill_modes_for_adapter(&profile.adapter)?,
        warnings: if health.available {
            Vec::new()
        } else {
            vec![format!("runtime healthcheck failed: {}", health.detail)]
        },
        locale,
        probed_at: timestamp(),
    };
    ledger.capability_probes.push(probe.clone());
    save_ledger(&layout, &ledger)?;
    Ok(AgentProbeResult { probe })
}

pub fn create_work_item(
    root: impl Into<PathBuf>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Result<CreateItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let now = timestamp();
    let item = WorkItem {
        id: ledger.next_id("work"),
        title: title.into(),
        description: description.into(),
        locale,
        status: WorkItemStatus::Ready,
        created_at: now.clone(),
        updated_at: now,
    };
    ledger.work_items.push(item.clone());
    save_ledger(&layout, &ledger)?;
    Ok(CreateItemResult { item })
}

pub fn list_work_items(root: impl Into<PathBuf>) -> Result<Vec<WorkItem>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_ledger(&layout)?.work_items)
}

pub fn get_work_item_snapshot(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<WorkItemSnapshot, NagareError> {
    let layout = ensure_project(root)?;
    let ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    Ok(WorkItemSnapshot::from_ledger(item, &ledger))
}

#[derive(Debug, Clone)]
pub struct WorkItemSnapshot {
    pub item: WorkItem,
    pub runs: Vec<AgentRun>,
    pub artifacts: Vec<Artifact>,
    pub evidence: Vec<Evidence>,
    pub verification_results: Vec<VerificationResult>,
    pub handoffs: Vec<HandoffPacket>,
    pub decisions: Vec<HumanDecision>,
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
}

impl WorkItemSnapshot {
    fn from_ledger(item: WorkItem, ledger: &Ledger) -> Self {
        let item_id = &item.id;
        Self {
            runs: ledger
                .runs
                .iter()
                .filter(|run| &run.work_item_id == item_id)
                .cloned()
                .collect(),
            artifacts: ledger
                .artifacts
                .iter()
                .filter(|artifact| &artifact.work_item_id == item_id)
                .cloned()
                .collect(),
            evidence: ledger
                .evidence
                .iter()
                .filter(|evidence| &evidence.work_item_id == item_id)
                .cloned()
                .collect(),
            verification_results: ledger
                .verification_results
                .iter()
                .filter(|verification| &verification.work_item_id == item_id)
                .cloned()
                .collect(),
            handoffs: ledger
                .handoffs
                .iter()
                .filter(|handoff| &handoff.work_item_id == item_id)
                .cloned()
                .collect(),
            decisions: ledger
                .decisions
                .iter()
                .filter(|decision| &decision.work_item_id == item_id)
                .cloned()
                .collect(),
            resolved_skill_contexts: ledger
                .resolved_skill_contexts
                .iter()
                .filter(|context| &context.work_item_id == item_id)
                .cloned()
                .collect(),
            resolved_run_packets: ledger
                .resolved_run_packets
                .iter()
                .filter(|packet| &packet.work_item_id == item_id)
                .cloned()
                .collect(),
            item,
        }
    }
}

pub fn run_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    agent_profile_id: &str,
    command: &str,
) -> Result<RunWorkItemResult, NagareError> {
    run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::Work,
        },
    )
}

pub fn run_work_item_with_input(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: RunWorkItemInput<'_>,
) -> Result<RunWorkItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();

    if input.purpose == AgentRunPurpose::Work {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::AgentRunning;
        item.updated_at = timestamp();
    }

    let run_id = ledger.next_id("run");
    let artifact_id = ledger.next_id("art");
    let evidence_id = ledger.next_id("ev");
    let run_packet_id = ledger.next_id("runpkt");
    let skill_context_id = ledger.next_id("skillctx");
    let agent_profile = get_agent_profile_from_layout(&layout, input.agent_profile_id)?;
    let adapter_id = normalize_adapter_id(&agent_profile.adapter)?;
    let working_dir = resolve_profile_working_dir(&layout, &agent_profile)?;
    let rule_resolution =
        resolve_rule_for_path_from_layout(&layout, input.path, Some(input.agent_profile_id))?;
    let capability_probe = latest_capability_probe(&ledger, input.agent_profile_id);
    let capabilities_in_force = capability_probe
        .map(|probe| probe.discovered_capabilities.clone())
        .unwrap_or_else(|| capabilities_for_adapter(adapter_id).unwrap_or_default());
    let instruction_sources = capability_probe
        .map(|probe| probe.instruction_sources.clone())
        .unwrap_or_else(|| instruction_sources(&layout));
    let goal = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(&item.title)
        .to_string();
    let run_packet = RunPacket {
        id: run_packet_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        adapter_id: adapter_id.to_string(),
        working_dir: path_uri(&working_dir),
        goal: goal.clone(),
    };
    let resolved_skill_context = ResolvedSkillContext {
        id: skill_context_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        capability_probe_id: capability_probe.map(|probe| probe.id.clone()),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        declared_skill_set_ids: rule_resolution.skill_set_ids.clone(),
        applied_skill_set_ids: rule_resolution.skill_set_ids.clone(),
        skipped_skill_set_ids: Vec::new(),
        capabilities_in_force,
        instruction_sources,
        artifact_uri: path_uri(
            &layout
                .artifacts_dir
                .join(format!("{skill_context_id}.json")),
        ),
        content_hash: format!("local:{}", skill_context_id),
        locale: locale.clone(),
        resolved_at: timestamp(),
    };
    let resolved_run_packet = ResolvedRunPacket {
        id: run_packet_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        adapter_id: adapter_id.to_string(),
        path: rule_resolution.path.clone(),
        permission_policy_id: rule_resolution.permission_policy_id.clone(),
        workspace_policy_id: rule_resolution.workspace_policy_id.clone(),
        resolved_skill_context_id: skill_context_id.clone(),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        verification: rule_resolution.verification.clone(),
        constraints: rule_resolution.warnings.clone(),
        artifact_uri: path_uri(&layout.artifacts_dir.join(format!("{run_packet_id}.json"))),
        content_hash: format!("local:{}", run_packet_id),
        locale: locale.clone(),
        created_at: timestamp(),
    };
    let prompt = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(goal.as_str());
    let request = AdapterRunRequest {
        working_dir: &working_dir,
        run_packet: &run_packet,
        prompt,
        dev_command: input.dev_command,
    };
    let started_at = timestamp();
    let output = adapter_for_id(adapter_id)?.run(&request)?;
    let ended_at = timestamp();
    let status = if output.exit_code == Some(0) {
        AgentRunStatus::Succeeded
    } else {
        AgentRunStatus::Failed
    };

    let log_path = layout.logs_dir.join(format!("{run_id}.log"));
    write_adapter_log(&log_path, &run_packet, &output)?;

    let artifact = Artifact {
        id: artifact_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_run_id: Some(run_id.clone()),
        artifact_type: "run_log".to_string(),
        uri: path_uri(&log_path),
        title: format!("{} {} log", input.agent_profile_id, input.purpose),
        locale: locale.clone(),
        created_at: ended_at.clone(),
    };
    let evidence = Evidence {
        id: evidence_id.clone(),
        work_item_id: work_item_id.to_string(),
        claim: agent_run_claim(&locale, input.purpose, status, input.agent_profile_id),
        basis: command_exit_basis(&locale, output.exit_code),
        artifact_id: Some(artifact_id.clone()),
        produced_by: input.agent_profile_id.to_string(),
        locale: locale.clone(),
        created_at: ended_at.clone(),
    };
    let run = AgentRun {
        id: run_id,
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        adapter: adapter_id.to_string(),
        purpose: input.purpose,
        command: output.command,
        status,
        exit_code: output.exit_code,
        started_at,
        ended_at,
        artifact_id,
        locale,
    };
    let item_status = if input.purpose == AgentRunPurpose::Work {
        if status == AgentRunStatus::Succeeded {
            WorkItemStatus::ReadyForReview
        } else {
            WorkItemStatus::FailedVerification
        }
    } else {
        item.status
    };

    ledger.runs.push(run.clone());
    ledger.artifacts.push(artifact);
    ledger.evidence.push(evidence);
    ledger
        .resolved_skill_contexts
        .push(resolved_skill_context.clone());
    ledger
        .resolved_run_packets
        .push(resolved_run_packet.clone());
    write_json_artifact(
        &layout,
        &format!("{}.json", resolved_skill_context.id),
        &resolved_skill_context,
    )?;
    write_json_artifact(
        &layout,
        &format!("{}.json", resolved_run_packet.id),
        &resolved_run_packet,
    )?;
    if input.purpose == AgentRunPurpose::Work {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = item_status;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;

    Ok(RunWorkItemResult {
        run,
        evidence_id,
        item_status,
    })
}

pub fn verify_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    command: &str,
) -> Result<VerifyResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let _ = ledger.work_item(work_item_id)?;

    let artifact_id = ledger.next_id("art");
    let evidence_id = ledger.next_id("ev");
    let verification_id = ledger.next_id("ver");
    let output = run_shell(command)?;
    let verified_at = timestamp();
    let log_path = layout.logs_dir.join(format!("{verification_id}.log"));
    write_command_log(&log_path, command, &output)?;

    let result = if output.exit_code == Some(0) {
        VerificationStatus::Passed
    } else {
        VerificationStatus::Failed
    };
    let item_status = if result == VerificationStatus::Passed {
        WorkItemStatus::ReadyForReview
    } else {
        WorkItemStatus::FailedVerification
    };
    let artifact = Artifact {
        id: artifact_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_run_id: None,
        artifact_type: "verification_log".to_string(),
        uri: path_uri(&log_path),
        title: localized_text(&locale, "verification log", "検証ログ").to_string(),
        locale: locale.clone(),
        created_at: verified_at.clone(),
    };
    let evidence = Evidence {
        id: evidence_id.clone(),
        work_item_id: work_item_id.to_string(),
        claim: verification_claim(&locale, result),
        basis: verification_basis(&locale, command, output.exit_code),
        artifact_id: Some(artifact_id.clone()),
        produced_by: "verification".to_string(),
        locale: locale.clone(),
        created_at: verified_at.clone(),
    };
    let verification = VerificationResult {
        id: verification_id,
        work_item_id: work_item_id.to_string(),
        command: command.to_string(),
        result,
        evidence_id,
        artifact_id,
        locale,
        verified_at,
    };

    ledger.artifacts.push(artifact);
    ledger.evidence.push(evidence);
    ledger.verification_results.push(verification.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = item_status;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;

    Ok(VerifyResult {
        verification,
        item_status,
    })
}

pub fn create_handoff(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    from_agent_profile: &str,
    to_agent_profile: &str,
    reason: &str,
    summary: &str,
) -> Result<HandoffResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let _ = ledger.work_item(work_item_id)?;
    let handoff = HandoffPacket {
        id: ledger.next_id("handoff"),
        work_item_id: work_item_id.to_string(),
        from_agent_profile: from_agent_profile.to_string(),
        to_agent_profile: to_agent_profile.to_string(),
        reason: reason.to_string(),
        summary: summary.to_string(),
        locale,
        created_at: timestamp(),
    };
    ledger.handoffs.push(handoff.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::NeedsHandoff;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;
    Ok(HandoffResult { handoff })
}

pub fn approve_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    rationale: &str,
) -> Result<DecisionResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?;
    if item.status != WorkItemStatus::ReadyForReview {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` must be ready_for_review before approval; current status is {}",
            item.status
        )));
    }
    let has_passing_verification = ledger.verification_results.iter().any(|verification| {
        verification.work_item_id == work_item_id
            && verification.result == VerificationStatus::Passed
    });
    if !has_passing_verification {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` needs a passing verification before approval"
        )));
    }

    let decision = HumanDecision {
        id: ledger.next_id("dec"),
        work_item_id: work_item_id.to_string(),
        decision_type: "approve".to_string(),
        rationale: if rationale.trim().is_empty() {
            default_approval_rationale(&locale).to_string()
        } else {
            rationale.to_string()
        },
        locale,
        created_at: timestamp(),
    };
    ledger.decisions.push(decision.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::Done;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;
    Ok(DecisionResult {
        decision,
        item_status: WorkItemStatus::Done,
    })
}

pub fn run_first_scenario(root: impl Into<PathBuf>) -> Result<ScenarioResult, NagareError> {
    let root = root.into();
    init_project(&root)?;
    let item = create_work_item(
        &root,
        "Repair failing agent run",
        "Demonstrate Codex CLI failure, Codex App Server handoff, verification, and approval.",
    )?
    .item;
    let codex_run = run_work_item(
        &root,
        &item.id,
        "codex-cli",
        scenario_command("codex attempt failed", false).as_str(),
    )?
    .run;
    let handoff = create_handoff(
        &root,
        &item.id,
        "codex-cli",
        "codex-app-server",
        "Codex agent profile produced a failing run",
        "Retry with Codex App Server agent profile using the captured run log as evidence.",
    )?
    .handoff;
    let codex_app_run = run_work_item(
        &root,
        &item.id,
        "codex-app-server",
        scenario_command("codex app server retry fixed the task", true).as_str(),
    )?
    .run;
    let verification = verify_work_item(
        &root,
        &item.id,
        scenario_command("verification passed", true).as_str(),
    )?
    .verification;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required verification passed after cross-agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        verification_id: verification.id,
        decision_id: decision.id,
        final_status,
    })
}

pub fn run_registered_agent_scenario(
    root: impl Into<PathBuf>,
) -> Result<ScenarioResult, NagareError> {
    let root = root.into();
    init_project(&root)?;
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-impl-smoke",
            display_name: "Codex CLI Smoke Implementer",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
        },
    )?;
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-app-smoke",
            display_name: "Codex App Server Smoke Implementer",
            runtime: "codex-app-local",
            adapter: "stdio.codex-app-server",
            role: "implementer",
            working_dir: ".",
        },
    )?;

    let item = create_work_item(
        &root,
        "Repair failing registered agent run",
        "Demonstrate registered Agent Profiles, handoff, verification, and approval.",
    )?
    .item;
    let codex_run = run_work_item(
        &root,
        &item.id,
        "codex-impl-smoke",
        scenario_command("registered codex attempt failed", false).as_str(),
    )?
    .run;
    let handoff = create_handoff(
        &root,
        &item.id,
        "codex-impl-smoke",
        "codex-app-smoke",
        "Registered Codex agent profile produced a failing run",
        "Retry with the registered Codex App Server profile using the captured run log as evidence.",
    )?
    .handoff;
    let codex_app_run = run_work_item(
        &root,
        &item.id,
        "codex-app-smoke",
        scenario_command("registered codex app server retry fixed the task", true).as_str(),
    )?
    .run;
    let verification = verify_work_item(
        &root,
        &item.id,
        scenario_command("registered verification passed", true).as_str(),
    )?
    .verification;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required verification passed after registered agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        verification_id: verification.id,
        decision_id: decision.id,
        final_status,
    })
}

fn ensure_project(root: impl Into<PathBuf>) -> Result<ProjectLayout, NagareError> {
    let layout = ProjectLayout::new(root);
    if !layout.config_path.exists() || !layout.ledger_path.exists() {
        init_project(&layout.root)?;
    }
    Ok(layout)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentProfileFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent_profile: Option<AgentProfileFileEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    agent_profiles: BTreeMap<String, AgentProfileFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectConfigFile {
    #[serde(default)]
    locale: LocaleSettings,
    #[serde(default)]
    nagare_agents: NagareAgentSettings,
    #[serde(default)]
    runtimes: BTreeMap<String, RuntimeDeclaration>,
    #[serde(default)]
    skill_sets: BTreeMap<String, SkillSetDeclaration>,
    #[serde(default)]
    permission_policies: BTreeMap<String, PermissionPolicyDeclaration>,
    #[serde(default)]
    workspace_policies: BTreeMap<String, WorkspacePolicyDeclaration>,
    #[serde(default)]
    project_rules: Vec<ProjectRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentProfileFileEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    display_name: String,
    runtime: String,
    adapter: String,
    role: String,
    #[serde(default = "default_working_dir")]
    working_dir: String,
}

fn load_agent_profiles(
    layout: &ProjectLayout,
) -> Result<BTreeMap<String, AgentProfile>, NagareError> {
    let mut profiles = BTreeMap::new();
    if layout.config_path.exists() {
        let raw = fs::read_to_string(&layout.config_path)?;
        merge_agent_profiles_from_toml(
            &mut profiles,
            &raw,
            AgentProfileSource::ProjectConfig,
            "project.toml",
        )?;
    }

    if layout.agents_dir.exists() {
        let mut paths = fs::read_dir(&layout.agents_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let raw = fs::read_to_string(&path)?;
            merge_agent_profiles_from_toml(
                &mut profiles,
                &raw,
                AgentProfileSource::ProjectAgentDirectory,
                &path.display().to_string(),
            )?;
        }
    }

    Ok(profiles)
}

fn merge_agent_profiles_from_toml(
    profiles: &mut BTreeMap<String, AgentProfile>,
    raw: &str,
    source: AgentProfileSource,
    source_name: &str,
) -> Result<(), NagareError> {
    let document: AgentProfileFile = toml::from_str(raw)?;
    if let Some(entry) = document.agent_profile {
        let id = entry.id.clone().ok_or_else(|| {
            NagareError::InvalidState(format!("`agent_profile.id` is required in {source_name}"))
        })?;
        profiles.insert(id.clone(), entry.into_profile(id, source)?);
    }
    for (id, entry) in document.agent_profiles {
        let profile_id = entry.id.clone().unwrap_or(id);
        profiles.insert(profile_id.clone(), entry.into_profile(profile_id, source)?);
    }
    Ok(())
}

impl AgentProfileFileEntry {
    fn into_profile(
        self,
        id: String,
        source: AgentProfileSource,
    ) -> Result<AgentProfile, NagareError> {
        validate_agent_profile_id(&id)?;
        let adapter = normalize_adapter_id(&self.adapter)?;
        Ok(AgentProfile {
            id,
            display_name: self.display_name,
            runtime: self.runtime,
            adapter: adapter.to_string(),
            role: self.role,
            working_dir: normalize_working_dir(&self.working_dir)?,
            source,
        })
    }
}

fn default_working_dir() -> String {
    ".".to_string()
}

fn get_agent_profile_from_layout(
    layout: &ProjectLayout,
    agent_profile_id: &str,
) -> Result<AgentProfile, NagareError> {
    load_agent_profiles(layout)?
        .remove(agent_profile_id)
        .ok_or_else(|| NagareError::NotFound(format!("agent profile `{agent_profile_id}`")))
}

fn get_runtime_declaration(
    layout: &ProjectLayout,
    runtime_id: &str,
) -> Result<RuntimeDeclaration, NagareError> {
    let document = load_project_config(layout)?;
    document
        .runtimes
        .get(runtime_id)
        .cloned()
        .ok_or_else(|| NagareError::NotFound(format!("runtime `{runtime_id}`")))
}

fn load_project_config(layout: &ProjectLayout) -> Result<ProjectConfigFile, NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    Ok(toml::from_str(&raw)?)
}

fn resolve_rule_for_path_from_layout(
    layout: &ProjectLayout,
    path: Option<&str>,
    agent_override: Option<&str>,
) -> Result<RuleResolution, NagareError> {
    let config = load_project_config(layout)?;
    let path = path
        .map(normalize_rule_path)
        .filter(|value| !value.trim().is_empty());
    let matched_rule = match path.as_deref() {
        Some(path) => best_matching_project_rule(&config.project_rules, path)?,
        None => None,
    };

    let agent_profile_id = agent_override
        .filter(|agent| !agent.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| matched_rule.and_then(|rule| rule.default_agent.clone()))
        .unwrap_or_else(|| config.nagare_agents.work_agent.clone());
    let review_agent_profile_id = matched_rule
        .and_then(|rule| rule.review_agent.clone())
        .unwrap_or_else(|| config.nagare_agents.review_agent.clone());

    validate_existing_agent_profile(layout, &agent_profile_id)?;
    validate_existing_agent_profile(layout, &review_agent_profile_id)?;

    let mut warnings = Vec::new();
    let skill_set_ids = matched_rule
        .map(|rule| rule.skill_sets.clone())
        .unwrap_or_default();
    for skill_set_id in &skill_set_ids {
        if !config.skill_sets.contains_key(skill_set_id) {
            warnings.push(format!(
                "skill set `{skill_set_id}` is referenced but not declared"
            ));
        }
    }

    let permission_policy_id = matched_rule.and_then(|rule| rule.permission_policy.clone());
    if let Some(policy_id) = &permission_policy_id {
        if !config.permission_policies.contains_key(policy_id) {
            warnings.push(format!(
                "permission policy `{policy_id}` is referenced but not declared"
            ));
        }
    }

    let workspace_policy_id = matched_rule.and_then(|rule| rule.workspace_policy.clone());
    if let Some(policy_id) = &workspace_policy_id {
        if !config.workspace_policies.contains_key(policy_id) {
            warnings.push(format!(
                "workspace policy `{policy_id}` is referenced but not declared"
            ));
        }
    }

    Ok(RuleResolution {
        path,
        matched_rule_id: matched_rule.map(|rule| rule.id.clone()),
        agent_profile_id,
        review_agent_profile_id,
        skill_set_ids,
        permission_policy_id,
        workspace_policy_id,
        verification: matched_rule
            .map(|rule| rule.verification.clone())
            .unwrap_or_default(),
        warnings,
    })
}

fn best_matching_project_rule<'a>(
    rules: &'a [ProjectRule],
    target_path: &str,
) -> Result<Option<&'a ProjectRule>, NagareError> {
    let mut best: Option<(&ProjectRule, usize)> = None;
    for rule in rules {
        let Some(score) = rule_match_score(rule, target_path) else {
            continue;
        };
        match best {
            Some((current, current_score)) if score == current_score && current.id != rule.id => {
                return Err(NagareError::InvalidState(format!(
                    "project rules `{}` and `{}` both match `{target_path}` with equal specificity",
                    current.id, rule.id
                )));
            }
            Some((_, current_score)) if score <= current_score => {}
            _ => best = Some((rule, score)),
        }
    }
    Ok(best.map(|(rule, _)| rule))
}

fn rule_match_score(rule: &ProjectRule, target_path: &str) -> Option<usize> {
    rule.match_patterns
        .iter()
        .filter_map(|pattern| pattern_match_score(pattern, target_path))
        .max()
}

fn pattern_match_score(pattern: &str, target_path: &str) -> Option<usize> {
    let pattern = normalize_rule_path(pattern);
    if pattern.is_empty() {
        return None;
    }
    if pattern == "**" || pattern == "*" {
        return Some(0);
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path_has_prefix(target_path, prefix).then_some(prefix.len());
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return direct_child_path(target_path, prefix).then_some(prefix.len());
    }
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        let matches = target_path.starts_with(prefix) && target_path.ends_with(suffix);
        return matches.then_some(prefix.len() + suffix.len());
    }
    (target_path == pattern || path_has_prefix(target_path, &pattern)).then_some(pattern.len())
}

fn path_has_prefix(target_path: &str, prefix: &str) -> bool {
    target_path == prefix || target_path.starts_with(&format!("{prefix}/"))
}

fn direct_child_path(target_path: &str, prefix: &str) -> bool {
    if !path_has_prefix(target_path, prefix) {
        return false;
    }
    let remainder = target_path
        .strip_prefix(prefix)
        .unwrap_or_default()
        .trim_start_matches('/');
    !remainder.is_empty() && !remainder.contains('/')
}

fn normalize_rule_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

fn write_nagare_agent_settings(
    layout: &ProjectLayout,
    settings: &NagareAgentSettings,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let settings_value = toml::Value::try_from(settings.clone())?;
    root_table.insert("nagare_agents".to_string(), settings_value);
    let rendered = toml::to_string_pretty(&value)?;
    fs::write(&layout.config_path, rendered)?;
    Ok(())
}

fn write_locale_settings(
    layout: &ProjectLayout,
    settings: &LocaleSettings,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let settings_value = toml::Value::try_from(settings.clone())?;
    root_table.insert("locale".to_string(), settings_value);
    let rendered = toml::to_string_pretty(&value)?;
    fs::write(&layout.config_path, rendered)?;
    Ok(())
}

fn validate_existing_agent_profile(
    layout: &ProjectLayout,
    agent_profile_id: &str,
) -> Result<(), NagareError> {
    get_agent_profile_from_layout(layout, agent_profile_id).map(|_| ())
}

fn latest_capability_probe<'a>(
    ledger: &'a Ledger,
    agent_profile_id: &str,
) -> Option<&'a CapabilityProbe> {
    ledger
        .capability_probes
        .iter()
        .rev()
        .find(|probe| probe.agent_profile_id == agent_profile_id)
}

fn default_work_agent_id() -> String {
    "codex-cli".to_string()
}

fn default_review_agent_id() -> String {
    "codex-app-server".to_string()
}

fn default_dispatch_agent_id() -> String {
    "codex-cli".to_string()
}

fn default_agent_run_purpose() -> AgentRunPurpose {
    AgentRunPurpose::Work
}

fn default_locale_language() -> String {
    env::var("NAGARE_LOCALE").unwrap_or_else(|_| "ja-JP".to_string())
}

fn default_locale_timezone() -> String {
    env::var("NAGARE_TIMEZONE").unwrap_or_else(|_| "Asia/Tokyo".to_string())
}

fn default_workspace_policy_kind() -> String {
    "project_root".to_string()
}

fn default_workspace_policy_cleanup() -> String {
    "keep".to_string()
}

fn validate_locale_language(language: &str) -> Result<(), NagareError> {
    if language.trim().is_empty()
        || !language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "locale language `{language}` must use letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

fn validate_timezone(timezone: &str) -> Result<(), NagareError> {
    if timezone.trim().is_empty()
        || timezone
            .chars()
            .any(|ch| ch.is_control() || ch == '\\' || ch == '"')
    {
        return Err(NagareError::InvalidState(format!(
            "timezone `{timezone}` is not valid"
        )));
    }
    Ok(())
}

fn runtime_healthcheck(runtime: &RuntimeDeclaration) -> ToolStatus {
    match runtime.healthcheck.split_first() {
        Some((command, args)) => check_command(command, args),
        None => check_command(&runtime.command, &runtime.args),
    }
}

fn validate_agent_profile_id(id: &str) -> Result<(), NagareError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "agent profile id `{id}` must use only ASCII letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

fn normalize_working_dir(working_dir: &str) -> Result<String, NagareError> {
    let value = working_dir.trim();
    if value.is_empty() || value == "." {
        return Ok(".".to_string());
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(NagareError::InvalidState(format!(
            "working_dir `{working_dir}` must be a relative path inside the project"
        )));
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

fn resolve_profile_working_dir(
    layout: &ProjectLayout,
    profile: &AgentProfile,
) -> Result<PathBuf, NagareError> {
    let normalized = normalize_working_dir(&profile.working_dir)?;
    let path = if normalized == "." {
        layout.root.clone()
    } else {
        layout.root.join(&normalized)
    };
    if !path.is_dir() {
        return Err(NagareError::InvalidState(format!(
            "working_dir `{}` for agent profile `{}` does not exist or is not a directory",
            profile.working_dir, profile.id
        )));
    }
    Ok(path)
}

fn capabilities_for_adapter(adapter_id: &str) -> Result<Vec<String>, NagareError> {
    let capabilities = match normalize_adapter_id(adapter_id)? {
        "process.codex-cli" => vec!["repo_read", "file_edit", "shell_command", "stdin_prompt"],
        "stdio.codex-app-server" => vec![
            "repo_read",
            "file_edit",
            "shell_command",
            "thread_state",
            "approval_flow",
            "event_stream",
        ],
        _ => unreachable!("normalize_adapter_id returned an unknown adapter"),
    };
    Ok(capabilities.into_iter().map(ToOwned::to_owned).collect())
}

fn skill_modes_for_adapter(adapter_id: &str) -> Result<Vec<String>, NagareError> {
    let modes = match normalize_adapter_id(adapter_id)? {
        "process.codex-cli" => vec!["prompt_injection", "file_reference"],
        "stdio.codex-app-server" => vec!["prompt_injection", "file_reference", "event_stream"],
        _ => unreachable!("normalize_adapter_id returned an unknown adapter"),
    };
    Ok(modes.into_iter().map(ToOwned::to_owned).collect())
}

fn instruction_sources(layout: &ProjectLayout) -> Vec<String> {
    ["AGENTS.md", ".codex/config.toml"]
        .iter()
        .filter(|source| layout.root.join(source).exists())
        .map(|source| source.to_string())
        .collect()
}

fn is_ja(locale: &str) -> bool {
    locale.to_ascii_lowercase().starts_with("ja")
}

fn localized_text<'a>(locale: &str, en: &'a str, ja: &'a str) -> &'a str {
    if is_ja(locale) { ja } else { en }
}

fn agent_run_claim(
    locale: &str,
    purpose: AgentRunPurpose,
    status: AgentRunStatus,
    agent_profile_id: &str,
) -> String {
    match (is_ja(locale), purpose, status) {
        (true, AgentRunPurpose::Work, AgentRunStatus::Succeeded) => {
            format!("Agent Profile `{agent_profile_id}` の実行が成功した")
        }
        (true, AgentRunPurpose::Work, AgentRunStatus::Failed) => {
            format!("Agent Profile `{agent_profile_id}` の実行が失敗した")
        }
        (true, AgentRunPurpose::DispatchPreview, AgentRunStatus::Succeeded) => {
            format!("Dispatch Agent `{agent_profile_id}` の実行前確認が成功した")
        }
        (true, AgentRunPurpose::DispatchPreview, AgentRunStatus::Failed) => {
            format!("Dispatch Agent `{agent_profile_id}` の実行前確認が失敗した")
        }
        (true, AgentRunPurpose::Review, AgentRunStatus::Succeeded) => {
            format!("Review Agent `{agent_profile_id}` の評価が成功した")
        }
        (true, AgentRunPurpose::Review, AgentRunStatus::Failed) => {
            format!("Review Agent `{agent_profile_id}` の評価が失敗した")
        }
        (false, AgentRunPurpose::Work, AgentRunStatus::Succeeded) => {
            format!("Agent run succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Work, AgentRunStatus::Failed) => {
            format!("Agent run failed with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::DispatchPreview, AgentRunStatus::Succeeded) => {
            format!("Dispatch preview succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::DispatchPreview, AgentRunStatus::Failed) => {
            format!("Dispatch preview failed with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Review, AgentRunStatus::Succeeded) => {
            format!("Review succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Review, AgentRunStatus::Failed) => {
            format!("Review failed with profile `{agent_profile_id}`")
        }
    }
}

fn command_exit_basis(locale: &str, exit_code: Option<i32>) -> String {
    if is_ja(locale) {
        format!("command の exit code は {exit_code:?}")
    } else {
        format!("command exit code {exit_code:?}")
    }
}

fn verification_claim(locale: &str, result: VerificationStatus) -> String {
    if is_ja(locale) {
        match result {
            VerificationStatus::Passed => "検証に成功した".to_string(),
            VerificationStatus::Failed => "検証に失敗した".to_string(),
        }
    } else {
        format!("Verification {result}")
    }
}

fn verification_basis(locale: &str, command: &str, exit_code: Option<i32>) -> String {
    if is_ja(locale) {
        format!("command `{command}` の exit code は {exit_code:?}")
    } else {
        format!("command `{command}` exit code {exit_code:?}")
    }
}

fn default_approval_rationale(locale: &str) -> &'static str {
    localized_text(
        locale,
        "Human approved after required verification",
        "必要な検証を確認したため、人間が承認した",
    )
}

fn run_shell(command: &str) -> io::Result<CommandRunOutput> {
    run_shell_in(command, None)
}

fn run_shell_in(command: &str, cwd: Option<&Path>) -> io::Result<CommandRunOutput> {
    let mut process = if cfg!(windows) {
        let mut command_builder = Command::new("cmd");
        command_builder.args(["/C", command]);
        command_builder
    } else {
        let mut command_builder = Command::new("sh");
        command_builder.args(["-lc", command]);
        command_builder
    };
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    let output = process.output()?;
    Ok(CommandRunOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
    })
}

fn write_command_log(path: &Path, command: &str, output: &CommandRunOutput) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "command: {command}\nexit_code: {:?}\n\n[stdout]\n{}\n[stderr]\n{}\n",
            output.exit_code, output.stdout, output.stderr
        ),
    )
}

fn write_adapter_log(
    path: &Path,
    run_packet: &RunPacket,
    output: &AdapterRunOutput,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "run_packet: {}\nwork_item: {}\nagent_profile: {}\nadapter: {}\nworking_dir: {}\ngoal: {}\ncommand: {}\nexit_code: {:?}\n\n[stdout]\n{}\n[stderr]\n{}\n",
            run_packet.id,
            run_packet.work_item_id,
            run_packet.agent_profile_id,
            run_packet.adapter_id,
            run_packet.working_dir,
            run_packet.goal,
            output.command,
            output.exit_code,
            output.stdout,
            output.stderr
        ),
    )
}

fn write_json_artifact<T: Serialize>(
    layout: &ProjectLayout,
    filename: &str,
    value: &T,
) -> Result<(), NagareError> {
    fs::create_dir_all(&layout.artifacts_dir)?;
    let path = layout.artifacts_dir.join(filename);
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{raw}\n"))?;
    Ok(())
}

fn run_dev_command(command: &str, cwd: &Path) -> Result<AdapterRunOutput, NagareError> {
    let output = run_shell_in(command, Some(cwd))?;
    Ok(AdapterRunOutput {
        command: format!("{command} [cwd={}]", cwd.display()),
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.exit_code,
    })
}

fn normalize_adapter_id(adapter_id: &str) -> Result<&'static str, NagareError> {
    match adapter_id {
        "process.codex-cli" | "process-codex-cli" => Ok("process.codex-cli"),
        "stdio.codex-app-server" | "stdio-codex-app-server" => Ok("stdio.codex-app-server"),
        _ => Err(NagareError::InvalidState(format!(
            "unsupported adapter `{adapter_id}`"
        ))),
    }
}

fn adapter_for_id(adapter_id: &str) -> Result<Box<dyn AgentAdapter>, NagareError> {
    match adapter_id {
        "process.codex-cli" => Ok(Box::new(ProcessCodexCliAdapter)),
        "stdio.codex-app-server" => Ok(Box::new(StdioCodexAppServerAdapter)),
        _ => Err(NagareError::InvalidState(format!(
            "unsupported adapter `{adapter_id}`"
        ))),
    }
}

fn path_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("{seconds}")
}

fn scenario_command(message: &str, success: bool) -> String {
    if cfg!(windows) {
        if success {
            format!("echo {message} && exit /B 0")
        } else {
            format!("echo {message} && exit /B 1")
        }
    } else if success {
        format!("echo {message}; exit 0")
    } else {
        format!("echo {message}; exit 1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                path: None,
                prompt: None,
                dev_command: Some(command.as_str()),
                purpose: AgentRunPurpose::DispatchPreview,
            },
        )
        .expect("dispatch preview should run");
        assert_eq!(preview.run.purpose, AgentRunPurpose::DispatchPreview);
        assert_eq!(preview.item_status, WorkItemStatus::Ready);

        let review = run_work_item_with_input(
            &root,
            &item.id,
            RunWorkItemInput {
                agent_profile_id: "codex-app-server",
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
        fs::remove_dir_all(root).ok();
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

        let rust_resolution =
            resolve_rule_for_path(&root, Some("crates/nagare-core/src/lib.rs"), None)
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
                path: Some("README.md"),
                prompt: None,
                dev_command: Some(command.as_str()),
                purpose: AgentRunPurpose::DispatchPreview,
            },
        )
        .expect("run should succeed");
        assert_eq!(result.run.purpose, AgentRunPurpose::DispatchPreview);

        let layout = ProjectLayout::new(&root);
        let ledger = load_ledger(&layout).expect("ledger should load");
        assert_eq!(ledger.resolved_skill_contexts.len(), 1);
        assert_eq!(ledger.resolved_run_packets.len(), 1);
        let context = &ledger.resolved_skill_contexts[0];
        let packet = &ledger.resolved_run_packets[0];
        assert_eq!(context.agent_profile_id, "codex-cli");
        assert_eq!(context.project_rule_ids, vec!["default".to_string()]);
        assert_eq!(
            context.applied_skill_set_ids,
            vec!["repo-default".to_string()]
        );
        assert_eq!(packet.resolved_skill_context_id, context.id);
        assert_eq!(packet.path.as_deref(), Some("README.md"));
        assert_eq!(packet.project_rule_ids, vec!["default".to_string()]);
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
}
