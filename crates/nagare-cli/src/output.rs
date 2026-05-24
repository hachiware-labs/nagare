use nagare_core::{
    AgentProfile, LocaleSettings, NagareAgentSettings, RuleResolution, VERSION, WorkItemSnapshot,
};

pub(crate) fn print_scenario_result(label: &str, result: &nagare_core::ScenarioResult) {
    println!("{label}");
    println!("work_item: {}", result.work_item_id);
    println!("codex_run: {}", result.codex_run_id);
    println!("handoff: {}", result.handoff_id);
    println!("codex_app_run: {}", result.codex_app_run_id);
    println!("verification: {}", result.verification_id);
    println!("decision: {}", result.decision_id);
    println!("final_status: {}", result.final_status);
}

pub(crate) fn print_snapshot(snapshot: &WorkItemSnapshot) {
    println!(
        "{}\t{}\t{}",
        snapshot.item.id, snapshot.item.status, snapshot.item.title
    );
    if !snapshot.item.description.is_empty() {
        println!("description: {}", snapshot.item.description);
    }
    println!("locale: {}", snapshot.item.locale);
    println!("runs:");
    for run in &snapshot.runs {
        println!(
            "  {}\t{}\t{}\t{}\texit={:?}\tartifact={}",
            run.id, run.purpose, run.agent_profile_id, run.status, run.exit_code, run.artifact_id
        );
    }
    println!("evidence:");
    for evidence in &snapshot.evidence {
        println!("  {}\t{}", evidence.id, evidence.claim);
    }
    println!("verification:");
    for verification in &snapshot.verification_results {
        println!("  {}\t{}", verification.id, verification.result);
    }
    println!("resolved_skill_contexts:");
    for context in &snapshot.resolved_skill_contexts {
        println!(
            "  {}\tagent={}\trules={}\tskills={}",
            context.id,
            context.agent_profile_id,
            comma_list(&context.project_rule_ids),
            comma_list(&context.applied_skill_set_ids)
        );
    }
    println!("resolved_run_packets:");
    for packet in &snapshot.resolved_run_packets {
        println!(
            "  {}\tagent={}\trules={}\tverification={}",
            packet.id,
            packet.agent_profile_id,
            comma_list(&packet.project_rule_ids),
            comma_list(&packet.verification)
        );
    }
    println!("handoffs:");
    for handoff in &snapshot.handoffs {
        println!(
            "  {}\t{} -> {}\t{}",
            handoff.id, handoff.from_agent_profile, handoff.to_agent_profile, handoff.reason
        );
    }
    println!("decisions:");
    for decision in &snapshot.decisions {
        println!("  {}\t{}", decision.id, decision.decision_type);
    }
    println!("dispatch_plans:");
    for plan in &snapshot.dispatch_plans {
        println!(
            "  {}\tdispatch_agent={}\ttarget_agent={}\trun={}\tsummary={}",
            plan.id,
            plan.dispatch_agent_profile_id,
            plan.target_agent_profile_id,
            plan.agent_run_id,
            plan.summary
        );
    }
}

pub(crate) fn print_agent_profile_row(profile: &AgentProfile) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        profile.id,
        profile.adapter,
        profile.runtime,
        profile.role,
        profile.working_dir,
        comma_list(&profile.specialties),
        profile.source
    );
}

pub(crate) fn print_agent_defaults(settings: &NagareAgentSettings) {
    println!("work_agent: {}", settings.work_agent);
    println!("review_agent: {}", settings.review_agent);
    println!("dispatch_agent: {}", settings.dispatch_agent);
}

pub(crate) fn print_rule_resolution(resolution: &RuleResolution) {
    println!("path: {}", resolution.path.as_deref().unwrap_or("-"));
    println!(
        "matched_rule: {}",
        resolution.matched_rule_id.as_deref().unwrap_or("-")
    );
    println!("agent_profile: {}", resolution.agent_profile_id);
    println!(
        "review_agent_profile: {}",
        resolution.review_agent_profile_id
    );
    println!("skill_sets: {}", comma_list(&resolution.skill_set_ids));
    println!(
        "permission_policy: {}",
        resolution.permission_policy_id.as_deref().unwrap_or("-")
    );
    println!(
        "workspace_policy: {}",
        resolution.workspace_policy_id.as_deref().unwrap_or("-")
    );
    println!("verification: {}", comma_list(&resolution.verification));
    for warning in &resolution.warnings {
        println!("warning: {warning}");
    }
}

pub(crate) fn dispatch_prompt(
    resolution: Option<&RuleResolution>,
    candidates: &[AgentProfile],
) -> String {
    let candidate_lines = if candidates.is_empty() {
        "- none".to_string()
    } else {
        candidates
            .iter()
            .map(compact_agent_candidate)
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (
        path,
        matched_rule,
        rule_target_agent,
        review_agent,
        skill_sets,
        permission_policy,
        workspace_policy,
        verification,
    ) = resolution
        .map(|resolution| {
            (
                resolution.path.as_deref().unwrap_or("-"),
                resolution.matched_rule_id.as_deref().unwrap_or("-"),
                resolution.agent_profile_id.as_str(),
                resolution.review_agent_profile_id.as_str(),
                comma_list(&resolution.skill_set_ids),
                resolution.permission_policy_id.as_deref().unwrap_or("-"),
                resolution.workspace_policy_id.as_deref().unwrap_or("-"),
                comma_list(&resolution.verification),
            )
        })
        .unwrap_or((
            "-",
            "-",
            "-",
            "-",
            "-".to_string(),
            "-",
            "-",
            "-".to_string(),
        ));
    format!(
        "Prepare a dispatch preview for path `{path}`.\nMatched rule: {matched_rule}\nRule target agent profile: {rule_target_agent}\nReview agent profile: {review_agent}\nSkill sets: {skill_sets}\nPermission policy: {permission_policy}\nWorkspace policy: {workspace_policy}\nVerification: {verification}\n\nCandidate agent profiles are intentionally compact. Select only from this list:\n{candidate_lines}\n\nReturn one JSON object only with keys: target_agent_profile_id, summary, risks, missing_information. Keep summary concise and do not include full instruction-source contents.",
    )
}

fn compact_agent_candidate(profile: &AgentProfile) -> String {
    format!(
        "- id: {} | role: {} | adapter: {} | working_dir: {} | specialties: {} | description: {}",
        profile.id,
        profile.role,
        profile.adapter,
        profile.working_dir,
        comma_list(&profile.specialties),
        compact_text(&profile.description, 160)
    )
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    let mut chars = trimmed.chars();
    let compact = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{compact}...")
    } else {
        compact
    }
}

pub(crate) fn print_locale_settings(settings: &LocaleSettings) {
    println!("language: {}", settings.language);
    println!("timezone: {}", settings.timezone);
}

pub(crate) fn print_created(label: &str, created: bool, path: &std::path::Path) {
    if created {
        println!("created {label}: {}", path.display());
    } else {
        println!("kept {label}: {}", path.display());
    }
}

pub(crate) fn bool_label(value: bool) -> &'static str {
    if value { "ok" } else { "missing" }
}

pub(crate) fn comma_list(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(",")
    }
}

pub(crate) fn empty_label(value: &str) -> &str {
    if value.trim().is_empty() { "-" } else { value }
}

pub(crate) fn print_help() {
    println!(
        "nagare {VERSION}

Usage:
  nagare init [--root <path>]
  nagare doctor [--root <path>]
  nagare locale show [--root <path>]
  nagare locale use [--language <locale>] [--timezone <timezone>] [--root <path>]
  nagare agent add --id <agent_profile_id> --runtime <runtime_id> --adapter <adapter_id> [--display-name <text>] [--role <role>] [--working-dir <relative_path>] [--description <text>] [--specialties <csv>] [--root <path>]
  nagare agent list [--root <path>]
  nagare agent show <agent_profile_id> [--root <path>]
  nagare agent defaults [--root <path>]
  nagare agent use [--work-agent <agent_profile_id>] [--review-agent <agent_profile_id>] [--dispatch-agent <agent_profile_id>] [--root <path>]
  nagare agent doctor <agent_profile_id> [--root <path>]
  nagare agent probe <agent_profile_id> [--root <path>]
  nagare rule check <path> [--agent <agent_profile_id>] [--root <path>]
  nagare item create --title <title> [--description <text>] [--root <path>]
  nagare item list [--root <path>]
  nagare item show <work_id> [--root <path>]
  nagare item preview <work_id> [--path <path>] [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item run <work_id> [--path <path>] [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item review <work_id> [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare verify <work_id> --command <command> [--root <path>]
  nagare handoff create <work_id> --from-agent <agent_profile_id> --to-agent <agent_profile_id> --reason <text> [--summary <text>] [--root <path>]
  nagare handoff dispatch <work_id> [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare decision approve <work_id> [--rationale <text>] [--root <path>]
  nagare status [--root <path>]
  nagare version
  nagare help
"
    );
}
