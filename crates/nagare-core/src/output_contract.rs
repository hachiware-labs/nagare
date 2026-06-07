use std::collections::BTreeMap;

use crate::*;

pub(crate) struct AgentOutputRecordInput<'a> {
    pub(crate) id: String,
    pub(crate) work_item_id: &'a str,
    pub(crate) agent_run_id: &'a str,
    pub(crate) agent_profile_id: &'a str,
    pub(crate) purpose: AgentRunPurpose,
    pub(crate) contract: &'a AgentOutputContract,
    pub(crate) stdout: &'a str,
    pub(crate) execution_record_id: &'a str,
    pub(crate) locale: &'a str,
    pub(crate) created_at: &'a str,
}

pub(crate) fn parse_agent_output_record(input: AgentOutputRecordInput<'_>) -> AgentOutputRecord {
    let mut warnings = Vec::new();
    let section_name = match input.purpose {
        AgentRunPurpose::Work => "nagare result",
        AgentRunPurpose::Review => "nagare review",
        AgentRunPurpose::DispatchPreview => "nagare dispatch",
        AgentRunPurpose::WorkflowSupervision => "nagare workflow decision",
    };
    let section = extract_markdown_section(input.stdout, section_name);
    let fields = section
        .as_deref()
        .map(parse_contract_fields)
        .unwrap_or_default();
    if section.is_none() && input.contract.required {
        warnings.push("output_contract_unparsed".to_string());
    }
    let questions = fields
        .get("questions")
        .or_else(|| fields.get("question"))
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|value| !is_empty_contract_value(value))
        .collect::<Vec<_>>();
    let next_action = fields
        .get("next_action")
        .and_then(|values| values.first())
        .map(|value| normalize_next_action(value));
    if let Some(action) = next_action.as_deref() {
        if !valid_next_action(action) {
            warnings.push(format!("invalid_next_action:{action}"));
        }
        if matches!(action, "answer_question" | "needs_input") && questions.is_empty() {
            warnings.push("next_action_without_question".to_string());
        }
    }
    if section.is_some()
        && matches!(
            input.purpose,
            AgentRunPurpose::Work | AgentRunPurpose::Review
        )
    {
        if has_nested_contract_keys(&fields) {
            warnings.push("nested_contract_fields".to_string());
        }
        if !has_contract_field(&fields, "completed") {
            warnings.push("missing_completed".to_string());
        }
        if !has_contract_field(&fields, "next_notes") {
            warnings.push("missing_next_notes".to_string());
        }
    }
    AgentOutputRecord {
        id: input.id,
        work_item_id: input.work_item_id.to_string(),
        agent_run_id: input.agent_run_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        purpose: input.purpose,
        contract: input.contract.contract.clone(),
        instruction_pack: input.contract.instruction_pack.clone(),
        parse_status: if section.is_some() {
            AgentOutputParseStatus::Parsed
        } else {
            AgentOutputParseStatus::Unparsed
        },
        fields,
        questions,
        next_action,
        warnings,
        execution_record_id: input.execution_record_id.to_string(),
        locale: input.locale.to_string(),
        created_at: input.created_at.to_string(),
    }
}

pub(crate) fn agent_output_requires_input(record: Option<&AgentOutputRecord>) -> bool {
    record.is_some_and(|record| !record.questions.is_empty())
}

pub(crate) fn agent_output_requests_handoff(record: Option<&AgentOutputRecord>) -> bool {
    record.is_some_and(|record| {
        record
            .next_action
            .as_deref()
            .is_some_and(|action| action == "handoff" || action == "create_handoff")
    })
}

pub(crate) fn prompt_with_output_contract(
    prompt: &str,
    purpose: AgentRunPurpose,
    contract: &AgentOutputContract,
    locale: &str,
) -> String {
    if contract.injection != AgentOutputInjection::PromptSuffix {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n{}",
        localized_output_contract_instruction(locale, purpose, contract)
    )
}

pub(crate) fn prompt_with_human_feedback(prompt: &str, context: &str, locale: &str) -> String {
    if context.trim().is_empty() {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n## {}\n{context}",
        localized_context_heading(locale, ContextHeading::HumanFeedback)
    )
}

pub(crate) fn prompt_with_handoff_context(prompt: &str, context: &str, locale: &str) -> String {
    if context.trim().is_empty() {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n## {}\n{context}",
        localized_context_heading(locale, ContextHeading::HandoffContext)
    )
}

pub(crate) fn human_feedback_prompt_context(ledger: &Ledger, work_item_id: &str) -> String {
    ledger
        .human_feedback
        .iter()
        .filter(|feedback| feedback.work_item_id == work_item_id)
        .map(|feedback| {
            format!(
                "- question: {}\n  answer: {}",
                feedback.question, feedback.answer
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn handoff_prompt_context(ledger: &Ledger, work_item_id: &str) -> String {
    ledger
        .handoffs
        .iter()
        .rev()
        .find(|handoff| handoff.work_item_id == work_item_id)
        .map(|handoff| {
            [
                format!("handoff_id: {}", handoff.id),
                format!("from_agent: {}", handoff.from_agent_profile),
                format!("to_agent: {}", handoff.to_agent_profile),
                format!("current_state: {}", handoff.current_state),
                format!("reason: {}", handoff.reason),
                format!("next_request: {}", handoff.next_request),
                format!("open_questions: {}", handoff.open_questions.join(", ")),
                format!("artifact_ids: {}", handoff.artifact_ids.join(", ")),
                format!(
                    "execution_record_ids: {}",
                    handoff.execution_record_ids.join(", ")
                ),
                format!(
                    "review_result_ids: {}",
                    handoff.review_result_ids.join(", ")
                ),
            ]
            .join("\n")
        })
        .unwrap_or_default()
}

fn extract_markdown_section(output: &str, section_name: &str) -> Option<String> {
    let output = dispatch_text_output(output);
    let mut capture = false;
    let mut lines = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let title = trimmed.trim_start_matches('#').trim().to_ascii_lowercase();
            if capture {
                break;
            }
            capture = title == section_name;
            continue;
        }
        if capture {
            lines.push(line);
        }
    }
    if capture {
        return Some(lines.join("\n"));
    }
    fallback_contract_section(&output, section_name)
}

fn parse_contract_fields(section: &str) -> BTreeMap<String, Vec<String>> {
    let mut fields = BTreeMap::new();
    let mut current_key: Option<String> = None;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if let Some((key, value)) = parse_contract_key_value(item) {
                current_key = Some(key.clone());
                fields.entry(key.clone()).or_insert_with(Vec::new);
                let value = value.trim();
                if !value.is_empty() {
                    fields.entry(key).or_default().push(value.to_string());
                }
                continue;
            }
            let key = current_key.clone().unwrap_or_else(|| "summary".to_string());
            fields
                .entry(key)
                .or_insert_with(Vec::new)
                .push(item.trim().to_string());
            continue;
        }
        if let Some((key, value)) = parse_contract_key_value(trimmed) {
            current_key = Some(key.clone());
            fields.entry(key.clone()).or_insert_with(Vec::new);
            let value = value.trim();
            if !value.is_empty() {
                fields.entry(key).or_default().push(value.to_string());
            }
            continue;
        }
        let key = current_key.clone().unwrap_or_else(|| "summary".to_string());
        fields.entry(key).or_default().push(trimmed.to_string());
    }
    fields
}

fn fallback_contract_section(output: &str, section_name: &str) -> Option<String> {
    let required_keys = match section_name {
        "nagare result" => &["status", "summary", "next_action"][..],
        "nagare review" => &["verdict", "summary", "next_action"][..],
        "nagare workflow decision" => &["action", "reason"][..],
        _ => return None,
    };
    let has_required_keys = required_keys.iter().all(|key| {
        output.lines().any(|line| {
            line.trim()
                .trim_start_matches("- ")
                .split_once(':')
                .is_some_and(|(candidate, _)| normalize_contract_key(candidate) == *key)
        })
    });
    has_required_keys.then(|| output.to_string())
}

fn parse_contract_key_value(line: &str) -> Option<(String, &str)> {
    let (key, value) = line.trim().split_once(':')?;
    let key = normalize_contract_key(key);
    known_contract_key(&key).then_some((key, value))
}

fn normalize_contract_key(key: &str) -> String {
    key.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn known_contract_key(key: &str) -> bool {
    matches!(
        key,
        "status"
            | "summary"
            | "completed"
            | "artifacts"
            | "evidence"
            | "questions"
            | "question"
            | "next_notes"
            | "next_action"
            | "verdict"
            | "findings"
            | "requested_changes"
            | "referenced_artifacts"
            | "criteria"
            | "criteria_results"
            | "action"
            | "reason"
            | "target_agent_profile_id"
            | "requires_human"
            | "confidence"
            | "command_hint"
    )
}

fn has_contract_field(fields: &BTreeMap<String, Vec<String>>, key: &str) -> bool {
    fields
        .get(key)
        .is_some_and(|values| values.iter().any(|value| !value.trim().is_empty()))
}

fn is_empty_contract_value(value: &str) -> bool {
    let normalized = value
        .trim()
        .trim_matches(|ch: char| ch == '-' || ch == ' ' || ch == '　')
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "" | "none" | "no" | "n/a" | "na" | "nil" | "null" | "なし" | "無し" | "ありません"
    )
}

fn has_nested_contract_keys(fields: &BTreeMap<String, Vec<String>>) -> bool {
    let known_keys = [
        "status:",
        "summary:",
        "completed:",
        "artifacts:",
        "evidence:",
        "questions:",
        "next_notes:",
        "next_action:",
        "verdict:",
        "findings:",
        "requested_changes:",
        "referenced_artifacts:",
    ];
    fields.iter().any(|(key, values)| {
        values.iter().any(|value| {
            value.lines().any(|line| {
                let normalized = line.trim().to_ascii_lowercase();
                known_keys.iter().any(|known_key| {
                    normalized.starts_with(known_key) && key != known_key.trim_end_matches(':')
                })
            })
        })
    })
}

fn normalize_next_action(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn valid_next_action(value: &str) -> bool {
    matches!(
        value,
        "review"
            | "run_agent"
            | "answer_question"
            | "needs_input"
            | "create_handoff"
            | "handoff"
            | "recover"
            | "approve"
            | "done"
            | "none"
            | "wait"
            | "stop"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bullet_nested_contract_keys_are_recovered() {
        let contract = default_work_output_contract();
        let output = parse_agent_output_record(AgentOutputRecordInput {
            id: "out_test".to_string(),
            work_item_id: "work_test",
            agent_run_id: "run_test",
            agent_profile_id: "worker",
            purpose: AgentRunPurpose::Work,
            contract: &contract,
            stdout: "## Nagare Result\nsummary:\n- status: succeeded\n- summary: answered the question\n- completed:\n- identified the expensive coffee categories\n- next_notes:\n- reviewer should check the source distinction\n- next_action: review\n",
            execution_record_id: "exec_test",
            locale: "en-US",
            created_at: "1",
        });

        assert_eq!(output.parse_status, AgentOutputParseStatus::Parsed);
        assert_eq!(
            output.fields.get("completed"),
            Some(&vec![
                "identified the expensive coffee categories".to_string()
            ])
        );
        assert_eq!(
            output.fields.get("next_notes"),
            Some(&vec![
                "reviewer should check the source distinction".to_string()
            ])
        );
        assert!(!output.warnings.contains(&"missing_completed".to_string()));
        assert!(!output.warnings.contains(&"missing_next_notes".to_string()));
    }

    #[test]
    fn review_contract_without_heading_can_be_recovered() {
        let contract = default_review_output_contract();
        let output = parse_agent_output_record(AgentOutputRecordInput {
            id: "out_test".to_string(),
            work_item_id: "work_test",
            agent_run_id: "run_test",
            agent_profile_id: "reviewer",
            purpose: AgentRunPurpose::Review,
            contract: &contract,
            stdout: "verdict: pass\nsummary:\n- answer is acceptable\ncompleted:\n- reviewed the answer\nfindings:\n- none\nquestions:\n- none\nnext_notes:\n- ready for approval\nnext_action: approve\n",
            execution_record_id: "exec_test",
            locale: "en-US",
            created_at: "1",
        });

        assert_eq!(output.parse_status, AgentOutputParseStatus::Parsed);
        assert_eq!(
            output.fields.get("verdict"),
            Some(&vec!["pass".to_string()])
        );
        assert_eq!(output.next_action.as_deref(), Some("approve"));
        assert!(
            !output
                .warnings
                .contains(&"output_contract_unparsed".to_string())
        );
    }
}
