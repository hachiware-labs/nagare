use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub id: String,
    pub work_item_id: String,
    pub agent_run_id: String,
    pub agent_profile_id: String,
    pub verdict: ReviewVerdict,
    #[serde(default)]
    pub summary: Vec<String>,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub requested_changes: Vec<String>,
    #[serde(default)]
    pub referenced_artifacts: Vec<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    pub next_action: Option<String>,
    pub artifact_id: Option<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Pass,
    RequestChanges,
    Blocked,
    Unknown,
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Pass => "pass",
            Self::RequestChanges => "request_changes",
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
        };
        f.write_str(value)
    }
}

pub(crate) fn review_result_from_agent_output(
    id: String,
    output: &AgentOutputRecord,
) -> ReviewResult {
    ReviewResult {
        id,
        work_item_id: output.work_item_id.clone(),
        agent_run_id: output.agent_run_id.clone(),
        agent_profile_id: output.agent_profile_id.clone(),
        verdict: output
            .fields
            .get("verdict")
            .and_then(|values| values.first())
            .map(|value| parse_review_verdict(value))
            .unwrap_or(ReviewVerdict::Unknown),
        summary: output.fields.get("summary").cloned().unwrap_or_default(),
        findings: output.fields.get("findings").cloned().unwrap_or_default(),
        requested_changes: output
            .fields
            .get("requested_changes")
            .cloned()
            .unwrap_or_default(),
        referenced_artifacts: output
            .fields
            .get("referenced_artifacts")
            .cloned()
            .unwrap_or_default(),
        questions: output.questions.clone(),
        next_action: output.next_action.clone(),
        artifact_id: output.artifact_id.clone(),
        locale: output.locale.clone(),
        created_at: output.created_at.clone(),
    }
}

pub(crate) fn review_work_item_status(
    review: &ReviewResult,
    current: WorkItemStatus,
) -> WorkItemStatus {
    if !review.questions.is_empty()
        || review
            .next_action
            .as_deref()
            .is_some_and(|action| action == "answer_question" || action == "needs_input")
    {
        return WorkItemStatus::NeedsInput;
    }
    match review.verdict {
        ReviewVerdict::Pass => WorkItemStatus::ReadyForVerification,
        ReviewVerdict::RequestChanges => WorkItemStatus::ChangesRequested,
        ReviewVerdict::Blocked => WorkItemStatus::NeedsInput,
        ReviewVerdict::Unknown => current,
    }
}

fn parse_review_verdict(value: &str) -> ReviewVerdict {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "pass" | "passed" | "approved" | "ok" => ReviewVerdict::Pass,
        "request_changes" | "changes_requested" | "needs_changes" => ReviewVerdict::RequestChanges,
        "blocked" | "block" | "needs_input" => ReviewVerdict::Blocked,
        _ => ReviewVerdict::Unknown,
    }
}
