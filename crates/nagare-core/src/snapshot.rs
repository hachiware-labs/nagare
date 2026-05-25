use crate::*;

#[derive(Debug, Clone)]
pub struct WorkItemSnapshot {
    pub item: WorkItem,
    pub runs: Vec<AgentRun>,
    pub artifacts: Vec<Artifact>,
    pub evidence: Vec<Evidence>,
    pub verification_results: Vec<VerificationResult>,
    pub handoffs: Vec<HandoffPacket>,
    pub decisions: Vec<HumanDecision>,
    pub human_feedback: Vec<HumanFeedback>,
    pub dispatch_plans: Vec<DispatchPlan>,
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
    pub agent_outputs: Vec<AgentOutputRecord>,
    pub timeline: Vec<WorkItemTimelineEvent>,
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
            timeline: Vec::new(),
            item,
        };
        snapshot.timeline = timeline_events(&snapshot);
        snapshot
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
    events.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then(left.id.cmp(&right.id))
    });
    events
}
