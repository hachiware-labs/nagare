use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::*;

pub(crate) fn ensure_project(root: impl Into<PathBuf>) -> Result<ProjectLayout, NagareError> {
    let root = root.into();
    let layout = ProjectLayout::new(root);
    if !layout.config_path.exists() || !layout.ledger_path.exists() {
        init_project(&layout.root)?;
    }
    migrate_legacy_default_agents(&layout)?;
    Ok(layout)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentProfileFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_profile: Option<AgentProfileFileEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) agent_profiles: BTreeMap<String, AgentProfileFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DomainProfileFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) domain_profile: Option<DomainProfileFileEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) domain_profiles: BTreeMap<String, DomainProfileFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DomainGroupFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) domain_group: Option<DomainGroupFileEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) domain_groups: BTreeMap<String, DomainGroupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectConfigFile {
    #[serde(default)]
    pub(crate) locale: LocaleSettings,
    #[serde(default)]
    pub(crate) workflow: WorkflowSettings,
    #[serde(default)]
    pub(crate) nagare_agents: NagareAgentSettings,
    #[serde(default)]
    pub(crate) runtimes: BTreeMap<String, RuntimeDeclaration>,
    #[serde(default)]
    pub(crate) skill_sets: BTreeMap<String, SkillSetDeclaration>,
    #[serde(default)]
    pub(crate) permission_policies: BTreeMap<String, PermissionPolicyDeclaration>,
    #[serde(default)]
    pub(crate) workspace_policies: BTreeMap<String, WorkspacePolicyDeclaration>,
    #[serde(default)]
    pub(crate) project_rules: Vec<ProjectRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AgentProfileFileEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    pub(crate) display_name: String,
    pub(crate) runtime: String,
    pub(crate) adapter: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) role: String,
    #[serde(default = "default_working_dir")]
    pub(crate) working_dir: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) specialties: Vec<String>,
    #[serde(default)]
    pub(crate) domain_group_ids: Vec<String>,
    #[serde(default)]
    pub(crate) domain_ids: Vec<String>,
    #[serde(default)]
    pub(crate) output_contracts: AgentOutputContracts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DomainProfileFileEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) group_id: Option<String>,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) artifact_types: Vec<String>,
    #[serde(default)]
    pub(crate) rubric: Vec<String>,
    #[serde(default)]
    pub(crate) dispatch_hints: Vec<String>,
    #[serde(default)]
    pub(crate) workflow: DomainWorkflowOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DomainGroupFileEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) shared_knowledge: Vec<String>,
    #[serde(default)]
    pub(crate) common_rubric: Vec<String>,
    #[serde(default)]
    pub(crate) dispatch_hints: Vec<String>,
    #[serde(default)]
    pub(crate) workflow: DomainWorkflowOverride,
}

pub(crate) fn load_agent_profiles(
    layout: &ProjectLayout,
) -> Result<BTreeMap<String, AgentProfile>, NagareError> {
    let mut profiles = BTreeMap::new();
    if layout.config_path.exists() {
        let raw = fs::read_to_string(&layout.config_path)?;
        merge_agent_profiles_from_toml(
            &mut profiles,
            &raw,
            AgentProfileSource::ProjectConfig,
            "project.toml",
        )?;
    }

    if layout.agents_dir.exists() {
        let mut paths = fs::read_dir(&layout.agents_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let raw = fs::read_to_string(&path)?;
            merge_agent_profiles_from_toml(
                &mut profiles,
                &raw,
                AgentProfileSource::ProjectAgentDirectory,
                &path.display().to_string(),
            )?;
        }
    }

    Ok(profiles)
}

pub(crate) fn merge_agent_profiles_from_toml(
    profiles: &mut BTreeMap<String, AgentProfile>,
    raw: &str,
    source: AgentProfileSource,
    source_name: &str,
) -> Result<(), NagareError> {
    let document: AgentProfileFile = toml::from_str(raw)?;
    if let Some(entry) = document.agent_profile {
        let id = entry.id.clone().ok_or_else(|| {
            NagareError::InvalidState(format!("`agent_profile.id` is required in {source_name}"))
        })?;
        profiles.insert(id.clone(), entry.into_profile(id, source)?);
    }
    for (id, entry) in document.agent_profiles {
        let profile_id = entry.id.clone().unwrap_or(id);
        profiles.insert(profile_id.clone(), entry.into_profile(profile_id, source)?);
    }
    Ok(())
}

pub(crate) fn load_domain_profiles(
    layout: &ProjectLayout,
) -> Result<BTreeMap<String, DomainProfile>, NagareError> {
    let mut domains = BTreeMap::new();
    if layout.domains_dir.exists() {
        let mut paths = fs::read_dir(&layout.domains_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let raw = fs::read_to_string(&path)?;
            merge_domain_profiles_from_toml(
                &mut domains,
                &raw,
                DomainProfileSource::ProjectDomainDirectory,
                &path.display().to_string(),
            )?;
        }
    }
    Ok(domains)
}

pub(crate) fn load_domain_groups(
    layout: &ProjectLayout,
) -> Result<BTreeMap<String, DomainGroup>, NagareError> {
    let mut groups = BTreeMap::new();
    if layout.domain_groups_dir.exists() {
        let mut paths = fs::read_dir(&layout.domain_groups_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            let raw = fs::read_to_string(&path)?;
            merge_domain_groups_from_toml(
                &mut groups,
                &raw,
                DomainGroupSource::ProjectDomainGroupDirectory,
                &path.display().to_string(),
            )?;
        }
    }
    Ok(groups)
}

pub(crate) fn merge_domain_groups_from_toml(
    groups: &mut BTreeMap<String, DomainGroup>,
    raw: &str,
    source: DomainGroupSource,
    source_name: &str,
) -> Result<(), NagareError> {
    let document: DomainGroupFile = toml::from_str(raw)?;
    if let Some(entry) = document.domain_group {
        let id = entry.id.clone().ok_or_else(|| {
            NagareError::InvalidState(format!("`domain_group.id` is required in {source_name}"))
        })?;
        groups.insert(id.clone(), entry.into_group(id, source)?);
    }
    for (id, entry) in document.domain_groups {
        let group_id = entry.id.clone().unwrap_or(id);
        groups.insert(group_id.clone(), entry.into_group(group_id, source)?);
    }
    Ok(())
}

pub(crate) fn merge_domain_profiles_from_toml(
    domains: &mut BTreeMap<String, DomainProfile>,
    raw: &str,
    source: DomainProfileSource,
    source_name: &str,
) -> Result<(), NagareError> {
    let document: DomainProfileFile = toml::from_str(raw)?;
    if let Some(entry) = document.domain_profile {
        let id = entry.id.clone().ok_or_else(|| {
            NagareError::InvalidState(format!("`domain_profile.id` is required in {source_name}"))
        })?;
        domains.insert(id.clone(), entry.into_domain(id, source)?);
    }
    for (id, entry) in document.domain_profiles {
        let domain_id = entry.id.clone().unwrap_or(id);
        domains.insert(domain_id.clone(), entry.into_domain(domain_id, source)?);
    }
    Ok(())
}

impl AgentProfileFileEntry {
    fn into_profile(
        self,
        id: String,
        source: AgentProfileSource,
    ) -> Result<AgentProfile, NagareError> {
        validate_agent_profile_id(&id)?;
        let adapter = normalize_adapter_id(&self.adapter)?;
        Ok(AgentProfile {
            id,
            display_name: self.display_name,
            runtime: self.runtime,
            adapter: adapter.to_string(),
            role: self.role,
            working_dir: normalize_working_dir(&self.working_dir)?,
            description: self.description,
            specialties: normalize_specialties(self.specialties),
            domain_group_ids: normalize_domain_group_ids(self.domain_group_ids)?,
            domain_ids: normalize_domain_profile_ids(self.domain_ids)?,
            output_contracts: self.output_contracts,
            source,
        })
    }
}

impl DomainProfileFileEntry {
    fn into_domain(
        self,
        id: String,
        source: DomainProfileSource,
    ) -> Result<DomainProfile, NagareError> {
        validate_domain_profile_id(&id)?;
        if let Some(group_id) = self.group_id.as_deref() {
            validate_domain_group_id(group_id)?;
        }
        Ok(DomainProfile {
            id,
            group_id: self.group_id.filter(|value| !value.trim().is_empty()),
            display_name: self.display_name,
            description: self.description.trim().to_string(),
            artifact_types: normalize_specialties(self.artifact_types),
            rubric: normalize_specialties(self.rubric),
            dispatch_hints: normalize_specialties(self.dispatch_hints),
            workflow: self.workflow,
            source,
        })
    }
}

impl DomainGroupFileEntry {
    fn into_group(self, id: String, source: DomainGroupSource) -> Result<DomainGroup, NagareError> {
        validate_domain_group_id(&id)?;
        Ok(DomainGroup {
            id,
            display_name: self.display_name,
            description: self.description.trim().to_string(),
            shared_knowledge: normalize_specialties(self.shared_knowledge),
            common_rubric: normalize_specialties(self.common_rubric),
            dispatch_hints: normalize_specialties(self.dispatch_hints),
            workflow: self.workflow,
            source,
        })
    }
}

fn migrate_legacy_default_agents(layout: &ProjectLayout) -> Result<(), NagareError> {
    if !layout.config_path.exists() {
        return Ok(());
    }
    let mut raw = fs::read_to_string(&layout.config_path)?;
    let has_workflow_agents = raw.contains("[agent_profiles.worker]")
        && raw.contains("[agent_profiles.reviewer]")
        && raw.contains("[agent_profiles.dispatcher]")
        && raw.contains("[agent_profiles.supervisor]");
    if !has_workflow_agents {
        if !raw.contains("[agent_profiles.codex-cli]")
            || !raw.contains("[agent_profiles.codex-app-server]")
        {
            return Ok(());
        }

        raw = raw
            .replace(
                "work_agent = \"codex-cli\"\nreview_agent = \"codex-app-server\"\ndispatch_agent = \"codex-cli\"\nsupervisor_agent = \"codex-cli\"",
                "work_agent = \"worker\"\nreview_agent = \"reviewer\"\ndispatch_agent = \"dispatcher\"\nsupervisor_agent = \"supervisor\"",
            )
            .replace(
                "[agent_profiles.codex-cli]\ndisplay_name = \"Codex CLI Implementer\"\nruntime = \"codex-local\"\nadapter = \"process-codex-cli\"\nrole = \"implementer\"\nworking_dir = \".\"\n\n[agent_profiles.codex-app-server]\ndisplay_name = \"Codex App Server Implementer\"\nruntime = \"codex-app-local\"\nadapter = \"stdio-codex-app-server\"\nrole = \"implementer\"\nworking_dir = \".\"",
                "[agent_profiles.worker]\ndisplay_name = \"Worker\"\nruntime = \"codex-local\"\nadapter = \"process-codex-cli\"\nrole = \"worker\"\nworking_dir = \".\"\ndescription = \"Implement the assigned work item. Prefer small, verifiable changes and leave concise completed work and next notes in the Nagare result.\"\n\n[agent_profiles.reviewer]\ndisplay_name = \"Reviewer\"\nruntime = \"codex-local\"\nadapter = \"process-codex-cli\"\nrole = \"reviewer\"\nworking_dir = \".\"\ndescription = \"Review the current work item against acceptance criteria, artifacts, and test evidence. Report pass/fail per criterion and concrete follow-up notes.\"\n\n[agent_profiles.dispatcher]\ndisplay_name = \"Dispatcher\"\nruntime = \"codex-local\"\nadapter = \"process-codex-cli\"\nrole = \"dispatcher\"\nworking_dir = \".\"\ndescription = \"Choose the most suitable target agent profile for the next work step. Return only the required dispatch JSON and keep the rationale concise.\"\n\n[agent_profiles.supervisor]\ndisplay_name = \"Supervisor\"\nruntime = \"codex-local\"\nadapter = \"process-codex-cli\"\nrole = \"supervisor\"\nworking_dir = \".\"\ndescription = \"Decide the next workflow action from the current state. Prefer forward progress, stop when human input is needed, and return the workflow decision contract.\"",
            );
    }
    raw = ensure_default_agent_role(raw, "worker", "worker");
    raw = ensure_default_agent_role(raw, "reviewer", "reviewer");
    raw = ensure_default_agent_role(raw, "dispatcher", "dispatcher");
    raw = ensure_default_agent_role(raw, "supervisor", "supervisor");
    raw = ensure_default_agent_instruction(
        raw,
        "worker",
        "Implement the assigned work item. Prefer small, verifiable changes and leave concise completed work and next notes in the Nagare result.",
    );
    raw = ensure_default_agent_instruction(
        raw,
        "reviewer",
        "Review the current work item against acceptance criteria, artifacts, and test evidence. Report pass/fail per criterion and concrete follow-up notes.",
    );
    raw = ensure_default_agent_instruction(
        raw,
        "dispatcher",
        "Choose the most suitable target agent profile for the next work step. Return only the required dispatch JSON and keep the rationale concise.",
    );
    raw = ensure_default_agent_instruction(
        raw,
        "supervisor",
        "Decide the next workflow action from the current state. Prefer forward progress, stop when human input is needed, and return the workflow decision contract.",
    );
    fs::write(&layout.config_path, raw)?;
    Ok(())
}

fn ensure_default_agent_role(mut raw: String, agent_id: &str, role: &str) -> String {
    let header = format!("[agent_profiles.{agent_id}]");
    let Some(section_start) = raw.find(&header) else {
        return raw;
    };
    let section_body_start = section_start + header.len();
    let next_section = raw[section_body_start..]
        .find("\n[")
        .map(|index| section_body_start + index)
        .unwrap_or(raw.len());
    if raw[section_body_start..next_section].contains("\nrole = ") {
        return raw;
    }
    let insert_at = next_section;
    raw.insert_str(insert_at, &format!("\nrole = \"{role}\""));
    raw
}

fn ensure_default_agent_instruction(mut raw: String, agent_id: &str, instruction: &str) -> String {
    let header = format!("[agent_profiles.{agent_id}]");
    let Some(section_start) = raw.find(&header) else {
        return raw;
    };
    let section_body_start = section_start + header.len();
    let next_section = raw[section_body_start..]
        .find("\n[")
        .map(|index| section_body_start + index)
        .unwrap_or(raw.len());
    if raw[section_body_start..next_section].contains("\ndescription = ") {
        return raw;
    }
    let insert_at = next_section;
    let escaped = instruction.replace('\\', "\\\\").replace('"', "\\\"");
    raw.insert_str(insert_at, &format!("\ndescription = \"{escaped}\""));
    raw
}

pub(crate) fn default_working_dir() -> String {
    ".".to_string()
}

pub(crate) fn normalize_specialties(specialties: Vec<String>) -> Vec<String> {
    specialties
        .into_iter()
        .map(|specialty| specialty.trim().to_string())
        .filter(|specialty| !specialty.is_empty())
        .collect()
}

pub(crate) fn get_agent_profile_from_layout(
    layout: &ProjectLayout,
    agent_profile_id: &str,
) -> Result<AgentProfile, NagareError> {
    load_agent_profiles(layout)?
        .remove(agent_profile_id)
        .ok_or_else(|| NagareError::NotFound(format!("agent profile `{agent_profile_id}`")))
}

pub(crate) fn get_runtime_declaration(
    layout: &ProjectLayout,
    runtime_id: &str,
) -> Result<RuntimeDeclaration, NagareError> {
    let document = load_project_config(layout)?;
    document
        .runtimes
        .get(runtime_id)
        .cloned()
        .ok_or_else(|| NagareError::NotFound(format!("runtime `{runtime_id}`")))
}

pub(crate) fn load_project_config(
    layout: &ProjectLayout,
) -> Result<ProjectConfigFile, NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    Ok(toml::from_str(&raw)?)
}

pub(crate) fn save_workflow_settings(
    layout: &ProjectLayout,
    settings: WorkflowSettings,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let rendered = render_workflow_settings_toml(settings);
    let updated = replace_or_append_table(&raw, "workflow", &rendered);
    fs::write(&layout.config_path, updated)?;
    Ok(())
}

fn render_workflow_settings_toml(settings: WorkflowSettings) -> String {
    format!(
        "[workflow]\ndefault_progress_mode = \"{}\"\napproval_policy = \"{}\"\n",
        settings.default_progress_mode, settings.approval_policy
    )
}

fn replace_or_append_table(raw: &str, table: &str, replacement: &str) -> String {
    let header = format!("[{table}]");
    let lines = raw.lines().collect::<Vec<_>>();
    let Some(start) = lines.iter().position(|line| line.trim() == header) else {
        let mut updated = raw.trim_end().to_string();
        if !updated.is_empty() {
            updated.push_str("\n\n");
        }
        updated.push_str(replacement.trim_end());
        updated.push('\n');
        return updated;
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| {
            let trimmed = line.trim();
            trimmed.starts_with('[') && trimmed.ends_with(']')
        })
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    let mut updated = Vec::new();
    updated.extend_from_slice(&lines[..start]);
    updated.extend(replacement.trim_end().lines());
    updated.extend_from_slice(&lines[end..]);
    let mut result = updated.join("\n");
    result.push('\n');
    result
}

pub(crate) fn resolve_rule_for_path_from_layout(
    layout: &ProjectLayout,
    path: Option<&str>,
    agent_override: Option<&str>,
) -> Result<RuleResolution, NagareError> {
    let config = load_project_config(layout)?;
    let path = path
        .map(normalize_rule_path)
        .filter(|value| !value.trim().is_empty());
    let matched_rule = match path.as_deref() {
        Some(path) => best_matching_project_rule(&config.project_rules, path)?,
        None => None,
    };

    let agent_profile_id = agent_override
        .filter(|agent| !agent.trim().is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| matched_rule.and_then(|rule| rule.default_agent.clone()))
        .unwrap_or_else(|| config.nagare_agents.work_agent.clone());
    let review_agent_profile_id = matched_rule
        .and_then(|rule| rule.review_agent.clone())
        .unwrap_or_else(|| config.nagare_agents.review_agent.clone());

    validate_existing_agent_profile(layout, &agent_profile_id)?;
    validate_existing_agent_profile(layout, &review_agent_profile_id)?;

    let mut warnings = Vec::new();
    let skill_set_ids = matched_rule
        .map(|rule| rule.skill_sets.clone())
        .unwrap_or_default();
    for skill_set_id in &skill_set_ids {
        if !config.skill_sets.contains_key(skill_set_id) {
            warnings.push(format!(
                "skill set `{skill_set_id}` is referenced but not declared"
            ));
        }
    }

    let permission_policy_id = matched_rule.and_then(|rule| rule.permission_policy.clone());
    if let Some(policy_id) = &permission_policy_id {
        if !config.permission_policies.contains_key(policy_id) {
            warnings.push(format!(
                "permission policy `{policy_id}` is referenced but not declared"
            ));
        }
    }

    let workspace_policy_id = matched_rule.and_then(|rule| rule.workspace_policy.clone());
    if let Some(policy_id) = &workspace_policy_id {
        if !config.workspace_policies.contains_key(policy_id) {
            warnings.push(format!(
                "workspace policy `{policy_id}` is referenced but not declared"
            ));
        }
    }

    Ok(RuleResolution {
        path,
        matched_rule_id: matched_rule.map(|rule| rule.id.clone()),
        agent_profile_id,
        review_agent_profile_id,
        skill_set_ids,
        permission_policy_id,
        workspace_policy_id,
        warnings,
    })
}

pub(crate) fn best_matching_project_rule<'a>(
    rules: &'a [ProjectRule],
    target_path: &str,
) -> Result<Option<&'a ProjectRule>, NagareError> {
    let mut best: Option<(&ProjectRule, usize)> = None;
    for rule in rules {
        let Some(score) = rule_match_score(rule, target_path) else {
            continue;
        };
        match best {
            Some((current, current_score)) if score == current_score && current.id != rule.id => {
                return Err(NagareError::InvalidState(format!(
                    "project rules `{}` and `{}` both match `{target_path}` with equal specificity",
                    current.id, rule.id
                )));
            }
            Some((_, current_score)) if score <= current_score => {}
            _ => best = Some((rule, score)),
        }
    }
    Ok(best.map(|(rule, _)| rule))
}

pub(crate) fn rule_match_score(rule: &ProjectRule, target_path: &str) -> Option<usize> {
    rule.match_patterns
        .iter()
        .filter_map(|pattern| pattern_match_score(pattern, target_path))
        .max()
}

pub(crate) fn pattern_match_score(pattern: &str, target_path: &str) -> Option<usize> {
    let pattern = normalize_rule_path(pattern);
    if pattern.is_empty() {
        return None;
    }
    if pattern == "**" || pattern == "*" {
        return Some(0);
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path_has_prefix(target_path, prefix).then_some(prefix.len());
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return direct_child_path(target_path, prefix).then_some(prefix.len());
    }
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        let matches = target_path.starts_with(prefix) && target_path.ends_with(suffix);
        return matches.then_some(prefix.len() + suffix.len());
    }
    (target_path == pattern || path_has_prefix(target_path, &pattern)).then_some(pattern.len())
}

pub(crate) fn path_has_prefix(target_path: &str, prefix: &str) -> bool {
    target_path == prefix || target_path.starts_with(&format!("{prefix}/"))
}

pub(crate) fn direct_child_path(target_path: &str, prefix: &str) -> bool {
    if !path_has_prefix(target_path, prefix) {
        return false;
    }
    let remainder = target_path
        .strip_prefix(prefix)
        .unwrap_or_default()
        .trim_start_matches('/');
    !remainder.is_empty() && !remainder.contains('/')
}

pub(crate) fn normalize_rule_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

pub(crate) fn write_nagare_agent_settings(
    layout: &ProjectLayout,
    settings: &NagareAgentSettings,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let settings_value = toml::Value::try_from(settings.clone())?;
    root_table.insert("nagare_agents".to_string(), settings_value);
    let rendered = toml::to_string_pretty(&value)?;
    fs::write(&layout.config_path, rendered)?;
    Ok(())
}

pub(crate) fn write_locale_settings(
    layout: &ProjectLayout,
    settings: &LocaleSettings,
) -> Result<(), NagareError> {
    let raw = fs::read_to_string(&layout.config_path)?;
    let mut value = raw.parse::<toml::Value>()?;
    let root_table = value.as_table_mut().ok_or_else(|| {
        NagareError::InvalidState("project config must be a TOML table".to_string())
    })?;
    let settings_value = toml::Value::try_from(settings.clone())?;
    root_table.insert("locale".to_string(), settings_value);
    let rendered = toml::to_string_pretty(&value)?;
    fs::write(&layout.config_path, rendered)?;
    Ok(())
}

pub(crate) fn validate_existing_agent_profile(
    layout: &ProjectLayout,
    agent_profile_id: &str,
) -> Result<(), NagareError> {
    get_agent_profile_from_layout(layout, agent_profile_id).map(|_| ())
}

pub(crate) fn validate_existing_domain_group(
    layout: &ProjectLayout,
    domain_group_id: &str,
) -> Result<(), NagareError> {
    load_domain_groups(layout)?
        .get(domain_group_id)
        .map(|_| ())
        .ok_or_else(|| NagareError::NotFound(format!("domain group `{domain_group_id}`")))
}

pub(crate) fn validate_existing_domain_group_ids(
    layout: &ProjectLayout,
    domain_group_ids: &[String],
) -> Result<(), NagareError> {
    let groups = load_domain_groups(layout)?;
    for domain_group_id in domain_group_ids {
        if !groups.contains_key(domain_group_id) {
            return Err(NagareError::NotFound(format!(
                "domain group `{domain_group_id}`"
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_existing_domain_profile_ids(
    layout: &ProjectLayout,
    domain_ids: &[String],
) -> Result<(), NagareError> {
    let domains = load_domain_profiles(layout)?;
    for domain_id in domain_ids {
        if !domains.contains_key(domain_id) {
            return Err(NagareError::NotFound(format!(
                "domain profile `{domain_id}`"
            )));
        }
    }
    Ok(())
}

pub(crate) fn latest_capability_probe<'a>(
    ledger: &'a Ledger,
    agent_profile_id: &str,
) -> Option<&'a CapabilityProbe> {
    ledger
        .capability_probes
        .iter()
        .rev()
        .find(|probe| probe.agent_profile_id == agent_profile_id)
}

pub(crate) fn default_probe_ttl_seconds() -> u64 {
    24 * 60 * 60
}

pub(crate) fn is_capability_probe_fresh(
    probe: &CapabilityProbe,
    profile: &AgentProfile,
    runtime_version: &str,
    now_seconds: u64,
) -> Result<bool, NagareError> {
    let adapter_id = normalize_adapter_id(&profile.adapter)?;
    if probe.runtime_id != profile.runtime || probe.adapter_id != adapter_id {
        return Ok(false);
    }
    if probe.runtime_version != runtime_version {
        return Ok(false);
    }
    let probed_at = probe.probed_at.parse::<u64>().map_err(|_| {
        NagareError::InvalidState(format!("invalid probed_at `{}`", probe.probed_at))
    })?;
    Ok(now_seconds.saturating_sub(probed_at) <= default_probe_ttl_seconds())
}

pub(crate) fn resolve_skill_sets_for_run(
    layout: &ProjectLayout,
    skill_set_ids: &[String],
    capabilities_in_force: &[String],
) -> Result<SkillSetResolution, NagareError> {
    let config = load_project_config(layout)?;
    let mut applied_skill_set_ids = Vec::new();
    let mut skipped_skill_set_ids = Vec::new();
    let mut warnings = Vec::new();

    for skill_set_id in skill_set_ids {
        let Some(skill_set) = config.skill_sets.get(skill_set_id) else {
            skipped_skill_set_ids.push(skill_set_id.clone());
            warnings.push(format!(
                "skill set `{skill_set_id}` was skipped because it is not declared"
            ));
            continue;
        };
        let missing_required = skill_set
            .required_capabilities
            .iter()
            .filter(|capability| !capabilities_in_force.contains(capability))
            .cloned()
            .collect::<Vec<_>>();
        if missing_required.is_empty() {
            applied_skill_set_ids.push(skill_set_id.clone());
        } else {
            skipped_skill_set_ids.push(skill_set_id.clone());
            warnings.push(format!(
                "skill set `{skill_set_id}` was skipped because required capabilities are missing: {}",
                missing_required.join(",")
            ));
        }
    }

    Ok(SkillSetResolution {
        declared_skill_set_ids: skill_set_ids.to_vec(),
        applied_skill_set_ids,
        skipped_skill_set_ids,
        warnings,
    })
}

pub(crate) fn default_work_agent_id() -> String {
    "worker".to_string()
}

pub(crate) fn default_review_agent_id() -> String {
    "reviewer".to_string()
}

pub(crate) fn default_dispatch_agent_id() -> String {
    "dispatcher".to_string()
}

pub(crate) fn default_supervisor_agent_id() -> String {
    "supervisor".to_string()
}

pub(crate) fn default_agent_run_purpose() -> AgentRunPurpose {
    AgentRunPurpose::Work
}

pub(crate) fn default_locale_language() -> String {
    env::var("NAGARE_LOCALE").unwrap_or_else(|_| "ja-JP".to_string())
}

pub(crate) fn default_locale_timezone() -> String {
    env::var("NAGARE_TIMEZONE").unwrap_or_else(|_| "Asia/Tokyo".to_string())
}

pub(crate) fn default_workspace_policy_kind() -> String {
    "project_root".to_string()
}

pub(crate) fn default_workspace_policy_cleanup() -> String {
    "keep".to_string()
}

pub(crate) fn validate_locale_language(language: &str) -> Result<(), NagareError> {
    if language.trim().is_empty()
        || !language
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "locale language `{language}` must use letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_timezone(timezone: &str) -> Result<(), NagareError> {
    if timezone.trim().is_empty()
        || timezone
            .chars()
            .any(|ch| ch.is_control() || ch == '\\' || ch == '"')
    {
        return Err(NagareError::InvalidState(format!(
            "timezone `{timezone}` is not valid"
        )));
    }
    Ok(())
}

pub(crate) fn runtime_healthcheck(runtime: &RuntimeDeclaration) -> ToolStatus {
    match runtime.healthcheck.split_first() {
        Some((command, args)) => check_command(command, args),
        None => check_command(&runtime.command, &runtime.args),
    }
}

pub(crate) fn validate_agent_profile_id(id: &str) -> Result<(), NagareError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "agent profile id `{id}` must use only ASCII letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_domain_profile_id(id: &str) -> Result<(), NagareError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "domain profile id `{id}` must use only ASCII letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_domain_group_id(id: &str) -> Result<(), NagareError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(NagareError::InvalidState(format!(
            "domain group id `{id}` must use only ASCII letters, numbers, '-' or '_'"
        )));
    }
    Ok(())
}

pub(crate) fn normalize_domain_group_ids(ids: Vec<String>) -> Result<Vec<String>, NagareError> {
    let mut normalized = Vec::new();
    for id in ids {
        let id = id.trim();
        if id.is_empty() || normalized.iter().any(|existing| existing == id) {
            continue;
        }
        validate_domain_group_id(id)?;
        normalized.push(id.to_string());
    }
    Ok(normalized)
}

pub(crate) fn normalize_domain_profile_ids(ids: Vec<String>) -> Result<Vec<String>, NagareError> {
    let mut normalized = Vec::new();
    for id in ids {
        let id = id.trim();
        if id.is_empty() || normalized.iter().any(|existing| existing == id) {
            continue;
        }
        validate_domain_profile_id(id)?;
        normalized.push(id.to_string());
    }
    Ok(normalized)
}

pub(crate) fn normalize_working_dir(working_dir: &str) -> Result<String, NagareError> {
    let value = working_dir.trim();
    if value.is_empty() || value == "." {
        return Ok(".".to_string());
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(NagareError::InvalidState(format!(
            "working_dir `{working_dir}` must be a relative path inside the project"
        )));
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn resolve_profile_working_dir(
    layout: &ProjectLayout,
    profile: &AgentProfile,
) -> Result<PathBuf, NagareError> {
    let normalized = normalize_working_dir(&profile.working_dir)?;
    let path = if normalized == "." {
        layout.root.clone()
    } else {
        layout.root.join(&normalized)
    };
    if !path.is_dir() {
        return Err(NagareError::InvalidState(format!(
            "working_dir `{}` for agent profile `{}` does not exist or is not a directory",
            profile.working_dir, profile.id
        )));
    }
    Ok(path)
}

pub(crate) fn capabilities_for_adapter(adapter_id: &str) -> Result<Vec<String>, NagareError> {
    let capabilities = match normalize_adapter_id(adapter_id)? {
        "process.codex-cli" => vec!["repo_read", "file_edit", "shell_command", "stdin_prompt"],
        "stdio.codex-app-server" => vec![
            "repo_read",
            "file_edit",
            "shell_command",
            "thread_state",
            "approval_flow",
            "event_stream",
        ],
        _ => unreachable!("normalize_adapter_id returned an unknown adapter"),
    };
    Ok(capabilities.into_iter().map(ToOwned::to_owned).collect())
}

pub(crate) fn skill_modes_for_adapter(adapter_id: &str) -> Result<Vec<String>, NagareError> {
    let modes = match normalize_adapter_id(adapter_id)? {
        "process.codex-cli" => vec!["prompt_injection", "file_reference"],
        "stdio.codex-app-server" => vec!["prompt_injection", "file_reference", "event_stream"],
        _ => unreachable!("normalize_adapter_id returned an unknown adapter"),
    };
    Ok(modes.into_iter().map(ToOwned::to_owned).collect())
}

pub(crate) fn instruction_sources(layout: &ProjectLayout) -> Vec<String> {
    ["AGENTS.md", ".codex/config.toml"]
        .iter()
        .filter(|source| layout.root.join(source).exists())
        .map(|source| source.to_string())
        .collect()
}
