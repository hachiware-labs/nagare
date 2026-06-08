pub(crate) fn agent_label(profiles: &[nagare_core::AgentProfile], agent_id: &str) -> String {
    profiles
        .iter()
        .find(|profile| profile.id == agent_id)
        .map(|profile| {
            if profile.display_name.trim().is_empty() || profile.display_name == profile.id {
                profile.id.clone()
            } else {
                format!("{} ({})", profile.display_name, profile.id)
            }
        })
        .unwrap_or_else(|| agent_id.to_string())
}

pub(crate) fn agent_meta(profiles: &[nagare_core::AgentProfile], agent_id: &str) -> String {
    profiles
        .iter()
        .find(|profile| profile.id == agent_id)
        .map(|profile| {
            let model = profile
                .model
                .model_ref()
                .unwrap_or_else(|| "model未設定".to_string());
            format!("{} / {}", tool_label(profile.tool_kind), model)
        })
        .unwrap_or_else(|| "Agent Profile details are not available.".to_string())
}

pub(crate) fn agent_label_with_meta(
    profiles: &[nagare_core::AgentProfile],
    agent_id: &str,
) -> String {
    format!(
        "{} - {}",
        agent_label(profiles, agent_id),
        agent_meta(profiles, agent_id)
    )
}

fn tool_label(tool_kind: nagare_core::AgentToolKind) -> &'static str {
    match tool_kind {
        nagare_core::AgentToolKind::Codex => "Codex",
        nagare_core::AgentToolKind::CodexCli => "Codex CLI",
        nagare_core::AgentToolKind::OpenClaw => "OpenClaw",
    }
}
