use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::*;

pub fn load_ledger(layout: &ProjectLayout) -> Result<Ledger, NagareError> {
    if !layout.ledger_path.exists() {
        return Ok(Ledger::default());
    }
    let raw = fs::read_to_string(&layout.ledger_path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn save_ledger(layout: &ProjectLayout, ledger: &Ledger) -> io::Result<()> {
    fs::create_dir_all(&layout.state_dir)?;
    let raw = serde_json::to_string_pretty(ledger).map_err(io::Error::other)?;
    fs::write(&layout.ledger_path, format!("{raw}\n"))
}

pub fn list_agent_profiles(root: impl Into<PathBuf>) -> Result<Vec<AgentProfile>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_agent_profiles(&layout)?.into_values().collect())
}

pub fn get_agent_profile(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentProfile, NagareError> {
    let layout = ensure_project(root)?;
    get_agent_profile_from_layout(&layout, agent_profile_id)
}

pub fn add_agent_profile(
    root: impl Into<PathBuf>,
    input: AddAgentProfileInput<'_>,
) -> Result<AddAgentProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(input.id)?;

    let mut existing = load_agent_profiles(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "agent profile `{}` already exists",
            input.id
        )));
    }

    let adapter = normalize_adapter_id(input.adapter)?;
    let profile = AgentProfile {
        id: input.id.to_string(),
        display_name: if input.display_name.trim().is_empty() {
            input.id.to_string()
        } else {
            input.display_name.to_string()
        },
        runtime: input.runtime.to_string(),
        adapter: adapter.to_string(),
        role: if input.role.trim().is_empty() {
            "implementer".to_string()
        } else {
            input.role.to_string()
        },
        working_dir: normalize_working_dir(input.working_dir)?,
        description: input.description.trim().to_string(),
        specialties: normalize_specialties(input.specialties),
        output_contracts: AgentOutputContracts::default(),
        source: AgentProfileSource::ProjectAgentDirectory,
    };
    existing.insert(profile.id.clone(), profile.clone());

    let path = write_agent_profile_file(&layout, &profile)?;

    Ok(AddAgentProfileResult { profile, path })
}

pub fn update_agent_profile(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
    input: UpdateAgentProfileInput<'_>,
) -> Result<UpdateAgentProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(agent_profile_id)?;
    let mut profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    if let Some(display_name) = input.display_name {
        profile.display_name = if display_name.trim().is_empty() {
            profile.id.clone()
        } else {
            display_name.trim().to_string()
        };
    }
    if let Some(role) = input.role {
        profile.role = if role.trim().is_empty() {
            "implementer".to_string()
        } else {
            role.trim().to_string()
        };
    }
    if let Some(working_dir) = input.working_dir {
        profile.working_dir = normalize_working_dir(working_dir)?;
    }
    if let Some(description) = input.description {
        profile.description = description.trim().to_string();
    }
    if let Some(specialties) = input.specialties {
        profile.specialties = normalize_specialties(specialties);
    }
    if let Some(update) = input.output_contract {
        apply_output_contract_update(&mut profile.output_contracts, update)?;
    }
    profile.source = AgentProfileSource::ProjectAgentDirectory;
    let path = write_agent_profile_file(&layout, &profile)?;
    Ok(UpdateAgentProfileResult { profile, path })
}

fn write_agent_profile_file(
    layout: &ProjectLayout,
    profile: &AgentProfile,
) -> Result<PathBuf, NagareError> {
    fs::create_dir_all(&layout.agents_dir)?;
    let path = layout.agents_dir.join(format!("{}.toml", profile.id));
    let document = AgentProfileFile {
        agent_profile: Some(AgentProfileFileEntry {
            id: Some(profile.id.clone()),
            display_name: profile.display_name.clone(),
            runtime: profile.runtime.clone(),
            adapter: profile.adapter.clone(),
            role: profile.role.clone(),
            working_dir: profile.working_dir.clone(),
            description: profile.description.clone(),
            specialties: profile.specialties.clone(),
            output_contracts: profile.output_contracts.clone(),
        }),
        agent_profiles: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;
    Ok(path)
}

fn apply_output_contract_update(
    contracts: &mut AgentOutputContracts,
    update: AgentOutputContractUpdate<'_>,
) -> Result<(), NagareError> {
    let purpose = update.purpose.ok_or_else(|| {
        NagareError::InvalidState("output contract purpose is required".to_string())
    })?;
    let target = match purpose {
        AgentOutputContractPurpose::Work => &mut contracts.work,
        AgentOutputContractPurpose::Review => &mut contracts.review,
        AgentOutputContractPurpose::Dispatch => &mut contracts.dispatch,
    };
    if let Some(contract) = update.contract {
        let contract = contract.trim();
        if contract.is_empty() {
            return Err(NagareError::InvalidState(
                "output contract cannot be empty".to_string(),
            ));
        }
        target.contract = contract.to_string();
    }
    if let Some(instruction_pack) = update.instruction_pack {
        let instruction_pack = instruction_pack.trim();
        if instruction_pack.is_empty() {
            return Err(NagareError::InvalidState(
                "output instruction pack cannot be empty".to_string(),
            ));
        }
        target.instruction_pack = instruction_pack.to_string();
    }
    if let Some(required) = update.required {
        target.required = required;
    }
    if let Some(injection) = update.injection {
        target.injection = injection;
    }
    Ok(())
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

pub fn get_nagare_agent_settings(
    root: impl Into<PathBuf>,
) -> Result<NagareAgentSettings, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_project_config(&layout)?.nagare_agents)
}

pub fn set_nagare_agent_settings(
    root: impl Into<PathBuf>,
    input: SetNagareAgentSettingsInput<'_>,
) -> Result<NagareAgentSettings, NagareError> {
    let layout = ensure_project(root)?;
    let mut settings = get_nagare_agent_settings(&layout.root)?;

    if let Some(agent) = input.work_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.work_agent = agent.to_string();
    }
    if let Some(agent) = input.review_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.review_agent = agent.to_string();
    }
    if let Some(agent) = input.dispatch_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.dispatch_agent = agent.to_string();
    }

    write_nagare_agent_settings(&layout, &settings)?;
    Ok(settings)
}

pub fn get_locale_settings(root: impl Into<PathBuf>) -> Result<LocaleSettings, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_project_config(&layout)?.locale)
}

pub fn resolve_rule_for_path(
    root: impl Into<PathBuf>,
    path: Option<&str>,
    agent_override: Option<&str>,
) -> Result<RuleResolution, NagareError> {
    let layout = ensure_project(root)?;
    resolve_rule_for_path_from_layout(&layout, path, agent_override)
}

pub fn set_locale_settings(
    root: impl Into<PathBuf>,
    input: SetLocaleInput<'_>,
) -> Result<LocaleSettings, NagareError> {
    let layout = ensure_project(root)?;
    let mut settings = get_locale_settings(&layout.root)?;
    if let Some(language) = input.language {
        validate_locale_language(language)?;
        settings.language = language.to_string();
    }
    if let Some(timezone) = input.timezone {
        validate_timezone(timezone)?;
        settings.timezone = timezone.to_string();
    }
    write_locale_settings(&layout, &settings)?;
    Ok(settings)
}

pub fn agent_doctor(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentDoctorReport, NagareError> {
    let layout = ensure_project(root)?;
    let profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    let runtime = get_runtime_declaration(&layout, &profile.runtime)?;
    let health = runtime_healthcheck(&runtime);
    Ok(AgentDoctorReport {
        profile,
        runtime,
        health,
    })
}

pub fn agent_probe(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentProbeResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    let mut ledger = load_ledger(&layout)?;
    let probe = create_capability_probe(&layout, &mut ledger, &locale, &profile)?;
    ledger.capability_probes.push(probe.clone());
    save_ledger(&layout, &ledger)?;
    Ok(AgentProbeResult { probe })
}

fn create_capability_probe(
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    locale: &str,
    profile: &AgentProfile,
) -> Result<CapabilityProbe, NagareError> {
    let runtime = get_runtime_declaration(layout, &profile.runtime)?;
    let health = runtime_healthcheck(&runtime);
    Ok(CapabilityProbe {
        id: ledger.next_id("probe"),
        agent_profile_id: profile.id.clone(),
        runtime_id: profile.runtime.clone(),
        adapter_id: normalize_adapter_id(&profile.adapter)?.to_string(),
        runtime_version: health.detail.clone(),
        available: health.available,
        discovered_capabilities: capabilities_for_adapter(&profile.adapter)?,
        instruction_sources: instruction_sources(layout),
        supported_skill_modes: skill_modes_for_adapter(&profile.adapter)?,
        warnings: if health.available {
            Vec::new()
        } else {
            vec![format!("runtime healthcheck failed: {}", health.detail)]
        },
        locale: locale.to_string(),
        probed_at: timestamp(),
    })
}

fn ensure_fresh_capability_probe(
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    locale: &str,
    profile: &AgentProfile,
) -> Result<CapabilityProbe, NagareError> {
    let runtime = get_runtime_declaration(layout, &profile.runtime)?;
    let health = runtime_healthcheck(&runtime);
    let now = timestamp_seconds();
    if let Some(probe) = latest_capability_probe(ledger, &profile.id) {
        if is_capability_probe_fresh(probe, profile, &health.detail, now)? {
            return Ok(probe.clone());
        }
    }
    let probe = CapabilityProbe {
        id: ledger.next_id("probe"),
        agent_profile_id: profile.id.clone(),
        runtime_id: profile.runtime.clone(),
        adapter_id: normalize_adapter_id(&profile.adapter)?.to_string(),
        runtime_version: health.detail.clone(),
        available: health.available,
        discovered_capabilities: capabilities_for_adapter(&profile.adapter)?,
        instruction_sources: instruction_sources(layout),
        supported_skill_modes: skill_modes_for_adapter(&profile.adapter)?,
        warnings: if health.available {
            Vec::new()
        } else {
            vec![format!("runtime healthcheck failed: {}", health.detail)]
        },
        locale: locale.to_string(),
        probed_at: timestamp(),
    };
    ledger.capability_probes.push(probe.clone());
    Ok(probe)
}

pub fn create_work_item(
    root: impl Into<PathBuf>,
    title: impl Into<String>,
    description: impl Into<String>,
) -> Result<CreateItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let now = timestamp();
    let item = WorkItem {
        id: ledger.next_id("work"),
        title: title.into(),
        description: description.into(),
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

pub fn run_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    agent_profile_id: &str,
    command: &str,
) -> Result<RunWorkItemResult, NagareError> {
    run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id,
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(command),
            purpose: AgentRunPurpose::Work,
        },
    )
}

pub fn run_work_item_with_input(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: RunWorkItemInput<'_>,
) -> Result<RunWorkItemResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();

    if input.purpose == AgentRunPurpose::Work {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::AgentRunning;
        item.updated_at = timestamp();
    }

    let run_id = ledger.next_id("run");
    let artifact_id = ledger.next_id("art");
    let evidence_id = ledger.next_id("ev");
    let run_packet_id = ledger.next_id("runpkt");
    let skill_context_id = ledger.next_id("skillctx");
    let dispatch_plan_id = if input.purpose == AgentRunPurpose::DispatchPreview {
        Some(ledger.next_id("dispatch"))
    } else {
        None
    };
    let agent_output_id = if input.purpose == AgentRunPurpose::DispatchPreview {
        None
    } else {
        Some(ledger.next_id("out"))
    };
    let review_result_id = if input.purpose == AgentRunPurpose::Review {
        Some(ledger.next_id("review"))
    } else {
        None
    };
    let agent_profile = get_agent_profile_from_layout(&layout, input.agent_profile_id)?;
    let adapter_id = normalize_adapter_id(&agent_profile.adapter)?;
    let working_dir = resolve_profile_working_dir(&layout, &agent_profile)?;
    let output_contract = agent_profile
        .output_contracts
        .for_purpose(input.purpose)
        .clone();
    let rule_resolution =
        resolve_rule_for_path_from_layout(&layout, input.path, Some(input.agent_profile_id))?;
    let dispatch_target_resolution = if input.purpose == AgentRunPurpose::DispatchPreview {
        Some(resolve_rule_for_path_from_layout(
            &layout, input.path, None,
        )?)
    } else {
        None
    };
    let capability_probe =
        ensure_fresh_capability_probe(&layout, &mut ledger, &locale, &agent_profile)?;
    let capabilities_in_force = capability_probe.discovered_capabilities.clone();
    let skill_set_resolution = resolve_skill_sets_for_run(
        &layout,
        &rule_resolution.skill_set_ids,
        &capabilities_in_force,
    )?;
    let instruction_sources = capability_probe.instruction_sources.clone();
    let goal = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(&item.title)
        .to_string();
    let human_feedback_context = human_feedback_prompt_context(&ledger, work_item_id);
    let resolved_skill_context = ResolvedSkillContext {
        id: skill_context_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        capability_probe_id: Some(capability_probe.id.clone()),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        declared_skill_set_ids: skill_set_resolution.declared_skill_set_ids.clone(),
        applied_skill_set_ids: skill_set_resolution.applied_skill_set_ids.clone(),
        skipped_skill_set_ids: skill_set_resolution.skipped_skill_set_ids.clone(),
        capabilities_in_force,
        instruction_sources,
        artifact_uri: path_uri(
            &layout
                .artifacts_dir
                .join(format!("{skill_context_id}.json")),
        ),
        content_hash: format!("local:{}", skill_context_id),
        locale: locale.clone(),
        resolved_at: timestamp(),
    };
    let resolved_run_packet = ResolvedRunPacket {
        id: run_packet_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        adapter_id: adapter_id.to_string(),
        purpose: input.purpose,
        working_dir: path_uri(&working_dir),
        goal: goal.clone(),
        path: rule_resolution.path.clone(),
        dispatch_plan_id: input.dispatch_plan_id.map(ToOwned::to_owned),
        permission_policy_id: rule_resolution.permission_policy_id.clone(),
        workspace_policy_id: rule_resolution.workspace_policy_id.clone(),
        resolved_skill_context_id: skill_context_id.clone(),
        output_contract: output_contract.clone(),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        verification: rule_resolution.verification.clone(),
        constraints: rule_resolution
            .warnings
            .iter()
            .chain(skill_set_resolution.warnings.iter())
            .cloned()
            .chain(
                (!human_feedback_context.is_empty())
                    .then(|| "human_feedback_context_applied".to_string()),
            )
            .collect(),
        artifact_uri: path_uri(&layout.artifacts_dir.join(format!("{run_packet_id}.json"))),
        content_hash: format!("local:{}", run_packet_id),
        locale: locale.clone(),
        created_at: timestamp(),
    };
    let prompt = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(goal.as_str());
    let prompt = prompt_with_output_contract(
        &prompt_with_human_feedback(prompt, &human_feedback_context),
        input.purpose,
        &output_contract,
    );
    let request = AdapterRunRequest {
        working_dir: &working_dir,
        run_packet: &resolved_run_packet,
        prompt: &prompt,
        dev_command: input.dev_command,
    };
    let started_at = timestamp();
    let output = adapter_for_id(adapter_id)?.run(&request)?;
    let ended_at = timestamp();
    let status = if output.exit_code == Some(0) {
        AgentRunStatus::Succeeded
    } else {
        AgentRunStatus::Failed
    };

    let log_path = layout.logs_dir.join(format!("{run_id}.log"));
    write_adapter_log(&log_path, &resolved_run_packet, &output)?;

    let artifact = Artifact {
        id: artifact_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_run_id: Some(run_id.clone()),
        artifact_type: "run_log".to_string(),
        uri: path_uri(&log_path),
        title: format!("{} {} log", input.agent_profile_id, input.purpose),
        locale: locale.clone(),
        created_at: ended_at.clone(),
    };
    let evidence = Evidence {
        id: evidence_id.clone(),
        work_item_id: work_item_id.to_string(),
        claim: agent_run_claim(&locale, input.purpose, status, input.agent_profile_id),
        basis: command_exit_basis(&locale, output.exit_code),
        artifact_id: Some(artifact_id.clone()),
        produced_by: input.agent_profile_id.to_string(),
        locale: locale.clone(),
        created_at: ended_at.clone(),
    };
    let collected_artifacts = collect_git_run_artifacts(
        &layout,
        &mut ledger,
        work_item_id,
        &run_id,
        &locale,
        &ended_at,
    )?;
    let run = AgentRun {
        id: run_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_profile_id: input.agent_profile_id.to_string(),
        adapter: adapter_id.to_string(),
        purpose: input.purpose,
        command: output.command,
        status,
        exit_code: output.exit_code,
        started_at,
        ended_at: ended_at.clone(),
        artifact_id: artifact_id.clone(),
        locale: locale.clone(),
    };
    let agent_output = agent_output_id.map(|id| {
        parse_agent_output_record(AgentOutputRecordInput {
            id,
            work_item_id,
            agent_run_id: &run_id,
            agent_profile_id: input.agent_profile_id,
            purpose: input.purpose,
            contract: &output_contract,
            stdout: &output.stdout,
            artifact_id: Some(&artifact_id),
            locale: &locale,
            created_at: &ended_at,
        })
    });
    let review_result = review_result_id
        .zip(agent_output.as_ref())
        .map(|(id, output)| review_result_from_agent_output(id, output));
    let item_status = if input.purpose == AgentRunPurpose::Work {
        if agent_output_requires_input(agent_output.as_ref()) {
            WorkItemStatus::NeedsInput
        } else if agent_output_requests_handoff(agent_output.as_ref()) {
            WorkItemStatus::NeedsHandoff
        } else if status == AgentRunStatus::Succeeded {
            WorkItemStatus::ReadyForReview
        } else {
            WorkItemStatus::FailedVerification
        }
    } else if input.purpose == AgentRunPurpose::Review {
        review_result
            .as_ref()
            .map(|review| review_work_item_status(review, item.status))
            .unwrap_or(item.status)
    } else {
        item.status
    };
    let dispatch_suggestion = parse_dispatch_plan_suggestion(&output.stdout);
    let valid_dispatch_targets = if dispatch_plan_id.is_some() {
        load_agent_profiles(&layout)?
    } else {
        BTreeMap::new()
    };
    let dispatch_plan = dispatch_plan_id.map(|id| {
        let fallback_target_agent_profile_id = dispatch_target_resolution
            .as_ref()
            .map(|resolution| resolution.agent_profile_id.clone())
            .unwrap_or_else(|| input.agent_profile_id.to_string());
        let mut selection_warnings = Vec::new();
        let target_agent_profile_id = match dispatch_suggestion
            .as_ref()
            .and_then(|suggestion| suggestion.target_agent_profile_id.as_deref())
        {
            Some(target) if valid_dispatch_targets.contains_key(target) => target.to_string(),
            Some(target) => {
                selection_warnings.push(format!(
                    "dispatch output target_agent_profile_id `{target}` is not registered; used fallback target `{fallback_target_agent_profile_id}`"
                ));
                fallback_target_agent_profile_id
            }
            None => {
                if dispatch_suggestion.is_some() {
                    selection_warnings.push(format!(
                        "dispatch output missing required target_agent_profile_id; used fallback target `{fallback_target_agent_profile_id}`"
                    ));
                } else {
                    selection_warnings.push(format!(
                        "dispatch output was not valid dispatch JSON; used fallback target `{fallback_target_agent_profile_id}`"
                    ));
                }
                fallback_target_agent_profile_id
            }
        };
        let summary = dispatch_suggestion
            .as_ref()
            .and_then(|suggestion| suggestion.summary.clone())
            .unwrap_or_else(|| {
                selection_warnings.push(
                    "dispatch output missing summary; summarized raw output".to_string(),
                );
                summarize_dispatch_output(&output.stdout)
            });
        let risks = dispatch_suggestion
            .as_ref()
            .map(|suggestion| suggestion.risks.clone())
            .filter(|risks| !risks.is_empty())
            .unwrap_or_else(|| extract_prefixed_lines(&output.stdout, "risk:"));
        let missing_information = dispatch_suggestion
            .as_ref()
            .map(|suggestion| suggestion.missing_information.clone())
            .filter(|missing_information| !missing_information.is_empty())
            .unwrap_or_else(|| extract_prefixed_lines(&output.stdout, "missing:"));
        DispatchPlan {
            id,
            work_item_id: work_item_id.to_string(),
            status: DispatchPlanStatus::Draft,
            agent_run_id: run.id.clone(),
            dispatch_agent_profile_id: input.agent_profile_id.to_string(),
            target_agent_profile_id,
            resolved_run_packet_id: resolved_run_packet.id.clone(),
            raw_output_artifact_id: artifact_id.clone(),
            path: rule_resolution.path.clone(),
            summary,
            risks,
            missing_information,
            selection_warnings,
            locale: locale.clone(),
            created_at: ended_at.clone(),
        }
    });

    ledger.runs.push(run.clone());
    ledger.artifacts.push(artifact);
    ledger.artifacts.extend(collected_artifacts);
    ledger.evidence.push(evidence);
    if let Some(record) = agent_output {
        ledger.agent_outputs.push(record);
    }
    if let Some(review) = review_result {
        ledger.review_results.push(review);
    }
    let dispatch_plan_id = dispatch_plan.as_ref().map(|plan| plan.id.clone());
    if let Some(plan) = dispatch_plan {
        for existing in &mut ledger.dispatch_plans {
            if existing.work_item_id == work_item_id && existing.status == DispatchPlanStatus::Draft
            {
                existing.status = DispatchPlanStatus::Superseded;
            }
        }
        ledger.dispatch_plans.push(plan);
    }
    ledger
        .resolved_skill_contexts
        .push(resolved_skill_context.clone());
    ledger
        .resolved_run_packets
        .push(resolved_run_packet.clone());
    write_json_artifact(
        &layout,
        &format!("{}.json", resolved_skill_context.id),
        &resolved_skill_context,
    )?;
    write_json_artifact(
        &layout,
        &format!("{}.json", resolved_run_packet.id),
        &resolved_run_packet,
    )?;
    if matches!(
        input.purpose,
        AgentRunPurpose::Work | AgentRunPurpose::Review
    ) {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = item_status;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;

    Ok(RunWorkItemResult {
        run,
        evidence_id,
        item_status,
        dispatch_plan_id,
    })
}

pub fn verify_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    command: &str,
) -> Result<VerifyResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let _ = ledger.work_item(work_item_id)?;

    let artifact_id = ledger.next_id("art");
    let evidence_id = ledger.next_id("ev");
    let verification_id = ledger.next_id("ver");
    let output = run_shell(command)?;
    let verified_at = timestamp();
    let log_path = layout.logs_dir.join(format!("{verification_id}.log"));
    write_command_log(&log_path, command, &output)?;

    let result = if output.exit_code == Some(0) {
        VerificationStatus::Passed
    } else {
        VerificationStatus::Failed
    };
    let item_status = if result == VerificationStatus::Passed {
        WorkItemStatus::ReadyForReview
    } else {
        WorkItemStatus::FailedVerification
    };
    let artifact = Artifact {
        id: artifact_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_run_id: None,
        artifact_type: "verification_log".to_string(),
        uri: path_uri(&log_path),
        title: localized_text(&locale, "verification log", "検証ログ").to_string(),
        locale: locale.clone(),
        created_at: verified_at.clone(),
    };
    let evidence = Evidence {
        id: evidence_id.clone(),
        work_item_id: work_item_id.to_string(),
        claim: verification_claim(&locale, result),
        basis: verification_basis(&locale, command, output.exit_code),
        artifact_id: Some(artifact_id.clone()),
        produced_by: "verification".to_string(),
        locale: locale.clone(),
        created_at: verified_at.clone(),
    };
    let verification = VerificationResult {
        id: verification_id,
        work_item_id: work_item_id.to_string(),
        command: command.to_string(),
        result,
        evidence_id,
        artifact_id,
        locale,
        verified_at,
    };

    ledger.artifacts.push(artifact);
    ledger.evidence.push(evidence);
    ledger.verification_results.push(verification.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = item_status;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;

    Ok(VerifyResult {
        verification,
        item_status,
    })
}

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
    let _ = ledger.work_item(work_item_id)?;
    let handoff = HandoffPacket {
        id: ledger.next_id("handoff"),
        work_item_id: work_item_id.to_string(),
        from_agent_profile: from_agent_profile.to_string(),
        to_agent_profile: to_agent_profile.to_string(),
        reason: reason.to_string(),
        summary: summary.to_string(),
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

pub fn approve_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    rationale: &str,
) -> Result<DecisionResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?;
    if item.status != WorkItemStatus::ReadyForReview {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` must be ready_for_review before approval; current status is {}",
            item.status
        )));
    }
    let has_passing_verification = ledger.verification_results.iter().any(|verification| {
        verification.work_item_id == work_item_id
            && verification.result == VerificationStatus::Passed
    });
    if !has_passing_verification {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` needs a passing verification before approval"
        )));
    }

    let decision = HumanDecision {
        id: ledger.next_id("dec"),
        work_item_id: work_item_id.to_string(),
        decision_type: "approve".to_string(),
        rationale: if rationale.trim().is_empty() {
            default_approval_rationale(&locale).to_string()
        } else {
            rationale.to_string()
        },
        locale,
        created_at: timestamp(),
    };
    ledger.decisions.push(decision.clone());
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::Done;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;
    Ok(DecisionResult {
        decision,
        item_status: WorkItemStatus::Done,
    })
}
