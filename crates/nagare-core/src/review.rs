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
    pub rubric_results: Vec<RubricReviewResult>,
    #[serde(default)]
    pub rubric_expected_count: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricReviewResult {
    pub item: String,
    #[serde(default)]
    pub points: Option<u32>,
    pub max_points: u32,
    #[serde(default)]
    pub verdict: RubricReviewVerdict,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub recorded: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RubricReviewVerdict {
    Pass,
    Partial,
    Fail,
    NotApplicable,
    #[default]
    Unknown,
}

impl std::fmt::Display for RubricReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Pass => "pass",
            Self::Partial => "partial",
            Self::Fail => "fail",
            Self::NotApplicable => "not_applicable",
            Self::Unknown => "unknown",
        };
        f.write_str(value)
    }
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
    rubric: &[String],
) -> ReviewResult {
    let rubric_definitions = rubric_definitions(rubric);
    let rubric_expected_count = rubric_definitions.len();
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
        rubric_results: rubric_results_from_output(output, &rubric_definitions),
        rubric_expected_count,
        questions: output.questions.clone(),
        next_action: output.next_action.clone(),
        execution_record_id: output.execution_record_id.clone(),
        locale: output.locale.clone(),
        created_at: output.created_at.clone(),
    }
}

#[derive(Clone)]
struct RubricDefinition {
    item: String,
    max_points: u32,
}

#[derive(Clone)]
struct ParsedRubricResult {
    item: String,
    points: Option<u32>,
    max_points: u32,
    verdict: RubricReviewVerdict,
    evidence: String,
}

fn rubric_definitions(rubric: &[String]) -> Vec<RubricDefinition> {
    rubric
        .iter()
        .flat_map(|entry| entry.lines())
        .filter_map(|line| {
            let heading = line.trim().strip_prefix("## ")?.trim();
            let (item, score) = heading.rsplit_once('(')?;
            let max_points = score.trim().strip_suffix(')')?.trim().parse().ok()?;
            let item = item.trim();
            (!item.is_empty()).then(|| RubricDefinition {
                item: item.to_string(),
                max_points,
            })
        })
        .collect()
}

fn rubric_results_from_output(
    output: &AgentOutputRecord,
    definitions: &[RubricDefinition],
) -> Vec<RubricReviewResult> {
    let parsed = output
        .fields
        .get("rubric_scores")
        .or_else(|| output.fields.get("rubric_results"))
        .into_iter()
        .flatten()
        .filter_map(|line| parse_rubric_result(line))
        .collect::<Vec<_>>();

    if definitions.is_empty() {
        return parsed
            .into_iter()
            .map(|result| RubricReviewResult {
                item: result.item,
                points: result.points,
                max_points: result.max_points,
                verdict: result.verdict,
                evidence: result.evidence,
                recorded: result.points.is_some() && result.max_points > 0,
            })
            .collect();
    }

    definitions
        .iter()
        .map(|definition| {
            let matched = parsed.iter().find(|result| {
                normalize_rubric_item(&result.item) == normalize_rubric_item(&definition.item)
            });
            let Some(result) = matched else {
                return RubricReviewResult {
                    item: definition.item.clone(),
                    points: None,
                    max_points: definition.max_points,
                    verdict: RubricReviewVerdict::Unknown,
                    evidence: "ルーブリック観点別得点が未記録です。".to_string(),
                    recorded: false,
                };
            };
            let max_mismatch = result.max_points != definition.max_points;
            let points = result
                .points
                .map(|points| points.min(definition.max_points));
            let evidence = if max_mismatch {
                format!(
                    "{} [出力配点 {}/定義配点 {}]",
                    result.evidence, result.max_points, definition.max_points
                )
            } else {
                result.evidence.clone()
            };
            RubricReviewResult {
                item: definition.item.clone(),
                points,
                max_points: definition.max_points,
                verdict: result.verdict,
                evidence,
                recorded: points.is_some() && result.max_points > 0,
            }
        })
        .collect()
}

fn parse_rubric_result(line: &str) -> Option<ParsedRubricResult> {
    let mut segments = line.split('|').map(str::trim);
    let item = segments.next()?.trim();
    if item.is_empty() || is_empty_rubric_value(item) {
        return None;
    }
    let mut points = None;
    let mut max_points = None;
    let mut verdict = RubricReviewVerdict::Unknown;
    let mut evidence = String::new();
    for segment in segments {
        let Some((key, value)) = segment.split_once('=').or_else(|| segment.split_once(':')) else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase().replace('-', "_");
        let value = value.trim();
        match key.as_str() {
            "points" | "score" => points = value.parse::<u32>().ok(),
            "max_points" | "max" => max_points = value.parse::<u32>().ok(),
            "verdict" | "status" => verdict = parse_rubric_verdict(value),
            "evidence" | "reason" => evidence = value.to_string(),
            _ => {}
        }
    }
    let max_points = max_points?;
    Some(ParsedRubricResult {
        item: item.to_string(),
        points,
        max_points,
        verdict,
        evidence,
    })
}

fn parse_rubric_verdict(value: &str) -> RubricReviewVerdict {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "pass" | "passed" | "ok" => RubricReviewVerdict::Pass,
        "partial" | "partially_met" | "minor_concern" => RubricReviewVerdict::Partial,
        "fail" | "failed" | "concern" => RubricReviewVerdict::Fail,
        "not_applicable" | "na" | "n_a" => RubricReviewVerdict::NotApplicable,
        _ => RubricReviewVerdict::Unknown,
    }
}

fn normalize_rubric_item(value: &str) -> String {
    rubric_item_name(value)
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '　', '-', '_', '・'], "")
}

fn rubric_item_name(value: &str) -> String {
    let value = value.trim().trim_start_matches('#').trim();
    if let Some((item, score)) = value.rsplit_once('(') {
        let score = score.trim().strip_suffix(')').unwrap_or(score).trim();
        if score.parse::<u32>().is_ok() {
            return item.trim().to_string();
        }
    }
    value.to_string()
}

fn is_empty_rubric_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "none" | "n/a" | "na" | "なし"
    )
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
            &[],
        );

        assert_eq!(review.criteria_results.len(), 1);
        assert_eq!(
            review.criteria_results[0].status,
            CriteriaReviewStatus::Passed
        );
    }

    #[test]
    fn rubric_results_are_recorded_against_expected_headings() {
        let mut output = sample_review_output();
        output.fields.insert(
            "rubric_scores".to_string(),
            vec![
                "## Correctness (40) | points=35 | max_points=40 | verdict=partial | evidence=One edge case is missing.".to_string(),
                "## Clarity (60) | points=60 | max_points=60 | verdict=pass | evidence=The structure is explicit.".to_string(),
            ],
        );

        let review = review_result_from_agent_output(
            "review_test".to_string(),
            &output,
            &[],
            &[
                "## Correctness (40)".to_string(),
                "## Clarity (60)".to_string(),
            ],
        );

        assert_eq!(review.rubric_expected_count, 2);
        assert_eq!(review.rubric_results.len(), 2);
        assert_eq!(review.rubric_results[0].points, Some(35));
        assert_eq!(review.rubric_results[0].max_points, 40);
        assert_eq!(
            review.rubric_results[0].verdict,
            RubricReviewVerdict::Partial
        );
        assert!(review.rubric_results[0].recorded);
    }

    #[test]
    fn omitted_rubric_result_is_marked_unrecorded_instead_of_zero() {
        let mut output = sample_review_output();
        output.fields.insert(
            "rubric_scores".to_string(),
            vec![
                "Correctness | points=40 | max_points=40 | verdict=pass | evidence=Covered."
                    .to_string(),
            ],
        );

        let review = review_result_from_agent_output(
            "review_test".to_string(),
            &output,
            &[],
            &[
                "## Correctness (40)".to_string(),
                "## Clarity (60)".to_string(),
            ],
        );

        assert_eq!(review.rubric_results[1].item, "Clarity");
        assert_eq!(review.rubric_results[1].points, None);
        assert!(!review.rubric_results[1].recorded);
    }

    fn sample_review_output() -> AgentOutputRecord {
        AgentOutputRecord {
            id: "out_test".to_string(),
            work_item_id: "work_test".to_string(),
            agent_run_id: "run_test".to_string(),
            agent_profile_id: "reviewer".to_string(),
            purpose: AgentRunPurpose::Review,
            contract: "nagare.review.v1".to_string(),
            instruction_pack: "nagare-review-writer.v1".to_string(),
            parse_status: AgentOutputParseStatus::Parsed,
            fields: BTreeMap::from([("verdict".to_string(), vec!["pass".to_string()])]),
            questions: Vec::new(),
            next_action: Some("approve".to_string()),
            warnings: Vec::new(),
            execution_record_id: "exec_test".to_string(),
            locale: "en-US".to_string(),
            created_at: "1".to_string(),
        }
    }
}
