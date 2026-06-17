use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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

pub fn list_artifact_types(root: impl Into<PathBuf>) -> Result<Vec<ArtifactType>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_artifact_types(&layout)?.into_values().collect())
}

pub fn list_domains(root: impl Into<PathBuf>) -> Result<Vec<Domain>, NagareError> {
    let layout = ensure_project(root)?;
    Ok(load_domains(&layout)?.into_values().collect())
}

pub fn get_artifact_type(
    root: impl Into<PathBuf>,
    artifact_type_id: &str,
) -> Result<ArtifactType, NagareError> {
    let layout = ensure_project(root)?;
    load_artifact_types(&layout)?
        .remove(artifact_type_id)
        .ok_or_else(|| NagareError::NotFound(format!("Artifact Type `{artifact_type_id}`")))
}

pub fn get_domain(root: impl Into<PathBuf>, domain_id: &str) -> Result<Domain, NagareError> {
    let layout = ensure_project(root)?;
    load_domains(&layout)?
        .remove(domain_id)
        .ok_or_else(|| NagareError::NotFound(format!("Domain `{domain_id}`")))
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

pub fn add_artifact_type(
    root: impl Into<PathBuf>,
    input: AddArtifactTypeInput<'_>,
) -> Result<AddArtifactTypeResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_artifact_type_id(input.id)?;
    let existing = load_artifact_types(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "Artifact Type `{}` already exists",
            input.id
        )));
    }
    let domain_id = input
        .domain_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if let Some(domain_id) = domain_id.as_deref() {
        validate_existing_domain(&layout, domain_id)?;
    }
    let domain = ArtifactType {
        id: input.id.to_string(),
        domain_id,
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
        source: ArtifactTypeSource::ProjectArtifactTypeDirectory,
    };
    let path = write_artifact_type_file(&layout, &domain)?;
    Ok(AddArtifactTypeResult { domain, path })
}

pub fn update_artifact_type(
    root: impl Into<PathBuf>,
    artifact_type_id: &str,
    input: UpdateArtifactTypeInput<'_>,
) -> Result<UpdateArtifactTypeResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_artifact_type_id(artifact_type_id)?;
    let mut domain = get_artifact_type(&layout.root, artifact_type_id)?;
    if let Some(domain_id) = input.domain_id {
        let domain_id = domain_id.map(str::trim).filter(|value| !value.is_empty());
        if let Some(domain_id) = domain_id {
            validate_existing_domain(&layout, domain_id)?;
        }
        domain.domain_id = domain_id.map(ToOwned::to_owned);
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
    domain.source = ArtifactTypeSource::ProjectArtifactTypeDirectory;
    let path = write_artifact_type_file(&layout, &domain)?;
    Ok(UpdateArtifactTypeResult { domain, path })
}

pub fn delete_artifact_type(
    root: impl Into<PathBuf>,
    artifact_type_id: &str,
) -> Result<ArtifactType, NagareError> {
    let layout = ensure_project(root)?;
    validate_artifact_type_id(artifact_type_id)?;
    let domain = get_artifact_type(&layout.root, artifact_type_id)?;
    if domain.source != ArtifactTypeSource::ProjectArtifactTypeDirectory {
        return Err(NagareError::InvalidState(format!(
            "Artifact Type `{artifact_type_id}` is not project-local and cannot be deleted"
        )));
    }
    let path = layout
        .artifact_types_dir
        .join(format!("{artifact_type_id}.toml"));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(domain)
}

pub fn add_domain(
    root: impl Into<PathBuf>,
    input: AddDomainInput<'_>,
) -> Result<AddDomainResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_id(input.id)?;
    let existing = load_domains(&layout)?;
    if existing.contains_key(input.id) {
        return Err(NagareError::InvalidState(format!(
            "Domain `{}` already exists",
            input.id
        )));
    }
    let group = Domain {
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
        source: DomainSource::ProjectDomainDirectory,
    };
    let path = write_domain_file(&layout, &group)?;
    Ok(AddDomainResult { group, path })
}

pub fn update_domain(
    root: impl Into<PathBuf>,
    domain_id: &str,
    input: UpdateDomainInput<'_>,
) -> Result<UpdateDomainResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_id(domain_id)?;
    let mut group = get_domain(&layout.root, domain_id)?;
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
    group.source = DomainSource::ProjectDomainDirectory;
    let path = write_domain_file(&layout, &group)?;
    Ok(UpdateDomainResult { group, path })
}

pub fn delete_domain(root: impl Into<PathBuf>, domain_id: &str) -> Result<Domain, NagareError> {
    let layout = ensure_project(root)?;
    validate_domain_id(domain_id)?;
    let group = get_domain(&layout.root, domain_id)?;
    if group.source != DomainSource::ProjectDomainDirectory {
        return Err(NagareError::InvalidState(format!(
            "Domain `{domain_id}` is not project-local and cannot be deleted"
        )));
    }
    let domains = load_artifact_types(&layout)?;
    if domains
        .values()
        .any(|domain| domain.domain_id.as_deref() == Some(domain_id))
    {
        return Err(NagareError::InvalidState(format!(
            "Domain `{domain_id}` still has domains"
        )));
    }
    let path = layout.domains_dir.join(format!("{domain_id}.toml"));
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
    let domain_ids = normalize_domain_ids(input.domain_ids)?;
    let artifact_type_ids = normalize_artifact_type_ids(input.artifact_type_ids)?;
    let skill_set_ids = normalize_skill_set_ids(input.skill_set_ids)?;
    validate_existing_skill_set_ids(&layout, &skill_set_ids)?;
    validate_existing_domain_ids(&layout, &domain_ids)?;
    validate_existing_artifact_type_ids(&layout, &artifact_type_ids)?;
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
        domain_ids,
        artifact_type_ids,
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
    if let Some(domain_ids) = input.domain_ids {
        profile.domain_ids = normalize_domain_ids(domain_ids)?;
        validate_existing_domain_ids(&layout, &profile.domain_ids)?;
    }
    if let Some(artifact_type_ids) = input.artifact_type_ids {
        profile.artifact_type_ids = normalize_artifact_type_ids(artifact_type_ids)?;
        validate_existing_artifact_type_ids(&layout, &profile.artifact_type_ids)?;
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
            installed_paths: package.installed_paths,
            install_scope: package.install_scope,
            installed_targets: package.installed_targets,
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
    let install_scope = normalize_skill_install_scope(source_kind, input.install_scope)?;
    let installed_targets =
        normalize_skill_install_targets(source_kind, input.install_targets, input.install)?;
    let mut source = input
        .source
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| input.path.map(|path| path.trim().to_string()))
        .unwrap_or_else(|| package_id.to_string());
    let import = if input.install {
        import_skill_package(
            &layout,
            source_kind,
            package_id,
            &source,
            input.skill_set_id,
            &install_scope,
            &installed_targets,
        )?
    } else {
        None
    };
    if let Some(import_source) = import
        .as_ref()
        .and_then(|import| import.source.as_ref())
        .filter(|value| !value.trim().is_empty())
    {
        source = import_source.clone();
    }
    let skill_set_id = input
        .skill_set_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or(import
            .as_ref()
            .and_then(|import| import.skill_name.as_deref()))
        .or(skill_md.name.as_deref())
        .unwrap_or(package_id);
    let skill_set_id = normalize_skill_set_ids(vec![skill_set_id.to_string()])?
        .into_iter()
        .next()
        .ok_or_else(|| NagareError::InvalidState("skill set id is required".to_string()))?;
    let installed_paths = input
        .path
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|path| vec![path.replace('\\', "/")])
        .or_else(|| {
            import.as_ref().map(|import| {
                import
                    .installed_paths
                    .iter()
                    .map(|path| path.replace('\\', "/"))
                    .collect::<Vec<_>>()
            })
        })
        .unwrap_or_default();
    let installed_path = installed_paths.first().cloned().unwrap_or_default();
    let source_paths_for_skills = if !installed_paths.is_empty() {
        installed_paths.clone()
    } else if installed_path.is_empty() {
        input
            .path
            .map(|path| vec![path.replace('\\', "/")])
            .unwrap_or_default()
    } else {
        vec![installed_path.clone()]
    };
    let skill_paths = normalize_skill_paths(input.skill_paths, source_paths_for_skills)?;
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
        installed_paths,
        install_scope,
        installed_targets,
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
            installed_paths: package.installed_paths,
            install_scope: package.install_scope,
            installed_targets: package.installed_targets,
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

pub fn uninstall_agent_skill_package(
    root: impl Into<PathBuf>,
    input: UninstallAgentSkillPackageInput<'_>,
) -> Result<UninstallAgentSkillPackageResult, NagareError> {
    let layout = ensure_project(root)?;
    validate_agent_profile_id(input.agent_profile_id)?;
    let skill_set_id = normalize_skill_set_ids(vec![input.skill_set_id.to_string()])?
        .into_iter()
        .next()
        .ok_or_else(|| NagareError::InvalidState("skill set id is required".to_string()))?;
    let profile = get_agent_profile_from_layout(&layout, input.agent_profile_id)?;
    let package = find_skill_package_for_skill_set(&layout, &skill_set_id)?;
    let mut warnings = Vec::new();
    let removed_from_agent = profile.skill_set_ids.iter().any(|id| id == &skill_set_id);
    if !removed_from_agent {
        warnings.push(format!(
            "skill set `{skill_set_id}` was not assigned to agent `{}`",
            profile.id
        ));
        return Ok(UninstallAgentSkillPackageResult {
            agent_profile_id: profile.id,
            skill_set_id,
            package_id: package.as_ref().map(|(id, _)| id.clone()),
            removed_from_agent: false,
            package_removed: false,
            installed_path_removed: false,
            warnings,
        });
    }

    let next_skill_set_ids = profile
        .skill_set_ids
        .iter()
        .filter(|id| *id != &skill_set_id)
        .cloned()
        .collect::<Vec<_>>();
    update_agent_profile(
        &layout.root,
        &profile.id,
        UpdateAgentProfileInput {
            skill_set_ids: Some(next_skill_set_ids),
            ..UpdateAgentProfileInput::default()
        },
    )?;

    let mut package_removed = false;
    let mut installed_path_removed = false;
    if input.uninstall_package {
        if let Some((package_id, package)) = package.as_ref() {
            let provided = if package.provided_skill_sets.is_empty() {
                vec![skill_set_id.clone()]
            } else {
                package.provided_skill_sets.clone()
            };
            if skill_sets_still_referenced(&layout, &provided)? {
                warnings.push(format!(
                    "skill package `{package_id}` is still used by another agent or project rule"
                ));
            } else {
                let body_result = uninstall_skill_package_body(
                    &layout,
                    package_id,
                    package,
                    profile.tool_kind,
                    &skill_set_id,
                    &mut warnings,
                )?;
                package_removed = body_result.package_removed;
                installed_path_removed = body_result.installed_path_removed;
            }
        } else {
            warnings.push(format!(
                "skill set `{skill_set_id}` has no registered skill package"
            ));
        }
    }

    Ok(UninstallAgentSkillPackageResult {
        agent_profile_id: profile.id,
        skill_set_id,
        package_id: package.map(|(id, _)| id),
        removed_from_agent,
        package_removed,
        installed_path_removed,
        warnings,
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
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| openclaw_command_error(command, args, error))?;
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

fn openclaw_command_error(command: &str, args: &[&str], error: std::io::Error) -> NagareError {
    if error.kind() == std::io::ErrorKind::NotFound {
        return NagareError::InvalidState(format!(
            "OpenClaw CLI is required to create or update an OpenClaw agent, but `{command}` was not found.\n\
Install OpenClaw, make sure `{command}` is available on PATH, or set NAGARE_OPENCLAW_COMMAND to the OpenClaw executable path.\n\
Command Nagare tried: {command} {}",
            args.join(" ")
        ));
    }
    NagareError::InvalidState(format!(
        "OpenClaw CLI could not be started: {command} {}\nerror: {error}",
        args.join(" ")
    ))
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
            domain_ids: profile.domain_ids.clone(),
            artifact_type_ids: profile.artifact_type_ids.clone(),
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

fn write_artifact_type_file(
    layout: &ProjectLayout,
    domain: &ArtifactType,
) -> Result<PathBuf, NagareError> {
    fs::create_dir_all(&layout.artifact_types_dir)?;
    let path = layout
        .artifact_types_dir
        .join(format!("{}.toml", domain.id));
    let document = ArtifactTypeFile {
        artifact_type: Some(ArtifactTypeFileEntry {
            id: Some(domain.id.clone()),
            domain_id: domain.domain_id.clone(),
            display_name: domain.display_name.clone(),
            description: domain.description.clone(),
            artifact_types: domain.artifact_types.clone(),
            rubric: domain.rubric.clone(),
            dispatch_hints: domain.dispatch_hints.clone(),
            workflow: domain.workflow,
        }),
        artifact_types: BTreeMap::new(),
    };
    let raw = toml::to_string_pretty(&document)?;
    fs::write(&path, raw)?;
    Ok(path)
}

fn write_domain_file(layout: &ProjectLayout, group: &Domain) -> Result<PathBuf, NagareError> {
    fs::create_dir_all(&layout.domains_dir)?;
    let path = layout.domains_dir.join(format!("{}.toml", group.id));
    let document = DomainFile {
        domain: Some(DomainFileEntry {
            id: Some(group.id.clone()),
            display_name: group.display_name.clone(),
            description: group.description.clone(),
            shared_knowledge: group.shared_knowledge.clone(),
            common_rubric: group.common_rubric.clone(),
            dispatch_hints: group.dispatch_hints.clone(),
            workflow: group.workflow,
        }),
        domains: BTreeMap::new(),
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

fn normalize_skill_install_scope(
    source_kind: &str,
    scope: Option<&str>,
) -> Result<String, NagareError> {
    let scope = scope.map(str::trim).filter(|value| !value.is_empty());
    match source_kind {
        "vercel" => match scope.unwrap_or("project").to_ascii_lowercase().as_str() {
            "project" => Ok("project".to_string()),
            "global" => Ok("global".to_string()),
            other => Err(NagareError::InvalidState(format!(
                "unsupported Vercel skill install scope `{other}`; expected project or global"
            ))),
        },
        "clawhub" => Ok(scope.unwrap_or("project").to_string()),
        _ => Ok(scope.unwrap_or("").to_string()),
    }
}

fn normalize_skill_install_targets(
    source_kind: &str,
    targets: Vec<String>,
    install: bool,
) -> Result<Vec<String>, NagareError> {
    let mut normalized = Vec::new();
    for target in normalize_specialties(targets) {
        let target = match target.to_ascii_lowercase().replace('_', "-").as_str() {
            "codex" | "codex-cli" | "codex-app" | "codex-app-server" => "codex",
            "openclaw" | "open-claw" => "openclaw",
            other => {
                return Err(NagareError::InvalidState(format!(
                    "unsupported skill install target `{other}`; expected codex or openclaw"
                )));
            }
        };
        if !normalized.iter().any(|value| value == target) {
            normalized.push(target.to_string());
        }
    }
    if normalized.is_empty() {
        if source_kind == "vercel" && install {
            normalized.push("codex".to_string());
        } else if source_kind == "clawhub" && install {
            normalized.push("openclaw".to_string());
        }
    }
    if source_kind == "vercel" && install && normalized.is_empty() {
        return Err(NagareError::InvalidState(
            "Vercel skill install requires at least one target tool".to_string(),
        ));
    }
    Ok(normalized)
}

#[derive(Debug, Clone, Default)]
struct SkillPackageImport {
    source: Option<String>,
    installed_paths: Vec<String>,
    skill_name: Option<String>,
}

fn import_skill_package(
    layout: &ProjectLayout,
    source_kind: &str,
    package_id: &str,
    source: &str,
    explicit_skill_set_id: Option<&str>,
    install_scope: &str,
    installed_targets: &[String],
) -> Result<Option<SkillPackageImport>, NagareError> {
    match source_kind {
        "vercel" => import_vercel_skill_package(
            layout,
            package_id,
            source,
            explicit_skill_set_id,
            install_scope,
            installed_targets,
        )
        .map(Some),
        "clawhub" => import_clawhub_skill_package(layout, package_id, source).map(Some),
        _ => Ok(None),
    }
}

fn import_vercel_skill_package(
    layout: &ProjectLayout,
    package_id: &str,
    source: &str,
    explicit_skill_set_id: Option<&str>,
    install_scope: &str,
    installed_targets: &[String],
) -> Result<SkillPackageImport, NagareError> {
    let roots = skill_dir_roots_for_install(layout, install_scope, installed_targets);
    let before = read_skill_dirs(&roots)?;
    let (source, skill_hint) = vercel_source_and_skill(package_id, source, explicit_skill_set_id);
    let mut args = vec![
        "skills".to_string(),
        "add".to_string(),
        source.clone(),
        if install_scope == "global" {
            "--global".to_string()
        } else {
            "--project".to_string()
        },
        "--agent".to_string(),
    ];
    args.extend(installed_targets.iter().cloned());
    args.extend(["-y".to_string(), "--copy".to_string()]);
    if let Some(skill) = skill_hint.as_deref() {
        args.push("--skill".to_string());
        args.push(skill.to_string());
    }
    let output = run_command_in_dir("npx", &args, &layout.root).map_err(|error| {
        NagareError::InvalidState(format!(
            "failed to run `npx skills add`; install Node.js/npm or run `npx skills add {source}` manually: {error}"
        ))
    })?;
    ensure_command_success("npx skills add", &output)?;
    let mut import = discover_skill_install(
        layout,
        before,
        read_skill_dirs(&roots)?,
        skill_hint.as_deref(),
        &source,
        &roots,
    )?;
    import.source = Some(source);
    Ok(import)
}

fn import_clawhub_skill_package(
    layout: &ProjectLayout,
    package_id: &str,
    source: &str,
) -> Result<SkillPackageImport, NagareError> {
    let before = common_skill_dirs(layout)?;
    let source = source.trim();
    let output = if command_exists("openclaw") {
        run_command_in_dir(
            "openclaw",
            &[
                "skill".to_string(),
                "install".to_string(),
                source.to_string(),
            ],
            &layout.root,
        )
    } else if command_exists("clawhub") {
        run_command_in_dir(
            "clawhub",
            &["install".to_string(), source.to_string()],
            &layout.root,
        )
    } else if command_exists("npx") {
        run_command_in_dir(
            "npx",
            &[
                "clawhub@latest".to_string(),
                "install".to_string(),
                source.to_string(),
            ],
            &layout.root,
        )
    } else {
        return Err(NagareError::InvalidState(
            "ClawHub import requires OpenClaw (`openclaw skill install`), ClawHub CLI (`npm i -g clawhub`), or npm/npx.".to_string(),
        ));
    }
    .map_err(|error| {
        NagareError::InvalidState(format!(
            "failed to run ClawHub install command; install OpenClaw or `npm i -g clawhub`: {error}"
        ))
    })?;
    ensure_command_success("ClawHub skill install", &output)?;
    let mut import = discover_common_skill_install(layout, before, Some(package_id), source)?;
    import.source = Some(source.to_string());
    Ok(import)
}

fn vercel_source_and_skill(
    package_id: &str,
    source: &str,
    explicit_skill_set_id: Option<&str>,
) -> (String, Option<String>) {
    let explicit_skill = explicit_skill_set_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if let Some((repo, skill)) = source.split_once('@') {
        return (
            repo.trim().to_string(),
            explicit_skill.or_else(|| Some(skill.trim().to_string())),
        );
    }
    if let Some((repo, skill)) = package_id.split_once('@') {
        return (
            repo.trim().to_string(),
            explicit_skill.or_else(|| Some(skill.trim().to_string())),
        );
    }
    let inferred_skill = if source.trim() != package_id.trim() && !package_id.contains('/') {
        Some(package_id.trim().to_string())
    } else {
        None
    };
    (source.trim().to_string(), explicit_skill.or(inferred_skill))
}

fn common_skill_dirs(layout: &ProjectLayout) -> Result<BTreeMap<String, PathBuf>, NagareError> {
    let dirs = common_skill_roots(layout);
    read_skill_dirs(&dirs)
}

fn common_skill_roots(layout: &ProjectLayout) -> Vec<PathBuf> {
    let mut dirs = vec![
        layout.root.join(".agents").join("skills"),
        layout.root.join(".openclaw").join("skills"),
        layout.root.join("skills"),
    ];
    if let Some(home) = home_dir() {
        dirs.push(home.join(".openclaw").join("skills"));
        dirs.push(home.join(".agents").join("skills"));
    }
    dirs
}

fn skill_dir_roots_for_install(
    layout: &ProjectLayout,
    install_scope: &str,
    installed_targets: &[String],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let global = install_scope == "global";
    for target in installed_targets {
        match target.as_str() {
            "codex" => {
                if global {
                    if let Some(home) = home_dir() {
                        roots.push(home.join(".agents").join("skills"));
                    }
                } else {
                    roots.push(layout.root.join(".agents").join("skills"));
                }
            }
            "openclaw" => {
                if global {
                    if let Some(home) = home_dir() {
                        roots.push(home.join(".openclaw").join("skills"));
                    }
                } else {
                    roots.push(layout.root.join("skills"));
                    roots.push(layout.root.join(".openclaw").join("skills"));
                }
            }
            _ => {}
        }
    }
    dedupe_paths(roots)
}

fn read_skill_dirs(dirs: &[PathBuf]) -> Result<BTreeMap<String, PathBuf>, NagareError> {
    let mut skills = BTreeMap::new();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() || !path.join("SKILL.md").is_file() {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                skills.insert(name.to_string(), path);
            }
        }
    }
    Ok(skills)
}

fn discover_common_skill_install(
    layout: &ProjectLayout,
    before: BTreeMap<String, PathBuf>,
    skill_hint: Option<&str>,
    source: &str,
) -> Result<SkillPackageImport, NagareError> {
    let roots = common_skill_roots(layout);
    let after = read_skill_dirs(&roots)?;
    discover_skill_install(layout, before, after, skill_hint, source, &roots)
}

fn discover_skill_install(
    layout: &ProjectLayout,
    before: BTreeMap<String, PathBuf>,
    after: BTreeMap<String, PathBuf>,
    skill_hint: Option<&str>,
    source: &str,
    roots: &[PathBuf],
) -> Result<SkillPackageImport, NagareError> {
    let mut candidates = Vec::new();
    if let Some(hint) = skill_hint.map(str::trim).filter(|value| !value.is_empty()) {
        if let Some(path) = after.get(hint) {
            candidates.push(path.clone());
        }
    }
    if candidates.is_empty() {
        if let Some(name) = source
            .trim_end_matches('/')
            .rsplit(['/', '\\', '@'])
            .next()
            .filter(|value| !value.is_empty())
        {
            if let Some(path) = after.get(name) {
                candidates.push(path.clone());
            }
        }
    }
    let new_paths = after
        .iter()
        .filter(|(name, _)| !before.contains_key(*name))
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        if new_paths.len() == 1 {
            candidates.push(new_paths[0].clone());
        } else if new_paths.len() > 1 {
            return Err(NagareError::InvalidState(format!(
                "skill import installed multiple skills; specify a single skill with Skill Set ID or `owner/repo@skill`: {}",
                new_paths
                    .iter()
                    .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
    }
    let Some(path) = candidates.into_iter().next() else {
        return Err(NagareError::InvalidState(format!(
            "skill import completed but Nagare could not locate an installed SKILL.md for `{source}`"
        )));
    };
    let metadata = skill_md_metadata_at(&path)?;
    let skill_name = metadata.name.or_else(|| {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
    });
    let installed_paths = skill_name
        .as_deref()
        .map(|name| installed_skill_paths(layout, roots, name))
        .filter(|paths| !paths.is_empty())
        .unwrap_or_else(|| vec![path_for_config(layout, &path)]);
    Ok(SkillPackageImport {
        source: None,
        installed_paths,
        skill_name,
    })
}

fn installed_skill_paths(
    layout: &ProjectLayout,
    roots: &[PathBuf],
    skill_name: &str,
) -> Vec<String> {
    roots
        .iter()
        .map(|root| root.join(skill_name))
        .filter(|path| path.join("SKILL.md").is_file())
        .map(|path| path_for_config(layout, &path))
        .collect()
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(path.to_string_lossy().replace('\\', "/")))
        .collect()
}

fn run_command_in_dir(
    command: &str,
    args: &[String],
    current_dir: &Path,
) -> io::Result<std::process::Output> {
    match Command::new(command)
        .args(args)
        .current_dir(current_dir)
        .output()
    {
        Ok(output) => Ok(output),
        Err(error) if cfg!(windows) && error.kind() == io::ErrorKind::NotFound => {
            Command::new(format!("{command}.cmd"))
                .args(args)
                .current_dir(current_dir)
                .output()
        }
        Err(error) => Err(error),
    }
}

fn ensure_command_success(command: &str, output: &std::process::Output) -> Result<(), NagareError> {
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(NagareError::InvalidState(format!(
        "`{command}` failed with {}. stdout: {} stderr: {}",
        output.status,
        first_nonempty_line(&stdout).unwrap_or_else(|| "-".to_string()),
        first_nonempty_line(&stderr).unwrap_or_else(|| "-".to_string())
    )))
}

#[derive(Debug, Clone, Copy, Default)]
struct SkillPackageBodyUninstallResult {
    package_removed: bool,
    installed_path_removed: bool,
}

fn find_skill_package_for_skill_set(
    layout: &ProjectLayout,
    skill_set_id: &str,
) -> Result<Option<(String, SkillPackageDeclaration)>, NagareError> {
    let config = load_project_config(layout)?;
    Ok(config.skill_packages.into_iter().find(|(id, package)| {
        id.as_str() == skill_set_id
            || package
                .provided_skill_sets
                .iter()
                .any(|skill| skill == skill_set_id)
    }))
}

fn uninstall_skill_from_external_tool(
    layout: &ProjectLayout,
    tool_kind: AgentToolKind,
    package: &SkillPackageDeclaration,
    skill_set_id: &str,
    warnings: &mut Vec<String>,
) {
    match package.source_kind.as_str() {
        "vercel" => {
            let agent_key = external_skill_agent_key(tool_kind);
            uninstall_vercel_skill_package_body(
                layout,
                package,
                skill_set_id,
                agent_key,
                warnings,
            );
        }
        "clawhub" if tool_kind == AgentToolKind::OpenClaw => {
            uninstall_clawhub_skill_package_body(layout, package, skill_set_id, warnings);
        }
        "clawhub" => warnings.push(
            "ClawHub uninstall is only applied for OpenClaw agents; removed Nagare project registration only"
                .to_string(),
        ),
        _ => {}
    }
}

fn external_skill_agent_key(tool_kind: AgentToolKind) -> &'static str {
    match tool_kind {
        AgentToolKind::Codex | AgentToolKind::CodexCli => "codex",
        AgentToolKind::OpenClaw => "openclaw",
    }
}

fn skill_sets_still_referenced(
    layout: &ProjectLayout,
    skill_set_ids: &[String],
) -> Result<bool, NagareError> {
    let ids = skill_set_ids.iter().collect::<BTreeSet<_>>();
    if ids.is_empty() {
        return Ok(false);
    }
    let agents = load_agent_profiles(layout)?;
    if agents.values().any(|profile| {
        profile
            .skill_set_ids
            .iter()
            .any(|skill_set_id| ids.contains(skill_set_id))
    }) {
        return Ok(true);
    }
    let config = load_project_config(layout)?;
    Ok(config.project_rules.iter().any(|rule| {
        rule.skill_sets
            .iter()
            .any(|skill_set_id| ids.contains(skill_set_id))
    }))
}

fn uninstall_skill_package_body(
    layout: &ProjectLayout,
    package_id: &str,
    package: &SkillPackageDeclaration,
    tool_kind: AgentToolKind,
    skill_set_id: &str,
    warnings: &mut Vec<String>,
) -> Result<SkillPackageBodyUninstallResult, NagareError> {
    uninstall_skill_from_external_tool(layout, tool_kind, package, skill_set_id, warnings);
    let mut installed_path_removed = false;
    for path in managed_skill_package_paths(layout, package, skill_set_id, tool_kind) {
        match safe_remove_skill_dir(layout, &path) {
            Ok(removed) => installed_path_removed |= removed,
            Err(error) => warnings.push(error.to_string()),
        }
    }
    let target = external_skill_agent_key(tool_kind);
    let has_remaining_targets = !package.installed_targets.is_empty()
        && package
            .installed_targets
            .iter()
            .any(|installed_target| installed_target != target);
    if has_remaining_targets {
        update_skill_package_after_target_uninstall(
            layout,
            package_id,
            skill_set_id,
            package,
            tool_kind,
        )?;
        return Ok(SkillPackageBodyUninstallResult {
            package_removed: false,
            installed_path_removed,
        });
    }
    if let Err(error) = remove_skill_from_skills_lock(layout, skill_set_id, package_id) {
        warnings.push(error.to_string());
    }
    remove_skill_package_from_project_config(layout, package_id, package)?;
    Ok(SkillPackageBodyUninstallResult {
        package_removed: true,
        installed_path_removed,
    })
}

fn uninstall_vercel_skill_package_body(
    layout: &ProjectLayout,
    package: &SkillPackageDeclaration,
    skill_set_id: &str,
    agent_key: &str,
    warnings: &mut Vec<String>,
) {
    let mut args = vec![
        "skills".to_string(),
        "remove".to_string(),
        skill_set_id.to_string(),
        "--agent".to_string(),
        agent_key.to_string(),
    ];
    if package.install_scope == "global" {
        args.push("--global".to_string());
    }
    args.push("-y".to_string());
    match run_command_in_dir("npx", &args, &layout.root) {
        Ok(output) => {
            if let Err(error) = ensure_command_success("npx skills remove --agent", &output) {
                warnings.push(error.to_string());
            }
        }
        Err(error) => warnings.push(format!(
            "failed to run `npx skills remove --agent {agent_key}`: {error}"
        )),
    }
}

fn uninstall_clawhub_skill_package_body(
    layout: &ProjectLayout,
    package: &SkillPackageDeclaration,
    skill_set_id: &str,
    warnings: &mut Vec<String>,
) {
    let source = package
        .source
        .trim()
        .is_empty()
        .then(|| skill_set_id.to_string())
        .unwrap_or_else(|| package.source.trim().to_string());
    let attempts = clawhub_uninstall_attempts(&source);
    let mut failure_messages = Vec::new();
    for (command, args, label) in attempts {
        if !command_exists(&command) && command != "npx" {
            continue;
        }
        match run_command_in_dir(&command, &args, &layout.root) {
            Ok(output) if output.status.success() => return,
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                failure_messages.push(format!(
                    "`{label}` failed with {}. stdout: {} stderr: {}",
                    output.status,
                    first_nonempty_line(&stdout).unwrap_or_else(|| "-".to_string()),
                    first_nonempty_line(&stderr).unwrap_or_else(|| "-".to_string())
                ));
            }
            Err(error) => failure_messages.push(format!("`{label}` failed: {error}")),
        }
    }
    if failure_messages.is_empty() {
        warnings.push(
            "ClawHub uninstall command was not found; removed Nagare project registration only"
                .to_string(),
        );
    } else {
        warnings.extend(failure_messages);
    }
}

fn clawhub_uninstall_attempts(source: &str) -> Vec<(String, Vec<String>, String)> {
    vec![
        (
            "openclaw".to_string(),
            vec![
                "skills".to_string(),
                "uninstall".to_string(),
                source.to_string(),
            ],
            format!("openclaw skills uninstall {source}"),
        ),
        (
            "openclaw".to_string(),
            vec![
                "skill".to_string(),
                "uninstall".to_string(),
                source.to_string(),
            ],
            format!("openclaw skill uninstall {source}"),
        ),
        (
            "clawhub".to_string(),
            vec!["uninstall".to_string(), source.to_string()],
            format!("clawhub uninstall {source}"),
        ),
        (
            "npx".to_string(),
            vec![
                "clawhub@latest".to_string(),
                "uninstall".to_string(),
                source.to_string(),
            ],
            format!("npx clawhub@latest uninstall {source}"),
        ),
    ]
}

fn managed_skill_package_paths(
    layout: &ProjectLayout,
    package: &SkillPackageDeclaration,
    skill_set_id: &str,
    tool_kind: AgentToolKind,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for path in package_installed_paths(package) {
        paths.push(skill_package_path_for_uninstall(layout, path));
    }
    if let Ok(config) = load_project_config(layout) {
        if let Some(skill_set) = config.skill_sets.get(skill_set_id) {
            for path in &skill_set.paths {
                paths.push(skill_package_path_for_uninstall(layout, path));
            }
        }
    }
    paths.push(
        layout
            .root
            .join(".agents")
            .join("skills")
            .join(skill_set_id),
    );
    paths.push(layout.root.join("skills").join(skill_set_id));
    let mut seen: BTreeSet<String> = BTreeSet::new();
    paths
        .into_iter()
        .filter(|path| path_matches_tool_kind(layout, path, tool_kind))
        .filter(|path| seen.insert(path.to_string_lossy().replace('\\', "/")))
        .collect()
}

fn package_installed_paths(package: &SkillPackageDeclaration) -> Vec<&str> {
    let mut paths = Vec::new();
    if !package.installed_path.trim().is_empty() {
        paths.push(package.installed_path.as_str());
    }
    for path in &package.installed_paths {
        if !path.trim().is_empty() && !paths.iter().any(|existing| *existing == path.as_str()) {
            paths.push(path.as_str());
        }
    }
    paths
}

fn path_matches_tool_kind(layout: &ProjectLayout, path: &Path, tool_kind: AgentToolKind) -> bool {
    tool_skill_roots(layout, tool_kind)
        .iter()
        .any(|root| path.starts_with(root))
}

fn tool_skill_roots(layout: &ProjectLayout, tool_kind: AgentToolKind) -> Vec<PathBuf> {
    let mut roots = match tool_kind {
        AgentToolKind::Codex | AgentToolKind::CodexCli => {
            vec![layout.root.join(".agents").join("skills")]
        }
        AgentToolKind::OpenClaw => vec![
            layout.root.join("skills"),
            layout.root.join(".openclaw").join("skills"),
        ],
    };
    if let Some(home) = home_dir() {
        match tool_kind {
            AgentToolKind::Codex | AgentToolKind::CodexCli => {
                roots.push(home.join(".agents").join("skills"));
            }
            AgentToolKind::OpenClaw => {
                roots.push(home.join(".openclaw").join("skills"));
            }
        }
    }
    dedupe_paths(roots)
}

fn skill_package_path_for_uninstall(layout: &ProjectLayout, path: &str) -> PathBuf {
    let path = Path::new(path.trim());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        layout.root.join(path)
    }
}

fn safe_remove_skill_dir(layout: &ProjectLayout, path: &Path) -> Result<bool, NagareError> {
    if !path.exists() {
        return Ok(false);
    }
    if !path.is_dir() || !path.join("SKILL.md").is_file() {
        return Ok(false);
    }
    let root = layout.root.canonicalize()?;
    let target = path.canonicalize()?;
    if target == root || !target.starts_with(&root) {
        return Err(NagareError::InvalidState(format!(
            "refusing to delete unmanaged skill path `{}`",
            path.display()
        )));
    }
    fs::remove_dir_all(path)?;
    Ok(true)
}

fn remove_skill_from_skills_lock(
    layout: &ProjectLayout,
    skill_set_id: &str,
    package_id: &str,
) -> Result<(), NagareError> {
    let lock_path = layout.root.join("skills-lock.json");
    if !lock_path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(&lock_path)?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)?;
    let mut changed = false;
    if let Some(skills) = value
        .get_mut("skills")
        .and_then(serde_json::Value::as_object_mut)
    {
        changed |= skills.remove(skill_set_id).is_some();
        if package_id != skill_set_id {
            changed |= skills.remove(package_id).is_some();
        }
    }
    if changed {
        fs::write(
            &lock_path,
            format!("{}\n", serde_json::to_string_pretty(&value)?),
        )?;
    }
    Ok(())
}

fn update_skill_package_after_target_uninstall(
    layout: &ProjectLayout,
    package_id: &str,
    skill_set_id: &str,
    package: &SkillPackageDeclaration,
    tool_kind: AgentToolKind,
) -> Result<(), NagareError> {
    let target = external_skill_agent_key(tool_kind);
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let remaining_targets = package
        .installed_targets
        .iter()
        .filter(|installed_target| installed_target.as_str() != target)
        .cloned()
        .collect::<Vec<_>>();
    let remaining_paths = package_installed_paths(package)
        .into_iter()
        .filter(|path| {
            !path_matches_tool_kind(
                layout,
                &skill_package_path_for_uninstall(layout, path),
                tool_kind,
            )
        })
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let skill_packages = root_table
        .get_mut("skill_packages")
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| {
            NagareError::InvalidState("skill_packages must be a TOML table".to_string())
        })?;
    if let Some(package_value) = skill_packages.get_mut(package_id) {
        let package_table = package_value.as_table_mut().ok_or_else(|| {
            NagareError::InvalidState("skill package entry must be a TOML table".to_string())
        })?;
        package_table.insert(
            "installed_targets".to_string(),
            toml::Value::Array(
                remaining_targets
                    .iter()
                    .map(|target| toml::Value::String(target.clone()))
                    .collect(),
            ),
        );
        package_table.insert(
            "installed_paths".to_string(),
            toml::Value::Array(
                remaining_paths
                    .iter()
                    .map(|path| toml::Value::String(path.clone()))
                    .collect(),
            ),
        );
        package_table.insert(
            "installed_path".to_string(),
            toml::Value::String(remaining_paths.first().cloned().unwrap_or_default()),
        );
    }
    if let Some(skill_sets) = root_table
        .get_mut("skill_sets")
        .and_then(toml::Value::as_table_mut)
    {
        if let Some(skill_set_value) = skill_sets.get_mut(skill_set_id) {
            let skill_set_table = skill_set_value.as_table_mut().ok_or_else(|| {
                NagareError::InvalidState("skill set entry must be a TOML table".to_string())
            })?;
            let existing_paths = skill_set_table
                .get("paths")
                .and_then(toml::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(toml::Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let mut remaining_skill_paths = existing_paths
                .into_iter()
                .filter(|path| {
                    !path_matches_tool_kind(
                        layout,
                        &skill_package_path_for_uninstall(layout, path),
                        tool_kind,
                    )
                })
                .collect::<Vec<_>>();
            if remaining_skill_paths.is_empty() {
                remaining_skill_paths = remaining_paths.clone();
            }
            skill_set_table.insert(
                "paths".to_string(),
                toml::Value::Array(
                    remaining_skill_paths
                        .into_iter()
                        .map(toml::Value::String)
                        .collect(),
                ),
            );
        }
    }
    fs::write(&layout.config_path, toml::to_string_pretty(&value)?)?;
    Ok(())
}

fn remove_skill_package_from_project_config(
    layout: &ProjectLayout,
    package_id: &str,
    package: &SkillPackageDeclaration,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let other_provided_skill_sets = value
        .get("skill_packages")
        .and_then(toml::Value::as_table)
        .map(|packages| {
            packages
                .iter()
                .filter(|(id, _)| id.as_str() != package_id)
                .flat_map(|(_, value)| provided_skill_sets_from_value(value))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    if let Some(skill_packages) = root_table
        .get_mut("skill_packages")
        .and_then(toml::Value::as_table_mut)
    {
        skill_packages.remove(package_id);
    }
    if let Some(skill_sets) = root_table
        .get_mut("skill_sets")
        .and_then(toml::Value::as_table_mut)
    {
        for skill_set_id in &package.provided_skill_sets {
            if !other_provided_skill_sets.contains(skill_set_id) {
                skill_sets.remove(skill_set_id);
            }
        }
        if package.provided_skill_sets.is_empty() && !other_provided_skill_sets.contains(package_id)
        {
            skill_sets.remove(package_id);
        }
    }
    fs::write(&layout.config_path, toml::to_string_pretty(&value)?)?;
    Ok(())
}

fn command_exists(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    Command::new(checker)
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn path_for_config(layout: &ProjectLayout, path: &Path) -> String {
    path.strip_prefix(&layout.root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn validate_skill_package_id(id: &str) -> Result<(), NagareError> {
    let id = id.trim();
    let valid = !id.is_empty()
        && id.split(['/', '@']).all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        });
    if !valid {
        return Err(NagareError::InvalidState(format!(
            "skill package id `{id}` contains unsupported characters"
        )));
    }
    Ok(())
}

fn normalize_skill_paths(
    paths: Vec<String>,
    source_paths: Vec<String>,
) -> Result<Vec<String>, NagareError> {
    let mut paths = normalize_specialties(paths);
    if paths.is_empty() {
        paths = normalize_specialties(source_paths)
            .into_iter()
            .map(|path| path.replace('\\', "/"))
            .collect();
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
    skill_md_metadata_at(&PathBuf::from(path))
}

fn skill_md_metadata_at(path: &Path) -> Result<SkillMdMetadata, NagareError> {
    let skill_path = path.join("SKILL.md");
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
    let mut old_provided_skill_sets = value
        .get("skill_packages")
        .and_then(toml::Value::as_table)
        .and_then(|packages| packages.get(package_id))
        .map(provided_skill_sets_from_value)
        .unwrap_or_default();
    if package_id != skill_set_id && !old_provided_skill_sets.iter().any(|id| id == package_id) {
        old_provided_skill_sets.push(package_id.to_string());
    }
    let skill_sets_provided_by_other_packages = value
        .get("skill_packages")
        .and_then(toml::Value::as_table)
        .map(|packages| {
            packages
                .iter()
                .filter(|(id, _)| id.as_str() != package_id)
                .flat_map(|(_, value)| provided_skill_sets_from_value(value))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let skill_sets = root_table
        .entry("skill_sets".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
        .as_table_mut()
        .ok_or_else(|| NagareError::InvalidState("skill_sets must be a TOML table".to_string()))?;
    for old_skill_set_id in old_provided_skill_sets {
        if old_skill_set_id != skill_set_id
            && !package.provided_skill_sets.contains(&old_skill_set_id)
            && !skill_sets_provided_by_other_packages.contains(&old_skill_set_id)
        {
            skill_sets.remove(&old_skill_set_id);
        }
    }
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

fn provided_skill_sets_from_value(value: &toml::Value) -> Vec<String> {
    value
        .get("provided_skill_sets")
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
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
    let domains = load_domains(layout)?;
    let artifact_types = load_artifact_types(layout)?;
    let artifact_type = item
        .artifact_type_id
        .as_deref()
        .and_then(|artifact_type_id| artifact_types.get(artifact_type_id));
    let domain_id = item
        .domain_id
        .as_deref()
        .or_else(|| artifact_type.and_then(|artifact_type| artifact_type.domain_id.as_deref()));
    let domain = domain_id.and_then(|domain_id| domains.get(domain_id));
    if domain.is_none() && artifact_type.is_none() {
        return Ok(String::new());
    }
    let mut lines = Vec::new();
    let mut has_rubric = false;
    if let Some(group) = domain {
        lines.push(format!(
            "{}: {} ({})",
            i18n.ui(UiTextKey::Domain),
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
        has_rubric |= !group.common_rubric.is_empty();
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
    if let Some(artifact_type) = artifact_type {
        lines.push(format!(
            "{}: {} ({})",
            i18n.ui(UiTextKey::ArtifactType),
            artifact_type.display_name,
            artifact_type.id
        ));
        if !artifact_type.description.trim().is_empty() {
            lines.push(format!(
                "{} {}: {}",
                i18n.ui(UiTextKey::ArtifactType),
                i18n.ui(UiTextKey::Description).to_ascii_lowercase(),
                artifact_type.description
            ));
        }
        append_context_list(&mut lines, "Artifact types", &artifact_type.artifact_types);
        append_context_list(
            &mut lines,
            &format!(
                "{} {}",
                i18n.ui(UiTextKey::ArtifactType),
                i18n.ui(UiTextKey::Rubric)
            ),
            &artifact_type.rubric,
        );
        has_rubric |= !artifact_type.rubric.is_empty();
        append_context_list(
            &mut lines,
            &format!(
                "{} {}",
                i18n.ui(UiTextKey::ArtifactType),
                i18n.ui(UiTextKey::DispatchHints)
            ),
            &artifact_type.dispatch_hints,
        );
    }
    if has_rubric {
        lines.push("Review scoring policy: Evaluate against the domain rubric on a 100-point scale. Follow Nagare's common Review Policy for any next-agent handling; do not invent domain-specific follow-up rules.".to_string());
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

    if matches!(
        input.purpose,
        AgentRunPurpose::Work | AgentRunPurpose::Synthesis
    ) {
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
    let mut item_status = if matches!(
        input.purpose,
        AgentRunPurpose::Work | AgentRunPurpose::Synthesis
    ) {
        if agent_output_requires_input(agent_output.as_ref()) {
            WorkItemStatus::NeedsInput
        } else if agent_output_requests_handoff(agent_output.as_ref()) {
            WorkItemStatus::NeedsHandoff
        } else if input.purpose == AgentRunPurpose::Synthesis && status == AgentRunStatus::Succeeded
        {
            WorkItemStatus::ReadyForReview
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
        AgentRunPurpose::Work
            | AgentRunPurpose::Synthesis
            | AgentRunPurpose::Review
            | AgentRunPurpose::DispatchPreview
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
    let domain = item.artifact_type_id.as_deref().unwrap_or("-");
    let group = item.domain_id.as_deref().unwrap_or("-");
    Some(DomainFallbackConfirmation {
        target_agent_profile_id: fallback_agent.clone(),
        message: format!(
            "No candidate agent is scoped to domain `{domain}` or Domain `{group}`; confirm whether to proceed with general fallback agent `{fallback_agent}`."
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
    let domain = item.artifact_type_id.as_deref().unwrap_or("-");
    let group = item.domain_id.as_deref().unwrap_or("-");
    Some(DomainFallbackConfirmation {
        target_agent_profile_id: fallback_agent.clone(),
        message: format!(
            "Domain-scoped agent is required for domain `{domain}` or Domain `{group}`, but no candidate agent is scoped to it; add a matching agent or change the domain agent policy before proceeding."
        ),
    })
}

fn domain_agent_missing(item: &WorkItem, candidates: &BTreeMap<String, AgentProfile>) -> bool {
    if item.artifact_type_id.is_none() && item.domain_id.is_none() {
        return false;
    }
    if item.artifact_type_id.as_deref() == Some("general")
        || item.domain_id.as_deref() == Some("general")
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
        .artifact_type_ids
        .iter()
        .any(|artifact_type_id| artifact_type_id == "general")
        || profile
            .domain_ids
            .iter()
            .any(|domain_id| domain_id == "general")
}

fn agent_matches_item_domain(profile: &AgentProfile, item: &WorkItem) -> bool {
    item.artifact_type_id
        .as_deref()
        .is_some_and(|artifact_type_id| {
            profile
                .artifact_type_ids
                .iter()
                .any(|profile_artifact_type_id| profile_artifact_type_id == artifact_type_id)
        })
        || item.domain_id.as_deref().is_some_and(|domain_id| {
            profile
                .domain_ids
                .iter()
                .any(|profile_domain_id| profile_domain_id == domain_id)
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
