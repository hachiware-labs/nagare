use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::*;

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
    pub review_results: Vec<ReviewResult>,
    #[serde(default)]
    pub handoffs: Vec<HandoffPacket>,
    #[serde(default)]
    pub decisions: Vec<HumanDecision>,
    #[serde(default)]
    pub human_feedback: Vec<HumanFeedback>,
    #[serde(default)]
    pub dispatch_plans: Vec<DispatchPlan>,
    #[serde(default)]
    pub recovery_plans: Vec<RecoveryPlan>,
    #[serde(default)]
    pub capability_probes: Vec<CapabilityProbe>,
    #[serde(default)]
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    #[serde(default)]
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
    #[serde(default)]
    pub agent_outputs: Vec<AgentOutputRecord>,
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
            review_results: Vec::new(),
            handoffs: Vec::new(),
            decisions: Vec::new(),
            human_feedback: Vec::new(),
            dispatch_plans: Vec::new(),
            recovery_plans: Vec::new(),
            capability_probes: Vec::new(),
            resolved_skill_contexts: Vec::new(),
            resolved_run_packets: Vec::new(),
            agent_outputs: Vec::new(),
        }
    }
}

impl Ledger {
    pub(crate) fn next_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}_{:04}", self.next_seq);
        self.next_seq += 1;
        id
    }

    pub(crate) fn work_item_mut(&mut self, id: &str) -> Result<&mut WorkItem, NagareError> {
        self.work_items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| NagareError::NotFound(format!("work item `{id}`")))
    }

    pub(crate) fn work_item(&self, id: &str) -> Result<&WorkItem, NagareError> {
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
    NeedsInput,
    FailedVerification,
    NeedsHandoff,
    ReadyForReview,
    ReadyForVerification,
    ChangesRequested,
    Done,
}

impl fmt::Display for WorkItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ready => "ready",
            Self::AgentRunning => "agent_running",
            Self::NeedsInput => "needs_input",
            Self::FailedVerification => "failed_verification",
            Self::NeedsHandoff => "needs_handoff",
            Self::ReadyForReview => "ready_for_review",
            Self::ReadyForVerification => "ready_for_verification",
            Self::ChangesRequested => "changes_requested",
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
pub struct HumanFeedback {
    pub id: String,
    pub work_item_id: String,
    pub source_agent_output_id: Option<String>,
    pub question: String,
    pub answer: String,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutputRecord {
    pub id: String,
    pub work_item_id: String,
    pub agent_run_id: String,
    pub agent_profile_id: String,
    pub purpose: AgentRunPurpose,
    pub contract: String,
    pub instruction_pack: String,
    pub parse_status: AgentOutputParseStatus,
    #[serde(default)]
    pub fields: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub questions: Vec<String>,
    pub next_action: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub artifact_id: Option<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentOutputParseStatus {
    Parsed,
    Unparsed,
}

impl fmt::Display for AgentOutputParseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parsed => f.write_str("parsed"),
            Self::Unparsed => f.write_str("unparsed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchPlan {
    pub id: String,
    pub work_item_id: String,
    #[serde(default = "default_dispatch_plan_status")]
    pub status: DispatchPlanStatus,
    pub agent_run_id: String,
    pub dispatch_agent_profile_id: String,
    pub target_agent_profile_id: String,
    pub resolved_run_packet_id: String,
    pub raw_output_artifact_id: String,
    pub path: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub missing_information: Vec<String>,
    #[serde(default)]
    pub selection_warnings: Vec<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPlanStatus {
    Draft,
    Accepted,
    Superseded,
}

impl fmt::Display for DispatchPlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => f.write_str("draft"),
            Self::Accepted => f.write_str("accepted"),
            Self::Superseded => f.write_str("superseded"),
        }
    }
}

pub fn default_dispatch_plan_status() -> DispatchPlanStatus {
    DispatchPlanStatus::Draft
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
    #[serde(default = "default_agent_run_purpose")]
    pub purpose: AgentRunPurpose,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub goal: String,
    pub path: Option<String>,
    #[serde(default)]
    pub dispatch_plan_id: Option<String>,
    pub permission_policy_id: Option<String>,
    pub workspace_policy_id: Option<String>,
    pub resolved_skill_context_id: String,
    #[serde(default)]
    pub output_contract: AgentOutputContract,
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
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub specialties: Vec<String>,
    #[serde(default)]
    pub output_contracts: AgentOutputContracts,
    #[serde(skip)]
    pub source: AgentProfileSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentOutputContracts {
    #[serde(default = "default_work_output_contract")]
    pub work: AgentOutputContract,
    #[serde(default = "default_review_output_contract")]
    pub review: AgentOutputContract,
    #[serde(default = "default_dispatch_output_contract")]
    pub dispatch: AgentOutputContract,
}

impl AgentOutputContracts {
    pub fn for_purpose(&self, purpose: AgentRunPurpose) -> &AgentOutputContract {
        match purpose {
            AgentRunPurpose::Review => &self.review,
            AgentRunPurpose::DispatchPreview => &self.dispatch,
            AgentRunPurpose::Work => &self.work,
        }
    }
}

impl Default for AgentOutputContracts {
    fn default() -> Self {
        Self {
            work: default_work_output_contract(),
            review: default_review_output_contract(),
            dispatch: default_dispatch_output_contract(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentOutputContract {
    pub contract: String,
    pub instruction_pack: String,
    #[serde(default = "default_output_contract_required")]
    pub required: bool,
    #[serde(default)]
    pub injection: AgentOutputInjection,
}

impl Default for AgentOutputContract {
    fn default() -> Self {
        default_work_output_contract()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentOutputInjection {
    #[default]
    PromptSuffix,
}

impl fmt::Display for AgentOutputInjection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptSuffix => f.write_str("prompt_suffix"),
        }
    }
}

pub fn default_output_contract_required() -> bool {
    true
}

pub fn default_work_output_contract() -> AgentOutputContract {
    AgentOutputContract {
        contract: "nagare.result.v1".to_string(),
        instruction_pack: "nagare-result-writer.v1".to_string(),
        required: true,
        injection: AgentOutputInjection::PromptSuffix,
    }
}

pub fn default_review_output_contract() -> AgentOutputContract {
    AgentOutputContract {
        contract: "nagare.review.v1".to_string(),
        instruction_pack: "nagare-review-writer.v1".to_string(),
        required: true,
        injection: AgentOutputInjection::PromptSuffix,
    }
}

pub fn default_dispatch_output_contract() -> AgentOutputContract {
    AgentOutputContract {
        contract: "nagare.dispatch.v1".to_string(),
        instruction_pack: "nagare-dispatch-writer.v1".to_string(),
        required: true,
        injection: AgentOutputInjection::PromptSuffix,
    }
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
    pub description: &'a str,
    pub specialties: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AddAgentProfileResult {
    pub profile: AgentProfile,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateAgentProfileInput<'a> {
    pub display_name: Option<&'a str>,
    pub role: Option<&'a str>,
    pub working_dir: Option<&'a str>,
    pub description: Option<&'a str>,
    pub specialties: Option<Vec<String>>,
    pub output_contract: Option<AgentOutputContractUpdate<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub enum AgentOutputContractPurpose {
    Work,
    Review,
    Dispatch,
}

impl AgentOutputContractPurpose {
    pub fn parse(value: &str) -> Result<Self, NagareError> {
        match value.trim() {
            "work" => Ok(Self::Work),
            "review" => Ok(Self::Review),
            "dispatch" => Ok(Self::Dispatch),
            other => Err(NagareError::InvalidState(format!(
                "unknown output contract purpose `{other}`; expected work, review, or dispatch"
            ))),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgentOutputContractUpdate<'a> {
    pub purpose: Option<AgentOutputContractPurpose>,
    pub contract: Option<&'a str>,
    pub instruction_pack: Option<&'a str>,
    pub required: Option<bool>,
    pub injection: Option<AgentOutputInjection>,
}

#[derive(Debug, Clone)]
pub struct UpdateAgentProfileResult {
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
pub(crate) struct SkillSetResolution {
    pub declared_skill_set_ids: Vec<String>,
    pub applied_skill_set_ids: Vec<String>,
    pub skipped_skill_set_ids: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RunWorkItemInput<'a> {
    pub agent_profile_id: &'a str,
    pub dispatch_plan_id: Option<&'a str>,
    pub path: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub dev_command: Option<&'a str>,
    pub purpose: AgentRunPurpose,
}

#[derive(Debug, Clone)]
pub struct AnswerWorkItemInput<'a> {
    pub question: Option<&'a str>,
    pub answer: &'a str,
}

#[derive(Debug, Clone)]
pub struct AnswerWorkItemResult {
    pub feedback: HumanFeedback,
    pub item_status: WorkItemStatus,
}

#[derive(Debug, Clone)]
pub struct SelectRunAgentInput<'a> {
    pub explicit_agent_profile_id: Option<&'a str>,
    pub dispatch_plan_id: Option<&'a str>,
    pub path: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunAgentSelectionSource {
    Explicit,
    DispatchPlan,
    ProjectRule,
    Default,
}

impl fmt::Display for RunAgentSelectionSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Explicit => f.write_str("explicit"),
            Self::DispatchPlan => f.write_str("dispatch_plan"),
            Self::ProjectRule => f.write_str("project_rule"),
            Self::Default => f.write_str("default"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectRunAgentResult {
    pub agent_profile_id: String,
    pub source: RunAgentSelectionSource,
    pub dispatch_plan_id: Option<String>,
    pub rule_resolution: Option<RuleResolution>,
}

#[derive(Debug, Clone)]
pub struct AdapterRunRequest<'a> {
    pub working_dir: &'a Path,
    pub run_packet: &'a ResolvedRunPacket,
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

pub(crate) trait AgentAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError>;
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
    pub dispatch_plan_id: Option<String>,
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
pub struct AcceptDispatchPlanResult {
    pub plan: DispatchPlan,
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
