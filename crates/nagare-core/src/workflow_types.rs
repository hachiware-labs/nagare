use crate::*;
use serde::{Deserialize, Serialize};

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
    RunSynthesis,
    CreateRecoveryPlan,
    AcceptRecoveryPlan,
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
            Self::RunSynthesis => "run_synthesis",
            Self::CreateRecoveryPlan => "create_recovery_plan",
            Self::AcceptRecoveryPlan => "accept_recovery_plan",
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
    pub synthesis_dev_command: Option<&'a str>,
    pub use_supervisor: bool,
    pub supervisor_dev_command: Option<&'a str>,
    pub auto_recover: bool,
    pub workflow_mode: Option<WorkflowMode>,
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
