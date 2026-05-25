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
    pub(crate) artifact_id: Option<&'a str>,
    pub(crate) locale: &'a str,
    pub(crate) created_at: &'a str,
}

pub(crate) fn parse_agent_output_record(input: AgentOutputRecordInput<'_>) -> AgentOutputRecord {
    let mut warnings = Vec::new();
    let section_name = match input.purpose {
        AgentRunPurpose::Work => "nagare result",
        AgentRunPurpose::Review => "nagare review",
        AgentRunPurpose::DispatchPreview => "nagare dispatch",
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
        .unwrap_or_default();
    let next_action = fields
        .get("next_action")
        .and_then(|values| values.first())
        .map(|value| normalize_next_action(value));
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
        artifact_id: input.artifact_id.map(ToOwned::to_owned),
        locale: input.locale.to_string(),
        created_at: input.created_at.to_string(),
    }
}

pub(crate) fn agent_output_requires_input(record: Option<&AgentOutputRecord>) -> bool {
    record.is_some_and(|record| {
        !record.questions.is_empty()
            || record
                .next_action
                .as_deref()
                .is_some_and(|action| action == "answer_question" || action == "needs_input")
    })
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
) -> String {
    if contract.injection != AgentOutputInjection::PromptSuffix {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n{}",
        output_contract_instruction(purpose, contract)
    )
}

pub(crate) fn prompt_with_human_feedback(prompt: &str, context: &str) -> String {
    if context.trim().is_empty() {
        return prompt.to_string();
    }
    format!("{prompt}\n\n## Nagare Human Feedback\n{context}")
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
    capture.then(|| lines.join("\n"))
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
            let key = current_key.clone().unwrap_or_else(|| "summary".to_string());
            fields
                .entry(key)
                .or_insert_with(Vec::new)
                .push(item.trim().to_string());
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = normalize_contract_key(key);
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

fn normalize_contract_key(key: &str) -> String {
    key.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn normalize_next_action(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn output_contract_instruction(purpose: AgentRunPurpose, contract: &AgentOutputContract) -> String {
    let required = if contract.required {
        "This final block is required."
    } else {
        "Include this final block when possible."
    };
    match purpose {
        AgentRunPurpose::DispatchPreview => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\nReturn one JSON object only with keys: target_agent_profile_id, summary, risks, missing_information. target_agent_profile_id must exactly match a registered candidate agent profile id.",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
        AgentRunPurpose::Review => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\nFinish with a Markdown section named `## Nagare Review` containing: verdict, summary, findings, referenced_artifacts, requested_changes, questions, next_action.",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
        AgentRunPurpose::Work => format!(
            "Nagare output contract: {contract_id}\nInstruction pack: {pack}\n{required}\nFinish with a Markdown section named `## Nagare Result` containing: status, summary, artifacts, evidence, questions, verification, next_action.",
            contract_id = contract.contract,
            pack = contract.instruction_pack,
        ),
    }
}
