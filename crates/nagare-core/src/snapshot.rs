use crate::*;

#[derive(Debug, Clone)]
pub struct WorkItemSnapshot {
    pub item: WorkItem,
    pub completion: WorkItemCompletion,
    pub approval_gate: WorkItemApprovalGate,
    pub runs: Vec<AgentRun>,
    pub artifacts: Vec<Artifact>,
    pub execution_records: Vec<ExecutionRecord>,
    pub evidence: Vec<Evidence>,
    pub review_results: Vec<ReviewResult>,
    pub handoffs: Vec<HandoffPacket>,
    pub decisions: Vec<HumanDecision>,
    pub human_feedback: Vec<HumanFeedback>,
    pub dispatch_plans: Vec<DispatchPlan>,
    pub recovery_plans: Vec<RecoveryPlan>,
    pub workflow_decisions: Vec<WorkflowDecision>,
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
    pub agent_outputs: Vec<AgentOutputRecord>,
    pub timeline: Vec<WorkItemTimelineEvent>,
    pub history_steps: Vec<WorkItemHistoryStep>,
}

#[derive(Debug, Clone)]
pub struct WorkItemCompletion {
    pub state: String,
    pub blocking_reason: Option<String>,
    pub next_action: String,
    pub next_command_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkItemApprovalGate {
    pub state: String,
    pub ready: bool,
    pub latest_review_id: Option<String>,
    pub criteria_passed: usize,
    pub criteria_total: usize,
    pub artifact_count: usize,
    pub recovery_count: usize,
    pub blockers: Vec<String>,
    pub command_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkItemTimelineEvent {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub status: String,
    pub actor: Option<String>,
    pub related_record_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct WorkItemHistoryStep {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub state: String,
    pub actor: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub summary: String,
    pub facts: Vec<WorkItemHistoryFact>,
    pub links: Vec<WorkItemHistoryLink>,
    pub source_record_ids: Vec<String>,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkItemHistoryFact {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct WorkItemHistoryLink {
    pub label: String,
    pub record_id: String,
    pub record_type: String,
}

impl WorkItemSnapshot {
    pub(crate) fn from_ledger(item: WorkItem, ledger: &Ledger) -> Self {
        let item_id = &item.id;
        let mut snapshot = Self {
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
            execution_records: ledger
                .execution_records
                .iter()
                .filter(|record| &record.work_item_id == item_id)
                .cloned()
                .collect(),
            evidence: ledger
                .evidence
                .iter()
                .filter(|evidence| &evidence.work_item_id == item_id)
                .cloned()
                .collect(),
            review_results: ledger
                .review_results
                .iter()
                .filter(|review| &review.work_item_id == item_id)
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
            human_feedback: ledger
                .human_feedback
                .iter()
                .filter(|feedback| &feedback.work_item_id == item_id)
                .cloned()
                .collect(),
            dispatch_plans: ledger
                .dispatch_plans
                .iter()
                .filter(|plan| &plan.work_item_id == item_id)
                .cloned()
                .collect(),
            recovery_plans: ledger
                .recovery_plans
                .iter()
                .filter(|plan| &plan.work_item_id == item_id)
                .cloned()
                .collect(),
            workflow_decisions: ledger
                .workflow_decisions
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
            agent_outputs: ledger
                .agent_outputs
                .iter()
                .filter(|output| &output.work_item_id == item_id)
                .cloned()
                .collect(),
            completion: WorkItemCompletion {
                state: "unknown".to_string(),
                blocking_reason: None,
                next_action: "inspect".to_string(),
                next_command_hint: None,
            },
            approval_gate: empty_approval_gate(),
            timeline: Vec::new(),
            history_steps: Vec::new(),
            item,
        };
        snapshot.timeline = timeline_events(&snapshot);
        snapshot.completion = completion_state(&snapshot);
        snapshot.approval_gate = approval_gate_state(&snapshot);
        snapshot.history_steps = history_steps(&snapshot);
        snapshot
    }
}

fn completion_state(snapshot: &WorkItemSnapshot) -> WorkItemCompletion {
    match snapshot.item.status {
        WorkItemStatus::Done => completion("done", None, "done", None),
        WorkItemStatus::AgentRunning => completion("in_progress", None, "wait", None),
        WorkItemStatus::NeedsInput => completion(
            "blocked",
            latest_question(snapshot).or_else(|| Some("needs_input".to_string())),
            "answer_question",
            Some(format!(
                "nagare item answer {} --answer <text>",
                snapshot.item.id
            )),
        ),
        WorkItemStatus::NeedsHandoff => {
            if snapshot.handoffs.is_empty() {
                completion(
                    "blocked",
                    latest_agent_next_action(snapshot)
                        .or_else(|| Some("needs_handoff".to_string())),
                    "create_handoff",
                    Some(format!(
                        "nagare handoff create {} --from-agent <agent> --to-agent <agent> --reason <text>",
                        snapshot.item.id
                    )),
                )
            } else if latest_dispatch_plan(snapshot, DispatchPlanStatus::Accepted).is_some() {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
                )
            } else if latest_dispatch_plan(snapshot, DispatchPlanStatus::Draft).is_some() {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
                )
            } else {
                completion(
                    "ready",
                    Some("handoff_created".to_string()),
                    "dispatch",
                    Some(format!("nagare handoff dispatch {}", snapshot.item.id)),
                )
            }
        }
        WorkItemStatus::ChangesRequested if latest_unparsed_output(snapshot).is_some() => {
            completion(
                "blocked",
                latest_unparsed_output(snapshot)
                    .map(|output| format!("contract_violation: {}", output.contract)),
                "recover",
                Some(format!("nagare item recover {}", snapshot.item.id)),
            )
        }
        WorkItemStatus::ChangesRequested => completion(
            "blocked",
            latest_requested_change(snapshot).or_else(|| Some("changes_requested".to_string())),
            "run_agent",
            Some(format!("nagare item run {}", snapshot.item.id)),
        ),
        WorkItemStatus::ReadyForReview => {
            if work_item_needs_synthesis(snapshot) {
                completion(
                    "ready_for_synthesis",
                    None,
                    "synthesize",
                    Some(format!("nagare item synthesize {}", snapshot.item.id)),
                )
            } else if latest_review_pass_after_latest_work(snapshot).is_some() {
                completion(
                    "ready_for_approval",
                    None,
                    "approve",
                    Some(format!("nagare decision approve {}", snapshot.item.id)),
                )
            } else {
                completion(
                    "ready_for_review",
                    None,
                    "review",
                    Some(format!("nagare item review {}", snapshot.item.id)),
                )
            }
        }
        WorkItemStatus::Ready => {
            if latest_dispatch_plan(snapshot, DispatchPlanStatus::Accepted).is_some() {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
                )
            } else if latest_dispatch_plan(snapshot, DispatchPlanStatus::Draft).is_some() {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
                )
            } else if has_answer_after_latest_question(snapshot) {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
                )
            } else {
                completion(
                    "ready",
                    None,
                    "dispatch",
                    Some(format!("nagare item preview {}", snapshot.item.id)),
                )
            }
        }
    }
}

fn completion(
    state: &str,
    blocking_reason: Option<String>,
    next_action: &str,
    next_command_hint: Option<String>,
) -> WorkItemCompletion {
    WorkItemCompletion {
        state: state.to_string(),
        blocking_reason,
        next_action: next_action.to_string(),
        next_command_hint,
    }
}

fn empty_approval_gate() -> WorkItemApprovalGate {
    WorkItemApprovalGate {
        state: "not_ready".to_string(),
        ready: false,
        latest_review_id: None,
        criteria_passed: 0,
        criteria_total: 0,
        artifact_count: 0,
        recovery_count: 0,
        blockers: Vec::new(),
        command_hint: None,
    }
}

fn approval_gate_state(snapshot: &WorkItemSnapshot) -> WorkItemApprovalGate {
    let latest_work_sequence = latest_work_run_sequence(snapshot);
    let latest_review = latest_review_pass_after(snapshot, latest_work_sequence);
    let criteria_total = snapshot.item.acceptance_criteria.len();
    let criteria_passed = latest_review.map(criteria_passed_count).unwrap_or(0);
    let done = snapshot.item.status == WorkItemStatus::Done;
    let mut blockers = Vec::new();

    if !done && snapshot.item.status != WorkItemStatus::ReadyForReview {
        blockers.push(format!("status:{}", snapshot.item.status));
    }
    if !done && latest_review.is_none() {
        blockers.push("review_not_passed".to_string());
    }
    if !done && work_item_needs_synthesis(snapshot) {
        blockers.push("synthesis_required".to_string());
    }
    if !done && criteria_total > 0 {
        if criteria_passed < criteria_total {
            blockers.push("criteria_not_satisfied".to_string());
        }
    }

    let ready = snapshot.item.status == WorkItemStatus::ReadyForReview && blockers.is_empty();
    let state = if done {
        "done"
    } else if ready {
        "ready"
    } else if snapshot.item.status == WorkItemStatus::ReadyForReview {
        "blocked"
    } else {
        "not_ready"
    };

    WorkItemApprovalGate {
        state: state.to_string(),
        ready,
        latest_review_id: latest_review.map(|review| review.id.clone()),
        criteria_passed,
        criteria_total,
        artifact_count: snapshot.artifacts.len(),
        recovery_count: snapshot
            .recovery_plans
            .iter()
            .filter(|plan| plan.status != RecoveryPlanStatus::Superseded)
            .count(),
        blockers,
        command_hint: if ready {
            Some(format!("nagare decision approve {}", snapshot.item.id))
        } else {
            None
        },
    }
}

fn latest_question(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| output.questions.first().cloned())
}

fn latest_agent_next_action(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find_map(|output| output.next_action.clone())
}

fn latest_requested_change(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .review_results
        .iter()
        .rev()
        .find(|review| review.verdict == ReviewVerdict::RequestChanges)
        .and_then(|review| review.requested_changes.first().cloned())
}

fn latest_unparsed_output(snapshot: &WorkItemSnapshot) -> Option<&AgentOutputRecord> {
    snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
}

fn latest_review_pass_after_latest_work(snapshot: &WorkItemSnapshot) -> Option<&ReviewResult> {
    latest_review_pass_after(snapshot, latest_work_run_sequence(snapshot))
}

pub(crate) fn work_item_needs_synthesis(snapshot: &WorkItemSnapshot) -> bool {
    let Some(latest_review) = latest_review_pass_after_latest_work(snapshot) else {
        return false;
    };
    let distinct_workers = snapshot
        .runs
        .iter()
        .filter(|run| {
            run.purpose == AgentRunPurpose::Work && run.status == AgentRunStatus::Succeeded
        })
        .map(|run| run.agent_profile_id.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if distinct_workers < 2 {
        return false;
    }
    let latest_review_sequence = id_sequence(&latest_review.id);
    !snapshot.runs.iter().any(|run| {
        run.purpose == AgentRunPurpose::Synthesis
            && run.status == AgentRunStatus::Succeeded
            && id_sequence(&run.id) > latest_review_sequence
    })
}

fn latest_review_pass_after(snapshot: &WorkItemSnapshot, sequence: u64) -> Option<&ReviewResult> {
    snapshot
        .review_results
        .iter()
        .rev()
        .find(|review| review.verdict == ReviewVerdict::Pass && id_sequence(&review.id) > sequence)
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

fn latest_work_run_sequence(snapshot: &WorkItemSnapshot) -> u64 {
    snapshot
        .runs
        .iter()
        .rev()
        .find(|run| run.purpose == AgentRunPurpose::Work)
        .map(|run| id_sequence(&run.id))
        .unwrap_or(0)
}

fn id_sequence(id: &str) -> u64 {
    id.rsplit('_')
        .next()
        .and_then(|suffix| suffix.parse::<u64>().ok())
        .unwrap_or(0)
}

fn criteria_passed_count(review: &ReviewResult) -> usize {
    review
        .criteria_results
        .iter()
        .filter(|result| result.status == CriteriaReviewStatus::Passed)
        .count()
}

fn history_steps(snapshot: &WorkItemSnapshot) -> Vec<WorkItemHistoryStep> {
    let mut steps = Vec::new();
    steps.push(WorkItemHistoryStep {
        id: format!("step_{}", snapshot.item.id),
        kind: "request".to_string(),
        title: "依頼を作成".to_string(),
        state: if snapshot.item.status == WorkItemStatus::Done {
            "done".to_string()
        } else {
            "recorded".to_string()
        },
        actor: Some("User".to_string()),
        started_at: Some(snapshot.item.created_at.clone()),
        ended_at: Some(snapshot.item.created_at.clone()),
        summary: snapshot.item.title.clone(),
        facts: history_facts([
            ("Title", snapshot.item.title.clone()),
            (
                "Acceptance",
                count_or_dash(snapshot.item.acceptance_criteria.len()),
            ),
            (
                "Expected artifacts",
                count_or_dash(snapshot.item.expected_artifacts.len()),
            ),
        ]),
        links: Vec::new(),
        source_record_ids: vec![snapshot.item.id.clone()],
        next_action: Some(snapshot.completion.next_action.clone()),
    });

    for plan in &snapshot.dispatch_plans {
        let run = snapshot.runs.iter().find(|run| run.id == plan.agent_run_id);
        let decision = latest_workflow_decision(
            snapshot,
            &[
                WorkflowDecisionAction::Dispatch,
                WorkflowDecisionAction::AcceptDispatch,
            ],
            Some(&plan.agent_run_id),
        );
        let mut facts = history_facts([
            ("Target", plan.target_agent_profile_id.clone()),
            ("Dispatch agent", plan.dispatch_agent_profile_id.clone()),
            ("Warnings", count_or_dash(plan.selection_warnings.len())),
            (
                "Missing info",
                count_or_dash(plan.missing_information.len()),
            ),
        ]);
        let mut links = vec![
            history_link("DispatchPlan", &plan.id, "dispatch_plan"),
            history_link(
                "Raw output",
                dispatch_plan_execution_record_id(plan),
                "execution_record",
            ),
        ];
        append_decision_context(&mut facts, &mut links, decision);
        let mut source_record_ids = vec![plan.id.clone(), plan.agent_run_id.clone()];
        if let Some(decision) = decision {
            source_record_ids.push(decision.id.clone());
        }
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", plan.id),
            kind: "dispatch".to_string(),
            title: "Agent 選定".to_string(),
            state: plan.status.to_string(),
            actor: Some(plan.dispatch_agent_profile_id.clone()),
            started_at: run.map(|run| run.started_at.clone()),
            ended_at: Some(plan.created_at.clone()),
            summary: format!(
                "{} を target agent に選定。{}",
                plan.target_agent_profile_id, plan.summary
            ),
            facts,
            links,
            source_record_ids,
            next_action: Some("run_agent".to_string()),
        });
    }

    for run in snapshot
        .runs
        .iter()
        .filter(|run| run.purpose == AgentRunPurpose::Work)
    {
        let output = snapshot
            .agent_outputs
            .iter()
            .find(|output| output.agent_run_id == run.id);
        let state = if output
            .is_some_and(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
        {
            "contract_invalid".to_string()
        } else if output.is_some_and(|output| !output.questions.is_empty()) {
            "needs_input".to_string()
        } else {
            run.status.to_string()
        };
        let summary = output
            .and_then(|output| {
                first_output_field(output, "summary")
                    .or_else(|| first_output_field(output, "completed"))
            })
            .unwrap_or_else(|| {
                format!(
                    "Process {} with exit {}.",
                    run.status,
                    format_exit_code(run.exit_code)
                )
            });
        let artifact_count = snapshot
            .artifacts
            .iter()
            .filter(|artifact| artifact.agent_run_id.as_deref() == Some(run.id.as_str()))
            .count();
        let artifact_titles = snapshot
            .artifacts
            .iter()
            .filter(|artifact| artifact.agent_run_id.as_deref() == Some(run.id.as_str()))
            .map(|artifact| artifact.title.clone())
            .collect::<Vec<_>>();
        let evidence_count = snapshot
            .evidence
            .iter()
            .filter(|evidence| {
                evidence.produced_by == run.id || evidence.produced_by == run.agent_profile_id
            })
            .count();
        let evidence_claims = snapshot
            .evidence
            .iter()
            .filter(|evidence| {
                evidence.produced_by == run.id || evidence.produced_by == run.agent_profile_id
            })
            .map(|evidence| evidence.claim.clone())
            .collect::<Vec<_>>();
        let decision = workflow_decision_for_run(snapshot, run).or_else(|| {
            latest_workflow_decision(snapshot, &[WorkflowDecisionAction::RunAgent], None)
        });
        let mut facts = history_facts([
            ("Agent", run.agent_profile_id.clone()),
            ("Process", run.status.to_string()),
            ("Exit", format_exit_code(run.exit_code)),
            ("Artifacts", count_or_dash(artifact_count)),
            ("Evidence", count_or_dash(evidence_count)),
        ]);
        let mut links = vec![history_link("Run", &run.id, "run")];
        links.push(history_link(
            "Run log",
            agent_run_execution_record_id(run),
            "execution_record",
        ));
        if !artifact_titles.is_empty() {
            facts.push(history_fact("artifact detail", artifact_titles.join(", ")));
        }
        if !evidence_claims.is_empty() {
            facts.push(history_fact("evidence detail", evidence_claims.join(", ")));
        }
        if let Some(output) = output {
            links.push(history_link("Agent Output", &output.id, "agent_output"));
            if let Some(completed) = first_output_field(output, "completed") {
                facts.push(history_fact("completed", completed));
            }
            if let Some(next_notes) = first_output_field(output, "next_notes") {
                facts.push(history_fact("next_notes", next_notes));
            }
            facts.push(history_fact(
                "questions",
                count_or_dash(output.questions.len()),
            ));
            let record_id = agent_output_execution_record_id(output);
            facts.push(history_fact("output record", record_id.to_string()));
            links.push(history_link("output record", record_id, "execution_record"));
        }
        append_decision_context(&mut facts, &mut links, decision);
        let mut source_record_ids = output
            .map(|output| vec![run.id.clone(), output.id.clone()])
            .unwrap_or_else(|| vec![run.id.clone()]);
        if let Some(decision) = decision {
            source_record_ids.push(decision.id.clone());
        }
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", run.id),
            kind: "work".to_string(),
            title: "作業実行".to_string(),
            state,
            actor: Some(run.agent_profile_id.clone()),
            started_at: Some(run.started_at.clone()),
            ended_at: Some(run.ended_at.clone()),
            summary,
            facts,
            links,
            source_record_ids,
            next_action: output.and_then(output_history_next_action),
        });
    }

    for run in snapshot
        .runs
        .iter()
        .filter(|run| run.purpose == AgentRunPurpose::Review)
        .filter(|run| {
            !snapshot
                .review_results
                .iter()
                .any(|review| review.agent_run_id == run.id)
        })
    {
        let output = snapshot
            .agent_outputs
            .iter()
            .find(|output| output.agent_run_id == run.id);
        let state = if output
            .is_some_and(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
        {
            "contract_invalid".to_string()
        } else {
            run.status.to_string()
        };
        let summary = output
            .and_then(|output| {
                first_output_field(output, "summary")
                    .or_else(|| first_output_field(output, "completed"))
            })
            .unwrap_or_else(|| format!("{} {}", run.agent_profile_id, run.purpose));
        let decision = workflow_decision_for_run(snapshot, run).or_else(|| {
            latest_workflow_decision(snapshot, &[WorkflowDecisionAction::RunReview], None)
        });
        let mut facts = history_facts([
            ("Agent", run.agent_profile_id.clone()),
            ("Status", state.replace('_', " ")),
            ("Process status", run.status.to_string()),
            ("Process exit", format_exit_code(run.exit_code)),
            (
                "Parse status",
                output
                    .map(|output| output.parse_status.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]);
        let mut links = vec![
            history_link("Run", &run.id, "run"),
            history_link(
                "Run log",
                agent_run_execution_record_id(run),
                "execution_record",
            ),
        ];
        if let Some(output) = output {
            links.push(history_link("Agent Output", &output.id, "agent_output"));
            if let Some(completed) = first_output_field(output, "completed") {
                facts.push(history_fact("completed", completed));
            }
            if let Some(next_notes) = first_output_field(output, "next_notes") {
                facts.push(history_fact("next_notes", next_notes));
            }
            facts.push(history_fact(
                "questions",
                count_or_dash(output.questions.len()),
            ));
            let record_id = agent_output_execution_record_id(output);
            facts.push(history_fact("output record", record_id.to_string()));
            links.push(history_link("output record", record_id, "execution_record"));
        }
        append_decision_context(&mut facts, &mut links, decision);
        let mut source_record_ids = output
            .map(|output| vec![run.id.clone(), output.id.clone()])
            .unwrap_or_else(|| vec![run.id.clone()]);
        if let Some(decision) = decision {
            source_record_ids.push(decision.id.clone());
        }
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", run.id),
            kind: "review".to_string(),
            title: "Review".to_string(),
            state,
            actor: Some(run.agent_profile_id.clone()),
            started_at: Some(run.started_at.clone()),
            ended_at: Some(run.ended_at.clone()),
            summary,
            facts,
            links,
            source_record_ids,
            next_action: output.and_then(output_history_next_action),
        });
    }

    for review in &snapshot.review_results {
        let run = snapshot
            .runs
            .iter()
            .find(|run| run.id == review.agent_run_id);
        let output = snapshot
            .agent_outputs
            .iter()
            .find(|output| output.agent_run_id == review.agent_run_id);
        let state = if output
            .is_some_and(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
        {
            "contract_invalid".to_string()
        } else {
            review.verdict.to_string()
        };
        let decision = latest_workflow_decision(
            snapshot,
            &[WorkflowDecisionAction::RunReview],
            Some(&review.agent_run_id),
        );
        let mut facts = history_facts([
            ("Status", state.replace('_', " ")),
            ("Verdict", review.verdict.to_string()),
            (
                "Process status",
                run.map(|run| run.status.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                "Process exit",
                run.map(|run| format_exit_code(run.exit_code))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                "Parse status",
                output
                    .map(|output| output.parse_status.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                "Criteria",
                format!(
                    "{}/{}",
                    criteria_passed_count(review),
                    review.criteria_results.len()
                ),
            ),
            ("Findings", count_or_dash(review.findings.len())),
            (
                "Requested changes",
                count_or_dash(review.requested_changes.len()),
            ),
        ]);
        let mut links = vec![history_link("ReviewResult", &review.id, "review")];
        if let Some(run) = run {
            links.push(history_link("Run", &run.id, "run"));
            links.push(history_link(
                "Run log",
                agent_run_execution_record_id(run),
                "execution_record",
            ));
        }
        if let Some(output) = output {
            links.push(history_link("Agent Output", &output.id, "agent_output"));
            if let Some(completed) = first_output_field(output, "completed") {
                facts.push(history_fact("completed", completed));
            }
            if let Some(next_notes) = first_output_field(output, "next_notes") {
                facts.push(history_fact("next_notes", next_notes));
            }
            facts.push(history_fact(
                "questions",
                count_or_dash(output.questions.len()),
            ));
            let record_id = agent_output_execution_record_id(output);
            facts.push(history_fact("output record", record_id.to_string()));
            links.push(history_link("output record", record_id, "execution_record"));
        }
        append_decision_context(&mut facts, &mut links, decision);
        let mut source_record_ids = vec![review.id.clone(), review.agent_run_id.clone()];
        if let Some(output) = output {
            source_record_ids.push(output.id.clone());
        }
        if let Some(decision) = decision {
            source_record_ids.push(decision.id.clone());
        }
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", review.id),
            kind: "review".to_string(),
            title: "Review".to_string(),
            state,
            actor: Some(review.agent_profile_id.clone()),
            started_at: run.map(|run| run.started_at.clone()),
            ended_at: Some(review.created_at.clone()),
            summary: review
                .summary
                .first()
                .cloned()
                .unwrap_or_else(|| review.verdict.to_string()),
            facts,
            links,
            source_record_ids,
            next_action: review_history_next_action(review),
        });
    }

    for run in snapshot
        .runs
        .iter()
        .filter(|run| run.purpose == AgentRunPurpose::Synthesis)
    {
        let output = snapshot
            .agent_outputs
            .iter()
            .find(|output| output.agent_run_id == run.id);
        let state = if output
            .is_some_and(|output| output.parse_status == AgentOutputParseStatus::Unparsed)
        {
            "contract_invalid".to_string()
        } else {
            run.status.to_string()
        };
        let summary = output
            .and_then(|output| {
                first_output_field(output, "summary")
                    .or_else(|| first_output_field(output, "completed"))
            })
            .unwrap_or_else(|| "複数Workerの結果を統合しました。".to_string());
        let decision = latest_workflow_decision(
            snapshot,
            &[WorkflowDecisionAction::RunSynthesis],
            Some(&run.id),
        );
        let mut facts = history_facts([
            ("Agent", run.agent_profile_id.clone()),
            ("Status", state.replace('_', " ")),
            ("Process status", run.status.to_string()),
            ("Process exit", format_exit_code(run.exit_code)),
            (
                "Parse status",
                output
                    .map(|output| output.parse_status.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ]);
        let mut links = vec![
            history_link("Run", &run.id, "run"),
            history_link(
                "Run log",
                agent_run_execution_record_id(run),
                "execution_record",
            ),
        ];
        if let Some(output) = output {
            links.push(history_link("Agent Output", &output.id, "agent_output"));
            if let Some(completed) = first_output_field(output, "completed") {
                facts.push(history_fact("completed", completed));
            }
            if let Some(next_notes) = first_output_field(output, "next_notes") {
                facts.push(history_fact("next_notes", next_notes));
            }
            let record_id = agent_output_execution_record_id(output);
            facts.push(history_fact("output record", record_id.to_string()));
            links.push(history_link("output record", record_id, "execution_record"));
        }
        append_decision_context(&mut facts, &mut links, decision);
        let mut source_record_ids = output
            .map(|output| vec![run.id.clone(), output.id.clone()])
            .unwrap_or_else(|| vec![run.id.clone()]);
        if let Some(decision) = decision {
            source_record_ids.push(decision.id.clone());
        }
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", run.id),
            kind: "synthesis".to_string(),
            title: "統合サマリー".to_string(),
            state,
            actor: Some(run.agent_profile_id.clone()),
            started_at: Some(run.started_at.clone()),
            ended_at: Some(run.ended_at.clone()),
            summary,
            facts,
            links,
            source_record_ids,
            next_action: output.and_then(output_history_next_action),
        });
    }

    for output in snapshot
        .agent_outputs
        .iter()
        .filter(|output| !output.questions.is_empty())
    {
        steps.push(WorkItemHistoryStep {
            id: format!("step_question_{}", output.id),
            kind: "input".to_string(),
            title: "Agent からの質問".to_string(),
            state: "needs_input".to_string(),
            actor: Some(output.agent_profile_id.clone()),
            started_at: Some(output.created_at.clone()),
            ended_at: None,
            summary: output.questions.first().cloned().unwrap_or_default(),
            facts: history_facts([
                ("Agent", output.agent_profile_id.clone()),
                ("Purpose", output.purpose.to_string()),
                ("Questions", count_or_dash(output.questions.len())),
            ]),
            links: vec![history_link("AgentOutput", &output.id, "agent_output")],
            source_record_ids: vec![output.id.clone()],
            next_action: Some("answer_question".to_string()),
        });
    }

    for feedback in &snapshot.human_feedback {
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", feedback.id),
            kind: "input".to_string(),
            title: "ユーザー回答".to_string(),
            state: "answered".to_string(),
            actor: Some("User".to_string()),
            started_at: Some(feedback.created_at.clone()),
            ended_at: Some(feedback.created_at.clone()),
            summary: feedback.answer.clone(),
            facts: history_facts([
                ("Question", feedback.question.clone()),
                ("Answer", feedback.answer.clone()),
            ]),
            links: vec![history_link(
                "HumanFeedback",
                &feedback.id,
                "human_feedback",
            )],
            source_record_ids: vec![feedback.id.clone()],
            next_action: Some("run_agent".to_string()),
        });
    }

    for handoff in &snapshot.handoffs {
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", handoff.id),
            kind: "handoff".to_string(),
            title: "Handoff".to_string(),
            state: "created".to_string(),
            actor: Some(handoff.from_agent_profile.clone()),
            started_at: Some(handoff.created_at.clone()),
            ended_at: Some(handoff.created_at.clone()),
            summary: handoff.summary.clone(),
            facts: history_facts([
                ("From", handoff.from_agent_profile.clone()),
                ("To", handoff.to_agent_profile.clone()),
                (
                    "Open questions",
                    count_or_dash(handoff.open_questions.len()),
                ),
                ("Artifacts", count_or_dash(handoff.artifact_ids.len())),
            ]),
            links: vec![history_link("Handoff", &handoff.id, "handoff")],
            source_record_ids: vec![handoff.id.clone()],
            next_action: Some("dispatch".to_string()),
        });
    }

    for plan in &snapshot.recovery_plans {
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", plan.id),
            kind: "recovery".to_string(),
            title: "Recovery".to_string(),
            state: plan.status.to_string(),
            actor: plan.target_agent_profile_id.clone(),
            started_at: Some(plan.created_at.clone()),
            ended_at: Some(plan.created_at.clone()),
            summary: plan.summary.clone(),
            facts: history_facts([
                ("Action", plan.action.to_string()),
                ("Failure", plan.failure_class.clone()),
                ("Reason", plan.reason.clone()),
                (
                    "Target",
                    plan.target_agent_profile_id
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]),
            links: vec![history_link("RecoveryPlan", &plan.id, "recovery")],
            source_record_ids: vec![plan.id.clone()],
            next_action: plan.command_hint.clone(),
        });
    }

    for decision in &snapshot.decisions {
        let is_reject = decision.decision_type == "reject";
        steps.push(WorkItemHistoryStep {
            id: format!("step_{}", decision.id),
            kind: "approval".to_string(),
            title: if is_reject {
                "承認せず差し戻し".to_string()
            } else {
                "承認して完了".to_string()
            },
            state: decision.decision_type.clone(),
            actor: Some("User".to_string()),
            started_at: Some(decision.created_at.clone()),
            ended_at: Some(decision.created_at.clone()),
            summary: decision.rationale.clone(),
            facts: history_facts([
                ("Decision", decision.decision_type.clone()),
                ("Rationale", decision.rationale.clone()),
            ]),
            links: vec![history_link("HumanDecision", &decision.id, "decision")],
            source_record_ids: vec![decision.id.clone()],
            next_action: Some(if is_reject {
                "dispatch".to_string()
            } else {
                "done".to_string()
            }),
        });
    }

    steps.sort_by(|left, right| {
        history_step_time(left)
            .cmp(history_step_time(right))
            .then(id_sequence(&left.id).cmp(&id_sequence(&right.id)))
            .then(left.id.cmp(&right.id))
    });
    steps
}

fn history_step_time(step: &WorkItemHistoryStep) -> &str {
    step.ended_at
        .as_deref()
        .or(step.started_at.as_deref())
        .unwrap_or("")
}

fn history_facts<const N: usize>(pairs: [(&str, String); N]) -> Vec<WorkItemHistoryFact> {
    pairs
        .into_iter()
        .filter(|(_, value)| !value.trim().is_empty())
        .map(|(label, value)| WorkItemHistoryFact {
            label: label.to_string(),
            value,
        })
        .collect()
}

fn history_fact(label: &str, value: String) -> WorkItemHistoryFact {
    WorkItemHistoryFact {
        label: label.to_string(),
        value,
    }
}

fn history_link(label: &str, record_id: &str, record_type: &str) -> WorkItemHistoryLink {
    WorkItemHistoryLink {
        label: label.to_string(),
        record_id: record_id.to_string(),
        record_type: record_type.to_string(),
    }
}

fn latest_workflow_decision<'a>(
    snapshot: &'a WorkItemSnapshot,
    actions: &[WorkflowDecisionAction],
    run_id: Option<&str>,
) -> Option<&'a WorkflowDecision> {
    snapshot.workflow_decisions.iter().rev().find(|decision| {
        actions.contains(&decision.action)
            && run_id.is_none_or(|run_id| {
                decision.agent_run_id.as_deref() == Some(run_id) || decision.agent_run_id.is_none()
            })
    })
}

fn workflow_decision_for_run<'a>(
    snapshot: &'a WorkItemSnapshot,
    run: &AgentRun,
) -> Option<&'a WorkflowDecision> {
    snapshot.workflow_decisions.iter().rev().find(|decision| {
        decision.agent_run_id.as_deref() == Some(run.id.as_str())
            || (decision.action == WorkflowDecisionAction::RunAgent
                && decision.target_agent_profile_id.as_deref()
                    == Some(run.agent_profile_id.as_str()))
            || (decision.action == WorkflowDecisionAction::RunSynthesis
                && run.purpose == AgentRunPurpose::Synthesis)
    })
}

fn append_decision_context(
    facts: &mut Vec<WorkItemHistoryFact>,
    links: &mut Vec<WorkItemHistoryLink>,
    decision: Option<&WorkflowDecision>,
) {
    let Some(decision) = decision else {
        return;
    };
    facts.push(history_fact(
        "Workflow Decision",
        decision.action.to_string(),
    ));
    facts.push(history_fact("Reason", decision.reason.clone()));
    facts.push(history_fact("Decision source", decision.source.to_string()));
    facts.push(history_fact(
        "Confidence",
        format!("{:.2}", decision.confidence),
    ));
    facts.push(history_fact(
        "Decision warnings",
        count_or_dash(decision.warnings.len()),
    ));
    links.push(history_link(
        "WorkflowDecision",
        &decision.id,
        "workflow_decision",
    ));
}

fn count_or_dash(count: usize) -> String {
    if count == 0 {
        "-".to_string()
    } else {
        count.to_string()
    }
}

fn format_exit_code(exit_code: Option<i32>) -> String {
    exit_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn first_output_field(output: &AgentOutputRecord, key: &str) -> Option<String> {
    output
        .fields
        .get(key)
        .and_then(|values| values.iter().find(|value| !value.trim().is_empty()))
        .cloned()
}

fn agent_run_execution_record_id(run: &AgentRun) -> &str {
    &run.execution_record_id
}

fn dispatch_plan_execution_record_id(plan: &DispatchPlan) -> &str {
    &plan.raw_output_execution_record_id
}

fn agent_output_execution_record_id(output: &AgentOutputRecord) -> &str {
    &output.execution_record_id
}

fn output_history_next_action(output: &AgentOutputRecord) -> Option<String> {
    match output.next_action.as_deref() {
        Some("answer_question" | "needs_input") if output.questions.is_empty() => None,
        _ => output.next_action.clone(),
    }
}

fn review_history_next_action(review: &ReviewResult) -> Option<String> {
    match review.next_action.as_deref() {
        Some("answer_question" | "needs_input") if review.questions.is_empty() => None,
        _ => review.next_action.clone(),
    }
}

fn timeline_events(snapshot: &WorkItemSnapshot) -> Vec<WorkItemTimelineEvent> {
    let mut events = Vec::new();
    events.push(WorkItemTimelineEvent {
        id: snapshot.item.id.clone(),
        event_type: "request".to_string(),
        title: snapshot.item.title.clone(),
        status: snapshot.item.status.to_string(),
        actor: None,
        related_record_id: None,
        created_at: snapshot.item.created_at.clone(),
    });
    for plan in &snapshot.dispatch_plans {
        events.push(WorkItemTimelineEvent {
            id: plan.id.clone(),
            event_type: "dispatch".to_string(),
            title: plan.summary.clone(),
            status: plan.status.to_string(),
            actor: Some(plan.dispatch_agent_profile_id.clone()),
            related_record_id: Some(dispatch_plan_execution_record_id(plan).to_string()),
            created_at: plan.created_at.clone(),
        });
    }
    for run in &snapshot.runs {
        events.push(WorkItemTimelineEvent {
            id: run.id.clone(),
            event_type: "run".to_string(),
            title: format!("{} {}", run.agent_profile_id, run.purpose),
            status: run.status.to_string(),
            actor: Some(run.agent_profile_id.clone()),
            related_record_id: Some(agent_run_execution_record_id(run).to_string()),
            created_at: run.ended_at.clone(),
        });
    }
    for record in &snapshot.execution_records {
        events.push(WorkItemTimelineEvent {
            id: record.id.clone(),
            event_type: "execution_record".to_string(),
            title: record.title.clone(),
            status: record.record_type.clone(),
            actor: record.agent_run_id.clone(),
            related_record_id: Some(record.id.clone()),
            created_at: record.created_at.clone(),
        });
    }
    for artifact in &snapshot.artifacts {
        events.push(WorkItemTimelineEvent {
            id: artifact.id.clone(),
            event_type: "artifact".to_string(),
            title: artifact.title.clone(),
            status: artifact.artifact_type.clone(),
            actor: artifact.agent_run_id.clone(),
            related_record_id: Some(artifact.id.clone()),
            created_at: artifact.created_at.clone(),
        });
    }
    for evidence in &snapshot.evidence {
        events.push(WorkItemTimelineEvent {
            id: evidence.id.clone(),
            event_type: "evidence".to_string(),
            title: evidence.claim.clone(),
            status: "recorded".to_string(),
            actor: Some(evidence.produced_by.clone()),
            related_record_id: evidence
                .artifact_id
                .clone()
                .or_else(|| evidence.execution_record_id.clone()),
            created_at: evidence.created_at.clone(),
        });
    }
    for output in &snapshot.agent_outputs {
        events.push(WorkItemTimelineEvent {
            id: output.id.clone(),
            event_type: if output.questions.is_empty() {
                "agent_output".to_string()
            } else {
                "question".to_string()
            },
            title: output
                .questions
                .first()
                .cloned()
                .or_else(|| output.next_action.clone())
                .unwrap_or_else(|| output.contract.clone()),
            status: output.parse_status.to_string(),
            actor: Some(output.agent_profile_id.clone()),
            related_record_id: Some(agent_output_execution_record_id(output).to_string()),
            created_at: output.created_at.clone(),
        });
    }
    for feedback in &snapshot.human_feedback {
        events.push(WorkItemTimelineEvent {
            id: feedback.id.clone(),
            event_type: "human_feedback".to_string(),
            title: feedback.answer.clone(),
            status: "answered".to_string(),
            actor: None,
            related_record_id: None,
            created_at: feedback.created_at.clone(),
        });
    }
    for review in &snapshot.review_results {
        events.push(WorkItemTimelineEvent {
            id: review.id.clone(),
            event_type: "review".to_string(),
            title: review.summary.first().cloned().unwrap_or_else(|| {
                review
                    .findings
                    .first()
                    .cloned()
                    .unwrap_or_else(|| review.verdict.to_string())
            }),
            status: review.verdict.to_string(),
            actor: Some(review.agent_profile_id.clone()),
            related_record_id: Some(review.execution_record_id.clone()),
            created_at: review.created_at.clone(),
        });
    }
    for handoff in &snapshot.handoffs {
        events.push(WorkItemTimelineEvent {
            id: handoff.id.clone(),
            event_type: "handoff".to_string(),
            title: handoff.reason.clone(),
            status: format!(
                "{} -> {}",
                handoff.from_agent_profile, handoff.to_agent_profile
            ),
            actor: Some(handoff.from_agent_profile.clone()),
            related_record_id: None,
            created_at: handoff.created_at.clone(),
        });
    }
    for decision in &snapshot.decisions {
        events.push(WorkItemTimelineEvent {
            id: decision.id.clone(),
            event_type: "decision".to_string(),
            title: decision.rationale.clone(),
            status: decision.decision_type.clone(),
            actor: None,
            related_record_id: None,
            created_at: decision.created_at.clone(),
        });
    }
    for plan in &snapshot.recovery_plans {
        events.push(WorkItemTimelineEvent {
            id: plan.id.clone(),
            event_type: "recovery".to_string(),
            title: plan.summary.clone(),
            status: plan.status.to_string(),
            actor: plan.target_agent_profile_id.clone(),
            related_record_id: None,
            created_at: plan.created_at.clone(),
        });
    }
    for decision in &snapshot.workflow_decisions {
        events.push(WorkItemTimelineEvent {
            id: decision.id.clone(),
            event_type: "workflow_decision".to_string(),
            title: decision.reason.clone(),
            status: decision.action.to_string(),
            actor: decision.target_agent_profile_id.clone(),
            related_record_id: None,
            created_at: decision.created_at.clone(),
        });
    }
    events.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then(id_sequence(&left.id).cmp(&id_sequence(&right.id)))
            .then(left.id.cmp(&right.id))
    });
    events
}
