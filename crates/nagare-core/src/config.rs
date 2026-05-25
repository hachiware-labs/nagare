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
pub(crate) struct ProjectConfigFile {
    #[serde(default)]
    pub(crate) locale: LocaleSettings,
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
    pub(crate) role: String,
    #[serde(default = "default_working_dir")]
    pub(crate) working_dir: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) specialties: Vec<String>,
    #[serde(default)]
    pub(crate) output_contracts: AgentOutputContracts,
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
            output_contracts: self.output_contracts,
            source,
        })
    }
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
        verification: matched_rule
            .map(|rule| rule.verification.clone())
            .unwrap_or_default(),
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
    "codex-cli".to_string()
}

pub(crate) fn default_review_agent_id() -> String {
    "codex-app-server".to_string()
}

pub(crate) fn default_dispatch_agent_id() -> String {
    "codex-cli".to_string()
}

pub(crate) fn default_supervisor_agent_id() -> String {
    "codex-cli".to_string()
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
