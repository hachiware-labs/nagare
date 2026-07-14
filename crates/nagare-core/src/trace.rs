use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::*;

const TRACE_SCHEMA: &str = "nagare.trace/1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRecord {
    pub schema: String,
    pub record: String,
    pub work_id: String,
    pub seq: u64,
    pub at: String,
    pub payload: Value,
}

pub fn list_work_trace(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<Vec<TraceRecord>, NagareError> {
    let layout = ensure_project(root)?;
    read_work_trace(&layout, work_item_id)
}

pub(crate) fn append_work_header_trace(
    layout: &ProjectLayout,
    item: &WorkItem,
) -> Result<TraceRecord, NagareError> {
    append_trace_record(
        layout,
        &item.id,
        "work_header",
        &item.created_at,
        json!({
            "request_text": request_text(item),
            "project_id": item.work_folder.as_deref().unwrap_or("default"),
            "confirmation_policy": item.approval_policy.to_string(),
            "created_at": item.created_at,
        }),
    )
}

pub(crate) fn append_agent_run_trace(
    layout: &ProjectLayout,
    item: &WorkItem,
    run: &AgentRun,
    profile: &AgentProfile,
    run_packet: &ResolvedRunPacket,
    skill_context: &ResolvedSkillContext,
    output: Option<&AgentOutputRecord>,
    review: Option<&ReviewResult>,
    dispatch_plan: Option<&DispatchPlan>,
    artifacts: &[Artifact],
) -> Result<Option<TraceRecord>, NagareError> {
    match run.purpose {
        AgentRunPurpose::DispatchPreview => {
            let Some(plan) = dispatch_plan else {
                return Ok(None);
            };
            let step_no = next_trace_step_no(layout, &item.id)?;
            Ok(Some(append_trace_record(
                layout,
                &item.id,
                "organizer_decision",
                &plan.created_at,
                json!({
                    "step_no": step_no,
                    "step_kind": "intake",
                    "agent": trace_agent(profile, "organizer"),
                    "runtime": trace_runtime(profile, run_packet),
                    "duration_ms": duration_ms(&run.started_at, &run.ended_at),
                    "status": trace_run_status(run.status),
                    "knowledge_refs": knowledge_refs(item),
                    "diagnostics": diagnostics(run, run_packet),
                    "interpreted_request": plan.summary,
                    "domain_id": item.domain_id.as_deref().unwrap_or("general"),
                    "artifact_type_id": item.artifact_type_id.as_deref().unwrap_or("general"),
                    "plan": [
                        { "step_no": step_no + 1, "step_kind": "create", "agent_id": plan.target_agent_profile_id }
                    ],
                    "assignments": [
                        {
                            "step_no": step_no + 1,
                            "agent_id": plan.target_agent_profile_id,
                            "rationale": assignment_rationale(plan)
                        }
                    ],
                    "candidates_considered": dispatch_candidates(plan),
                }),
            )?))
        }
        AgentRunPurpose::Work => {
            let step_no = next_trace_step_no(layout, &item.id)?;
            Ok(Some(append_trace_record(
                layout,
                &item.id,
                "worker_output",
                &run.ended_at,
                json!({
                    "step_no": step_no,
                    "step_kind": "create",
                    "agent": trace_agent(profile, "worker"),
                    "runtime": trace_runtime(profile, run_packet),
                    "duration_ms": duration_ms(&run.started_at, &run.ended_at),
                    "status": trace_run_status(run.status),
                    "knowledge_refs": knowledge_refs(item),
                    "diagnostics": diagnostics(run, run_packet),
                    "inputs": {
                        "summary": run_packet.goal,
                        "refs": worker_input_refs(item, skill_context, run_packet),
                    },
                    "actions_summary": output.and_then(output_summary).unwrap_or_else(|| {
                        format!("{} exited with {}.", run.agent_profile_id, format_exit(run.exit_code))
                    }),
                    "artifacts": trace_artifacts_for_run(artifacts, &run.id),
                    "answer": output.and_then(output_answer).unwrap_or_else(|| {
                        output_summary_text(run, output)
                    }),
                    "question": output.and_then(trace_question),
                }),
            )?))
        }
        AgentRunPurpose::Synthesis => {
            let step_no = next_trace_step_no(layout, &item.id)?;
            Ok(Some(append_trace_record(
                layout,
                &item.id,
                "organizer_summary",
                &run.ended_at,
                json!({
                    "step_no": step_no,
                    "step_kind": "synthesis",
                    "agent": trace_agent(profile, "organizer"),
                    "runtime": trace_runtime(profile, run_packet),
                    "duration_ms": duration_ms(&run.started_at, &run.ended_at),
                    "status": trace_run_status(run.status),
                    "knowledge_refs": knowledge_refs(item),
                    "diagnostics": diagnostics(run, run_packet),
                    "inputs": {
                        "summary": run_packet.goal,
                        "refs": worker_input_refs(item, skill_context, run_packet),
                    },
                    "actions_summary": output.and_then(output_summary).unwrap_or_else(|| {
                        output_summary_text(run, output)
                    }),
                    "artifacts": trace_artifacts_for_run(artifacts, &run.id),
                    "answer": output.and_then(output_answer).unwrap_or_else(|| {
                        output_summary_text(run, output)
                    }),
                    "question": output.and_then(trace_question),
                }),
            )?))
        }
        AgentRunPurpose::Review => {
            let Some(review) = review else {
                return Ok(None);
            };
            let step_no = next_trace_step_no(layout, &item.id)?;
            let item_verdicts = trace_review_items(review);
            let max_score = item_verdicts.len() as u64;
            let total_score = review
                .criteria_results
                .iter()
                .filter(|result| result.status == CriteriaReviewStatus::Passed)
                .count() as u64;
            let overall_score = output.and_then(review_overall_score);
            Ok(Some(append_trace_record(
                layout,
                &item.id,
                "reviewer_verdict",
                &review.created_at,
                json!({
                    "step_no": step_no,
                    "step_kind": "review",
                    "agent": trace_agent(profile, "reviewer"),
                    "runtime": trace_runtime(profile, run_packet),
                    "duration_ms": duration_ms(&run.started_at, &run.ended_at),
                    "status": trace_run_status(run.status),
                    "knowledge_refs": knowledge_refs(item),
                    "diagnostics": diagnostics(run, run_packet),
                    "rubric_ref": {
                        "id": item.artifact_type_id.as_deref().unwrap_or("acceptance_criteria"),
                        "version": 1,
                    },
                    "target_artifacts": trace_artifact_paths(artifacts, &item.id),
                    "item_verdicts": item_verdicts,
                    "total_score": total_score,
                    "max_score": max_score,
                    "overall_score": overall_score,
                    "overall_max_score": overall_score.map(|_| 100),
                    "recommendation": review_recommendation(review),
                    "summary": first_nonempty(&review.summary).unwrap_or_else(|| review.verdict.to_string()),
                }),
            )?))
        }
        AgentRunPurpose::WorkflowSupervision => Ok(None),
    }
}

pub(crate) fn append_human_feedback_trace(
    layout: &ProjectLayout,
    feedback: &HumanFeedback,
) -> Result<TraceRecord, NagareError> {
    append_trace_record(
        layout,
        &feedback.work_item_id,
        "human_decision",
        &feedback.created_at,
        json!({
            "kind": "answer",
            "refs": feedback
                .source_agent_output_id
                .as_ref()
                .map(|id| vec![format!("worker_output#{id}")])
                .unwrap_or_default(),
            "content": {
                "selected": null,
                "note": feedback.answer,
                "question": feedback.question,
            },
        }),
    )
}

pub(crate) fn append_human_decision_trace(
    layout: &ProjectLayout,
    decision: &HumanDecision,
) -> Result<TraceRecord, NagareError> {
    append_trace_record(
        layout,
        &decision.work_item_id,
        "human_decision",
        &decision.created_at,
        json!({
            "kind": decision.decision_type,
            "refs": [],
            "content": human_decision_content(decision),
        }),
    )
}

pub(crate) fn append_recovery_event_trace(
    layout: &ProjectLayout,
    plan: &RecoveryPlan,
) -> Result<TraceRecord, NagareError> {
    append_trace_record(
        layout,
        &plan.work_item_id,
        "recovery_event",
        &plan.created_at,
        json!({
            "step_no": next_trace_step_no(layout, &plan.work_item_id)?,
            "cause": plan.reason,
            "impact": plan.summary,
            "handoff": {
                "completed": [],
                "pending": recovery_pending(plan),
            },
            "diagnostics": {
                "source_event_id": plan.source_event_id,
                "command_hint": plan.command_hint,
            },
        }),
    )
}

pub(crate) fn append_recovery_choice_trace(
    layout: &ProjectLayout,
    plan: &RecoveryPlan,
) -> Result<TraceRecord, NagareError> {
    append_trace_record(
        layout,
        &plan.work_item_id,
        "human_decision",
        &plan.created_at,
        json!({
            "kind": "recovery_choice",
            "refs": [format!("recovery_event#{}", plan.id)],
            "content": {
                "option": plan.action.to_string(),
                "plan_id": plan.id,
            },
        }),
    )
}

fn read_work_trace(
    layout: &ProjectLayout,
    work_item_id: &str,
) -> Result<Vec<TraceRecord>, NagareError> {
    let path = work_trace_path(layout, work_item_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str::<TraceRecord>(line).map_err(NagareError::from))
        .collect()
}

fn append_trace_record(
    layout: &ProjectLayout,
    work_item_id: &str,
    record: &str,
    at: &str,
    payload: Value,
) -> Result<TraceRecord, NagareError> {
    let path = work_trace_path(layout, work_item_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let trace = TraceRecord {
        schema: TRACE_SCHEMA.to_string(),
        record: record.to_string(),
        work_id: work_item_id.to_string(),
        seq: next_trace_seq(&path)?,
        at: at.to_string(),
        payload,
    };
    let raw = serde_json::to_string(&trace)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{raw}")?;
    Ok(trace)
}

fn work_trace_path(layout: &ProjectLayout, work_item_id: &str) -> PathBuf {
    layout
        .nagare_dir
        .join("works")
        .join(work_item_id)
        .join("trace.jsonl")
}

fn next_trace_seq(path: &Path) -> Result<u64, NagareError> {
    if !path.exists() {
        return Ok(1);
    }
    let raw = fs::read_to_string(path)?;
    Ok(raw.lines().filter(|line| !line.trim().is_empty()).count() as u64 + 1)
}

fn next_trace_step_no(layout: &ProjectLayout, work_item_id: &str) -> Result<u64, NagareError> {
    let max_step = read_work_trace(layout, work_item_id)?
        .into_iter()
        .filter(|record| {
            matches!(
                record.record.as_str(),
                "organizer_decision" | "worker_output" | "reviewer_verdict"
            )
        })
        .filter_map(|record| record.payload.get("step_no").and_then(Value::as_u64))
        .max()
        .unwrap_or(0);
    Ok(max_step + 1)
}

fn request_text(item: &WorkItem) -> String {
    if item.description.trim().is_empty() {
        item.title.clone()
    } else {
        item.description.clone()
    }
}

fn trace_agent(profile: &AgentProfile, fallback_role: &str) -> Value {
    let role = normalized_trace_role(&profile.role, fallback_role);
    json!({
        "id": profile.id,
        "name": profile.display_name,
        "role": role,
        "builtin": profile.source == AgentProfileSource::ProjectConfig,
    })
}

fn normalized_trace_role(role: &str, fallback_role: &str) -> String {
    let normalized = role.to_ascii_lowercase();
    if normalized.contains("organizer")
        || normalized.contains("dispatch")
        || normalized.contains("supervisor")
    {
        "organizer".to_string()
    } else if normalized.contains("review") {
        "reviewer".to_string()
    } else if normalized.contains("worker") || normalized.contains("implement") {
        "worker".to_string()
    } else {
        fallback_role.to_string()
    }
}

fn trace_runtime(profile: &AgentProfile, run_packet: &ResolvedRunPacket) -> Value {
    json!({
        "id": profile.runtime,
        "model": model_label(&run_packet.model),
    })
}

fn model_label(model: &AgentModelSelection) -> String {
    if model.id.trim().is_empty() {
        "runtime-default".to_string()
    } else if model.provider.trim().is_empty() {
        model.id.clone()
    } else {
        format!("{}/{}", model.provider, model.id)
    }
}

fn duration_ms(started_at: &str, ended_at: &str) -> u64 {
    let started = started_at.parse::<u64>().unwrap_or_default();
    let ended = ended_at.parse::<u64>().unwrap_or(started);
    ended.saturating_sub(started) * 1000
}

fn trace_run_status(status: AgentRunStatus) -> &'static str {
    match status {
        AgentRunStatus::Succeeded => "completed",
        AgentRunStatus::Failed => "failed",
    }
}

fn knowledge_refs(item: &WorkItem) -> Vec<Value> {
    item.domain_id
        .iter()
        .chain(item.artifact_type_id.iter())
        .map(|id| json!({ "id": id, "version": 1 }))
        .collect()
}

fn diagnostics(run: &AgentRun, run_packet: &ResolvedRunPacket) -> Value {
    json!({
        "runtime": run.adapter,
        "session_ref": run.id,
        "hint": run.command,
        "run_packet": run_packet.id,
    })
}

fn assignment_rationale(plan: &DispatchPlan) -> String {
    plan.selection_warnings
        .first()
        .cloned()
        .unwrap_or_else(|| plan.summary.clone())
}

fn dispatch_candidates(plan: &DispatchPlan) -> Vec<Value> {
    plan.selection_warnings
        .iter()
        .map(|warning| {
            json!({
                "agent_id": plan.target_agent_profile_id,
                "reason_rejected": warning,
            })
        })
        .collect()
}

fn worker_input_refs(
    item: &WorkItem,
    skill_context: &ResolvedSkillContext,
    run_packet: &ResolvedRunPacket,
) -> Vec<String> {
    let mut refs = Vec::new();
    refs.extend(item.domain_id.iter().map(|id| format!("domain:{id}")));
    refs.extend(
        item.artifact_type_id
            .iter()
            .map(|id| format!("artifact_type:{id}")),
    );
    refs.extend(
        skill_context
            .applied_skill_set_ids
            .iter()
            .map(|id| format!("skill_set:{id}")),
    );
    refs.extend(
        run_packet
            .dispatch_plan_id
            .iter()
            .map(|id| format!("dispatch_plan:{id}")),
    );
    refs
}

fn output_summary(output: &AgentOutputRecord) -> Option<String> {
    first_output_field(output, "summary").or_else(|| first_output_field(output, "completed"))
}

fn output_answer(output: &AgentOutputRecord) -> Option<String> {
    output_summary(output).or_else(|| first_output_field(output, "answer"))
}

fn output_summary_text(run: &AgentRun, output: Option<&AgentOutputRecord>) -> String {
    output.and_then(output_summary).unwrap_or_else(|| {
        format!(
            "{} exited with {}.",
            run.agent_profile_id,
            format_exit(run.exit_code)
        )
    })
}

fn first_output_field(output: &AgentOutputRecord, key: &str) -> Option<String> {
    output
        .fields
        .get(key)
        .and_then(|values| values.iter().find(|value| !value.trim().is_empty()))
        .cloned()
}

fn trace_question(output: &AgentOutputRecord) -> Option<Value> {
    output.questions.first().map(|question| {
        json!({
            "id": output.id,
            "text": question,
            "options": [],
        })
    })
}

fn trace_artifacts_for_run(artifacts: &[Artifact], run_id: &str) -> Vec<Value> {
    artifacts
        .iter()
        .filter(|artifact| artifact.agent_run_id.as_deref() == Some(run_id))
        .map(trace_artifact)
        .collect()
}

fn trace_artifact_paths(artifacts: &[Artifact], work_item_id: &str) -> Vec<String> {
    artifacts
        .iter()
        .filter(|artifact| artifact.work_item_id == work_item_id)
        .map(|artifact| artifact.uri.clone())
        .collect()
}

fn trace_artifact(artifact: &Artifact) -> Value {
    json!({
        "path": artifact.uri,
        "change": artifact_change(artifact),
        "lines": 0,
    })
}

fn artifact_change(artifact: &Artifact) -> &'static str {
    match artifact.artifact_type.as_str() {
        "deleted_file" => "deleted",
        "new_file" => "new",
        _ => "modified",
    }
}

fn trace_review_items(review: &ReviewResult) -> Vec<Value> {
    review
        .criteria_results
        .iter()
        .map(|result| {
            let passed = result.status == CriteriaReviewStatus::Passed;
            json!({
                "item": result.criterion,
                "max_points": 1,
                "points": if passed { 1 } else { 0 },
                "verdict": if passed { "pass" } else { "concern" },
                "evidence": if result.note.trim().is_empty() { result.status.to_string() } else { result.note.clone() },
                "concern_note": if passed { Value::Null } else { Value::String(result.note.clone()) },
            })
        })
        .collect()
}

fn review_recommendation(review: &ReviewResult) -> &'static str {
    match review.verdict {
        ReviewVerdict::Pass => "approve",
        ReviewVerdict::RequestChanges | ReviewVerdict::Blocked | ReviewVerdict::Unknown => "revise",
    }
}

fn review_overall_score(output: &AgentOutputRecord) -> Option<u8> {
    output
        .fields
        .get("overall_score")
        .and_then(|values| values.first())
        .and_then(|value| value.trim().parse::<u8>().ok())
        .filter(|score| *score <= 100)
}

fn first_nonempty(values: &[String]) -> Option<String> {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn human_decision_content(decision: &HumanDecision) -> Value {
    match decision.decision_type.as_str() {
        "approve" => json!({ "rationale": decision.rationale }),
        "reject" => json!({
            "comment": decision.rationale,
            "cited_concerns": cited_concerns_from_rationale(&decision.rationale)
        }),
        other => json!({ "kind": other, "note": decision.rationale }),
    }
}

fn cited_concerns_from_rationale(rationale: &str) -> Vec<String> {
    let mut in_citations = false;
    let mut concerns = Vec::new();
    for line in rationale.lines().map(str::trim) {
        if line.contains("引用したレビュー懸念") || line.eq_ignore_ascii_case("cited concerns:")
        {
            in_citations = true;
            continue;
        }
        if in_citations && line.ends_with(':') && !line.starts_with('-') {
            break;
        }
        if in_citations {
            if let Some(item) = line.strip_prefix("- ") {
                let item = item.trim();
                if !item.is_empty() {
                    concerns.push(item.to_string());
                }
            }
        }
    }
    concerns
}

fn recovery_pending(plan: &RecoveryPlan) -> Vec<String> {
    plan.prompt_hint
        .iter()
        .chain(plan.command_hint.iter())
        .cloned()
        .collect()
}

fn format_exit(exit_code: Option<i32>) -> String {
    exit_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_decision_content_extracts_cited_concerns() {
        let decision = HumanDecision {
            id: "decision_1".to_string(),
            work_item_id: "work_1".to_string(),
            decision_type: "reject".to_string(),
            rationale: "引用したレビュー懸念:\n- 読みやすさ: 補足が長い\n- 形式: 見出し不足\n修正指示:\n- 見出しを追加".to_string(),
            locale: "ja".to_string(),
            created_at: "2026-07-05T00:00:00+09:00".to_string(),
        };

        let content = human_decision_content(&decision);

        assert_eq!(content["cited_concerns"][0], "読みやすさ: 補足が長い");
        assert_eq!(content["cited_concerns"][1], "形式: 見出し不足");
        assert_eq!(
            content["cited_concerns"].as_array().expect("array").len(),
            2
        );
    }
}
