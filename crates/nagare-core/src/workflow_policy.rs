use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowMode {
    ConfirmFirst,
    FinishFirst,
}

impl WorkflowMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "confirm_first" | "confirm" => Ok(Self::ConfirmFirst),
            "finish_first" | "finish" => Ok(Self::FinishFirst),
            other => Err(format!(
                "unknown workflow mode `{other}`; expected confirm_first or finish_first"
            )),
        }
    }
}

impl Default for WorkflowMode {
    fn default() -> Self {
        Self::ConfirmFirst
    }
}

impl fmt::Display for WorkflowMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::ConfirmFirst => "confirm_first",
            Self::FinishFirst => "finish_first",
        };
        f.write_str(value)
    }
}

pub fn default_workflow_mode() -> WorkflowMode {
    WorkflowMode::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalPolicy {
    ManualFinalApproval,
    AutoCompleteOnReviewPass,
}

impl ApprovalPolicy {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "manual_final_approval" | "manual" | "approve" => Ok(Self::ManualFinalApproval),
            "auto_complete_on_review_pass" | "auto_complete" | "auto" => {
                Ok(Self::AutoCompleteOnReviewPass)
            }
            other => Err(format!(
                "unknown approval policy `{other}`; expected manual_final_approval or auto_complete_on_review_pass"
            )),
        }
    }
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        Self::ManualFinalApproval
    }
}

impl fmt::Display for ApprovalPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::ManualFinalApproval => "manual_final_approval",
            Self::AutoCompleteOnReviewPass => "auto_complete_on_review_pass",
        };
        f.write_str(value)
    }
}

pub fn default_approval_policy() -> ApprovalPolicy {
    ApprovalPolicy::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkflowSettings {
    pub default_progress_mode: WorkflowMode,
    pub approval_policy: ApprovalPolicy,
}

impl Default for WorkflowSettings {
    fn default() -> Self {
        Self {
            default_progress_mode: WorkflowMode::ConfirmFirst,
            approval_policy: ApprovalPolicy::ManualFinalApproval,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DomainWorkflowOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_mode: Option<WorkflowMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<ApprovalPolicy>,
}
