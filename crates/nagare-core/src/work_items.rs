use std::path::PathBuf;

use crate::*;

pub fn create_work_item(
    root: impl Into<PathBuf>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Result<CreateItemResult, NagareError> {
    create_work_item_with_input(
        root,
        CreateWorkItemInput {
            title: title.into(),
            description: description.into(),
            ..CreateWorkItemInput::default()
        },
    )
}

pub fn create_work_item_with_input(
    root: impl Into<PathBuf>,
    input: CreateWorkItemInput,
) -> Result<CreateItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let config = load_project_config(&layout)?;
    let locale = config.locale.language.clone();
    let domains = load_domains(&layout)?;
    let artifact_type = input
        .artifact_type_id
        .as_deref()
        .map(|artifact_type_id| {
            load_artifact_types(&layout)?
                .remove(artifact_type_id)
                .ok_or_else(|| NagareError::NotFound(format!("Artifact Type `{artifact_type_id}`")))
        })
        .transpose()?;
    let domain_id = match (input.domain_id.as_deref(), artifact_type.as_ref()) {
        (Some(input_domain_id), Some(artifact_type)) => {
            validate_existing_domain(&layout, input_domain_id)?;
            if let Some(domain_id) = artifact_type.domain_id.as_deref() {
                if domain_id != input_domain_id {
                    return Err(NagareError::InvalidState(format!(
                        "Artifact Type `{}` belongs to domain `{domain_id}`, not `{input_domain_id}`",
                        artifact_type.id
                    )));
                }
            }
            Some(input_domain_id.to_string())
        }
        (Some(input_domain_id), None) => {
            validate_existing_domain(&layout, input_domain_id)?;
            Some(input_domain_id.to_string())
        }
        (None, Some(artifact_type)) => artifact_type.domain_id.clone(),
        (None, None) => None,
    };
    let domain = domain_id
        .as_deref()
        .and_then(|domain_id| domains.get(domain_id));
    let workflow_mode = input
        .workflow_mode
        .or_else(|| {
            artifact_type
                .as_ref()
                .and_then(|artifact_type| artifact_type.workflow.progress_mode)
        })
        .or_else(|| domain.and_then(|domain| domain.workflow.progress_mode))
        .unwrap_or(config.workflow.default_progress_mode);
    let approval_policy = input
        .approval_policy
        .or_else(|| {
            artifact_type
                .as_ref()
                .and_then(|artifact_type| artifact_type.workflow.approval_policy)
        })
        .or_else(|| domain.and_then(|domain| domain.workflow.approval_policy))
        .unwrap_or(config.workflow.approval_policy);
    let mut ledger = load_ledger(&layout)?;
    let work_folder = input
        .work_folder
        .as_deref()
        .map(normalize_working_dir)
        .transpose()?
        .filter(|path| path != ".");
    let now = timestamp();
    let item = WorkItem {
        id: ledger.next_id("work"),
        title: input.title,
        description: input.description,
        acceptance_criteria: normalize_text_list(input.acceptance_criteria),
        expected_artifacts: normalize_text_list(input.expected_artifacts),
        work_folder,
        constraints: normalize_text_list(input.constraints),
        domain_id,
        artifact_type_id: input.artifact_type_id,
        domain_agent_policy: input.domain_agent_policy,
        require_domain_agent: false,
        workflow_mode,
        approval_policy,
        locale,
        status: WorkItemStatus::Ready,
        created_at: now.clone(),
        updated_at: now,
    };
    ledger.work_items.push(item.clone());
    save_ledger(&layout, &ledger)?;
    Ok(CreateItemResult { item })
}

pub fn list_work_items(root: impl Into<PathBuf>) -> Result<Vec<WorkItem>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_ledger(&layout)?.work_items)
}

pub fn delete_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<WorkItem, NagareError> {
    let layout = ensure_project(root)?;
    let mut ledger = load_ledger(&layout)?;
    let index = ledger
        .work_items
        .iter()
        .position(|item| item.id == work_item_id)
        .ok_or_else(|| NagareError::NotFound(format!("work item `{work_item_id}`")))?;
    let item = ledger.work_items.remove(index);

    ledger.runs.retain(|run| run.work_item_id != work_item_id);
    ledger
        .artifacts
        .retain(|artifact| artifact.work_item_id != work_item_id);
    ledger
        .evidence
        .retain(|evidence| evidence.work_item_id != work_item_id);
    ledger
        .review_results
        .retain(|review| review.work_item_id != work_item_id);
    ledger
        .handoffs
        .retain(|handoff| handoff.work_item_id != work_item_id);
    ledger
        .decisions
        .retain(|decision| decision.work_item_id != work_item_id);
    ledger
        .human_feedback
        .retain(|feedback| feedback.work_item_id != work_item_id);
    ledger
        .dispatch_plans
        .retain(|plan| plan.work_item_id != work_item_id);
    ledger
        .recovery_plans
        .retain(|plan| plan.work_item_id != work_item_id);
    ledger
        .workflow_decisions
        .retain(|decision| decision.work_item_id != work_item_id);
    ledger
        .resolved_skill_contexts
        .retain(|context| context.work_item_id != work_item_id);
    ledger
        .resolved_run_packets
        .retain(|packet| packet.work_item_id != work_item_id);
    ledger
        .agent_outputs
        .retain(|output| output.work_item_id != work_item_id);

    save_ledger(&layout, &ledger)?;
    Ok(item)
}

pub fn get_work_item_snapshot(
    root: impl Into<PathBuf>,
    work_item_id: &str,
) -> Result<WorkItemSnapshot, NagareError> {
    let layout = ensure_project(root)?;
    let ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    Ok(WorkItemSnapshot::from_ledger(item, &ledger))
}

pub fn answer_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: AnswerWorkItemInput<'_>,
) -> Result<AnswerWorkItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let answer = input.answer.trim();
    if answer.is_empty() {
        return Err(NagareError::InvalidState(
            "answer cannot be empty".to_string(),
        ));
    }
    let latest_question = latest_agent_question(&ledger, work_item_id);
    let question = input
        .question
        .map(str::trim)
        .filter(|question| !question.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            latest_question
                .as_ref()
                .map(|(_, question)| question.clone())
        })
        .unwrap_or_else(|| "(unspecified question)".to_string());
    let source_agent_output_id = latest_question.map(|(id, _)| id);
    let feedback = HumanFeedback {
        id: ledger.next_id("feedback"),
        work_item_id: work_item_id.to_string(),
        source_agent_output_id,
        question,
        answer: answer.to_string(),
        locale,
        created_at: timestamp(),
    };
    ledger.human_feedback.push(feedback.clone());
    let item = ledger.work_item_mut(work_item_id)?;
    item.status = WorkItemStatus::Ready;
    item.updated_at = timestamp();
    let item_status = item.status;
    save_ledger(&layout, &ledger)?;
    Ok(AnswerWorkItemResult {
        feedback,
        item_status,
    })
}

pub(crate) fn work_item_goal_prompt_for_locale(item: &WorkItem, locale: &str) -> String {
    let i18n = I18n::new(locale);
    if item.description.trim().is_empty()
        && item.acceptance_criteria.is_empty()
        && item.expected_artifacts.is_empty()
        && item.constraints.is_empty()
    {
        return item.title.clone();
    }
    let mut lines = vec![item.title.clone()];
    if !item.description.trim().is_empty() {
        lines.push(item.description.clone());
    }
    if !item.acceptance_criteria.is_empty() {
        lines.push(format!("## {}", i18n.ui(UiTextKey::AcceptanceCriteria)));
        lines.extend(
            item.acceptance_criteria
                .iter()
                .map(|criterion| format!("- {criterion}")),
        );
    }
    if !item.expected_artifacts.is_empty() {
        lines.push(format!("## {}", i18n.ui(UiTextKey::ExpectedArtifacts)));
        lines.extend(
            item.expected_artifacts
                .iter()
                .map(|artifact| format!("- {artifact}")),
        );
    }
    if !item.constraints.is_empty() {
        lines.push(format!("## {}", i18n.ui(UiTextKey::Constraints)));
        lines.extend(
            item.constraints
                .iter()
                .map(|constraint| format!("- {constraint}")),
        );
    }
    lines.join("\n")
}

fn latest_agent_question(ledger: &Ledger, work_item_id: &str) -> Option<(String, String)> {
    ledger
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.work_item_id == work_item_id && !output.questions.is_empty())
        .and_then(|output| {
            output
                .questions
                .first()
                .map(|question| (output.id.clone(), question.clone()))
        })
}

fn normalize_text_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}
