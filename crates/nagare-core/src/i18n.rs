use std::env;
use std::process::Command;

use crate::{AgentOutputContract, AgentRunPurpose};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NagareLanguage {
    En,
    Ja,
}

impl NagareLanguage {
    pub fn is_ja(self) -> bool {
        matches!(self, Self::Ja)
    }
}

#[derive(Debug, Clone)]
pub struct I18n {
    locale: String,
    language: NagareLanguage,
}

impl I18n {
    pub fn new(locale: impl AsRef<str>) -> Self {
        let locale = normalize_locale(locale.as_ref());
        let language = language_from_locale(&locale);
        Self { locale, language }
    }

    pub fn environment() -> Self {
        Self::new(detect_environment_locale())
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn language(&self) -> NagareLanguage {
        self.language
    }

    pub fn ui(&self, key: UiTextKey) -> &'static str {
        ui_text(self.language, key)
    }

    pub fn agent_default_description(&self, agent_id: &str) -> &'static str {
        match (self.language, agent_id) {
            (NagareLanguage::Ja, "worker") => {
                "割り当てられたWork Itemを実装します。小さく検証可能な変更を優先し、Nagare Resultに完了内容と次のメモを簡潔に残します。"
            }
            (NagareLanguage::Ja, "reviewer") => {
                "現在のWork Itemを受け入れ条件、成果物、テスト証跡に照らしてレビューします。条件ごとのpass/failと具体的なフォローアップを報告します。"
            }
            (NagareLanguage::Ja, "dispatcher") => {
                "次の作業ステップに最も適したAgent Profileを選びます。必要なdispatch JSONだけを返し、理由は簡潔にします。"
            }
            (NagareLanguage::Ja, "supervisor") => {
                "現在の状態から次のworkflow actionを判断します。前進を優先し、人の入力が必要な場合は停止し、workflow decision contractを返します。"
            }
            (_, "worker") => {
                "Implement the assigned work item. Prefer small, verifiable changes and leave concise completed work and next notes in the Nagare result."
            }
            (_, "reviewer") => {
                "Review the current work item against acceptance criteria, artifacts, and test evidence. Report pass/fail per criterion and concrete follow-up notes."
            }
            (_, "dispatcher") => {
                "Choose the most suitable target agent profile for the next work step. Return only the required dispatch JSON and keep the rationale concise."
            }
            (_, "supervisor") => {
                "Decide the next workflow action from the current state. Prefer forward progress, stop when human input is needed, and return the workflow decision contract."
            }
            _ => "",
        }
    }

    pub fn general_domain_toml(&self) -> String {
        match self.language {
            NagareLanguage::Ja => r#"[domain]
id = "general"
display_name = "汎用"
description = "専門ドメインを必要としない汎用作業。"
shared_knowledge = [
    "明確な証跡を残し、小さくレビュー可能な変更を優先する。",
    "Work Itemが変更を求めていない限り、既存Projectの慣習を維持する。",
]
common_rubric = [
    "Work Itemの結果が明確で検証可能である。",
    "関連するテストまたは確認を実行している。未実施の場合は理由を明示している。",
    "成果物とメモがhandoffまたはreviewに十分な簡潔さで残っている。",
]
dispatch_hints = [
    "より狭いDomainが適していない場合にこのGroupを使う。",
]
"#
            .to_string(),
            NagareLanguage::En => r#"[domain]
id = "general"
display_name = "General"
description = "General-purpose work that does not need a specialized domain."
shared_knowledge = [
    "Prefer small, reviewable changes with clear evidence.",
    "Preserve existing project conventions unless the work item asks for a change.",
]
common_rubric = [
    "The work item outcome is clear and verifiable.",
    "Relevant tests or checks are run, or any gap is explicitly reported.",
    "Artifacts and notes are concise enough for handoff or review.",
]
dispatch_hints = [
    "Use this group when no narrower Domain is a better fit.",
]
"#
            .to_string(),
        }
    }

    pub fn general_artifact_type_toml(&self) -> String {
        match self.language {
            NagareLanguage::Ja => r#"[artifact_type]
id = "general"
domain_id = "general"
display_name = "汎用"
description = "実装、レビュー、ドキュメント、保守を含む汎用作業。"
artifact_types = [
    "code",
    "documentation",
    "test_output",
    "notes",
]
rubric = [
    "要求された振る舞いまたは回答が、指定範囲に対して完了している。",
    "変更はWork Itemの範囲に収まり、無関係な変更を避けている。",
    "最終結果に実施した検証と残リスクが含まれている。",
]
dispatch_hints = [
    "複合的または未分類の作業にはこのDomainを使う。",
]
"#
            .to_string(),
            NagareLanguage::En => r#"[artifact_type]
id = "general"
domain_id = "general"
display_name = "General"
description = "General implementation, review, documentation, and maintenance work."
artifact_types = [
    "code",
    "documentation",
    "test_output",
    "notes",
]
rubric = [
    "The requested behavior or answer is complete for the stated scope.",
    "Changes are scoped to the work item and avoid unrelated churn.",
    "The final result includes the verification performed and any remaining risk.",
]
dispatch_hints = [
    "Use this domain for mixed or uncategorized work.",
]
"#
            .to_string(),
        }
    }

    pub fn default_config_toml(&self, timezone: &str) -> String {
        format!(
            r#"# Nagare local project configuration.

[project]
name = "nagare-local"

[storage]
kind = "json-ledger"
path = ".nagare/state/ledger.json"
sqlite_future_path = ".nagare/state/nagare.db"

[locale]
language = "{language}"
timezone = "{timezone}"

[workflow]
default_progress_mode = "confirm_first"
approval_policy = "manual_final_approval"

[nagare_agents]
work_agent = "worker"
review_agent = "reviewer"
dispatch_agent = "dispatcher"
supervisor_agent = "supervisor"

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

[runtimes.openclaw-local]
kind = "process"
command = "openclaw"
args = ["agent"]
healthcheck = ["openclaw", "--version"]

[adapters.process-codex-cli]
kind = "process.codex-cli"
runtime_kind = "process"
known_capabilities = ["repo_read", "file_edit", "shell_command", "stdin_prompt"]

[adapters.stdio-codex-app-server]
kind = "stdio.codex-app-server"
runtime_kind = "stdio"
known_capabilities = ["repo_read", "file_edit", "shell_command", "thread_state", "approval_flow", "event_stream"]

[adapters.process-openclaw-agent]
kind = "process.openclaw-agent"
runtime_kind = "process"
known_capabilities = ["repo_read", "file_edit", "shell_command", "thread_state", "provider_model_selection"]

[agent_profiles.worker]
display_name = "{worker_name}"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "worker"
working_dir = "."
managed_by = "nagare"
description = "{worker_description}"
domain_ids = ["general"]
artifact_type_ids = ["general"]

[agent_profiles.worker.external]
provider = "codex-cli"
agent_id = "worker"
managed = true
source = "created"

[agent_profiles.reviewer]
display_name = "{reviewer_name}"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "reviewer"
working_dir = "."
managed_by = "nagare"
description = "{reviewer_description}"
domain_ids = ["general"]
artifact_type_ids = ["general"]

[agent_profiles.reviewer.external]
provider = "codex-cli"
agent_id = "reviewer"
managed = true
source = "created"

[agent_profiles.dispatcher]
display_name = "{dispatcher_name}"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "dispatcher"
working_dir = "."
managed_by = "nagare"
description = "{dispatcher_description}"
domain_ids = ["general"]
artifact_type_ids = ["general"]

[agent_profiles.dispatcher.external]
provider = "codex-cli"
agent_id = "dispatcher"
managed = true
source = "created"

[agent_profiles.supervisor]
display_name = "{supervisor_name}"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "supervisor"
working_dir = "."
managed_by = "nagare"
description = "{supervisor_description}"
domain_ids = ["general"]
artifact_type_ids = ["general"]

[agent_profiles.supervisor.external]
provider = "codex-cli"
agent_id = "supervisor"
managed = true
source = "created"

[permission_policies.medium-code-task]
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]

[workspace_policies.project-root]
kind = "project_root"
isolate_per_work_item = false
cleanup = "keep"
"#,
            language = toml_escape(self.locale()),
            timezone = toml_escape(timezone),
            worker_name = toml_escape(self.ui(UiTextKey::Worker)),
            reviewer_name = toml_escape(self.ui(UiTextKey::Reviewer)),
            dispatcher_name = toml_escape(self.ui(UiTextKey::Dispatcher)),
            supervisor_name = toml_escape(self.ui(UiTextKey::Supervisor)),
            worker_description = toml_escape(self.agent_default_description("worker")),
            reviewer_description = toml_escape(self.agent_default_description("reviewer")),
            dispatcher_description = toml_escape(self.agent_default_description("dispatcher")),
            supervisor_description = toml_escape(self.agent_default_description("supervisor")),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiTextKey {
    WorkQueue,
    Settings,
    CreateNewItem,
    CreateNewAgent,
    Agents,
    Domains,
    Domain,
    ArtifactTypes,
    ArtifactType,
    Workflow,
    Profiles,
    Registered,
    CreateNewDomain,
    CreateNewArtifactType,
    ClearFilters,
    Agent,
    Type,
    Model,
    ModelProvider,
    BaseUrl,
    DomainScope,
    Workdir,
    Instruction,
    Source,
    Actions,
    Description,
    SharedKnowledge,
    Rubric,
    DispatchHints,
    Group,
    General,
    Worker,
    Reviewer,
    Dispatcher,
    Supervisor,
    Title,
    WorkFolder,
    AcceptanceCriteria,
    ExpectedArtifacts,
    Constraints,
    ProgressMode,
    FinalApproval,
    DomainAgentPolicy,
    SettingsLead,
    DomainsLead,
    ArtifactTypesLead,
    ProjectDefault,
    SaveWorkflowSettings,
    Edit,
    Delete,
    Name,
    DisplayName,
    Role,
    Instructions,
    Specialties,
    MoreContext,
    Prompt,
    Groups,
    Work,
    Review,
    Dispatch,
    CreateDomain,
    SaveDomain,
    CreateArtifactType,
    SaveArtifactType,
    CreateAgent,
    SaveAgent,
    DeleteAgent,
    DomainFormLead,
    ArtifactTypeFormLead,
    AgentFormLead,
    CommonRubric,
    ProgressModeOverride,
    FinalApprovalOverride,
    ProgressModeDefault,
    FinalApprovalDefault,
    NoGroup,
    InheritProjectDefault,
    ExternalAgentType,
    NoWorkItemsYet,
    ManualContinuation,
    All,
    NeedsAttention,
    Failed,
    Approval,
    Running,
    IdFolder,
    State,
    Next,
    Mode,
    DeleteWorkItem,
    AddItemLead,
}

pub fn ui_text(language: NagareLanguage, key: UiTextKey) -> &'static str {
    match language {
        NagareLanguage::Ja => match key {
            UiTextKey::WorkQueue => "作業キュー",
            UiTextKey::Settings => "設定",
            UiTextKey::CreateNewItem => "Work Itemを作成",
            UiTextKey::CreateNewAgent => "エージェントを作成",
            UiTextKey::Agents => "エージェント",
            UiTextKey::Domains => "ドメイン",
            UiTextKey::Domain => "ドメイン",
            UiTextKey::ArtifactTypes => "成果物種別",
            UiTextKey::ArtifactType => "成果物種別",
            UiTextKey::Workflow => "ワークフロー",
            UiTextKey::Profiles => "プロファイル",
            UiTextKey::Registered => "登録済み",
            UiTextKey::CreateNewDomain => "ドメインを作成",
            UiTextKey::CreateNewArtifactType => "成果物種別を作成",
            UiTextKey::ClearFilters => "フィルタ解除",
            UiTextKey::Agent => "エージェント",
            UiTextKey::Type => "種別",
            UiTextKey::Model => "モデル",
            UiTextKey::ModelProvider => "モデルプロバイダー",
            UiTextKey::BaseUrl => "Base URL",
            UiTextKey::DomainScope => "担当ドメイン",
            UiTextKey::Workdir => "作業ディレクトリ",
            UiTextKey::Instruction => "指示",
            UiTextKey::Source => "由来",
            UiTextKey::Actions => "操作",
            UiTextKey::Description => "説明",
            UiTextKey::SharedKnowledge => "共通知識",
            UiTextKey::Rubric => "評価基準",
            UiTextKey::DispatchHints => "振り分けヒント",
            UiTextKey::Group => "グループ",
            UiTextKey::General => "汎用",
            UiTextKey::Worker => "Worker",
            UiTextKey::Reviewer => "Reviewer",
            UiTextKey::Dispatcher => "Dispatcher",
            UiTextKey::Supervisor => "Supervisor",
            UiTextKey::Title => "タイトル",
            UiTextKey::WorkFolder => "作業フォルダ",
            UiTextKey::AcceptanceCriteria => "受け入れ条件",
            UiTextKey::ExpectedArtifacts => "期待する成果物",
            UiTextKey::Constraints => "制約",
            UiTextKey::ProgressMode => "進行モード",
            UiTextKey::FinalApproval => "最終承認",
            UiTextKey::DomainAgentPolicy => "専門ドメインエージェントの扱い",
            UiTextKey::SettingsLead => {
                "ワークフロー、ドメイン、エージェントプロファイルを設定します"
            }
            UiTextKey::DomainsLead => "共通知識、共通評価基準、振り分けヒント、ワークフロー既定値",
            UiTextKey::ArtifactTypesLead => {
                "ドメイン所属、評価基準、振り分けヒント、ワークフロー上書き"
            }
            UiTextKey::ProjectDefault => "プロジェクト既定",
            UiTextKey::SaveWorkflowSettings => "ワークフロー設定を保存",
            UiTextKey::Edit => "編集",
            UiTextKey::Delete => "削除",
            UiTextKey::Name => "名前",
            UiTextKey::DisplayName => "表示名",
            UiTextKey::Role => "役割",
            UiTextKey::Instructions => "指示",
            UiTextKey::Specialties => "専門性",
            UiTextKey::MoreContext => "追加コンテキスト",
            UiTextKey::Prompt => "依頼内容",
            UiTextKey::Groups => "グループ",
            UiTextKey::Work => "作業",
            UiTextKey::Review => "レビュー",
            UiTextKey::Dispatch => "振り分け",
            UiTextKey::CreateDomain => "ドメインを作成",
            UiTextKey::SaveDomain => "ドメインを保存",
            UiTextKey::CreateArtifactType => "成果物種別を作成",
            UiTextKey::SaveArtifactType => "成果物種別を保存",
            UiTextKey::CreateAgent => "エージェントを作成",
            UiTextKey::SaveAgent => "エージェントを保存",
            UiTextKey::DeleteAgent => "エージェントを削除",
            UiTextKey::DomainFormLead => {
                "ドメインの共通知識、評価基準、ワークフロー既定値を設定します"
            }
            UiTextKey::ArtifactTypeFormLead => {
                "成果物種別ごとの評価基準とDispatchヒントを設定します"
            }
            UiTextKey::AgentFormLead => "エージェントプロファイルを設定します",
            UiTextKey::CommonRubric => "共通評価基準",
            UiTextKey::ProgressModeOverride => "進行モードの上書き",
            UiTextKey::FinalApprovalOverride => "最終承認の上書き",
            UiTextKey::ProgressModeDefault => "進行モード既定値",
            UiTextKey::FinalApprovalDefault => "最終承認既定値",
            UiTextKey::NoGroup => "グループなし",
            UiTextKey::InheritProjectDefault => "プロジェクト既定を継承",
            UiTextKey::ExternalAgentType => "外部エージェント種別",
            UiTextKey::NoWorkItemsYet => "Work Itemはまだありません",
            UiTextKey::ManualContinuation => "手動継続",
            UiTextKey::All => "すべて",
            UiTextKey::NeedsAttention => "確認が必要",
            UiTextKey::Failed => "失敗",
            UiTextKey::Approval => "承認",
            UiTextKey::Running => "処理中",
            UiTextKey::IdFolder => "ID / フォルダ",
            UiTextKey::State => "状態",
            UiTextKey::Next => "次",
            UiTextKey::Mode => "モード",
            UiTextKey::DeleteWorkItem => "Work Itemを削除",
            UiTextKey::AddItemLead => "Work Itemを追加し、バックグラウンドで処理を進めます",
        },
        NagareLanguage::En => match key {
            UiTextKey::WorkQueue => "Work Queue",
            UiTextKey::Settings => "Settings",
            UiTextKey::CreateNewItem => "Create New Item",
            UiTextKey::CreateNewAgent => "Create New Agent",
            UiTextKey::Agents => "Agents",
            UiTextKey::Domains => "Domains",
            UiTextKey::Domain => "Domain",
            UiTextKey::ArtifactTypes => "Artifact Types",
            UiTextKey::ArtifactType => "Artifact Type",
            UiTextKey::Workflow => "Workflow",
            UiTextKey::Profiles => "profiles",
            UiTextKey::Registered => "registered",
            UiTextKey::CreateNewDomain => "Create New Domain",
            UiTextKey::CreateNewArtifactType => "Create New Artifact Type",
            UiTextKey::ClearFilters => "Clear filters",
            UiTextKey::Agent => "Agent",
            UiTextKey::Type => "Type",
            UiTextKey::Model => "Model",
            UiTextKey::ModelProvider => "Model Provider",
            UiTextKey::BaseUrl => "Base URL",
            UiTextKey::DomainScope => "Domain scope",
            UiTextKey::Workdir => "Workdir",
            UiTextKey::Instruction => "Instruction",
            UiTextKey::Source => "Source",
            UiTextKey::Actions => "Actions",
            UiTextKey::Description => "Description",
            UiTextKey::SharedKnowledge => "Shared knowledge",
            UiTextKey::Rubric => "Rubric",
            UiTextKey::DispatchHints => "Dispatch hints",
            UiTextKey::Group => "Group",
            UiTextKey::General => "General",
            UiTextKey::Worker => "Worker",
            UiTextKey::Reviewer => "Reviewer",
            UiTextKey::Dispatcher => "Dispatcher",
            UiTextKey::Supervisor => "Supervisor",
            UiTextKey::Title => "Title",
            UiTextKey::WorkFolder => "Work folder",
            UiTextKey::AcceptanceCriteria => "Acceptance Criteria",
            UiTextKey::ExpectedArtifacts => "Expected Artifacts",
            UiTextKey::Constraints => "Constraints",
            UiTextKey::ProgressMode => "Progress mode",
            UiTextKey::FinalApproval => "Final approval",
            UiTextKey::DomainAgentPolicy => "Domain agent policy",
            UiTextKey::SettingsLead => "Workflow policy, domains, and agent profiles",
            UiTextKey::DomainsLead => {
                "Shared knowledge, common rubric, dispatch hints, and workflow defaults"
            }
            UiTextKey::ArtifactTypesLead => {
                "Domain membership, rubric, dispatch hints, and workflow overrides"
            }
            UiTextKey::ProjectDefault => "project default",
            UiTextKey::SaveWorkflowSettings => "Save Workflow Settings",
            UiTextKey::Edit => "Edit",
            UiTextKey::Delete => "Delete",
            UiTextKey::Name => "Name",
            UiTextKey::DisplayName => "Display Name",
            UiTextKey::Role => "Role",
            UiTextKey::Instructions => "Instructions",
            UiTextKey::Specialties => "Specialties",
            UiTextKey::MoreContext => "More context",
            UiTextKey::Prompt => "Prompt",
            UiTextKey::Groups => "groups",
            UiTextKey::Work => "work",
            UiTextKey::Review => "review",
            UiTextKey::Dispatch => "dispatch",
            UiTextKey::CreateDomain => "Create Domain",
            UiTextKey::SaveDomain => "Save Domain",
            UiTextKey::CreateArtifactType => "Create Artifact Type",
            UiTextKey::SaveArtifactType => "Save Artifact Type",
            UiTextKey::CreateAgent => "Create Agent",
            UiTextKey::SaveAgent => "Save Agent",
            UiTextKey::DeleteAgent => "Delete Agent",
            UiTextKey::DomainFormLead => {
                "Configure domain knowledge, rubric, and workflow defaults"
            }
            UiTextKey::ArtifactTypeFormLead => "Configure artifact type rubric and dispatch hints",
            UiTextKey::AgentFormLead => "Configure this agent profile",
            UiTextKey::CommonRubric => "Common rubric",
            UiTextKey::ProgressModeOverride => "Progress mode override",
            UiTextKey::FinalApprovalOverride => "Final approval override",
            UiTextKey::ProgressModeDefault => "Progress mode default",
            UiTextKey::FinalApprovalDefault => "Final approval default",
            UiTextKey::NoGroup => "No group",
            UiTextKey::InheritProjectDefault => "Inherit project default",
            UiTextKey::ExternalAgentType => "External Agent Type",
            UiTextKey::NoWorkItemsYet => "No work items yet",
            UiTextKey::ManualContinuation => "manual continuation",
            UiTextKey::All => "All",
            UiTextKey::NeedsAttention => "Needs attention",
            UiTextKey::Failed => "Failed",
            UiTextKey::Approval => "Approval",
            UiTextKey::Running => "Running",
            UiTextKey::IdFolder => "ID / Folder",
            UiTextKey::State => "State",
            UiTextKey::Next => "Next",
            UiTextKey::Mode => "Mode",
            UiTextKey::DeleteWorkItem => "Delete Work Item",
            UiTextKey::AddItemLead => "Add a work item and start background execution",
        },
    }
}

pub(crate) fn localized_output_contract_instruction(
    locale: &str,
    purpose: AgentRunPurpose,
    contract: &AgentOutputContract,
) -> String {
    let i18n = I18n::new(locale);
    let required = match (i18n.language(), contract.required) {
        (NagareLanguage::Ja, true) => "この最終ブロックは必須です。",
        (NagareLanguage::Ja, false) => "可能であればこの最終ブロックを含めてください。",
        (_, true) => "This final block is required.",
        (_, false) => "Include this final block when possible.",
    };
    let common = match i18n.language() {
        NagareLanguage::Ja => {
            "指定されたMarkdown契約形を最後に出力してください。各contract keyは必ず行頭に置き、summaryの下や箇条書き本文の中に入れず、code fenceで囲まないでください。"
        }
        NagareLanguage::En => {
            "Finish with this exact Markdown contract shape. Put each contract key at the start of its own line. Do not nest contract keys under summary, do not put contract keys inside bullet text, and do not wrap the block in a code fence."
        }
    };
    match purpose {
        AgentRunPurpose::DispatchPreview => match i18n.language() {
            NagareLanguage::Ja => format!(
                "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\ntarget_agent_profile_id, summary, risks, missing_information をkeyに持つJSON objectを1つだけ返してください。target_agent_profile_idは登録済み候補Agent Profile idと完全一致させてください。JSONの周囲にMarkdownを付けないでください。",
                contract_id = contract.contract,
                pack = contract.instruction_pack,
            ),
            NagareLanguage::En => format!(
                "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\nReturn one JSON object only with keys: target_agent_profile_id, summary, risks, missing_information. target_agent_profile_id must exactly match a registered candidate agent profile id. Do not add Markdown around the JSON.",
                contract_id = contract.contract,
                pack = contract.instruction_pack,
            ),
        },
        AgentRunPurpose::Review => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\n{common}\n\n## Nagare Review\nverdict: pass|request_changes|blocked\nsummary:\n- concise review summary\ncompleted:\n- what you reviewed, including CI/tests/checks when applicable\ncriteria:\n- <criterion>: passed|failed|unknown - note\nfindings:\n- finding or none\nreferenced_artifacts:\n- requested deliverable artifact id/path or none\nrequested_changes:\n- requested change or none\nquestions:\n- question or none\nnext_notes:\n- handoff hint for the next dispatch or agent\nnext_action: approve|run_agent|answer_question|stop",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
        AgentRunPurpose::Work => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\n{common}\n\n## Nagare Result\nstatus: succeeded|blocked|failed\nsummary:\n- final user-facing result or concise answer\ncompleted:\n- completed work item\nartifacts:\n- requested deliverable artifact path/id or none\nevidence:\n- evidence or none\nquestions:\n- question or none\nnext_notes:\n- handoff hint for the next dispatch or agent\nnext_action: review|answer_question|handoff|stop",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
        AgentRunPurpose::Synthesis => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\n{common}\n\n## Nagare Result\nstatus: succeeded|blocked|failed\nsummary:\n- final user-facing conclusion across all reviewed workers\ncompleted:\n- worker-by-worker summary and what was completed\nartifacts:\n- referenced deliverable artifact path/id or none\nevidence:\n- evidence or reviewed source steps\nquestions:\n- question or none\nnext_notes:\n- approval note or remaining risk\nnext_action: approve|answer_question|stop",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
        AgentRunPurpose::WorkflowSupervision => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\n{common}\n\n## Nagare Workflow Decision\naction: dispatch|run_agent|run_review|recover|approve|stop\nreason: concise reason\ntarget_agent_profile_id: agent id or none\nrequires_human: true|false\nconfidence: 0.0-1.0\ncommand_hint: nagare command or none",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
    }
}

pub(crate) fn localized_context_heading(locale: &str, key: ContextHeading) -> &'static str {
    match (language_from_locale(locale), key) {
        (NagareLanguage::Ja, ContextHeading::AgentInstructions) => "Nagare Agent Instructions",
        (NagareLanguage::Ja, ContextHeading::DomainContext) => "Nagare Domain Context",
        (NagareLanguage::Ja, ContextHeading::HumanFeedback) => "Nagare Human Feedback",
        (NagareLanguage::Ja, ContextHeading::HandoffContext) => "Nagare Handoff Context",
        (_, ContextHeading::AgentInstructions) => "Nagare Agent Instructions",
        (_, ContextHeading::DomainContext) => "Nagare Domain Context",
        (_, ContextHeading::HumanFeedback) => "Nagare Human Feedback",
        (_, ContextHeading::HandoffContext) => "Nagare Handoff Context",
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ContextHeading {
    AgentInstructions,
    DomainContext,
    HumanFeedback,
    HandoffContext,
}

pub fn language_from_locale(locale: &str) -> NagareLanguage {
    let base = locale
        .trim()
        .split('.')
        .next()
        .unwrap_or(locale)
        .split('@')
        .next()
        .unwrap_or(locale);
    let lower = base.to_ascii_lowercase().replace('_', "-");
    if lower == "ja" || lower.starts_with("ja-") {
        NagareLanguage::Ja
    } else {
        NagareLanguage::En
    }
}

pub fn detect_environment_locale() -> String {
    env_locale_candidates()
        .into_iter()
        .find_map(|value| parse_locale_candidate(&value))
        .or_else(detect_windows_locale)
        .unwrap_or_else(|| "ja-JP".to_string())
}

pub fn detect_environment_timezone() -> String {
    env::var("NAGARE_TIMEZONE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| env::var("TZ").ok().filter(|value| !value.trim().is_empty()))
        .or_else(detect_windows_timezone)
        .unwrap_or_else(|| "UTC".to_string())
}

fn env_locale_candidates() -> Vec<String> {
    ["NAGARE_LOCALE", "LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
        .iter()
        .filter_map(|key| env::var(key).ok())
        .collect()
}

fn parse_locale_candidate(value: &str) -> Option<String> {
    let first = value
        .split([':', ';'])
        .map(str::trim)
        .find(|part| !part.is_empty())?;
    let without_encoding = first.split('.').next().unwrap_or(first);
    let without_modifier = without_encoding
        .split('@')
        .next()
        .unwrap_or(without_encoding);
    let normalized = normalize_locale(without_modifier);
    (!normalized.is_empty()
        && normalized != "C"
        && normalized != "POSIX"
        && normalized != "c"
        && normalized != "posix")
        .then_some(normalized)
}

fn normalize_locale(value: &str) -> String {
    let value = value.trim().replace('_', "-");
    if let Some((language, region)) = value.split_once('-') {
        format!(
            "{}-{}",
            language.to_ascii_lowercase(),
            region.to_ascii_uppercase()
        )
    } else {
        value.to_ascii_lowercase()
    }
}

fn detect_windows_locale() -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[System.Globalization.CultureInfo]::CurrentCulture.Name",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_locale_candidate(stdout.trim())
}

fn detect_windows_timezone() -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-TimeZone).Id"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    match value.as_str() {
        "Tokyo Standard Time" => Some("Asia/Tokyo".to_string()),
        "" => None,
        other => Some(other.to_string()),
    }
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_work_output_contract;

    #[test]
    fn locale_prefix_selects_supported_language() {
        assert_eq!(language_from_locale("ja-JP"), NagareLanguage::Ja);
        assert_eq!(language_from_locale("ja_JP.UTF-8"), NagareLanguage::Ja);
        assert_eq!(language_from_locale("en-US"), NagareLanguage::En);
    }

    #[test]
    fn japanese_catalog_covers_seed_and_prompt_text() {
        let i18n = I18n::new("ja-JP");
        assert!(
            i18n.default_config_toml("Asia/Tokyo")
                .contains("割り当てられたWork Item")
        );
        assert!(i18n.general_domain_toml().contains("専門ドメイン"));
        let instruction = localized_output_contract_instruction(
            "ja-JP",
            AgentRunPurpose::Work,
            &default_work_output_contract(),
        );
        assert!(instruction.contains("この最終ブロックは必須です。"));
        assert!(instruction.contains("## Nagare Result"));
    }
}
