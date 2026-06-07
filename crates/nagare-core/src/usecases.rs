use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

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
    let tool_kind = AgentToolKind::infer(input.runtime, adapter);
    validate_tool_kind_for_runtime_adapter(tool_kind, input.runtime, adapter)?;
    let managed_by = normalize_managed_by(input.managed_by.unwrap_or("nagare"))?;
    let model = normalize_agent_model_selection(input.model)?;
    validate_model_for_adapter(adapter, &model)?;
    let external = normalize_external_agent_binding(default_external_binding(
        input.id,
        adapter,
        input.external,
    ))?;
    let domain_group_ids = normalize_domain_group_ids(input.domain_group_ids)?;
    let domain_ids = normalize_domain_profile_ids(input.domain_ids)?;
    let skill_set_ids = normalize_skill_set_ids(input.skill_set_ids)?;
    validate_existing_skill_set_ids(&layout, &skill_set_ids)?;
    validate_existing_domain_group_ids(&layout, &domain_group_ids)?;
    validate_existing_domain_profile_ids(&layout, &domain_ids)?;
    let prompt = AgentPromptConfig {
        instructions: input.description.trim().to_string(),
        version: default_agent_prompt_version(),
    };
    let profile = AgentProfile {
        id: input.id.to_string(),
        display_name: if input.display_name.trim().is_empty() {
            input.id.to_string()
        } else {
            input.display_name.to_string()
        },
        tool_kind,
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
        skill_set_ids,
        domain_group_ids,
        domain_ids,
        managed_by,
        model,
        external,
        prompt,
        output_contracts: AgentOutputContracts::default(),
        source: AgentProfileSource::ProjectAgentDirectory,
    };
    create_external_agent_if_needed(&layout, &profile)?;
    existing.insert(profile.id.clone(), profile.clone());

    let path = write_agent_profile_file(&layout, &profile)?;

    Ok(AddAgentProfileResult { profile, path })
}

fn default_external_binding(
    agent_profile_id: &str,
    adapter: &str,
    external: ExternalAgentBinding,
) -> ExternalAgentBinding {
    if !external.provider.is_empty()
        || !external.agent_id.is_empty()
        || !external.source.is_empty()
        || external.managed
    {
        return external;
    }
    let provider = match adapter {
        "process.codex-cli" => "codex-cli",
        "stdio.codex-app-server" => "codex",
        "process.openclaw-agent" => "openclaw",
        _ => return external,
    };
    ExternalAgentBinding {
        provider: provider.to_string(),
        agent_id: agent_profile_id.to_string(),
        managed: true,
        source: "created".to_string(),
    }
}

pub fn update_agent_profile(
    root: impl Into<PathBuf>,
    agent_profile_id: &str,
    input: UpdateAgentProfileInput<'_>,
) -> Result<UpdateAgentProfileResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(agent_profile_id)?;
    let mut profile = get_agent_profile_from_layout(&layout, agent_profile_id)?;
    let previous_profile = profile.clone();
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
    profile.tool_kind = AgentToolKind::infer(&profile.runtime, &profile.adapter);
    validate_tool_kind_for_runtime_adapter(profile.tool_kind, &profile.runtime, &profile.adapter)?;
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
        if profile.prompt.instructions.trim().is_empty() {
            profile.prompt.instructions = profile.description.clone();
        }
    }
    if let Some(specialties) = input.specialties {
        profile.specialties = normalize_specialties(specialties);
    }
    if let Some(skill_set_ids) = input.skill_set_ids {
        profile.skill_set_ids = normalize_skill_set_ids(skill_set_ids)?;
        validate_existing_skill_set_ids(&layout, &profile.skill_set_ids)?;
    }
    if let Some(domain_group_ids) = input.domain_group_ids {
        profile.domain_group_ids = normalize_domain_group_ids(domain_group_ids)?;
        validate_existing_domain_group_ids(&layout, &profile.domain_group_ids)?;
    }
    if let Some(domain_ids) = input.domain_ids {
        profile.domain_ids = normalize_domain_profile_ids(domain_ids)?;
        validate_existing_domain_profile_ids(&layout, &profile.domain_ids)?;
    }
    if let Some(managed_by) = input.managed_by {
        profile.managed_by = normalize_managed_by(managed_by)?;
    }
    if let Some(model) = input.model {
        profile.model =
            normalize_agent_model_selection(merge_agent_model_selection(&profile.model, model))?;
    }
    if let Some(external) = input.external {
        profile.external = normalize_external_agent_binding(merge_external_agent_binding(
            &profile.external,
            external,
        ))?;
    }
    validate_model_for_adapter(&profile.adapter, &profile.model)?;
    if let Some(update) = input.output_contract {
        apply_output_contract_update(&mut profile.output_contracts, update)?;
    }
    profile.source = AgentProfileSource::ProjectAgentDirectory;
    sync_external_agent_after_update(&layout, &previous_profile, &profile)?;
    let path = write_agent_profile_file(&layout, &profile)?;
    Ok(UpdateAgentProfileResult { profile, path })
}

fn merge_agent_model_selection(
    current: &AgentModelSelection,
    update: AgentModelSelection,
) -> AgentModelSelection {
    AgentModelSelection {
        provider: if update.provider.is_empty() {
            current.provider.clone()
        } else {
            update.provider
        },
        id: if update.id.is_empty() {
            current.id.clone()
        } else {
            update.id
        },
        base_url: if update.base_url.is_empty() {
            current.base_url.clone()
        } else {
            update.base_url
        },
        api_key_env: if update.api_key_env.is_empty() {
            current.api_key_env.clone()
        } else {
            update.api_key_env
        },
    }
}

fn merge_external_agent_binding(
    current: &ExternalAgentBinding,
    update: ExternalAgentBinding,
) -> ExternalAgentBinding {
    ExternalAgentBinding {
        provider: if update.provider.is_empty() {
            current.provider.clone()
        } else {
            update.provider
        },
        agent_id: if update.agent_id.is_empty() {
            current.agent_id.clone()
        } else {
            update.agent_id
        },
        managed: update.managed || current.managed,
        source: if update.source.is_empty() {
            current.source.clone()
        } else {
            update.source
        },
    }
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
    delete_external_agent_if_needed(&profile)?;
    Ok(profile)
}

pub fn list_skill_set_catalog(
    root: impl Into<PathBuf>,
) -> Result<Vec<SkillSetCatalogEntry>, NagareError> {
    let layout = ensure_project(root)?;
    let config = load_project_config(&layout)?;
    Ok(config
        .skill_sets
        .into_iter()
        .map(|(id, skill_set)| SkillSetCatalogEntry {
            id,
            paths: skill_set.paths,
            required_capabilities: skill_set.required_capabilities,
            optional_capabilities: skill_set.optional_capabilities,
        })
        .collect())
}

pub fn list_skill_packages(
    root: impl Into<PathBuf>,
) -> Result<Vec<SkillPackageCatalogEntry>, NagareError> {
    let layout = ensure_project(root)?;
    let config = load_project_config(&layout)?;
    Ok(config
        .skill_packages
        .into_iter()
        .map(|(id, package)| SkillPackageCatalogEntry {
            id,
            source_kind: package.source_kind,
            source: package.source,
            reference: package.reference,
            checksum: package.checksum,
            installed_path: package.installed_path,
            provided_skill_sets: package.provided_skill_sets,
        })
        .collect())
}

pub fn add_skill_package(
    root: impl Into<PathBuf>,
    input: AddSkillPackageInput<'_>,
) -> Result<SkillPackageInstallResult, NagareError> {
    let layout = ensure_project(root)?;
    let source_kind = normalize_skill_source_kind(input.source_kind)?;
    let skill_md = skill_md_metadata(input.path)?;
    let package_id = input
        .id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or(skill_md.name.as_deref())
        .ok_or_else(|| {
            NagareError::InvalidState(
                "--id is required when the source does not provide SKILL.md name".to_string(),
            )
        })?;
    validate_skill_package_id(package_id)?;
    let skill_set_id = input
        .skill_set_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(package_id);
    let skill_set_id = normalize_skill_set_ids(vec![skill_set_id.to_string()])?
        .into_iter()
        .next()
        .ok_or_else(|| NagareError::InvalidState("skill set id is required".to_string()))?;
    let source = input
        .source
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| input.path.map(|path| path.trim().to_string()))
        .unwrap_or_else(|| package_id.to_string());
    let installed_path = input
        .path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("")
        .replace('\\', "/");
    let skill_paths = normalize_skill_paths(input.skill_paths, input.path)?;
    let skill_set = SkillSetDeclaration {
        paths: skill_paths,
        required_capabilities: normalize_specialties(input.required_capabilities),
        optional_capabilities: normalize_specialties(input.optional_capabilities),
    };
    let package = SkillPackageDeclaration {
        source_kind: source_kind.to_string(),
        source,
        reference: input.reference.unwrap_or("").trim().to_string(),
        checksum: input.checksum.unwrap_or("").trim().to_string(),
        installed_path,
        provided_skill_sets: vec![skill_set_id.clone()],
    };
    write_skill_package_to_project_config(
        &layout,
        package_id,
        &package,
        &skill_set_id,
        &skill_set,
    )?;
    Ok(SkillPackageInstallResult {
        package: SkillPackageCatalogEntry {
            id: package_id.to_string(),
            source_kind: package.source_kind,
            source: package.source,
            reference: package.reference,
            checksum: package.checksum,
            installed_path: package.installed_path,
            provided_skill_sets: package.provided_skill_sets,
        },
        skill_set: SkillSetCatalogEntry {
            id: skill_set_id,
            paths: skill_set.paths,
            required_capabilities: skill_set.required_capabilities,
            optional_capabilities: skill_set.optional_capabilities,
        },
    })
}

fn create_external_agent_if_needed(
    layout: &ProjectLayout,
    profile: &AgentProfile,
) -> Result<(), NagareError> {
    if !profile.external.is_nagare_managed(&profile.managed_by)
        || profile.external.provider != "openclaw"
        || profile.external.source != "created"
    {
        return Ok(());
    }
    let command = openclaw_command();
    if !profile.model.provider.is_empty() && !profile.model.base_url.is_empty() {
        let provider_json = openclaw_provider_config_json(&profile.model)?;
        run_openclaw_command(
            &command,
            &[
                "config",
                "set",
                &format!("models.providers.{}", profile.model.provider),
                &provider_json,
                "--strict-json",
                "--merge",
            ],
        )?;
    }
    let workspace = resolve_profile_working_dir(layout, profile)?;
    let workspace = workspace.display().to_string();
    let mut args = vec![
        "agents",
        "add",
        &profile.external.agent_id,
        "--workspace",
        &workspace,
    ];
    let model = profile.model.model_ref();
    if let Some(model) = model.as_deref() {
        args.push("--model");
        args.push(model);
    }
    args.extend(["--non-interactive", "--json"]);
    run_openclaw_command(&command, &args)?;
    set_openclaw_agent_identity_if_needed(layout, profile)?;
    Ok(())
}

fn sync_external_agent_after_update(
    layout: &ProjectLayout,
    previous: &AgentProfile,
    current: &AgentProfile,
) -> Result<(), NagareError> {
    let previous_managed = is_managed_openclaw_created(previous);
    let current_managed = is_managed_openclaw_created(current);
    match (previous_managed, current_managed) {
        (false, false) => Ok(()),
        (false, true) => create_external_agent_if_needed(layout, current),
        (true, false) => delete_external_agent_if_needed(previous),
        (true, true) => {
            if openclaw_agent_requires_recreate(previous, current, layout)? {
                delete_external_agent_if_needed(previous)?;
                create_external_agent_if_needed(layout, current)?;
            } else if previous.display_name != current.display_name {
                set_openclaw_agent_identity_if_needed(layout, current)?;
            } else if previous.model != current.model && !current.model.base_url.is_empty() {
                configure_openclaw_model_provider(current)?;
            }
            Ok(())
        }
    }
}

fn is_managed_openclaw_created(profile: &AgentProfile) -> bool {
    profile.external.is_nagare_managed(&profile.managed_by)
        && profile.external.provider == "openclaw"
        && profile.external.source == "created"
}

fn openclaw_agent_requires_recreate(
    previous: &AgentProfile,
    current: &AgentProfile,
    layout: &ProjectLayout,
) -> Result<bool, NagareError> {
    Ok(previous.external.agent_id != current.external.agent_id
        || previous.model.model_ref() != current.model.model_ref()
        || resolve_profile_working_dir(layout, previous)?
            != resolve_profile_working_dir(layout, current)?)
}

fn configure_openclaw_model_provider(profile: &AgentProfile) -> Result<(), NagareError> {
    if profile.model.provider.is_empty() || profile.model.base_url.is_empty() {
        return Ok(());
    }
    let command = openclaw_command();
    let provider_json = openclaw_provider_config_json(&profile.model)?;
    run_openclaw_command(
        &command,
        &[
            "config",
            "set",
            &format!("models.providers.{}", profile.model.provider),
            &provider_json,
            "--strict-json",
            "--merge",
        ],
    )
}

fn set_openclaw_agent_identity_if_needed(
    layout: &ProjectLayout,
    profile: &AgentProfile,
) -> Result<(), NagareError> {
    if profile.display_name.trim().is_empty() || profile.display_name == profile.external.agent_id {
        return Ok(());
    }
    let command = openclaw_command();
    let workspace = resolve_profile_working_dir(layout, profile)?;
    run_openclaw_command(
        &command,
        &[
            "agents",
            "set-identity",
            "--agent",
            &profile.external.agent_id,
            "--workspace",
            &workspace.display().to_string(),
            "--name",
            &profile.display_name,
            "--json",
        ],
    )
}

fn delete_external_agent_if_needed(profile: &AgentProfile) -> Result<(), NagareError> {
    if !is_managed_openclaw_created(profile) {
        return Ok(());
    }
    let command = openclaw_command();
    run_openclaw_command(
        &command,
        &[
            "agents",
            "delete",
            &profile.external.agent_id,
            "--force",
            "--json",
        ],
    )
}

fn openclaw_command() -> String {
    std::env::var("NAGARE_OPENCLAW_COMMAND").unwrap_or_else(|_| "openclaw".to_string())
}

fn openclaw_provider_config_json(model: &AgentModelSelection) -> Result<String, NagareError> {
    let mut object = serde_json::Map::new();
    object.insert(
        "baseUrl".to_string(),
        serde_json::Value::String(model.base_url.clone()),
    );
    if !model.api_key_env.is_empty() {
        object.insert(
            "apiKey".to_string(),
            serde_json::json!({
                "source": "env",
                "provider": "default",
                "id": model.api_key_env,
            }),
        );
    }
    let model_id = model
        .id
        .rsplit_once('/')
        .map(|(_, id)| id)
        .unwrap_or(&model.id);
    object.insert(
        "models".to_string(),
        serde_json::json!([{ "id": model_id }]),
    );
    Ok(serde_json::Value::Object(object).to_string())
}

fn run_openclaw_command(command: &str, args: &[&str]) -> Result<(), NagareError> {
    let output = Command::new(command).args(args).output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(NagareError::InvalidState(format!(
        "openclaw command failed: {} {}\nstdout: {}\nstderr: {}",
        command,
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

fn managed_candidate_profiles(
    profiles: BTreeMap<String, AgentProfile>,
) -> BTreeMap<String, AgentProfile> {
    let managed = profiles
        .iter()
        .filter(|(_, profile)| profile.external.is_nagare_managed(&profile.managed_by))
        .map(|(id, profile)| (id.clone(), profile.clone()))
        .collect::<BTreeMap<_, _>>();
    if managed.is_empty() {
        profiles
    } else {
        managed
    }
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
            tool_kind: Some(profile.tool_kind),
            runtime: profile.runtime.clone(),
            adapter: profile.adapter.clone(),
            role: profile.role.clone(),
            working_dir: profile.working_dir.clone(),
            description: profile.description.clone(),
            specialties: profile.specialties.clone(),
            skill_set_ids: profile.skill_set_ids.clone(),
            domain_group_ids: profile.domain_group_ids.clone(),
            domain_ids: profile.domain_ids.clone(),
            managed_by: profile.managed_by.clone(),
            model: profile.model.clone(),
            external: profile.external.clone(),
            prompt: profile.prompt.clone(),
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

fn prompt_with_agent_instructions(prompt: &str, instructions: &str, locale: &str) -> String {
    let instructions = instructions.trim();
    if instructions.is_empty() {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n## {}\n{instructions}",
        localized_context_heading(locale, ContextHeading::AgentInstructions)
    )
}

fn prompt_with_nagare_agent_context(
    prompt: &str,
    profile: &AgentProfile,
    work_item_id: &str,
    output_contract: &AgentOutputContract,
) -> String {
    let model = profile.model.model_ref().unwrap_or_else(|| "-".to_string());
    format!(
        "{prompt}\n\n## Nagare Agent Context\n- managed_by: {}\n- agent_profile_id: {}\n- external_provider: {}\n- external_agent_id: {}\n- model: {}\n- work_item_id: {}\n- required_output_contract: {}\n",
        empty_marker(&profile.managed_by),
        profile.id,
        empty_marker(&profile.external.provider),
        empty_marker(&profile.external.agent_id),
        model,
        work_item_id,
        output_contract.contract
    )
}

fn empty_marker(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

#[derive(Default)]
struct SkillMdMetadata {
    name: Option<String>,
}

fn normalize_skill_source_kind(kind: &str) -> Result<&'static str, NagareError> {
    match kind.trim().to_ascii_lowercase().replace('_', "-").as_str() {
        "local" => Ok("local"),
        "git" => Ok("git"),
        "clawhub" | "claw-hub" => Ok("clawhub"),
        "vercel" | "vercel-skills" => Ok("vercel"),
        "skill-creator" | "skillcreator" => Ok("skill_creator"),
        other => Err(NagareError::InvalidState(format!(
            "unsupported skill source kind `{other}`; expected local, git, clawhub, vercel, or skill-creator"
        ))),
    }
}

fn validate_skill_package_id(id: &str) -> Result<(), NagareError> {
    let id = id.trim();
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(NagareError::InvalidState(format!(
            "skill package id `{id}` contains unsupported characters"
        )));
    }
    Ok(())
}

fn normalize_skill_paths(
    paths: Vec<String>,
    source_path: Option<&str>,
) -> Result<Vec<String>, NagareError> {
    let mut paths = normalize_specialties(paths);
    if paths.is_empty() {
        if let Some(path) = source_path.map(str::trim).filter(|path| !path.is_empty()) {
            paths.push(path.replace('\\', "/"));
        }
    }
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    Ok(paths)
}

fn skill_md_metadata(path: Option<&str>) -> Result<SkillMdMetadata, NagareError> {
    let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(SkillMdMetadata::default());
    };
    let skill_path = PathBuf::from(path).join("SKILL.md");
    if !skill_path.exists() {
        return Ok(SkillMdMetadata::default());
    }
    let raw = fs::read_to_string(skill_path)?;
    let mut lines = raw.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Ok(SkillMdMetadata::default());
    }
    let mut metadata = SkillMdMetadata::default();
    for line in lines {
        let line = line.trim();
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("name:") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                metadata.name = Some(value.to_string());
            }
        }
    }
    Ok(metadata)
}

fn write_skill_package_to_project_config(
    layout: &ProjectLayout,
    package_id: &str,
    package: &SkillPackageDeclaration,
    skill_set_id: &str,
    skill_set: &SkillSetDeclaration,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let skill_sets = root_table
        .entry("skill_sets".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| NagareError::InvalidState("skill_sets must be a TOML table".to_string()))?;
    skill_sets.insert(
        skill_set_id.to_string(),
        toml::Value::try_from(skill_set.clone())?,
    );
    let skill_packages = root_table
        .entry("skill_packages".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| {
            NagareError::InvalidState("skill_packages must be a TOML table".to_string())
        })?;
    skill_packages.insert(
        package_id.to_string(),
        toml::Value::try_from(package.clone())?,
    );
    let rendered = toml::to_string_pretty(&value)?;
    fs::write(&layout.config_path, rendered)?;
    Ok(())
}

fn validate_existing_skill_set_ids(
    layout: &ProjectLayout,
    skill_set_ids: &[String],
) -> Result<(), NagareError> {
    if skill_set_ids.is_empty() {
        return Ok(());
    }
    let config = load_project_config(layout)?;
    for skill_set_id in skill_set_ids {
        if !config.skill_sets.contains_key(skill_set_id) {
            return Err(NagareError::InvalidState(format!(
                "skill set `{skill_set_id}` is not declared"
            )));
        }
    }
    Ok(())
}

fn prompt_with_domain_context(prompt: &str, context: &str, locale: &str) -> String {
    let context = context.trim();
    if context.is_empty() {
        return prompt.to_string();
    }
    format!(
        "{prompt}\n\n## {}\n{context}",
        localized_context_heading(locale, ContextHeading::DomainContext)
    )
}

fn domain_prompt_context(
    layout: &ProjectLayout,
    item: &WorkItem,
    locale: &str,
) -> Result<String, NagareError> {
    let i18n = I18n::new(locale);
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
            "{}: {} ({})",
            i18n.ui(UiTextKey::DomainGroup),
            group.display_name,
            group.id
        ));
        if !group.description.trim().is_empty() {
            lines.push(format!(
                "{} {}: {}",
                i18n.ui(UiTextKey::Group),
                i18n.ui(UiTextKey::Description).to_ascii_lowercase(),
                group.description
            ));
        }
        append_context_list(
            &mut lines,
            i18n.ui(UiTextKey::SharedKnowledge),
            &group.shared_knowledge,
        );
        append_context_list(&mut lines, "Common rubric", &group.common_rubric);
        append_context_list(
            &mut lines,
            &format!(
                "{} {}",
                i18n.ui(UiTextKey::Group),
                i18n.ui(UiTextKey::DispatchHints)
            ),
            &group.dispatch_hints,
        );
    }
    if let Some(domain) = domain {
        lines.push(format!(
            "{}: {} ({})",
            i18n.ui(UiTextKey::Domain),
            domain.display_name,
            domain.id
        ));
        if !domain.description.trim().is_empty() {
            lines.push(format!(
                "{} {}: {}",
                i18n.ui(UiTextKey::Domain),
                i18n.ui(UiTextKey::Description).to_ascii_lowercase(),
                domain.description
            ));
        }
        append_context_list(&mut lines, "Artifact types", &domain.artifact_types);
        append_context_list(
            &mut lines,
            &format!(
                "{} {}",
                i18n.ui(UiTextKey::Domain),
                i18n.ui(UiTextKey::Rubric)
            ),
            &domain.rubric,
        );
        append_context_list(
            &mut lines,
            &format!(
                "{} {}",
                i18n.ui(UiTextKey::Domain),
                i18n.ui(UiTextKey::DispatchHints)
            ),
            &domain.dispatch_hints,
        );
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
    let skill_set_ids =
        merged_skill_set_ids(&rule_resolution.skill_set_ids, &agent_profile.skill_set_ids);
    let skill_set_resolution =
        resolve_skill_sets_for_run(&layout, &skill_set_ids, &capabilities_in_force)?;
    let instruction_sources = capability_probe.instruction_sources.clone();
    let default_goal = work_item_goal_prompt_for_locale(&item, &locale);
    let goal = input
        .prompt
        .filter(|prompt| !prompt.trim().is_empty())
        .unwrap_or(default_goal.as_str())
        .to_string();
    let human_feedback_context = human_feedback_prompt_context(&ledger, work_item_id);
    let handoff_context = handoff_prompt_context(&ledger, work_item_id);
    let domain_context = domain_prompt_context(&layout, &item, &locale)?;
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
        model: agent_profile.model.clone(),
        external: agent_profile.external.clone(),
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
        &prompt_with_nagare_agent_context(
            &prompt_with_agent_instructions(
                &prompt_with_domain_context(
                    &prompt_with_handoff_context(
                        &prompt_with_human_feedback(prompt, &human_feedback_context, &locale),
                        &handoff_context,
                        &locale,
                    ),
                    &domain_context,
                    &locale,
                ),
                agent_profile
                    .prompt
                    .effective_instructions(&agent_profile.description),
                &locale,
            ),
            &agent_profile,
            work_item_id,
            &output_contract,
        ),
        input.purpose,
        &output_contract,
        &locale,
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
    let mut item_status = if input.purpose == AgentRunPurpose::Work {
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
        managed_candidate_profiles(load_agent_profiles(&layout)?)
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
    let domain_agent_policy = effective_domain_agent_policy(&item);
    let domain_agent_missing =
        dispatch_plan_id.is_some() && domain_agent_missing(&item, &valid_dispatch_targets);
    let domain_fallback_confirmation = if domain_agent_missing
        && domain_agent_policy == DomainAgentPolicy::ConfirmGeneralFallback
    {
        domain_fallback_confirmation(&item, &valid_dispatch_targets, &agent_settings.work_agent)
    } else if domain_agent_missing && domain_agent_policy == DomainAgentPolicy::RequireDomainAgent {
        required_domain_agent_confirmation(
            &item,
            &valid_dispatch_targets,
            &agent_settings.work_agent,
        )
    } else {
        None
    };
    let auto_general_fallback =
        domain_agent_missing && domain_agent_policy == DomainAgentPolicy::AutoGeneralFallback;
    let domain_fallback_target = if auto_general_fallback || domain_fallback_confirmation.is_some()
    {
        general_fallback_agent_id(&valid_dispatch_targets, &agent_settings.work_agent)
    } else {
        agent_settings.work_agent.clone()
    };
    let dispatch_plan = dispatch_plan_id.map(|id| {
        let fallback_target_agent_profile_id = domain_fallback_confirmation
            .as_ref()
            .map(|confirmation| confirmation.target_agent_profile_id.clone())
            .or_else(|| {
                dispatch_target_resolution
            .as_ref()
            .map(|resolution| resolution.agent_profile_id.clone())
                .filter(|target| valid_dispatch_targets.contains_key(target))
            })
            .unwrap_or_else(|| domain_fallback_target.clone());
        let mut selection_warnings = Vec::new();
        let target_agent_profile_id = if domain_fallback_confirmation.is_some()
            || auto_general_fallback
        {
            fallback_target_agent_profile_id.clone()
        } else {
            match dispatch_suggestion
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
        let mut missing_information = dispatch_suggestion
            .as_ref()
            .map(|suggestion| suggestion.missing_information.clone())
            .filter(|missing_information| !missing_information.is_empty())
            .unwrap_or_else(|| extract_prefixed_lines(&output.stdout, "missing:"));
        if let Some(confirmation) = domain_fallback_confirmation.as_ref() {
            if !missing_information.contains(&confirmation.message) {
                missing_information.push(confirmation.message.clone());
            }
            selection_warnings.push(confirmation.message.clone());
        }
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
    if domain_fallback_confirmation.is_some() {
        item_status = WorkItemStatus::NeedsInput;
    }
    if matches!(
        input.purpose,
        AgentRunPurpose::Work | AgentRunPurpose::Review | AgentRunPurpose::DispatchPreview
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

fn merged_skill_set_ids(
    rule_skill_set_ids: &[String],
    agent_skill_set_ids: &[String],
) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut merged = Vec::new();
    for id in rule_skill_set_ids.iter().chain(agent_skill_set_ids.iter()) {
        if seen.insert(id.clone()) {
            merged.push(id.clone());
        }
    }
    merged
}

struct DomainFallbackConfirmation {
    target_agent_profile_id: String,
    message: String,
}

fn effective_domain_agent_policy(item: &WorkItem) -> DomainAgentPolicy {
    if item.require_domain_agent
        && item.domain_agent_policy == DomainAgentPolicy::AutoGeneralFallback
    {
        return DomainAgentPolicy::ConfirmGeneralFallback;
    }
    item.domain_agent_policy
}

fn domain_fallback_confirmation(
    item: &WorkItem,
    candidates: &BTreeMap<String, AgentProfile>,
    default_work_agent_id: &str,
) -> Option<DomainFallbackConfirmation> {
    if !domain_agent_missing(item, candidates) {
        return None;
    }
    let fallback_agent = general_fallback_agent_id(candidates, default_work_agent_id);
    let domain = item.domain_id.as_deref().unwrap_or("-");
    let group = item.domain_group_id.as_deref().unwrap_or("-");
    Some(DomainFallbackConfirmation {
        target_agent_profile_id: fallback_agent.clone(),
        message: format!(
            "No candidate agent is scoped to domain `{domain}` or domain group `{group}`; confirm whether to proceed with general fallback agent `{fallback_agent}`."
        ),
    })
}

fn required_domain_agent_confirmation(
    item: &WorkItem,
    candidates: &BTreeMap<String, AgentProfile>,
    default_work_agent_id: &str,
) -> Option<DomainFallbackConfirmation> {
    if !domain_agent_missing(item, candidates) {
        return None;
    }
    let fallback_agent = general_fallback_agent_id(candidates, default_work_agent_id);
    let domain = item.domain_id.as_deref().unwrap_or("-");
    let group = item.domain_group_id.as_deref().unwrap_or("-");
    Some(DomainFallbackConfirmation {
        target_agent_profile_id: fallback_agent.clone(),
        message: format!(
            "Domain-scoped agent is required for domain `{domain}` or domain group `{group}`, but no candidate agent is scoped to it; add a matching agent or change the domain agent policy before proceeding."
        ),
    })
}

fn domain_agent_missing(item: &WorkItem, candidates: &BTreeMap<String, AgentProfile>) -> bool {
    if item.domain_id.is_none() && item.domain_group_id.is_none() {
        return false;
    }
    if item.domain_id.as_deref() == Some("general")
        || item.domain_group_id.as_deref() == Some("general")
    {
        return false;
    }
    if candidates
        .values()
        .any(|profile| agent_matches_item_domain(profile, item))
    {
        return false;
    }
    true
}

fn general_fallback_agent_id(
    candidates: &BTreeMap<String, AgentProfile>,
    default_work_agent_id: &str,
) -> String {
    if candidates
        .get(default_work_agent_id)
        .is_some_and(agent_is_general)
    {
        return default_work_agent_id.to_string();
    }
    candidates
        .iter()
        .find(|(_, profile)| agent_is_general(profile))
        .map(|(id, _)| id.clone())
        .or_else(|| {
            candidates
                .get(default_work_agent_id)
                .map(|_| default_work_agent_id.to_string())
        })
        .or_else(|| candidates.keys().next().cloned())
        .unwrap_or_else(|| default_work_agent_id.to_string())
}

fn agent_is_general(profile: &AgentProfile) -> bool {
    profile
        .domain_ids
        .iter()
        .any(|domain_id| domain_id == "general")
        || profile
            .domain_group_ids
            .iter()
            .any(|domain_group_id| domain_group_id == "general")
}

fn agent_matches_item_domain(profile: &AgentProfile, item: &WorkItem) -> bool {
    item.domain_id.as_deref().is_some_and(|domain_id| {
        profile
            .domain_ids
            .iter()
            .any(|profile_domain_id| profile_domain_id == domain_id)
    }) || item
        .domain_group_id
        .as_deref()
        .is_some_and(|domain_group_id| {
            profile
                .domain_group_ids
                .iter()
                .any(|profile_domain_group_id| profile_domain_group_id == domain_group_id)
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
