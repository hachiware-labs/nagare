use crate::*;

#[derive(Debug, Clone)]
pub struct WorkItemSnapshot {
    pub item: WorkItem,
    pub completion: WorkItemCompletion,
    pub runs: Vec<AgentRun>,
    pub artifacts: Vec<Artifact>,
    pub evidence: Vec<Evidence>,
    pub verification_results: Vec<VerificationResult>,
    pub review_results: Vec<ReviewResult>,
    pub handoffs: Vec<HandoffPacket>,
    pub decisions: Vec<HumanDecision>,
    pub human_feedback: Vec<HumanFeedback>,
    pub dispatch_plans: Vec<DispatchPlan>,
    pub recovery_plans: Vec<RecoveryPlan>,
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
    pub agent_outputs: Vec<AgentOutputRecord>,
    pub timeline: Vec<WorkItemTimelineEvent>,
}

#[derive(Debug, Clone)]
pub struct WorkItemCompletion {
    pub state: String,
    pub blocking_reason: Option<String>,
    pub next_action: String,
    pub next_command_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkItemTimelineEvent {
    pub id: String,
    pub event_type: String,
    pub title: String,
    pub status: String,
    pub actor: Option<String>,
    pub artifact_id: Option<String>,
    pub created_at: String,
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
            timeline: Vec::new(),
            item,
        };
        snapshot.timeline = timeline_events(&snapshot);
        snapshot.completion = completion_state(&snapshot);
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
            } else {
                completion(
                    "blocked",
                    Some("handoff_created".to_string()),
                    "redispatch",
                    Some(format!("nagare handoff dispatch {}", snapshot.item.id)),
                )
            }
        }
        WorkItemStatus::FailedVerification => completion(
            "blocked",
            latest_failed_verification(snapshot)
                .or_else(|| Some("agent_or_verification_failed".to_string())),
            "recover",
            Some(format!("nagare item recover {}", snapshot.item.id)),
        ),
        WorkItemStatus::ChangesRequested => completion(
            "blocked",
            latest_requested_change(snapshot).or_else(|| Some("changes_requested".to_string())),
            "run_agent",
            Some(format!("nagare item run {}", snapshot.item.id)),
        ),
        WorkItemStatus::ReadyForVerification => completion(
            "ready_for_verification",
            None,
            "verify",
            Some(format!(
                "nagare verify {} --command <command>",
                snapshot.item.id
            )),
        ),
        WorkItemStatus::ReadyForReview => {
            if has_passing_verification(snapshot) {
                completion(
                    "ready_for_approval",
                    None,
                    "approve",
                    Some(format!("nagare decision approve {}", snapshot.item.id)),
                )
            } else if latest_review_requests_verification(snapshot) {
                completion(
                    "ready_for_verification",
                    None,
                    "verify",
                    Some(format!(
                        "nagare verify {} --command <command>",
                        snapshot.item.id
                    )),
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
            if snapshot.runs.is_empty() && snapshot.dispatch_plans.is_empty() {
                completion(
                    "ready",
                    None,
                    "dispatch",
                    Some(format!("nagare item preview {}", snapshot.item.id)),
                )
            } else {
                completion(
                    "ready",
                    None,
                    "run_agent",
                    Some(format!("nagare item run {}", snapshot.item.id)),
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

fn latest_failed_verification(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .verification_results
        .iter()
        .rev()
        .find(|verification| verification.result == VerificationStatus::Failed)
        .map(|verification| format!("verification_failed: {}", verification.command))
}

fn latest_requested_change(snapshot: &WorkItemSnapshot) -> Option<String> {
    snapshot
        .review_results
        .iter()
        .rev()
        .find(|review| review.verdict == ReviewVerdict::RequestChanges)
        .and_then(|review| review.requested_changes.first().cloned())
}

fn has_passing_verification(snapshot: &WorkItemSnapshot) -> bool {
    snapshot
        .verification_results
        .iter()
        .any(|verification| verification.result == VerificationStatus::Passed)
}

fn latest_review_requests_verification(snapshot: &WorkItemSnapshot) -> bool {
    snapshot
        .review_results
        .iter()
        .rev()
        .any(|review| review.verdict == ReviewVerdict::Pass)
}

fn timeline_events(snapshot: &WorkItemSnapshot) -> Vec<WorkItemTimelineEvent> {
    let mut events = Vec::new();
    events.push(WorkItemTimelineEvent {
        id: snapshot.item.id.clone(),
        event_type: "request".to_string(),
        title: snapshot.item.title.clone(),
        status: snapshot.item.status.to_string(),
        actor: None,
        artifact_id: None,
        created_at: snapshot.item.created_at.clone(),
    });
    for plan in &snapshot.dispatch_plans {
        events.push(WorkItemTimelineEvent {
            id: plan.id.clone(),
            event_type: "dispatch".to_string(),
            title: plan.summary.clone(),
            status: plan.status.to_string(),
            actor: Some(plan.dispatch_agent_profile_id.clone()),
            artifact_id: Some(plan.raw_output_artifact_id.clone()),
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
            artifact_id: Some(run.artifact_id.clone()),
            created_at: run.ended_at.clone(),
        });
    }
    for artifact in &snapshot.artifacts {
        events.push(WorkItemTimelineEvent {
            id: artifact.id.clone(),
            event_type: "artifact".to_string(),
            title: artifact.title.clone(),
            status: artifact.artifact_type.clone(),
            actor: artifact.agent_run_id.clone(),
            artifact_id: Some(artifact.id.clone()),
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
            artifact_id: evidence.artifact_id.clone(),
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
            artifact_id: output.artifact_id.clone(),
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
            artifact_id: None,
            created_at: feedback.created_at.clone(),
        });
    }
    for verification in &snapshot.verification_results {
        events.push(WorkItemTimelineEvent {
            id: verification.id.clone(),
            event_type: "verification".to_string(),
            title: verification.command.clone(),
            status: verification.result.to_string(),
            actor: None,
            artifact_id: Some(verification.artifact_id.clone()),
            created_at: verification.verified_at.clone(),
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
            artifact_id: review.artifact_id.clone(),
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
            artifact_id: None,
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
            artifact_id: None,
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
            artifact_id: None,
            created_at: plan.created_at.clone(),
        });
    }
    events.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then(left.id.cmp(&right.id))
    });
    events
}
