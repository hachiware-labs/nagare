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
    pub dispatch_plans: Vec<DispatchPlan>,
    pub resolved_skill_contexts: Vec<ResolvedSkillContext>,
    pub resolved_run_packets: Vec<ResolvedRunPacket>,
}

impl WorkItemSnapshot {
    pub(crate) fn from_ledger(item: WorkItem, ledger: &Ledger) -> Self {
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
            item,
        }
    }
}
