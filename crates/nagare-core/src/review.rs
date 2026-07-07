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
    pub execution_record_id: String,
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
        execution_record_id: output.execution_record_id.clone(),
        locale: output.locale.clone(),
        created_at: output.created_at.clone(),
    }
}

pub(crate) fn review_work_item_status(
    review: &ReviewResult,
    current_status: WorkItemStatus,
) -> WorkItemStatus {
    if !review.questions.is_empty() {
        return WorkItemStatus::NeedsInput;
    }
    match review.verdict {
        ReviewVerdict::Pass if criteria_results_pass(review) => WorkItemStatus::ReadyForReview,
        ReviewVerdict::Pass => WorkItemStatus::ChangesRequested,
        ReviewVerdict::RequestChanges => WorkItemStatus::ChangesRequested,
        ReviewVerdict::Blocked => WorkItemStatus::NeedsInput,
        ReviewVerdict::Unknown if current_status == WorkItemStatus::ReadyForReview => {
            WorkItemStatus::ChangesRequested
        }
        ReviewVerdict::Unknown => current_status,
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
            let note = best_criteria_note(&lines, criterion).unwrap_or_default();
            CriteriaReviewResult {
                criterion: criterion.clone(),
                status: parse_criteria_status(&note),
                note,
            }
        })
        .collect()
}

fn best_criteria_note(lines: &[String], criterion: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| contains_normalized(line, criterion))
        .cloned()
        .or_else(|| {
            lines
                .iter()
                .filter(|line| parse_criteria_status(line) != CriteriaReviewStatus::Unknown)
                .filter_map(|line| {
                    let score = token_overlap_score(line, criterion);
                    (score >= minimum_overlap_score(criterion)).then(|| (score, line.clone()))
                })
                .max_by_key(|(score, _)| *score)
                .map(|(_, line)| line)
        })
}

fn contains_normalized(line: &str, needle: &str) -> bool {
    line.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn token_overlap_score(line: &str, criterion: &str) -> usize {
    let line_tokens = normalized_tokens(line);
    normalized_tokens(criterion)
        .into_iter()
        .filter(|token| line_tokens.contains(token))
        .count()
}

fn minimum_overlap_score(criterion: &str) -> usize {
    let token_count = normalized_tokens(criterion).len();
    if token_count <= 2 { token_count } else { 2 }
}

fn normalized_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() > 2)
        .filter(|token| {
            !matches!(
                token.as_str(),
                "the" | "and" | "for" | "with" | "that" | "this" | "from" | "into"
            )
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn criteria_result_matches_natural_review_wording() {
        let output = AgentOutputRecord {
            id: "out_test".to_string(),
            work_item_id: "work_test".to_string(),
            agent_run_id: "run_test".to_string(),
            agent_profile_id: "reviewer".to_string(),
            purpose: AgentRunPurpose::Review,
            contract: "nagare.review.v1".to_string(),
            instruction_pack: "nagare-review-writer.v1".to_string(),
            parse_status: AgentOutputParseStatus::Parsed,
            fields: BTreeMap::from([
                ("verdict".to_string(), vec!["pass".to_string()]),
                (
                    "criteria".to_string(),
                    vec![
                        "No more than three steps: passed - `## Steps` contains exactly three numbered steps.".to_string(),
                    ],
                ),
            ]),
            questions: Vec::new(),
            next_action: Some("approve".to_string()),
            warnings: Vec::new(),
            execution_record_id: "exec_test".to_string(),
            locale: "en-US".to_string(),
            created_at: "1".to_string(),
        };

        let review = review_result_from_agent_output(
            "review_test".to_string(),
            &output,
            &["The guide includes no more than three steps".to_string()],
        );

        assert_eq!(review.criteria_results.len(), 1);
        assert_eq!(
            review.criteria_results[0].status,
            CriteriaReviewStatus::Passed
        );
    }
}
