use crate::*;

#[derive(Debug, Clone)]
pub struct CreateItemResult {
    pub item: WorkItem,
}

#[derive(Debug, Clone, Default)]
pub struct CreateWorkItemInput {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub expected_artifacts: Vec<String>,
    pub work_folder: Option<String>,
    pub constraints: Vec<String>,
    pub domain_id: Option<String>,
    pub artifact_type_id: Option<String>,
    pub domain_agent_policy: DomainAgentPolicy,
    pub workflow_mode: Option<WorkflowMode>,
    pub approval_policy: Option<ApprovalPolicy>,
}

#[derive(Debug, Clone)]
pub struct RunWorkItemResult {
    pub run: AgentRun,
    pub evidence_id: String,
    pub item_status: WorkItemStatus,
    pub dispatch_plan_id: Option<String>,
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
    pub review_id: String,
    pub decision_id: String,
    pub final_status: WorkItemStatus,
}
