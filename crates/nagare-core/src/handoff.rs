use std::path::PathBuf;

use crate::*;

pub fn create_handoff(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    from_agent_profile: &str,
    to_agent_profile: &str,
    reason: &str,
    summary: &str,
) -> Result<HandoffResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let snapshot = WorkItemSnapshot::from_ledger(item, &ledger);
    let handoff = HandoffPacket {
        id: ledger.next_id("handoff"),
        work_item_id: work_item_id.to_string(),
        from_agent_profile: from_agent_profile.to_string(),
        to_agent_profile: to_agent_profile.to_string(),
        reason: reason.to_string(),
        summary: handoff_summary(from_agent_profile, to_agent_profile, reason, summary),
        current_state: snapshot.item.status.to_string(),
        open_questions: open_questions(&snapshot),
        artifact_ids: artifact_ids(&snapshot),
        diff_artifact_ids: diff_artifact_ids(&snapshot),
        failed_verification_ids: failed_verification_ids(&snapshot),
        review_result_ids: review_result_ids(&snapshot),
        next_request: handoff_next_request(work_item_id, &snapshot, summary),
        locale,
        created_at: timestamp(),
    };
    ledger.handoffs.push(handoff.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::NeedsHandoff;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;
    Ok(HandoffResult { handoff })
}

fn handoff_summary(from_agent: &str, to_agent: &str, reason: &str, summary: &str) -> String {
    if summary.trim().is_empty() {
        format!("Handoff from {from_agent} to {to_agent}: {reason}")
    } else {
        summary.to_string()
    }
}

fn handoff_next_request(work_item_id: &str, snapshot: &WorkItemSnapshot, summary: &str) -> String {
    if summary.trim().is_empty() {
        format!(
            "Continue Work Item `{}` from state `{}` and address the handoff reason.",
            work_item_id, snapshot.item.status
        )
    } else {
        summary.to_string()
    }
}

fn open_questions(snapshot: &WorkItemSnapshot) -> Vec<String> {
    snapshot
        .agent_outputs
        .iter()
        .flat_map(|output| output.questions.clone())
        .collect()
}

fn artifact_ids(snapshot: &WorkItemSnapshot) -> Vec<String> {
    snapshot
        .artifacts
        .iter()
        .map(|artifact| artifact.id.clone())
        .collect()
}

fn diff_artifact_ids(snapshot: &WorkItemSnapshot) -> Vec<String> {
    snapshot
        .artifacts
        .iter()
        .filter(|artifact| artifact.artifact_type == "diff_patch")
        .map(|artifact| artifact.id.clone())
        .collect()
}

fn failed_verification_ids(snapshot: &WorkItemSnapshot) -> Vec<String> {
    snapshot
        .verification_results
        .iter()
        .filter(|verification| verification.result == VerificationStatus::Failed)
        .map(|verification| verification.id.clone())
        .collect()
}

fn review_result_ids(snapshot: &WorkItemSnapshot) -> Vec<String> {
    snapshot
        .review_results
        .iter()
        .map(|review| review.id.clone())
        .collect()
}
