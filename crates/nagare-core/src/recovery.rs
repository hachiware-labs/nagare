use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub work_item_id: String,
    #[serde(default = "default_recovery_plan_status")]
    pub status: RecoveryPlanStatus,
    pub action: RecoveryAction,
    pub target_agent_profile_id: Option<String>,
    #[serde(default = "default_recovery_failure_class")]
    pub failure_class: String,
    pub reason: String,
    pub summary: String,
    pub source_event_id: Option<String>,
    pub command_hint: Option<String>,
    pub prompt_hint: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPlanStatus {
    Draft,
    Accepted,
    Superseded,
}

impl std::fmt::Display for RecoveryPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Draft => "draft",
            Self::Accepted => "accepted",
            Self::Superseded => "superseded",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    RerunSameAgent,
    RerunWithContractReminder,
    Handoff,
    AskHuman,
    RequestChanges,
    Redispatch,
}

impl std::fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::RerunSameAgent => "rerun_same_agent",
            Self::RerunWithContractReminder => "rerun_with_contract_reminder",
            Self::Handoff => "handoff",
            Self::AskHuman => "ask_human",
            Self::RequestChanges => "request_changes",
            Self::Redispatch => "redispatch",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone)]
pub struct CreateRecoveryPlanResult {
    pub plan: RecoveryPlan,
}

#[derive(Debug, Clone)]
pub struct AcceptRecoveryPlanResult {
    pub plan: RecoveryPlan,
}

#[derive(Debug, Clone)]
pub struct ApplyRecoveryPlanInput<'a> {
    pub recovery_plan_id: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub dev_command: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct ApplyRecoveryPlanResult {
    pub plan: RecoveryPlan,
    pub run: RunWorkItemResult,
}

pub fn create_recovery_plan(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<CreateRecoveryPlanResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let snapshot = WorkItemSnapshot::from_ledger(item, &ledger);
    let mut plans = recovery_plans_for_snapshot(&snapshot, &layout, &mut ledger, &locale)?;
    for existing in &mut ledger.recovery_plans {
        if existing.work_item_id == work_item_id && existing.status == RecoveryPlanStatus::Draft {
            existing.status = RecoveryPlanStatus::Superseded;
        }
    }
    for plan in &mut plans {
        plan.status = RecoveryPlanStatus::Draft;
    }
    let plan = plans
        .first()
        .cloned()
        .ok_or_else(|| NagareError::InvalidState("no recovery plan candidate".to_string()))?;
    ledger.recovery_plans.extend(plans);
    save_ledger(&layout, &ledger)?;
    Ok(CreateRecoveryPlanResult { plan })
}

pub fn accept_recovery_plan(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    recovery_plan_id: Option<&str>,
) -> Result<AcceptRecoveryPlanResult, NagareError> {
    let layout = ensure_project(root)?;
    let mut ledger = load_ledger(&layout)?;
    ledger.work_item(work_item_id)?;
    let selected_index = match recovery_plan_id {
        Some(id) => ledger
            .recovery_plans
            .iter()
            .position(|plan| plan.work_item_id == work_item_id && plan.id == id),
        None => ledger.recovery_plans.iter().rposition(|plan| {
            plan.work_item_id == work_item_id && plan.status == RecoveryPlanStatus::Draft
        }),
    }
    .ok_or_else(|| {
        let target = recovery_plan_id.unwrap_or("latest draft");
        NagareError::NotFound(format!("recovery plan `{target}` for `{work_item_id}`"))
    })?;
    let selected_id = ledger.recovery_plans[selected_index].id.clone();
    for plan in &mut ledger.recovery_plans {
        if plan.work_item_id == work_item_id && plan.status != RecoveryPlanStatus::Superseded {
            plan.status = if plan.id == selected_id {
                RecoveryPlanStatus::Accepted
            } else {
                RecoveryPlanStatus::Superseded
            };
        }
    }
    let plan = ledger.recovery_plans[selected_index].clone();
    save_ledger(&layout, &ledger)?;
    Ok(AcceptRecoveryPlanResult { plan })
}

pub fn apply_recovery_plan(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: ApplyRecoveryPlanInput<'_>,
) -> Result<ApplyRecoveryPlanResult, NagareError> {
    let root = root.into();
    let layout = ensure_project(&root)?;
    let ledger = load_ledger(&layout)?;
    ledger.work_item(work_item_id)?;
    let plan = recovery_plan_for_apply(&ledger, work_item_id, input.recovery_plan_id)?.clone();
    if !matches!(
        plan.action,
        RecoveryAction::RerunSameAgent | RecoveryAction::RerunWithContractReminder
    ) {
        return Err(NagareError::InvalidState(format!(
            "recovery plan `{}` action `{}` cannot be applied as an agent rerun",
            plan.id, plan.action
        )));
    }
    let agent_profile_id = plan.target_agent_profile_id.clone().ok_or_else(|| {
        NagareError::InvalidState(format!(
            "recovery plan `{}` has no target agent profile",
            plan.id
        ))
    })?;
    let purpose = recovery_rerun_purpose(&ledger, &plan);
    let run = run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: &agent_profile_id,
            dispatch_plan_id: None,
            path: None,
            prompt: input.prompt.or(plan.prompt_hint.as_deref()),
            dev_command: input.dev_command,
            purpose,
        },
    )?;
    Ok(ApplyRecoveryPlanResult { plan, run })
}

fn recovery_rerun_purpose(ledger: &Ledger, plan: &RecoveryPlan) -> AgentRunPurpose {
    plan.source_event_id
        .as_deref()
        .and_then(|source_event_id| {
            ledger
                .agent_outputs
                .iter()
                .find(|output| output.id == source_event_id)
        })
        .map(|output| output.purpose)
        .filter(|purpose| matches!(purpose, AgentRunPurpose::Work | AgentRunPurpose::Review))
        .unwrap_or(AgentRunPurpose::Work)
}

fn recovery_plan_for_apply<'a>(
    ledger: &'a Ledger,
    work_item_id: &str,
    recovery_plan_id: Option<&str>,
) -> Result<&'a RecoveryPlan, NagareError> {
    let plan = match recovery_plan_id {
        Some(id) => ledger
            .recovery_plans
            .iter()
            .find(|plan| plan.work_item_id == work_item_id && plan.id == id),
        None => ledger
            .recovery_plans
            .iter()
            .rposition(|plan| {
                plan.work_item_id == work_item_id && plan.status == RecoveryPlanStatus::Accepted
            })
            .map(|index| &ledger.recovery_plans[index]),
    }
    .ok_or_else(|| {
        let target = recovery_plan_id.unwrap_or("latest accepted");
        NagareError::NotFound(format!("recovery plan `{target}` for `{work_item_id}`"))
    })?;
    if plan.status != RecoveryPlanStatus::Accepted {
        return Err(NagareError::InvalidState(format!(
            "recovery plan `{}` must be accepted before apply; current status is {}",
            plan.id, plan.status
        )));
    }
    Ok(plan)
}

fn recovery_plans_for_snapshot(
    snapshot: &WorkItemSnapshot,
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    locale: &str,
) -> Result<Vec<RecoveryPlan>, NagareError> {
    let primary = recovery_plan_for_snapshot(snapshot, layout, ledger, locale)?;
    let mut plans = vec![primary];
    plans.extend(additional_recovery_candidates(
        snapshot, layout, ledger, locale,
    )?);
    Ok(plans)
}

fn recovery_plan_for_snapshot(
    snapshot: &WorkItemSnapshot,
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    locale: &str,
) -> Result<RecoveryPlan, NagareError> {
    let latest_work_agent = snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work)
        .map(|run| run.agent_profile_id.clone())
        .or_else(|| {
            load_project_config(layout)
                .ok()
                .map(|config| config.nagare_agents.work_agent)
        });
    let unparsed = latest_unparsed_output(snapshot);
    let latest_review_change = snapshot
        .review_results
        .iter()
        .rev()
        .find(|review| review.verdict == ReviewVerdict::RequestChanges);
    let notes_missing_output = latest_output_missing_notes(snapshot);

    let (
        action,
        target_agent_profile_id,
        reason,
        summary,
        source_event_id,
        command_hint,
        prompt_hint,
        failure_class,
    ) = if let Some(output) = unparsed {
        (
            RecoveryAction::RerunWithContractReminder,
            Some(output.agent_profile_id.clone()),
            "output_contract_missing".to_string(),
            format!(
                "Ask `{}` to restate the final output using `{}`.",
                output.agent_profile_id, output.contract
            ),
            Some(output.id.clone()),
            Some(format!("nagare item recover apply {}", snapshot.item.id)),
            Some(format!(
                "Restate the previous run output for Work Item `{}` using the required `{}` contract. Return only the final contract block.",
                snapshot.item.id, output.contract
            )),
            "contract_violation".to_string(),
        )
    } else if snapshot.item.status == WorkItemStatus::NeedsInput {
        (
            RecoveryAction::AskHuman,
            None,
            "needs_input".to_string(),
            snapshot
                .completion
                .blocking_reason
                .clone()
                .unwrap_or_else(|| "Answer the open agent question.".to_string()),
            latest_question_event(snapshot),
            snapshot.completion.next_command_hint.clone(),
            None,
            "missing_input".to_string(),
        )
    } else if snapshot.item.status == WorkItemStatus::NeedsHandoff && snapshot.handoffs.is_empty() {
        (
            RecoveryAction::Handoff,
            latest_work_agent.clone(),
            "needs_handoff".to_string(),
            "Create a HandoffPacket to move the Work Item to a better agent.".to_string(),
            latest_agent_output_event(snapshot),
            snapshot.completion.next_command_hint.clone(),
            None,
            "needs_handoff".to_string(),
        )
    } else if snapshot.item.status == WorkItemStatus::NeedsHandoff {
        (
            RecoveryAction::Redispatch,
            latest_work_agent.clone(),
            "handoff_created".to_string(),
            "Run handoff dispatch and accept a target agent before continuing.".to_string(),
            snapshot.handoffs.last().map(|handoff| handoff.id.clone()),
            snapshot.completion.next_command_hint.clone(),
            None,
            "needs_handoff".to_string(),
        )
    } else if let Some(review) = latest_review_change {
        (
            RecoveryAction::RerunSameAgent,
            latest_work_agent.clone(),
            "changes_requested".to_string(),
            review
                .requested_changes
                .first()
                .cloned()
                .unwrap_or_else(|| "Address requested review changes.".to_string()),
            Some(review.id.clone()),
            Some(format!("nagare item run {}", snapshot.item.id)),
            None,
            "review_changes".to_string(),
        )
    } else if let Some(output) = notes_missing_output {
        let missing = missing_output_notes(output);
        (
            RecoveryAction::RerunWithContractReminder,
            Some(output.agent_profile_id.clone()),
            "output_notes_missing".to_string(),
            format!(
                "Ask `{}` to restate `{}` with missing notes: {}.",
                output.agent_profile_id,
                output.contract,
                missing.join(", ")
            ),
            Some(output.id.clone()),
            Some(format!("nagare item recover apply {}", snapshot.item.id)),
            Some(format!(
                "Restate the previous run output for Work Item `{}` using `{}` and include: {}. Return only the final contract block.",
                snapshot.item.id,
                output.contract,
                missing.join(", ")
            )),
            "output_notes_missing".to_string(),
        )
    } else {
        (
            RecoveryAction::RerunSameAgent,
            latest_work_agent.clone(),
            "continue_workflow".to_string(),
            "Continue with the next Work Item action.".to_string(),
            snapshot.timeline.last().map(|event| event.id.clone()),
            snapshot.completion.next_command_hint.clone(),
            None,
            "continue_workflow".to_string(),
        )
    };

    Ok(RecoveryPlan {
        id: ledger.next_id("recovery"),
        work_item_id: snapshot.item.id.clone(),
        status: RecoveryPlanStatus::Draft,
        action,
        target_agent_profile_id,
        failure_class,
        reason,
        summary,
        source_event_id,
        command_hint,
        prompt_hint,
        warnings: Vec::new(),
        locale: locale.to_string(),
        created_at: timestamp(),
    })
}

fn additional_recovery_candidates(
    snapshot: &WorkItemSnapshot,
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    locale: &str,
) -> Result<Vec<RecoveryPlan>, NagareError> {
    let mut plans = Vec::new();
    let latest_work_agent = snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work)
        .map(|run| run.agent_profile_id.clone())
        .or_else(|| {
            load_project_config(layout)
                .ok()
                .map(|config| config.nagare_agents.work_agent)
        });
    let latest_work_run = snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work);
    let has_diff = snapshot
        .artifacts
        .iter()
        .any(|artifact| artifact.artifact_type == "diff_patch");

    let missing_expected_artifacts = missing_expected_artifacts(snapshot);
    if !missing_expected_artifacts.is_empty() {
        plans.push(RecoveryPlan {
            id: ledger.next_id("recovery"),
            work_item_id: snapshot.item.id.clone(),
            status: RecoveryPlanStatus::Draft,
            action: RecoveryAction::RequestChanges,
            target_agent_profile_id: latest_work_agent.clone(),
            failure_class: "missing_artifact".to_string(),
            reason: "expected_artifact_missing".to_string(),
            summary: format!(
                "Expected artifacts are missing from the latest agent output: {}.",
                missing_expected_artifacts.join(", ")
            ),
            source_event_id: latest_work_run.map(|run| run.id.clone()),
            command_hint: Some(format!("nagare item run {}", snapshot.item.id)),
            prompt_hint: Some(format!(
                "Produce or reference the missing expected artifacts: {}.",
                missing_expected_artifacts.join(", ")
            )),
            warnings: Vec::new(),
            locale: locale.to_string(),
            created_at: timestamp(),
        });
    }

    if latest_work_run.is_some() && !has_diff && !snapshot.item.expected_artifacts.is_empty() {
        plans.push(RecoveryPlan {
            id: ledger.next_id("recovery"),
            work_item_id: snapshot.item.id.clone(),
            status: RecoveryPlanStatus::Draft,
            action: RecoveryAction::RerunSameAgent,
            target_agent_profile_id: latest_work_agent,
            failure_class: "no_diff".to_string(),
            reason: "no_diff_artifact".to_string(),
            summary: "No diff artifact was collected after the latest work run.".to_string(),
            source_event_id: latest_work_run.map(|run| run.id.clone()),
            command_hint: Some(format!("nagare item run {}", snapshot.item.id)),
            prompt_hint: Some(
                "Create concrete workspace changes or explain why no diff is expected.".to_string(),
            ),
            warnings: Vec::new(),
            locale: locale.to_string(),
            created_at: timestamp(),
        });
    }

    Ok(plans)
}

fn missing_expected_artifacts(snapshot: &WorkItemSnapshot) -> Vec<String> {
    if snapshot.item.expected_artifacts.is_empty() {
        return Vec::new();
    }
    let reported = snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.purpose == AgentRunPurpose::Work)
        .and_then(|output| output.fields.get("artifacts"))
        .cloned()
        .unwrap_or_default();
    snapshot
        .item
        .expected_artifacts
        .iter()
        .filter(|expected| {
            !reported
                .iter()
                .any(|artifact| artifact.contains(expected.as_str()))
        })
        .cloned()
        .collect()
}

fn latest_unparsed_output(snapshot: &WorkItemSnapshot) -> Option<&AgentOutputRecord> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
}

fn latest_output_missing_notes(snapshot: &WorkItemSnapshot) -> Option<&AgentOutputRecord> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| {
            matches!(
                output.purpose,
                AgentRunPurpose::Work | AgentRunPurpose::Review
            )
        })
        .filter(|output| !missing_output_notes(output).is_empty())
}

fn missing_output_notes(output: &AgentOutputRecord) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if output
        .warnings
        .iter()
        .any(|warning| warning == "missing_completed")
    {
        missing.push("completed");
    }
    if output
        .warnings
        .iter()
        .any(|warning| warning == "missing_next_notes")
    {
        missing.push("next_notes");
    }
    missing
}

fn latest_question_event(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .timeline
        .iter()
        .rev()
        .find(|event| event.event_type == "question")
        .map(|event| event.id.clone())
}

fn latest_agent_output_event(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .timeline
        .iter()
        .rev()
        .find(|event| event.event_type == "agent_output")
        .map(|event| event.id.clone())
}

fn default_recovery_plan_status() -> RecoveryPlanStatus {
    RecoveryPlanStatus::Draft
}

fn default_recovery_failure_class() -> String {
    "general".to_string()
}
