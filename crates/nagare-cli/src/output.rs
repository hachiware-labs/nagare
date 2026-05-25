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
    println!(
        "acceptance_criteria: {}",
        comma_list(&snapshot.item.acceptance_criteria)
    );
    println!(
        "expected_artifacts: {}",
        comma_list(&snapshot.item.expected_artifacts)
    );
    println!(
        "verification_hint: {}",
        snapshot.item.verification_hint.as_deref().unwrap_or("-")
    );
    println!(
        "work_folder: {}",
        snapshot.item.work_folder.as_deref().unwrap_or("-")
    );
    println!("constraints: {}", comma_list(&snapshot.item.constraints));
    println!("locale: {}", snapshot.item.locale);
    println!(
        "completion: state={} next_action={} blocking_reason={} hint={}",
        snapshot.completion.state,
        snapshot.completion.next_action,
        snapshot
            .completion
            .blocking_reason
            .as_deref()
            .unwrap_or("-"),
        snapshot
            .completion
            .next_command_hint
            .as_deref()
            .unwrap_or("-")
    );
    println!("timeline:");
    for event in &snapshot.timeline {
        println!(
            "  {}\t{}\t{}\t{}\tactor={}\tartifact={}",
            event.id,
            event.event_type,
            event.status,
            event.title,
            event.actor.as_deref().unwrap_or("-"),
            event.artifact_id.as_deref().unwrap_or("-")
        );
    }
    println!("runs:");
    for run in &snapshot.runs {
        println!(
            "  {}\t{}\t{}\t{}\texit={:?}\tartifact={}",
            run.id, run.purpose, run.agent_profile_id, run.status, run.exit_code, run.artifact_id
        );
    }
    println!("artifacts:");
    for artifact in &snapshot.artifacts {
        println!(
            "  {}\t{}\tagent_run={}\t{}",
            artifact.id,
            artifact.artifact_type,
            artifact.agent_run_id.as_deref().unwrap_or("-"),
            artifact.title
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
    println!("review_results:");
    for review in &snapshot.review_results {
        println!(
            "  {}\t{}\tagent={}\tcriteria={}\tfindings={}\trequested_changes={}",
            review.id,
            review.verdict,
            review.agent_profile_id,
            review
                .criteria_results
                .iter()
                .map(|result| format!("{}:{}", result.criterion, result.status))
                .collect::<Vec<_>>()
                .join(","),
            comma_list(&review.findings),
            comma_list(&review.requested_changes)
        );
    }
    println!("resolved_skill_contexts:");
    for context in &snapshot.resolved_skill_contexts {
        println!(
            "  {}\tagent={}\tcontext_refs={}\tskills={}",
            context.id,
            context.agent_profile_id,
            comma_list(&context.project_rule_ids),
            comma_list(&context.applied_skill_set_ids)
        );
    }
    println!("resolved_run_packets:");
    for packet in &snapshot.resolved_run_packets {
        println!(
            "  {}\tagent={}\tcontext_refs={}\tverification={}",
            packet.id,
            packet.agent_profile_id,
            comma_list(&packet.project_rule_ids),
            comma_list(&packet.verification)
        );
    }
    println!("agent_outputs:");
    for output in &snapshot.agent_outputs {
        println!(
            "  {}\t{}\t{}\tnext_action={}\tquestions={}\twarnings={}",
            output.id,
            output.purpose,
            output.parse_status,
            output.next_action.as_deref().unwrap_or("-"),
            comma_list(&output.questions),
            comma_list(&output.warnings)
        );
    }
    println!("handoffs:");
    for handoff in &snapshot.handoffs {
        println!(
            "  {}\t{} -> {}\tstate={}\treason={}\tnext_request={}\tartifacts={}\tdiffs={}",
            handoff.id,
            handoff.from_agent_profile,
            handoff.to_agent_profile,
            handoff.current_state,
            handoff.reason,
            handoff.next_request,
            comma_list(&handoff.artifact_ids),
            comma_list(&handoff.diff_artifact_ids)
        );
    }
    println!("decisions:");
    for decision in &snapshot.decisions {
        println!("  {}\t{}", decision.id, decision.decision_type);
    }
    println!("human_feedback:");
    for feedback in &snapshot.human_feedback {
        println!(
            "  {}\tquestion={}\tanswer={}",
            feedback.id, feedback.question, feedback.answer
        );
    }
    println!("dispatch_plans:");
    for plan in &snapshot.dispatch_plans {
        println!(
            "  {}\t{}\tdispatch_agent={}\ttarget_agent={}\trun={}\twarnings={}\tsummary={}",
            plan.id,
            plan.status,
            plan.dispatch_agent_profile_id,
            plan.target_agent_profile_id,
            plan.agent_run_id,
            comma_list(&plan.selection_warnings),
            plan.summary
        );
    }
    println!("recovery_plans:");
    for plan in &snapshot.recovery_plans {
        println!(
            "  {}\t{}\taction={}\tfailure_class={}\ttarget_agent={}\treason={}\thint={}",
            plan.id,
            plan.status,
            plan.action,
            plan.failure_class,
            plan.target_agent_profile_id.as_deref().unwrap_or("-"),
            plan.reason,
            plan.command_hint.as_deref().unwrap_or("-")
        );
    }
    println!("workflow_decisions:");
    for decision in &snapshot.workflow_decisions {
        println!(
            "  {}\t{}\tsource={}\trequires_human={}\ttarget_agent={}\treason={}",
            decision.id,
            decision.action,
            decision.source,
            decision.requires_human,
            decision.target_agent_profile_id.as_deref().unwrap_or("-"),
            decision.reason
        );
    }
}

pub(crate) fn print_agent_profile_row(profile: &AgentProfile) {
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\twork_contract={}\treview_contract={}\tdispatch_contract={}\tsupervision_contract={}\t{}",
        profile.id,
        profile.adapter,
        profile.runtime,
        profile.role,
        profile.working_dir,
        comma_list(&profile.specialties),
        profile.output_contracts.work.contract,
        profile.output_contracts.review.contract,
        profile.output_contracts.dispatch.contract,
        profile.output_contracts.supervision.contract,
        profile.source
    );
}

pub(crate) fn print_agent_defaults(settings: &NagareAgentSettings) {
    println!("work_agent: {}", settings.work_agent);
    println!("review_agent: {}", settings.review_agent);
    println!("dispatch_agent: {}", settings.dispatch_agent);
    println!("supervisor_agent: {}", settings.supervisor_agent);
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
        matched_context,
        resolved_target_agent,
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
        "Prepare a dispatch preview for path `{path}`.\nMatched context: {matched_context}\nResolved target agent profile: {resolved_target_agent}\nReview agent profile: {review_agent}\nSkill sets: {skill_sets}\nPermission policy: {permission_policy}\nWorkspace policy: {workspace_policy}\nVerification: {verification}\n\nCandidate agent profiles are intentionally compact. Select only from this list:\n{candidate_lines}\n\nReturn one JSON object only. Required keys: target_agent_profile_id, summary. Optional keys: risks, missing_information. target_agent_profile_id must exactly match one candidate id. Keep summary concise and do not include full instruction-source contents.",
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
  nagare agent update <agent_profile_id> [--display-name <text>] [--role <role>] [--working-dir <relative_path>] [--description <text>] [--specialties <csv>] [--output-purpose work|review|dispatch|supervision] [--output-contract <id>] [--instruction-pack <id>] [--output-required true|false] [--output-injection prompt_suffix] [--root <path>]
  nagare agent list [--root <path>]
  nagare agent show <agent_profile_id> [--root <path>]
  nagare agent defaults [--root <path>]
  nagare agent use [--work-agent <agent_profile_id>] [--review-agent <agent_profile_id>] [--dispatch-agent <agent_profile_id>] [--supervisor-agent <agent_profile_id>] [--root <path>]
  nagare agent doctor <agent_profile_id> [--root <path>]
  nagare agent probe <agent_profile_id> [--root <path>]
  nagare item create --title <title> [--description <text>] [--acceptance <csv>] [--artifact <csv>] [--verification <command_or_hint>] [--work-folder <relative_path>] [--constraint <csv>] [--root <path>]
  nagare item list [--root <path>]
  nagare item show <work_id> [--root <path>]
  nagare item answer <work_id> --answer <text> [--question <text>] [--root <path>]
  nagare item preview <work_id> [--path <path>] [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item dispatch accept <work_id> [--dispatch-plan <dispatch_plan_id>] [--root <path>]
  nagare item recover <work_id> [--root <path>]
  nagare item recover accept <work_id> [--recovery-plan <recovery_plan_id>] [--root <path>]
  nagare item recover apply <work_id> [--recovery-plan <recovery_plan_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item run <work_id> [--path <path>] [--agent <agent_profile_id>] [--dispatch-plan <dispatch_plan_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item review <work_id> [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
  nagare item advance <work_id> [--until-blocked true|false] [--max-steps <n>] [--supervisor true|false] [--path <path>] [--prompt <text>] [--command <command>] [--dispatch-command <command>] [--review-command <command>] [--verify-command <command>] [--supervisor-command <command>] [--root <path>]
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
