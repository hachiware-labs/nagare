use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDecision {
    pub id: String,
    pub work_item_id: String,
    pub action: WorkflowDecisionAction,
    pub source: WorkflowDecisionSource,
    pub reason: String,
    pub requires_human: bool,
    pub target_agent_profile_id: Option<String>,
    pub agent_run_id: Option<String>,
    pub confidence: f32,
    pub command_hint: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowDecisionAction {
    Dispatch,
    AcceptDispatch,
    RunAgent,
    RunReview,
    RunVerification,
    CreateRecoveryPlan,
    ApplyRecoveryPlan,
    AskHuman,
    CreateHandoff,
    Approve,
    Wait,
    Done,
    Stop,
}

impl std::fmt::Display for WorkflowDecisionAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Dispatch => "dispatch",
            Self::AcceptDispatch => "accept_dispatch",
            Self::RunAgent => "run_agent",
            Self::RunReview => "run_review",
            Self::RunVerification => "run_verification",
            Self::CreateRecoveryPlan => "create_recovery_plan",
            Self::ApplyRecoveryPlan => "apply_recovery_plan",
            Self::AskHuman => "ask_human",
            Self::CreateHandoff => "create_handoff",
            Self::Approve => "approve",
            Self::Wait => "wait",
            Self::Done => "done",
            Self::Stop => "stop",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowDecisionSource {
    Deterministic,
    SupervisorAgent,
}

impl std::fmt::Display for WorkflowDecisionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::SupervisorAgent => f.write_str("supervisor_agent"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateWorkflowDecisionResult {
    pub decision: WorkflowDecision,
}

#[derive(Debug, Clone, Default)]
pub struct AdvanceWorkItemInput<'a> {
    pub path: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub dev_command: Option<&'a str>,
    pub dispatch_dev_command: Option<&'a str>,
    pub review_dev_command: Option<&'a str>,
    pub verification_command: Option<&'a str>,
    pub use_supervisor: bool,
    pub supervisor_dev_command: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct AdvanceWorkItemResult {
    pub decision: WorkflowDecision,
    pub advanced: bool,
    pub item_status: WorkItemStatus,
    pub message: String,
    pub run_id: Option<String>,
    pub dispatch_plan_id: Option<String>,
    pub recovery_plan_id: Option<String>,
    pub verification_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdvanceUntilBlockedInput<'a> {
    pub step: AdvanceWorkItemInput<'a>,
    pub max_steps: usize,
}

#[derive(Debug, Clone)]
pub struct AdvanceUntilBlockedResult {
    pub steps: Vec<AdvanceWorkItemResult>,
    pub final_status: WorkItemStatus,
    pub stopped_reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct SupervisorWorkflowDecisionInput<'a> {
    pub path: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub dev_command: Option<&'a str>,
}

pub fn create_workflow_decision(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<CreateWorkflowDecisionResult, NagareError> {
    let decision = record_workflow_decision(root, work_item_id)?;
    Ok(CreateWorkflowDecisionResult { decision })
}

pub fn create_supervisor_workflow_decision(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: SupervisorWorkflowDecisionInput<'_>,
) -> Result<CreateWorkflowDecisionResult, NagareError> {
    let decision = record_supervisor_workflow_decision(root, work_item_id, input)?;
    Ok(CreateWorkflowDecisionResult { decision })
}

pub fn advance_work_item_once(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: AdvanceWorkItemInput<'_>,
) -> Result<AdvanceWorkItemResult, NagareError> {
    let root = root.into();
    let decision = if input.use_supervisor {
        record_supervisor_workflow_decision(
            &root,
            work_item_id,
            SupervisorWorkflowDecisionInput {
                path: input.path,
                prompt: input.prompt,
                dev_command: input.supervisor_dev_command,
            },
        )?
    } else {
        record_workflow_decision(&root, work_item_id)?
    };
    let snapshot = get_work_item_snapshot(&root, work_item_id)?;
    let item_status = snapshot.item.status;

    match decision.action {
        WorkflowDecisionAction::Dispatch => {
            let settings = get_nagare_agent_settings(&root)?;
            let agent = decision
                .target_agent_profile_id
                .as_deref()
                .unwrap_or(settings.dispatch_agent.as_str());
            let run = run_work_item_with_input(
                &root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: agent,
                    dispatch_plan_id: None,
                    path: input.path.or(snapshot.item.work_folder.as_deref()),
                    prompt: input.prompt,
                    dev_command: input.dispatch_dev_command.or(input.dev_command),
                    purpose: AgentRunPurpose::DispatchPreview,
                },
            )?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: run.item_status,
                message: "dispatch preview recorded".to_string(),
                run_id: Some(run.run.id),
                dispatch_plan_id: run.dispatch_plan_id,
                recovery_plan_id: None,
                verification_id: None,
            })
        }
        WorkflowDecisionAction::AcceptDispatch => {
            let accepted = accept_dispatch_plan(&root, work_item_id, None)?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status,
                message: format!("dispatch plan {} accepted", accepted.plan.id),
                run_id: None,
                dispatch_plan_id: Some(accepted.plan.id),
                recovery_plan_id: None,
                verification_id: None,
            })
        }
        WorkflowDecisionAction::RunAgent => {
            let selection = select_agent_for_work_item_run(
                &root,
                work_item_id,
                SelectRunAgentInput {
                    explicit_agent_profile_id: decision.target_agent_profile_id.as_deref(),
                    dispatch_plan_id: None,
                    path: input.path.or(snapshot.item.work_folder.as_deref()),
                },
            )?;
            let run = run_work_item_with_input(
                &root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: selection.agent_profile_id.as_str(),
                    dispatch_plan_id: selection.dispatch_plan_id.as_deref(),
                    path: input.path.or(snapshot.item.work_folder.as_deref()),
                    prompt: input.prompt,
                    dev_command: input.dev_command,
                    purpose: AgentRunPurpose::Work,
                },
            )?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: run.item_status,
                message: "work agent run completed".to_string(),
                run_id: Some(run.run.id),
                dispatch_plan_id: run.dispatch_plan_id.or(selection.dispatch_plan_id),
                recovery_plan_id: None,
                verification_id: None,
            })
        }
        WorkflowDecisionAction::RunReview => {
            let settings = get_nagare_agent_settings(&root)?;
            let run = run_work_item_with_input(
                &root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: settings.review_agent.as_str(),
                    dispatch_plan_id: None,
                    path: input.path.or(snapshot.item.work_folder.as_deref()),
                    prompt: input.prompt,
                    dev_command: input.review_dev_command.or(input.dev_command),
                    purpose: AgentRunPurpose::Review,
                },
            )?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: run.item_status,
                message: "review run completed".to_string(),
                run_id: Some(run.run.id),
                dispatch_plan_id: None,
                recovery_plan_id: None,
                verification_id: None,
            })
        }
        WorkflowDecisionAction::RunVerification => {
            let command = input
                .verification_command
                .or(snapshot.item.verification_hint.as_deref());
            let Some(command) = command else {
                return Ok(blocked_advance(
                    decision,
                    item_status,
                    "verification command required",
                ));
            };
            let verification = verify_work_item(&root, work_item_id, command)?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: verification.item_status,
                message: "verification completed".to_string(),
                run_id: None,
                dispatch_plan_id: None,
                recovery_plan_id: None,
                verification_id: Some(verification.verification.id),
            })
        }
        WorkflowDecisionAction::CreateRecoveryPlan => {
            let recovery = create_recovery_plan(&root, work_item_id)?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status,
                message: "recovery plan created".to_string(),
                run_id: None,
                dispatch_plan_id: None,
                recovery_plan_id: Some(recovery.plan.id),
                verification_id: None,
            })
        }
        WorkflowDecisionAction::AskHuman
        | WorkflowDecisionAction::CreateHandoff
        | WorkflowDecisionAction::Approve
        | WorkflowDecisionAction::Wait
        | WorkflowDecisionAction::Done
        | WorkflowDecisionAction::Stop
        | WorkflowDecisionAction::ApplyRecoveryPlan => Ok(blocked_advance(
            decision,
            item_status,
            "workflow requires external action",
        )),
    }
}

pub fn advance_work_item_until_blocked(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: AdvanceUntilBlockedInput<'_>,
) -> Result<AdvanceUntilBlockedResult, NagareError> {
    let root = root.into();
    let max_steps = input.max_steps.max(1);
    let mut steps = Vec::new();
    let mut stopped_reason = "max_steps_reached".to_string();

    for _ in 0..max_steps {
        let result = advance_work_item_once(&root, work_item_id, input.step.clone())?;
        let stop = !result.advanced
            || matches!(
                result.decision.action,
                WorkflowDecisionAction::AskHuman
                    | WorkflowDecisionAction::CreateHandoff
                    | WorkflowDecisionAction::CreateRecoveryPlan
                    | WorkflowDecisionAction::Approve
                    | WorkflowDecisionAction::Wait
                    | WorkflowDecisionAction::Done
                    | WorkflowDecisionAction::Stop
            );
        stopped_reason = if stop {
            result.message.clone()
        } else {
            stopped_reason
        };
        steps.push(result);
        if stop {
            break;
        }
    }

    let final_status = get_work_item_snapshot(&root, work_item_id)?.item.status;
    Ok(AdvanceUntilBlockedResult {
        steps,
        final_status,
        stopped_reason,
    })
}

fn record_workflow_decision(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<WorkflowDecision, NagareError> {
    let layout = ensure_project(root)?;
    let config = load_project_config(&layout)?;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let snapshot = WorkItemSnapshot::from_ledger(item, &ledger);
    let decision = workflow_decision_for_snapshot(
        &snapshot,
        &config.nagare_agents,
        ledger.next_id("wfd"),
        config.locale.language,
        None,
        WorkflowDecisionSource::Deterministic,
    );
    ledger.workflow_decisions.push(decision.clone());
    save_ledger(&layout, &ledger)?;
    Ok(decision)
}

fn record_supervisor_workflow_decision(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: SupervisorWorkflowDecisionInput<'_>,
) -> Result<WorkflowDecision, NagareError> {
    let root = root.into();
    let settings = get_nagare_agent_settings(&root)?;
    let run = run_work_item_with_input(
        &root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: settings.supervisor_agent.as_str(),
            dispatch_plan_id: None,
            path: input.path,
            prompt: input.prompt,
            dev_command: input.dev_command,
            purpose: AgentRunPurpose::WorkflowSupervision,
        },
    )?;
    let layout = ensure_project(&root)?;
    let config = load_project_config(&layout)?;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let snapshot = WorkItemSnapshot::from_ledger(item, &ledger);
    let fallback = workflow_decision_for_snapshot(
        &snapshot,
        &config.nagare_agents,
        ledger.next_id("wfd"),
        config.locale.language,
        Some(run.run.id.clone()),
        WorkflowDecisionSource::SupervisorAgent,
    );
    let output = ledger
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.agent_run_id == run.run.id);
    let decision = match output {
        Some(output) => workflow_decision_from_agent_output(fallback, output),
        None => fallback,
    };
    ledger.workflow_decisions.push(decision.clone());
    save_ledger(&layout, &ledger)?;
    Ok(decision)
}

fn workflow_decision_from_agent_output(
    mut decision: WorkflowDecision,
    output: &AgentOutputRecord,
) -> WorkflowDecision {
    if let Some(action) = first_field(output, "action") {
        match parse_workflow_decision_action(action) {
            Some(action) => decision.action = action,
            None => decision.warnings.push(format!(
                "unknown supervisor action `{action}`; used fallback"
            )),
        }
    }
    if let Some(reason) = first_field(output, "reason") {
        decision.reason = reason.to_string();
    }
    if let Some(target) = first_field(output, "target_agent_profile_id") {
        let target = target.trim();
        if !target.is_empty() && target != "-" {
            decision.target_agent_profile_id = Some(target.to_string());
        }
    }
    if let Some(requires_human) = first_field(output, "requires_human") {
        decision.requires_human = parse_workflow_bool(requires_human);
    }
    if let Some(confidence) = first_field(output, "confidence") {
        if let Ok(value) = confidence.trim().parse::<f32>() {
            decision.confidence = value;
        }
    }
    if let Some(command_hint) = first_field(output, "command_hint") {
        decision.command_hint = Some(command_hint.to_string());
    }
    decision
}

fn first_field<'a>(output: &'a AgentOutputRecord, key: &str) -> Option<&'a str> {
    output
        .fields
        .get(key)
        .and_then(|values| values.first())
        .map(String::as_str)
}

fn parse_workflow_bool(value: &str) -> bool {
    matches!(value.trim(), "true" | "yes" | "1")
}

fn parse_workflow_decision_action(value: &str) -> Option<WorkflowDecisionAction> {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "dispatch" => Some(WorkflowDecisionAction::Dispatch),
        "accept_dispatch" => Some(WorkflowDecisionAction::AcceptDispatch),
        "run_agent" => Some(WorkflowDecisionAction::RunAgent),
        "run_review" => Some(WorkflowDecisionAction::RunReview),
        "run_verification" => Some(WorkflowDecisionAction::RunVerification),
        "create_recovery_plan" => Some(WorkflowDecisionAction::CreateRecoveryPlan),
        "apply_recovery_plan" => Some(WorkflowDecisionAction::ApplyRecoveryPlan),
        "ask_human" => Some(WorkflowDecisionAction::AskHuman),
        "create_handoff" => Some(WorkflowDecisionAction::CreateHandoff),
        "approve" => Some(WorkflowDecisionAction::Approve),
        "wait" => Some(WorkflowDecisionAction::Wait),
        "done" => Some(WorkflowDecisionAction::Done),
        "stop" => Some(WorkflowDecisionAction::Stop),
        _ => None,
    }
}

fn blocked_advance(
    decision: WorkflowDecision,
    item_status: WorkItemStatus,
    message: &str,
) -> AdvanceWorkItemResult {
    AdvanceWorkItemResult {
        decision,
        advanced: false,
        item_status,
        message: message.to_string(),
        run_id: None,
        dispatch_plan_id: None,
        recovery_plan_id: None,
        verification_id: None,
    }
}

pub(crate) fn workflow_decision_for_snapshot(
    snapshot: &WorkItemSnapshot,
    settings: &NagareAgentSettings,
    id: String,
    locale: String,
    agent_run_id: Option<String>,
    source: WorkflowDecisionSource,
) -> WorkflowDecision {
    let accepted_dispatch = latest_dispatch_plan(snapshot, DispatchPlanStatus::Accepted);
    let draft_dispatch = latest_dispatch_plan(snapshot, DispatchPlanStatus::Draft);
    let passing_verification = snapshot
        .verification_results
        .iter()
        .any(|verification| verification.result == VerificationStatus::Passed);
    let latest_review_pass = snapshot
        .review_results
        .iter()
        .rev()
        .any(|review| review.verdict == ReviewVerdict::Pass);

    let (action, reason, requires_human, target_agent_profile_id, command_hint) =
        match snapshot.item.status {
            WorkItemStatus::Done => (
                WorkflowDecisionAction::Done,
                "work_item_done".to_string(),
                false,
                None,
                None,
            ),
            WorkItemStatus::AgentRunning => (
                WorkflowDecisionAction::Wait,
                "agent_running".to_string(),
                false,
                None,
                None,
            ),
            WorkItemStatus::NeedsInput => (
                WorkflowDecisionAction::AskHuman,
                snapshot
                    .completion
                    .blocking_reason
                    .clone()
                    .unwrap_or_else(|| "needs_input".to_string()),
                true,
                None,
                snapshot.completion.next_command_hint.clone(),
            ),
            WorkItemStatus::NeedsHandoff if snapshot.handoffs.is_empty() => (
                WorkflowDecisionAction::CreateHandoff,
                "needs_handoff".to_string(),
                true,
                None,
                snapshot.completion.next_command_hint.clone(),
            ),
            WorkItemStatus::NeedsHandoff => (
                WorkflowDecisionAction::Dispatch,
                "handoff_created".to_string(),
                false,
                Some(settings.dispatch_agent.clone()),
                Some(format!("nagare handoff dispatch {}", snapshot.item.id)),
            ),
            WorkItemStatus::FailedVerification => (
                WorkflowDecisionAction::CreateRecoveryPlan,
                "failed_verification".to_string(),
                false,
                None,
                Some(format!("nagare item recover {}", snapshot.item.id)),
            ),
            WorkItemStatus::ChangesRequested => (
                WorkflowDecisionAction::CreateRecoveryPlan,
                "changes_requested".to_string(),
                false,
                None,
                Some(format!("nagare item recover {}", snapshot.item.id)),
            ),
            WorkItemStatus::ReadyForVerification => (
                WorkflowDecisionAction::RunVerification,
                "ready_for_verification".to_string(),
                snapshot.item.verification_hint.is_none(),
                None,
                Some(format!(
                    "nagare verify {} --command <command>",
                    snapshot.item.id
                )),
            ),
            WorkItemStatus::ReadyForReview if passing_verification => (
                WorkflowDecisionAction::Approve,
                "verification_passed".to_string(),
                true,
                None,
                Some(format!("nagare decision approve {}", snapshot.item.id)),
            ),
            WorkItemStatus::ReadyForReview if latest_review_pass => (
                WorkflowDecisionAction::RunVerification,
                "review_passed".to_string(),
                snapshot.item.verification_hint.is_none(),
                None,
                Some(format!(
                    "nagare verify {} --command <command>",
                    snapshot.item.id
                )),
            ),
            WorkItemStatus::ReadyForReview => (
                WorkflowDecisionAction::RunReview,
                "ready_for_review".to_string(),
                false,
                Some(settings.review_agent.clone()),
                Some(format!("nagare item review {}", snapshot.item.id)),
            ),
            WorkItemStatus::Ready if accepted_dispatch.is_some() => (
                WorkflowDecisionAction::RunAgent,
                "accepted_dispatch_plan".to_string(),
                false,
                accepted_dispatch.map(|plan| plan.target_agent_profile_id.clone()),
                Some(format!("nagare item run {}", snapshot.item.id)),
            ),
            WorkItemStatus::Ready if draft_dispatch.is_some() => (
                WorkflowDecisionAction::AcceptDispatch,
                "draft_dispatch_plan".to_string(),
                false,
                draft_dispatch.map(|plan| plan.target_agent_profile_id.clone()),
                Some(format!("nagare item dispatch accept {}", snapshot.item.id)),
            ),
            WorkItemStatus::Ready => (
                WorkflowDecisionAction::Dispatch,
                "ready_without_dispatch".to_string(),
                false,
                Some(settings.dispatch_agent.clone()),
                Some(format!("nagare item preview {}", snapshot.item.id)),
            ),
        };

    WorkflowDecision {
        id,
        work_item_id: snapshot.item.id.clone(),
        action,
        source,
        reason,
        requires_human,
        target_agent_profile_id,
        agent_run_id,
        confidence: 0.7,
        command_hint,
        warnings: Vec::new(),
        locale,
        created_at: timestamp(),
    }
}

fn latest_dispatch_plan(
    snapshot: &WorkItemSnapshot,
    status: DispatchPlanStatus,
) -> Option<&DispatchPlan> {
    snapshot
        .dispatch_plans
        .iter()
        .rev()
        .find(|plan| plan.status == status)
}
