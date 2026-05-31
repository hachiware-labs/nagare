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

pub fn list_domain_profiles(root: impl Into<PathBuf>) -> Result<Vec<DomainProfile>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_domain_profiles(&layout)?.into_values().collect())
}

pub fn list_domain_groups(root: impl Into<PathBuf>) -> Result<Vec<DomainGroup>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_domain_groups(&layout)?.into_values().collect())
}

pub fn get_domain_profile(
    root: impl Into<PathBuf>,
    domain_profile_id: &str,
) -> Result<DomainProfile, NagareError> {
    let layout = ensure_project(root)?;
    load_domain_profiles(&layout)?
        .remove(domain_profile_id)
        .ok_or_else(|| NagareError::NotFound(format!("domain profile `{domain_profile_id}`")))
}

pub fn get_domain_group(
    root: impl Into<PathBuf>,
    domain_group_id: &str,
) -> Result<DomainGroup, NagareError> {
    let layout = ensure_project(root)?;
    load_domain_groups(&layout)?
        .remove(domain_group_id)
        .ok_or_else(|| NagareError::NotFound(format!("domain group `{domain_group_id}`")))
}

pub fn get_workflow_settings(root: impl Into<PathBuf>) -> Result<WorkflowSettings, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_project_config(&layout)?.workflow)
}

pub fn set_workflow_settings(
    root: impl Into<PathBuf>,
    settings: WorkflowSettings,
) -> Result<WorkflowSettings, NagareError> {
    let layout = ensure_project(root)?;
    save_workflow_settings(&layout, settings)?;
    Ok(settings)
}

pub fn add_domain_profile(
    root: impl Into<PathBuf>,
    input: AddDomainProfileInput<'_>,
) -> Result<AddDomainProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_profile_id(input.id)?;
    let existing = load_domain_profiles(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "domain profile `{}` already exists",
            input.id
        )));
    }
    let group_id = input
        .group_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if let Some(group_id) = group_id.as_deref() {
        validate_existing_domain_group(&layout, group_id)?;
    }
    let domain = DomainProfile {
        id: input.id.to_string(),
        group_id,
        display_name: if input.display_name.trim().is_empty() {
            input.id.to_string()
        } else {
            input.display_name.trim().to_string()
        },
        description: input.description.trim().to_string(),
        artifact_types: normalize_specialties(input.artifact_types),
        rubric: normalize_specialties(input.rubric),
        dispatch_hints: normalize_specialties(input.dispatch_hints),
        workflow: input.workflow,
        source: DomainProfileSource::ProjectDomainDirectory,
    };
    let path = write_domain_profile_file(&layout, &domain)?;
    Ok(AddDomainProfileResult { domain, path })
}

pub fn update_domain_profile(
    root: impl Into<PathBuf>,
    domain_profile_id: &str,
    input: UpdateDomainProfileInput<'_>,
) -> Result<UpdateDomainProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_profile_id(domain_profile_id)?;
    let mut domain = get_domain_profile(&layout.root, domain_profile_id)?;
    if let Some(group_id) = input.group_id {
        let group_id = group_id.map(str::trim).filter(|value| !value.is_empty());
        if let Some(group_id) = group_id {
            validate_existing_domain_group(&layout, group_id)?;
        }
        domain.group_id = group_id.map(ToOwned::to_owned);
    }
    if let Some(display_name) = input.display_name {
        domain.display_name = if display_name.trim().is_empty() {
            domain.id.clone()
        } else {
            display_name.trim().to_string()
        };
    }
    if let Some(description) = input.description {
        domain.description = description.trim().to_string();
    }
    if let Some(artifact_types) = input.artifact_types {
        domain.artifact_types = normalize_specialties(artifact_types);
    }
    if let Some(rubric) = input.rubric {
        domain.rubric = normalize_specialties(rubric);
    }
    if let Some(dispatch_hints) = input.dispatch_hints {
        domain.dispatch_hints = normalize_specialties(dispatch_hints);
    }
    if let Some(workflow) = input.workflow {
        domain.workflow = workflow;
    }
    domain.source = DomainProfileSource::ProjectDomainDirectory;
    let path = write_domain_profile_file(&layout, &domain)?;
    Ok(UpdateDomainProfileResult { domain, path })
}

pub fn delete_domain_profile(
    root: impl Into<PathBuf>,
    domain_profile_id: &str,
) -> Result<DomainProfile, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_profile_id(domain_profile_id)?;
    let domain = get_domain_profile(&layout.root, domain_profile_id)?;
    if domain.source != DomainProfileSource::ProjectDomainDirectory {
        return Err(NagareError::InvalidState(format!(
            "domain profile `{domain_profile_id}` is not project-local and cannot be deleted"
        )));
    }
    let path = layout.domains_dir.join(format!("{domain_profile_id}.toml"));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(domain)
}

pub fn add_domain_group(
    root: impl Into<PathBuf>,
    input: AddDomainGroupInput<'_>,
) -> Result<AddDomainGroupResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_group_id(input.id)?;
    let existing = load_domain_groups(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "domain group `{}` already exists",
            input.id
        )));
    }
    let group = DomainGroup {
        id: input.id.to_string(),
        display_name: if input.display_name.trim().is_empty() {
            input.id.to_string()
        } else {
            input.display_name.trim().to_string()
        },
        description: input.description.trim().to_string(),
        shared_knowledge: normalize_specialties(input.shared_knowledge),
        common_rubric: normalize_specialties(input.common_rubric),
        dispatch_hints: normalize_specialties(input.dispatch_hints),
        workflow: input.workflow,
        source: DomainGroupSource::ProjectDomainGroupDirectory,
    };
    let path = write_domain_group_file(&layout, &group)?;
    Ok(AddDomainGroupResult { group, path })
}

pub fn update_domain_group(
    root: impl Into<PathBuf>,
    domain_group_id: &str,
    input: UpdateDomainGroupInput<'_>,
) -> Result<UpdateDomainGroupResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_group_id(domain_group_id)?;
    let mut group = get_domain_group(&layout.root, domain_group_id)?;
    if let Some(display_name) = input.display_name {
        group.display_name = if display_name.trim().is_empty() {
            group.id.clone()
        } else {
            display_name.trim().to_string()
        };
    }
    if let Some(description) = input.description {
        group.description = description.trim().to_string();
    }
    if let Some(shared_knowledge) = input.shared_knowledge {
        group.shared_knowledge = normalize_specialties(shared_knowledge);
    }
    if let Some(common_rubric) = input.common_rubric {
        group.common_rubric = normalize_specialties(common_rubric);
    }
    if let Some(dispatch_hints) = input.dispatch_hints {
        group.dispatch_hints = normalize_specialties(dispatch_hints);
    }
    if let Some(workflow) = input.workflow {
        group.workflow = workflow;
    }
    group.source = DomainGroupSource::ProjectDomainGroupDirectory;
    let path = write_domain_group_file(&layout, &group)?;
    Ok(UpdateDomainGroupResult { group, path })
}

pub fn delete_domain_group(
    root: impl Into<PathBuf>,
    domain_group_id: &str,
) -> Result<DomainGroup, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_group_id(domain_group_id)?;
    let group = get_domain_group(&layout.root, domain_group_id)?;
    if group.source != DomainGroupSource::ProjectDomainGroupDirectory {
        return Err(NagareError::InvalidState(format!(
            "domain group `{domain_group_id}` is not project-local and cannot be deleted"
        )));
    }
    let domains = load_domain_profiles(&layout)?;
    if domains
        .values()
        .any(|domain| domain.group_id.as_deref() == Some(domain_group_id))
    {
        return Err(NagareError::InvalidState(format!(
            "domain group `{domain_group_id}` still has domains"
        )));
    }
    let path = layout
        .domain_groups_dir
        .join(format!("{domain_group_id}.toml"));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(group)
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
    let domain_group_ids = normalize_domain_group_ids(input.domain_group_ids)?;
    let domain_ids = normalize_domain_profile_ids(input.domain_ids)?;
    validate_existing_domain_group_ids(&layout, &domain_group_ids)?;
    validate_existing_domain_profile_ids(&layout, &domain_ids)?;
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
        domain_group_ids,
        domain_ids,
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
    if let Some(runtime) = input.runtime {
        profile.runtime = runtime.trim().to_string();
    }
    if let Some(adapter) = input.adapter {
        profile.adapter = normalize_adapter_id(adapter)?.to_string();
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
    if let Some(domain_group_ids) = input.domain_group_ids {
        profile.domain_group_ids = normalize_domain_group_ids(domain_group_ids)?;
        validate_existing_domain_group_ids(&layout, &profile.domain_group_ids)?;
    }
    if let Some(domain_ids) = input.domain_ids {
        profile.domain_ids = normalize_domain_profile_ids(domain_ids)?;
        validate_existing_domain_profile_ids(&layout, &profile.domain_ids)?;
    }
    if let Some(update) = input.output_contract {
        apply_output_contract_update(&mut profile.output_contracts, update)?;
    }
    profile.source = AgentProfileSource::ProjectAgentDirectory;
    let path = write_agent_profile_file(&layout, &profile)?;
    Ok(UpdateAgentProfileResult { profile, path })
}

pub fn delete_agent_profile(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
) -> Result<AgentProfile, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(agent_profile_id)?;
    let profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    if profile.source != AgentProfileSource::ProjectAgentDirectory {
        return Err(NagareError::InvalidState(format!(
            "agent profile `{agent_profile_id}` is not project-local and cannot be deleted"
        )));
    }
    let path = layout.agents_dir.join(format!("{agent_profile_id}.toml"));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(profile)
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
            domain_group_ids: profile.domain_group_ids.clone(),
            domain_ids: profile.domain_ids.clone(),
            output_contracts: profile.output_contracts.clone(),
        }),
        agent_profiles: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;
    Ok(path)
}

fn write_domain_profile_file(
    layout: &ProjectLayout,
    domain: &DomainProfile,
) -> Result<PathBuf, NagareError> {
    fs::create_dir_all(&layout.domains_dir)?;
    let path = layout.domains_dir.join(format!("{}.toml", domain.id));
    let document = DomainProfileFile {
        domain_profile: Some(DomainProfileFileEntry {
            id: Some(domain.id.clone()),
            group_id: domain.group_id.clone(),
            display_name: domain.display_name.clone(),
            description: domain.description.clone(),
            artifact_types: domain.artifact_types.clone(),
            rubric: domain.rubric.clone(),
            dispatch_hints: domain.dispatch_hints.clone(),
            workflow: domain.workflow,
        }),
        domain_profiles: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;
    Ok(path)
}

fn write_domain_group_file(
    layout: &ProjectLayout,
    group: &DomainGroup,
) -> Result<PathBuf, NagareError> {
    fs::create_dir_all(&layout.domain_groups_dir)?;
    let path = layout.domain_groups_dir.join(format!("{}.toml", group.id));
    let document = DomainGroupFile {
        domain_group: Some(DomainGroupFileEntry {
            id: Some(group.id.clone()),
            display_name: group.display_name.clone(),
            description: group.description.clone(),
            shared_knowledge: group.shared_knowledge.clone(),
            common_rubric: group.common_rubric.clone(),
            dispatch_hints: group.dispatch_hints.clone(),
            workflow: group.workflow,
        }),
        domain_groups: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;
    Ok(path)
}

fn prompt_with_agent_instructions(prompt: &str, instructions: &str) -> String {
    let instructions = instructions.trim();
    if instructions.is_empty() {
        return prompt.to_string();
    }
    format!("{prompt}\n\n## Nagare Agent Instructions\n{instructions}")
}

fn prompt_with_domain_context(prompt: &str, context: &str) -> String {
    let context = context.trim();
    if context.is_empty() {
        return prompt.to_string();
    }
    format!("{prompt}\n\n## Nagare Domain Context\n{context}")
}

fn domain_prompt_context(layout: &ProjectLayout, item: &WorkItem) -> Result<String, NagareError> {
    let domain_groups = load_domain_groups(layout)?;
    let domains = load_domain_profiles(layout)?;
    let domain = item
        .domain_id
        .as_deref()
        .and_then(|domain_id| domains.get(domain_id));
    let domain_group_id = item
        .domain_group_id
        .as_deref()
        .or_else(|| domain.and_then(|domain| domain.group_id.as_deref()));
    let domain_group = domain_group_id.and_then(|group_id| domain_groups.get(group_id));
    if domain.is_none() && domain_group.is_none() {
        return Ok(String::new());
    }
    let mut lines = Vec::new();
    if let Some(group) = domain_group {
        lines.push(format!(
            "Domain group: {} ({})",
            group.display_name, group.id
        ));
        if !group.description.trim().is_empty() {
            lines.push(format!("Group description: {}", group.description));
        }
        append_context_list(&mut lines, "Shared knowledge", &group.shared_knowledge);
        append_context_list(&mut lines, "Common rubric", &group.common_rubric);
        append_context_list(&mut lines, "Group dispatch hints", &group.dispatch_hints);
    }
    if let Some(domain) = domain {
        lines.push(format!("Domain: {} ({})", domain.display_name, domain.id));
        if !domain.description.trim().is_empty() {
            lines.push(format!("Domain description: {}", domain.description));
        }
        append_context_list(&mut lines, "Artifact types", &domain.artifact_types);
        append_context_list(&mut lines, "Domain rubric", &domain.rubric);
        append_context_list(&mut lines, "Domain dispatch hints", &domain.dispatch_hints);
    }
    Ok(lines.join("\n"))
}

fn append_context_list(lines: &mut Vec<String>, label: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    lines.extend(values.iter().map(|value| format!("- {value}")));
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
        AgentOutputContractPurpose::Supervision => &mut contracts.supervision,
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
    if let Some(agent) = input.supervisor_agent {
        validate_existing_agent_profile(&layout, agent)?;
        settings.supervisor_agent = agent.to_string();
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
    let project_config = load_project_config(&layout)?;
    let locale = project_config.locale.language.clone();
    let agent_settings = project_config.nagare_agents;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let effective_path = input.path.or(item.work_folder.as_deref());

    if input.purpose == AgentRunPurpose::Work {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::AgentRunning;
        item.updated_at = timestamp();
    }

    let run_id = ledger.next_id("run");
    let execution_record_id = ledger.next_id("exec");
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
        resolve_rule_for_path_from_layout(&layout, effective_path, Some(input.agent_profile_id))?;
    let dispatch_target_resolution = if input.purpose == AgentRunPurpose::DispatchPreview {
        Some(resolve_rule_for_path_from_layout(
            &layout,
            effective_path,
            None,
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
    let default_goal = work_item_goal_prompt(&item);
    let goal = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(default_goal.as_str())
        .to_string();
    let human_feedback_context = human_feedback_prompt_context(&ledger, work_item_id);
    let handoff_context = handoff_prompt_context(&ledger, work_item_id);
    let domain_context = domain_prompt_context(&layout, &item)?;
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
        execution_record_uri: path_uri(&layout.logs_dir.join(format!("{skill_context_id}.json"))),
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
        work_folder: item.work_folder.clone(),
        dispatch_plan_id: input.dispatch_plan_id.map(ToOwned::to_owned),
        permission_policy_id: rule_resolution.permission_policy_id.clone(),
        workspace_policy_id: rule_resolution.workspace_policy_id.clone(),
        resolved_skill_context_id: skill_context_id.clone(),
        output_contract: output_contract.clone(),
        project_rule_ids: rule_resolution.matched_rule_id.iter().cloned().collect(),
        constraints: rule_resolution
            .warnings
            .iter()
            .chain(skill_set_resolution.warnings.iter())
            .cloned()
            .chain(
                (!human_feedback_context.is_empty())
                    .then(|| "human_feedback_context_applied".to_string()),
            )
            .chain((!handoff_context.is_empty()).then(|| "handoff_context_applied".to_string()))
            .chain((!domain_context.is_empty()).then(|| "domain_context_applied".to_string()))
            .chain(
                (!item.acceptance_criteria.is_empty())
                    .then(|| "acceptance_criteria_context_applied".to_string()),
            )
            .collect(),
        execution_record_uri: path_uri(&layout.logs_dir.join(format!("{run_packet_id}.json"))),
        content_hash: format!("local:{}", run_packet_id),
        locale: locale.clone(),
        created_at: timestamp(),
    };
    let prompt = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(goal.as_str());
    let prompt = prompt_with_output_contract(
        &prompt_with_agent_instructions(
            &prompt_with_domain_context(
                &prompt_with_handoff_context(
                    &prompt_with_human_feedback(prompt, &human_feedback_context),
                    &handoff_context,
                ),
                &domain_context,
            ),
            &agent_profile.description,
        ),
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

    let execution_record = ExecutionRecord {
        id: execution_record_id.clone(),
        work_item_id: work_item_id.to_string(),
        agent_run_id: Some(run_id.clone()),
        record_type: "run_log".to_string(),
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
        artifact_id: None,
        execution_record_id: Some(execution_record_id.clone()),
        produced_by: input.agent_profile_id.to_string(),
        locale: locale.clone(),
        created_at: ended_at.clone(),
    };
    let collected_execution_records = collect_git_execution_records(
        &layout,
        &mut ledger,
        work_item_id,
        &run_id,
        &locale,
        &ended_at,
    )?;
    let collected_artifacts =
        collect_expected_artifacts(&layout, &mut ledger, &item, &run_id, &locale, &ended_at);
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
        execution_record_id: execution_record_id.clone(),
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
            execution_record_id: &execution_record_id,
            locale: &locale,
            created_at: &ended_at,
        })
    });
    let review_result = review_result_id
        .zip(agent_output.as_ref())
        .map(|(id, output)| review_result_from_agent_output(id, output, &item.acceptance_criteria));
    let item_status = if input.purpose == AgentRunPurpose::Work {
        if agent_output_requires_input(agent_output.as_ref()) {
            WorkItemStatus::NeedsInput
        } else if agent_output_requests_handoff(agent_output.as_ref()) {
            WorkItemStatus::NeedsHandoff
        } else if status == AgentRunStatus::Succeeded {
            WorkItemStatus::ReadyForReview
        } else {
            WorkItemStatus::ChangesRequested
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
            .into_iter()
            .filter(|(id, _)| {
                id != input.agent_profile_id
                    && id != &agent_settings.dispatch_agent
                    && id != &agent_settings.supervisor_agent
            })
            .collect()
    } else {
        BTreeMap::new()
    };
    let dispatch_plan = dispatch_plan_id.map(|id| {
        let fallback_target_agent_profile_id = dispatch_target_resolution
            .as_ref()
            .map(|resolution| resolution.agent_profile_id.clone())
            .filter(|target| valid_dispatch_targets.contains_key(target))
            .unwrap_or_else(|| agent_settings.work_agent.clone());
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
            raw_output_execution_record_id: execution_record_id.clone(),
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
    ledger.execution_records.push(execution_record);
    ledger.execution_records.extend(collected_execution_records);
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
    write_json_execution_record(
        &layout,
        &format!("{}.json", resolved_skill_context.id),
        &resolved_skill_context,
    )?;
    write_json_execution_record(
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

pub fn approve_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    rationale: &str,
) -> Result<DecisionResult, NagareError> {
    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    if item.status != WorkItemStatus::ReadyForReview {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` must be ready_for_review before approval; current status is {}",
            item.status
        )));
    }
    let snapshot = WorkItemSnapshot::from_ledger(item.clone(), &ledger);
    if snapshot
        .approval_gate
        .blockers
        .iter()
        .any(|blocker| blocker == "criteria_not_satisfied" || blocker == "review_not_passed")
    {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` needs a passing review for all acceptance criteria before approval"
        )));
    }
    if !snapshot.approval_gate.ready {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` approval gate is blocked: {}",
            snapshot.approval_gate.blockers.join(",")
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

pub fn reject_work_item(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    rationale: &str,
) -> Result<DecisionResult, NagareError> {
    let rationale = rationale.trim();
    if rationale.is_empty() {
        return Err(NagareError::InvalidState(
            "reject rationale is required".to_string(),
        ));
    }

    let layout = ensure_project(root)?;
    let locale = load_project_config(&layout)?.locale.language;
    let mut ledger = load_ledger(&layout)?;
    let item = ledger.work_item(work_item_id)?.clone();
    let snapshot = WorkItemSnapshot::from_ledger(item.clone(), &ledger);
    if !snapshot.approval_gate.ready {
        return Err(NagareError::InvalidState(format!(
            "work item `{work_item_id}` can only be rejected at approval gate; current gate is {}",
            snapshot.approval_gate.state
        )));
    }

    let decision = HumanDecision {
        id: ledger.next_id("dec"),
        work_item_id: work_item_id.to_string(),
        decision_type: "reject".to_string(),
        rationale: rationale.to_string(),
        locale,
        created_at: timestamp(),
    };
    ledger.decisions.push(decision.clone());
    for plan in &mut ledger.dispatch_plans {
        if plan.work_item_id == work_item_id && plan.status != DispatchPlanStatus::Superseded {
            plan.status = DispatchPlanStatus::Superseded;
        }
    }
    for plan in &mut ledger.recovery_plans {
        if plan.work_item_id == work_item_id && plan.status != RecoveryPlanStatus::Superseded {
            plan.status = RecoveryPlanStatus::Superseded;
        }
    }
    {
        let item = ledger.work_item_mut(work_item_id)?;
        item.status = WorkItemStatus::Ready;
        item.updated_at = timestamp();
    }
    save_ledger(&layout, &ledger)?;
    Ok(DecisionResult {
        decision,
        item_status: WorkItemStatus::Ready,
    })
}
