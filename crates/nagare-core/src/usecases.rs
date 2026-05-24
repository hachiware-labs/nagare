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
        source: AgentProfileSource::ProjectAgentDirectory,
    };
    existing.insert(profile.id.clone(), profile.clone());

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
        }),
        agent_profiles: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;

    Ok(AddAgentProfileResult { profile, path })
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
    let agent_profile = get_agent_profile_from_layout(&layout, input.agent_profile_id)?;
    let adapter_id = normalize_adapter_id(&agent_profile.adapter)?;
    let working_dir = resolve_profile_working_dir(&layout, &agent_profile)?;
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
        permission_policy_id: rule_resolution.permission_policy_id.clone(),
        workspace_policy_id: rule_resolution.workspace_policy_id.clone(),
        resolved_skill_context_id: skill_context_id.clone(),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        verification: rule_resolution.verification.clone(),
        constraints: rule_resolution
            .warnings
            .iter()
            .chain(skill_set_resolution.warnings.iter())
            .cloned()
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
    let request = AdapterRunRequest {
        working_dir: &working_dir,
        run_packet: &resolved_run_packet,
        prompt,
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
    let run = AgentRun {
        id: run_id,
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
    let item_status = if input.purpose == AgentRunPurpose::Work {
        if status == AgentRunStatus::Succeeded {
            WorkItemStatus::ReadyForReview
        } else {
            WorkItemStatus::FailedVerification
        }
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
        let target_agent_profile_id = dispatch_suggestion
            .as_ref()
            .and_then(|suggestion| suggestion.target_agent_profile_id.as_deref())
            .filter(|target| valid_dispatch_targets.contains_key(*target))
            .map(ToOwned::to_owned)
            .unwrap_or(fallback_target_agent_profile_id);
        let summary = dispatch_suggestion
            .as_ref()
            .and_then(|suggestion| suggestion.summary.clone())
            .unwrap_or_else(|| summarize_dispatch_output(&output.stdout));
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
            agent_run_id: run.id.clone(),
            dispatch_agent_profile_id: input.agent_profile_id.to_string(),
            target_agent_profile_id,
            resolved_run_packet_id: resolved_run_packet.id.clone(),
            raw_output_artifact_id: artifact_id.clone(),
            path: rule_resolution.path.clone(),
            summary,
            risks,
            missing_information,
            locale: locale.clone(),
            created_at: ended_at.clone(),
        }
    });

    ledger.runs.push(run.clone());
    ledger.artifacts.push(artifact);
    ledger.evidence.push(evidence);
    let dispatch_plan_id = dispatch_plan.as_ref().map(|plan| plan.id.clone());
    if let Some(plan) = dispatch_plan {
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
    if input.purpose == AgentRunPurpose::Work {
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
