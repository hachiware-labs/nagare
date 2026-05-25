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
    pub criteria_results: Vec<CriteriaReviewResult>,
    #[serde(default)]
    pub questions: Vec<String>,
    pub next_action: Option<String>,
    pub artifact_id: Option<String>,
    #[serde(default = "default_locale_language")]
    pub locale: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriteriaReviewResult {
    pub criterion: String,
    pub status: CriteriaReviewStatus,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriteriaReviewStatus {
    Passed,
    Failed,
    Unknown,
}

impl std::fmt::Display for CriteriaReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        };
        f.write_str(value)
    }
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
    acceptance_criteria: &[String],
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
        criteria_results: criteria_results_from_output(output, acceptance_criteria),
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
        ReviewVerdict::Pass if criteria_results_pass(review) => {
            WorkItemStatus::ReadyForVerification
        }
        ReviewVerdict::Pass => WorkItemStatus::ChangesRequested,
        ReviewVerdict::RequestChanges => WorkItemStatus::ChangesRequested,
        ReviewVerdict::Blocked => WorkItemStatus::NeedsInput,
        ReviewVerdict::Unknown => current,
    }
}

pub(crate) fn criteria_results_pass(review: &ReviewResult) -> bool {
    review
        .criteria_results
        .iter()
        .all(|result| result.status == CriteriaReviewStatus::Passed)
}

fn criteria_results_from_output(
    output: &AgentOutputRecord,
    acceptance_criteria: &[String],
) -> Vec<CriteriaReviewResult> {
    if acceptance_criteria.is_empty() {
        return Vec::new();
    }
    let lines = output
        .fields
        .get("criteria")
        .or_else(|| output.fields.get("criteria_results"))
        .cloned()
        .unwrap_or_default();
    acceptance_criteria
        .iter()
        .map(|criterion| {
            let note = lines
                .iter()
                .find(|line| contains_normalized(line, criterion))
                .cloned()
                .unwrap_or_default();
            CriteriaReviewResult {
                criterion: criterion.clone(),
                status: parse_criteria_status(&note),
                note,
            }
        })
        .collect()
}

fn contains_normalized(line: &str, needle: &str) -> bool {
    line.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn parse_criteria_status(value: &str) -> CriteriaReviewStatus {
    let normalized = value.to_ascii_lowercase();
    if normalized.contains("pass") || normalized.contains("ok") || normalized.contains("satisfied")
    {
        CriteriaReviewStatus::Passed
    } else if normalized.contains("fail")
        || normalized.contains("missing")
        || normalized.contains("request")
        || normalized.contains("not satisfied")
    {
        CriteriaReviewStatus::Failed
    } else {
        CriteriaReviewStatus::Unknown
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
