use crate::*;
use std::path::PathBuf;

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
    let auto_recover = effective_auto_recover(&snapshot, &input);

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
            })
        }
        WorkflowDecisionAction::AcceptRecoveryPlan => {
            let draft_plan = latest_unapplied_draft_recovery(&snapshot);
            if !auto_recover {
                return Ok(AdvanceWorkItemResult {
                    decision,
                    advanced: false,
                    item_status,
                    message: "recovery plan acceptance required".to_string(),
                    run_id: None,
                    dispatch_plan_id: None,
                    recovery_plan_id: draft_plan.map(|plan| plan.id.clone()),
                });
            }
            if draft_plan
                .map(|plan| !recovery_plan_can_auto_apply(plan))
                .unwrap_or(false)
            {
                return Ok(AdvanceWorkItemResult {
                    decision,
                    advanced: false,
                    item_status,
                    message: "recovery plan requires external action".to_string(),
                    run_id: None,
                    dispatch_plan_id: None,
                    recovery_plan_id: draft_plan.map(|plan| plan.id.clone()),
                });
            }
            let accepted = accept_recovery_plan(&root, work_item_id, None)?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status,
                message: format!("recovery plan {} accepted", accepted.plan.id),
                run_id: None,
                dispatch_plan_id: None,
                recovery_plan_id: Some(accepted.plan.id),
            })
        }
        WorkflowDecisionAction::ApplyRecoveryPlan => {
            let recovery = apply_recovery_plan(
                &root,
                work_item_id,
                ApplyRecoveryPlanInput {
                    recovery_plan_id: None,
                    prompt: input.prompt,
                    dev_command: input.dev_command,
                },
            )?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: recovery.run.item_status,
                message: format!("recovery plan {} applied", recovery.plan.id),
                run_id: Some(recovery.run.run.id),
                dispatch_plan_id: recovery.run.dispatch_plan_id,
                recovery_plan_id: Some(recovery.plan.id),
            })
        }
        WorkflowDecisionAction::Done if item_status == WorkItemStatus::ReadyForReview => {
            let approved = approve_work_item(
                &root,
                work_item_id,
                "review passed and approval_policy=auto_complete_on_review_pass",
            )?;
            Ok(AdvanceWorkItemResult {
                decision,
                advanced: true,
                item_status: approved.item_status,
                message: "work item auto-completed after passing review".to_string(),
                run_id: None,
                dispatch_plan_id: None,
                recovery_plan_id: None,
            })
        }
        WorkflowDecisionAction::AskHuman
        | WorkflowDecisionAction::CreateHandoff
        | WorkflowDecisionAction::Approve
        | WorkflowDecisionAction::Wait
        | WorkflowDecisionAction::Done
        | WorkflowDecisionAction::Stop => Ok(blocked_advance(
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
                    | WorkflowDecisionAction::Approve
                    | WorkflowDecisionAction::Wait
                    | WorkflowDecisionAction::Done
                    | WorkflowDecisionAction::Stop
            )
            || (result.decision.action == WorkflowDecisionAction::CreateRecoveryPlan
                && !advance_auto_recover_enabled(&root, work_item_id, &input.step)?);
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
        "create_recovery_plan" => Some(WorkflowDecisionAction::CreateRecoveryPlan),
        "accept_recovery_plan" => Some(WorkflowDecisionAction::AcceptRecoveryPlan),
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
    }
}

fn effective_auto_recover(snapshot: &WorkItemSnapshot, input: &AdvanceWorkItemInput<'_>) -> bool {
    input.auto_recover
        || input.workflow_mode.unwrap_or(snapshot.item.workflow_mode) == WorkflowMode::FinishFirst
}

fn advance_auto_recover_enabled(
    root: &std::path::Path,
    work_item_id: &str,
    input: &AdvanceWorkItemInput<'_>,
) -> Result<bool, NagareError> {
    if input.auto_recover {
        return Ok(true);
    }
    if let Some(mode) = input.workflow_mode {
        return Ok(mode == WorkflowMode::FinishFirst);
    }
    Ok(get_work_item_snapshot(root, work_item_id)?
        .item
        .workflow_mode
        == WorkflowMode::FinishFirst)
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
    let latest_work_sequence = latest_work_run_sequence(snapshot);
    let latest_review_pass = snapshot.review_results.iter().rev().any(|review| {
        review.verdict == ReviewVerdict::Pass && id_sequence(&review.id) > latest_work_sequence
    });
    let accepted_recovery = latest_unapplied_accepted_recovery(snapshot);
    let draft_recovery = latest_unapplied_draft_recovery(snapshot);

    let (action, reason, requires_human, target_agent_profile_id, command_hint) =
        if let Some(plan) = accepted_recovery {
            (
                WorkflowDecisionAction::ApplyRecoveryPlan,
                format!("accepted_recovery_plan: {}", plan.failure_class),
                false,
                plan.target_agent_profile_id.clone(),
                Some(format!("nagare item recover apply {}", snapshot.item.id)),
            )
        } else if let Some(plan) = draft_recovery {
            (
                WorkflowDecisionAction::AcceptRecoveryPlan,
                format!("draft_recovery_plan: {}", plan.failure_class),
                true,
                plan.target_agent_profile_id.clone(),
                Some(format!("nagare item recover accept {}", snapshot.item.id)),
            )
        } else {
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
                WorkItemStatus::NeedsHandoff if accepted_dispatch.is_some() => (
                    WorkflowDecisionAction::RunAgent,
                    "accepted_handoff_dispatch_plan".to_string(),
                    false,
                    accepted_dispatch.map(|plan| plan.target_agent_profile_id.clone()),
                    Some(format!("nagare item run {}", snapshot.item.id)),
                ),
                WorkItemStatus::NeedsHandoff if draft_dispatch.is_some() => (
                    WorkflowDecisionAction::AcceptDispatch,
                    "draft_handoff_dispatch_plan".to_string(),
                    false,
                    draft_dispatch.map(|plan| plan.target_agent_profile_id.clone()),
                    Some(format!("nagare item dispatch accept {}", snapshot.item.id)),
                ),
                WorkItemStatus::NeedsHandoff => (
                    WorkflowDecisionAction::Dispatch,
                    "handoff_created".to_string(),
                    false,
                    Some(settings.dispatch_agent.clone()),
                    Some(format!("nagare handoff dispatch {}", snapshot.item.id)),
                ),
                WorkItemStatus::ChangesRequested => (
                    WorkflowDecisionAction::CreateRecoveryPlan,
                    "changes_requested".to_string(),
                    false,
                    None,
                    Some(format!("nagare item recover {}", snapshot.item.id)),
                ),
                WorkItemStatus::ReadyForReview if latest_review_pass => {
                    if snapshot.item.approval_policy == ApprovalPolicy::AutoCompleteOnReviewPass {
                        (
                            WorkflowDecisionAction::Done,
                            "review_passed_auto_complete".to_string(),
                            false,
                            None,
                            Some(format!("nagare item advance {}", snapshot.item.id)),
                        )
                    } else {
                        (
                            WorkflowDecisionAction::Approve,
                            "review_passed".to_string(),
                            true,
                            None,
                            Some(format!("nagare decision approve {}", snapshot.item.id)),
                        )
                    }
                }
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
                WorkItemStatus::Ready if has_answer_after_latest_question(snapshot) => (
                    WorkflowDecisionAction::RunAgent,
                    "human_feedback_answered".to_string(),
                    false,
                    Some(settings.work_agent.clone()),
                    Some(format!("nagare item run {}", snapshot.item.id)),
                ),
                WorkItemStatus::Ready => (
                    WorkflowDecisionAction::Dispatch,
                    "ready_without_dispatch".to_string(),
                    false,
                    Some(settings.dispatch_agent.clone()),
                    Some(format!("nagare item preview {}", snapshot.item.id)),
                ),
            }
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

fn latest_unapplied_accepted_recovery(snapshot: &WorkItemSnapshot) -> Option<&RecoveryPlan> {
    snapshot.recovery_plans.iter().rev().find(|plan| {
        plan.status == RecoveryPlanStatus::Accepted
            && !snapshot.runs.iter().any(|run| {
                run.purpose == AgentRunPurpose::Work && id_sequence(&run.id) > id_sequence(&plan.id)
            })
    })
}

fn latest_unapplied_draft_recovery(snapshot: &WorkItemSnapshot) -> Option<&RecoveryPlan> {
    snapshot.recovery_plans.iter().rev().find(|plan| {
        plan.status == RecoveryPlanStatus::Draft
            && !snapshot.runs.iter().any(|run| {
                run.purpose == AgentRunPurpose::Work && id_sequence(&run.id) > id_sequence(&plan.id)
            })
    })
}

fn recovery_plan_can_auto_apply(plan: &RecoveryPlan) -> bool {
    matches!(
        plan.action,
        RecoveryAction::RerunSameAgent | RecoveryAction::RerunWithContractReminder
    )
}

fn latest_work_run_sequence(snapshot: &WorkItemSnapshot) -> u64 {
    snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work)
        .map(|run| id_sequence(&run.id))
        .unwrap_or(0)
}

fn has_answer_after_latest_question(snapshot: &WorkItemSnapshot) -> bool {
    let Some(output) = snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| !output.questions.is_empty())
    else {
        return false;
    };
    snapshot.human_feedback.iter().rev().any(|feedback| {
        feedback.source_agent_output_id.as_deref() == Some(output.id.as_str())
            && id_sequence(&feedback.id) > id_sequence(&output.id)
    })
}

fn id_sequence(id: &str) -> u64 {
    id.rsplit('_')
        .next()
        .and_then(|suffix| suffix.parse::<u64>().ok())
        .unwrap_or(0)
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
