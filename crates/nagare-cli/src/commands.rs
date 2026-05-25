use std::collections::BTreeSet;
use std::path::Path;

use nagare_core::{
    AddAgentProfileInput, AgentOutputContractPurpose, AgentOutputContractUpdate,
    AgentOutputInjection, AgentProfile, AgentRunPurpose, AnswerWorkItemInput, RuleResolution,
    RunWorkItemInput, SelectRunAgentInput, SetLocaleInput, SetNagareAgentSettingsInput,
    UpdateAgentProfileInput, VERSION, accept_dispatch_plan, add_agent_profile, agent_doctor,
    agent_probe, answer_work_item, approve_work_item, create_handoff, create_work_item, doctor,
    get_agent_profile, get_locale_settings, get_nagare_agent_settings, get_work_item_snapshot,
    init_project, list_agent_profiles, list_work_items, resolve_rule_for_path, run_first_scenario,
    run_registered_agent_scenario, run_work_item_with_input, select_agent_for_work_item_run,
    set_locale_settings, set_nagare_agent_settings, update_agent_profile, verify_work_item,
};

use crate::args::ParsedArgs;
use crate::output::*;

const DISPATCH_AGENT_CANDIDATE_LIMIT: usize = 5;

pub(crate) fn run(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("version") | Some("--version") | Some("-V") => {
            println!("nagare {VERSION}");
            Ok(())
        }
        Some("init") => init_command(&args[1..]),
        Some("doctor") => doctor_command(&args[1..]),
        Some("agent") => agent_command(&args[1..]),
        Some("locale") => locale_command(&args[1..]),
        Some("item") => item_command(&args[1..]),
        Some("verify") => verify_command(&args[1..]),
        Some("handoff") => handoff_command(&args[1..]),
        Some("decision") => decision_command(&args[1..]),
        Some("dev") => dev_command(&args[1..]),
        Some("status") => item_list_command(&args[1..]),
        Some(command) => Err(format!("unknown command `{command}`")),
    }
}

fn init_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let root = parsed.root()?;
    let result = init_project(root).map_err(|error| error.to_string())?;

    println!("initialized {}", result.layout.nagare_dir.display());
    print_created(
        "project config",
        result.created_config,
        &result.layout.config_path,
    );
    print_created("ledger", result.created_ledger, &result.layout.ledger_path);
    Ok(())
}

fn doctor_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let report = doctor(parsed.root()?);

    println!("nagare {VERSION}");
    println!("root: {}", report.root.display());
    println!("git: {}", bool_label(report.has_git));
    println!("project_config: {}", bool_label(report.has_config));
    println!("ledger: {}", bool_label(report.has_ledger));
    for tool in report.tools {
        println!("{tool}");
    }
    Ok(())
}

fn agent_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("add") => agent_add_command(&args[1..]),
        Some("update") => agent_update_command(&args[1..]),
        Some("list") => agent_list_command(&args[1..]),
        Some("show") => agent_show_command(&args[1..]),
        Some("defaults") => agent_defaults_command(&args[1..]),
        Some("use") => agent_use_command(&args[1..]),
        Some("doctor") => agent_doctor_command(&args[1..]),
        Some("probe") => agent_probe_command(&args[1..]),
        Some(command) => Err(format!("unknown agent command `{command}`")),
        None => Err(
            "agent command required: add, update, list, show, defaults, use, doctor, probe"
                .to_string(),
        ),
    }
}

fn agent_add_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let id = parsed.required("--id")?;
    let runtime = parsed.required("--runtime")?;
    let adapter = parsed.required("--adapter")?;
    let display_name = parsed.optional("--display-name").unwrap_or(id);
    let role = parsed.optional("--role").unwrap_or("implementer");
    let working_dir = parsed.optional("--working-dir").unwrap_or(".");
    let description = parsed.optional("--description").unwrap_or("");
    let specialties = parse_comma_list(parsed.optional("--specialties"));
    let result = add_agent_profile(
        parsed.root()?,
        AddAgentProfileInput {
            id,
            display_name,
            runtime,
            adapter,
            role,
            working_dir,
            description,
            specialties,
        },
    )
    .map_err(|e| e.to_string())?;
    println!(
        "agent {} added adapter={} runtime={} role={} working_dir={} path={}",
        result.profile.id,
        result.profile.adapter,
        result.profile.runtime,
        result.profile.role,
        result.profile.working_dir,
        result.path.display()
    );
    Ok(())
}

fn agent_update_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let agent_profile_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "agent update requires an agent profile id".to_string())?;
    let has_update = parsed.optional("--display-name").is_some()
        || parsed.optional("--role").is_some()
        || parsed.optional("--working-dir").is_some()
        || parsed.optional("--description").is_some()
        || parsed.optional("--specialties").is_some()
        || parsed.optional("--output-purpose").is_some()
        || parsed.optional("--output-contract").is_some()
        || parsed.optional("--instruction-pack").is_some()
        || parsed.optional("--output-required").is_some()
        || parsed.optional("--output-injection").is_some();
    if !has_update {
        return Err("agent update requires a profile field or output contract option".to_string());
    }
    let output_contract = parse_output_contract_update(&parsed)?;
    let result = update_agent_profile(
        parsed.root()?,
        agent_profile_id,
        UpdateAgentProfileInput {
            display_name: parsed.optional("--display-name"),
            role: parsed.optional("--role"),
            working_dir: parsed.optional("--working-dir"),
            description: parsed.optional("--description"),
            specialties: parsed
                .optional("--specialties")
                .map(|value| parse_comma_list(Some(value))),
            output_contract,
        },
    )
    .map_err(|e| e.to_string())?;
    println!(
        "agent {} updated role={} working_dir={} specialties={} path={}",
        result.profile.id,
        result.profile.role,
        result.profile.working_dir,
        comma_list(&result.profile.specialties),
        result.path.display()
    );
    Ok(())
}

fn agent_list_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let profiles = list_agent_profiles(parsed.root()?).map_err(|e| e.to_string())?;
    if profiles.is_empty() {
        println!("no agent profiles");
        return Ok(());
    }
    for profile in profiles {
        print_agent_profile_row(&profile);
    }
    Ok(())
}

fn agent_show_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let agent_profile_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "agent show requires an agent profile id".to_string())?;
    let profile = get_agent_profile(parsed.root()?, agent_profile_id).map_err(|e| e.to_string())?;
    println!("id: {}", profile.id);
    println!("display_name: {}", profile.display_name);
    println!("runtime: {}", profile.runtime);
    println!("adapter: {}", profile.adapter);
    println!("role: {}", profile.role);
    println!("working_dir: {}", profile.working_dir);
    println!("description: {}", empty_label(&profile.description));
    println!("specialties: {}", comma_list(&profile.specialties));
    println!(
        "output_contract.work: {} / {} / required={} / injection={}",
        profile.output_contracts.work.contract,
        profile.output_contracts.work.instruction_pack,
        profile.output_contracts.work.required,
        profile.output_contracts.work.injection
    );
    println!(
        "output_contract.review: {} / {} / required={} / injection={}",
        profile.output_contracts.review.contract,
        profile.output_contracts.review.instruction_pack,
        profile.output_contracts.review.required,
        profile.output_contracts.review.injection
    );
    println!(
        "output_contract.dispatch: {} / {} / required={} / injection={}",
        profile.output_contracts.dispatch.contract,
        profile.output_contracts.dispatch.instruction_pack,
        profile.output_contracts.dispatch.required,
        profile.output_contracts.dispatch.injection
    );
    println!("source: {}", profile.source);
    Ok(())
}

fn parse_output_contract_update<'a>(
    parsed: &'a ParsedArgs,
) -> Result<Option<AgentOutputContractUpdate<'a>>, String> {
    let has_output_update = parsed.optional("--output-purpose").is_some()
        || parsed.optional("--output-contract").is_some()
        || parsed.optional("--instruction-pack").is_some()
        || parsed.optional("--output-required").is_some()
        || parsed.optional("--output-injection").is_some();
    if !has_output_update {
        return Ok(None);
    }
    let purpose = parsed
        .required("--output-purpose")
        .and_then(|value| AgentOutputContractPurpose::parse(value).map_err(|e| e.to_string()))?;
    let required = parsed
        .optional("--output-required")
        .map(parse_bool)
        .transpose()?;
    let injection = parsed
        .optional("--output-injection")
        .map(parse_output_injection)
        .transpose()?;
    Ok(Some(AgentOutputContractUpdate {
        purpose: Some(purpose),
        contract: parsed.optional("--output-contract"),
        instruction_pack: parsed.optional("--instruction-pack"),
        required,
        injection,
    }))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim() {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        other => Err(format!("expected boolean true or false, got `{other}`")),
    }
}

fn parse_output_injection(value: &str) -> Result<AgentOutputInjection, String> {
    match value.trim() {
        "prompt_suffix" => Ok(AgentOutputInjection::PromptSuffix),
        other => Err(format!(
            "unknown output injection `{other}`; expected prompt_suffix"
        )),
    }
}

fn parse_comma_list(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn agent_defaults_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let settings = get_nagare_agent_settings(parsed.root()?).map_err(|e| e.to_string())?;
    print_agent_defaults(&settings);
    Ok(())
}

fn agent_use_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_agent = parsed.optional("--work-agent");
    let review_agent = parsed.optional("--review-agent");
    let dispatch_agent = parsed.optional("--dispatch-agent");
    if work_agent.is_none() && review_agent.is_none() && dispatch_agent.is_none() {
        return Err(
            "agent use requires --work-agent, --review-agent, or --dispatch-agent".to_string(),
        );
    }
    let settings = set_nagare_agent_settings(
        parsed.root()?,
        SetNagareAgentSettingsInput {
            work_agent,
            review_agent,
            dispatch_agent,
        },
    )
    .map_err(|e| e.to_string())?;
    print_agent_defaults(&settings);
    Ok(())
}

fn agent_doctor_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let agent_profile_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "agent doctor requires an agent profile id".to_string())?;
    let report = agent_doctor(parsed.root()?, agent_profile_id).map_err(|e| e.to_string())?;
    println!("agent: {}", report.profile.id);
    println!("display_name: {}", report.profile.display_name);
    println!("runtime: {}", report.profile.runtime);
    println!("adapter: {}", report.profile.adapter);
    println!("runtime_kind: {}", report.runtime.kind);
    println!("{}", report.health);
    Ok(())
}

fn agent_probe_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let agent_profile_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "agent probe requires an agent profile id".to_string())?;
    let result = agent_probe(parsed.root()?, agent_profile_id).map_err(|e| e.to_string())?;
    let probe = result.probe;
    println!(
        "probe {} agent={} available={} runtime={} adapter={}",
        probe.id, probe.agent_profile_id, probe.available, probe.runtime_id, probe.adapter_id
    );
    println!("runtime_version: {}", probe.runtime_version);
    println!(
        "capabilities: {}",
        comma_list(&probe.discovered_capabilities)
    );
    println!("skill_modes: {}", comma_list(&probe.supported_skill_modes));
    if !probe.instruction_sources.is_empty() {
        println!(
            "instruction_sources: {}",
            comma_list(&probe.instruction_sources)
        );
    }
    for warning in probe.warnings {
        println!("warning: {warning}");
    }
    Ok(())
}

fn locale_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("show") => locale_show_command(&args[1..]),
        Some("use") => locale_use_command(&args[1..]),
        Some(command) => Err(format!("unknown locale command `{command}`")),
        None => Err("locale command required: show, use".to_string()),
    }
}

fn locale_show_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let settings = get_locale_settings(parsed.root()?).map_err(|e| e.to_string())?;
    print_locale_settings(&settings);
    Ok(())
}

fn locale_use_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let language = parsed.optional("--language");
    let timezone = parsed.optional("--timezone");
    if language.is_none() && timezone.is_none() {
        return Err("locale use requires --language or --timezone".to_string());
    }
    let settings = set_locale_settings(parsed.root()?, SetLocaleInput { language, timezone })
        .map_err(|e| e.to_string())?;
    print_locale_settings(&settings);
    Ok(())
}

fn item_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("create") => item_create_command(&args[1..]),
        Some("list") => item_list_command(&args[1..]),
        Some("show") => item_show_command(&args[1..]),
        Some("preview") => item_preview_command(&args[1..]),
        Some("dispatch") => item_dispatch_command(&args[1..]),
        Some("run") => item_run_command(&args[1..]),
        Some("review") => item_review_command(&args[1..]),
        Some("answer") => item_answer_command(&args[1..]),
        Some(command) => Err(format!("unknown item command `{command}`")),
        None => Err(
            "item command required: create, list, show, preview, dispatch, run, review, answer"
                .to_string(),
        ),
    }
}

fn item_create_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let title = parsed.required("--title")?;
    let description = parsed.optional("--description").unwrap_or_default();
    let result = create_work_item(parsed.root()?, title, description).map_err(|e| e.to_string())?;
    println!("created {} {}", result.item.id, result.item.status);
    Ok(())
}

fn item_list_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let items = list_work_items(parsed.root()?).map_err(|e| e.to_string())?;
    if items.is_empty() {
        println!("no work items");
        return Ok(());
    }
    for item in items {
        println!("{}\t{}\t{}", item.id, item.status, item.title);
    }
    Ok(())
}

fn item_show_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "item show requires a work item id".to_string())?;
    let snapshot =
        get_work_item_snapshot(parsed.root()?, work_item_id).map_err(|e| e.to_string())?;
    print_snapshot(&snapshot);
    Ok(())
}

fn item_answer_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "item answer requires a work item id".to_string())?;
    let result = answer_work_item(
        parsed.root()?,
        work_item_id,
        AnswerWorkItemInput {
            question: parsed.optional("--question"),
            answer: parsed.required("--answer")?,
        },
    )
    .map_err(|e| e.to_string())?;
    println!(
        "feedback {} recorded item_status={}",
        result.feedback.id, result.item_status
    );
    Ok(())
}

fn item_run_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let root = parsed.root()?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "item run requires a work item id".to_string())?;
    let path = parsed.optional("--path");
    let agent_selection = select_agent_for_work_item_run(
        &root,
        work_item_id,
        SelectRunAgentInput {
            explicit_agent_profile_id: parsed.optional("--agent"),
            dispatch_plan_id: parsed.optional("--dispatch-plan"),
            path,
        },
    )
    .map_err(|e| e.to_string())?;
    if let Some(dispatch_plan_id) = &agent_selection.dispatch_plan_id {
        println!(
            "selected_agent: {} source={} dispatch_plan={}",
            agent_selection.agent_profile_id, agent_selection.source, dispatch_plan_id
        );
    } else {
        println!(
            "selected_agent: {} source={}",
            agent_selection.agent_profile_id, agent_selection.source
        );
    }
    let command = parsed.optional("--command");
    let prompt = parsed.optional("--prompt");
    if command.is_none() && prompt.is_none() {
        return Err("item run requires --prompt or --command".to_string());
    }
    let result = run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: agent_selection.agent_profile_id.as_str(),
            dispatch_plan_id: agent_selection.dispatch_plan_id.as_deref(),
            path,
            prompt,
            dev_command: command,
            purpose: AgentRunPurpose::Work,
        },
    )
    .map_err(|e| e.to_string())?;
    println!(
        "run {} {} agent={} exit={:?} evidence={} item_status={}",
        result.run.id,
        result.run.status,
        result.run.agent_profile_id,
        result.run.exit_code,
        result.evidence_id,
        result.item_status
    );
    Ok(())
}

fn item_dispatch_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("accept") => item_dispatch_accept_command(&args[1..]),
        Some(command) => Err(format!("unknown item dispatch command `{command}`")),
        None => Err("item dispatch command required: accept".to_string()),
    }
}

fn item_dispatch_accept_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "item dispatch accept requires a work item id".to_string())?;
    let result = accept_dispatch_plan(
        parsed.root()?,
        work_item_id,
        parsed
            .optional("--dispatch-plan")
            .or(parsed.optional("--plan")),
    )
    .map_err(|e| e.to_string())?;
    println!(
        "dispatch_plan {} {} target_agent={}",
        result.plan.id, result.plan.status, result.plan.target_agent_profile_id
    );
    Ok(())
}

fn item_preview_command(args: &[String]) -> Result<(), String> {
    run_item_with_nagare_agent(args, AgentRunPurpose::DispatchPreview, "dispatch_agent")
}

fn item_review_command(args: &[String]) -> Result<(), String> {
    run_item_with_nagare_agent(args, AgentRunPurpose::Review, "review_agent")
}

fn run_item_with_nagare_agent(
    args: &[String],
    purpose: AgentRunPurpose,
    default_agent_key: &str,
) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let root = parsed.root()?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| format!("item {purpose} requires a work item id"))?;
    let defaults = get_nagare_agent_settings(&root).map_err(|e| e.to_string())?;
    let default_agent = match default_agent_key {
        "dispatch_agent" => defaults.dispatch_agent.as_str(),
        "review_agent" => defaults.review_agent.as_str(),
        _ => defaults.work_agent.as_str(),
    };
    let agent = parsed.optional("--agent").unwrap_or(default_agent);
    let command = parsed.optional("--command");
    let path = parsed.optional("--path");
    let resolution = if purpose == AgentRunPurpose::DispatchPreview {
        match path {
            Some(path) => {
                Some(resolve_rule_for_path(&root, Some(path), None).map_err(|e| e.to_string())?)
            }
            None => None,
        }
    } else {
        None
    };
    let generated_prompt =
        if purpose == AgentRunPurpose::DispatchPreview && parsed.optional("--prompt").is_none() {
            let candidates = dispatch_agent_candidates(
                &root,
                &defaults.work_agent,
                &defaults.review_agent,
                resolution.as_ref(),
            )?;
            Some(dispatch_prompt(resolution.as_ref(), &candidates))
        } else {
            None
        };
    let prompt = match parsed.optional("--prompt") {
        Some(prompt) => Some(prompt),
        None => generated_prompt.as_deref(),
    };
    let result = run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: agent,
            dispatch_plan_id: None,
            path,
            prompt,
            dev_command: command,
            purpose,
        },
    )
    .map_err(|e| e.to_string())?;
    println!(
        "run {} {} purpose={} agent={} exit={:?} evidence={} dispatch_plan={} item_status={}",
        result.run.id,
        result.run.status,
        result.run.purpose,
        result.run.agent_profile_id,
        result.run.exit_code,
        result.evidence_id,
        result.dispatch_plan_id.as_deref().unwrap_or("-"),
        result.item_status
    );
    Ok(())
}

fn dispatch_agent_candidates(
    root: &Path,
    default_work_agent: &str,
    default_review_agent: &str,
    resolution: Option<&RuleResolution>,
) -> Result<Vec<AgentProfile>, String> {
    let profiles = list_agent_profiles(root).map_err(|e| e.to_string())?;
    let mut selected_ids = Vec::new();
    let mut seen = BTreeSet::new();

    if let Some(resolution) = resolution {
        push_unique(&mut selected_ids, &mut seen, &resolution.agent_profile_id);
        push_unique(
            &mut selected_ids,
            &mut seen,
            &resolution.review_agent_profile_id,
        );
    }
    push_unique(&mut selected_ids, &mut seen, default_work_agent);
    push_unique(&mut selected_ids, &mut seen, default_review_agent);
    for profile in &profiles {
        push_unique(&mut selected_ids, &mut seen, &profile.id);
        if selected_ids.len() >= DISPATCH_AGENT_CANDIDATE_LIMIT {
            break;
        }
    }

    let mut candidates = Vec::new();
    for selected_id in selected_ids {
        if let Some(profile) = profiles.iter().find(|profile| profile.id == selected_id) {
            candidates.push(profile.clone());
        }
        if candidates.len() >= DISPATCH_AGENT_CANDIDATE_LIMIT {
            break;
        }
    }
    Ok(candidates)
}

fn push_unique(selected_ids: &mut Vec<String>, seen: &mut BTreeSet<String>, id: &str) {
    if !id.trim().is_empty() && seen.insert(id.to_string()) {
        selected_ids.push(id.to_string());
    }
}

fn verify_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "verify requires a work item id".to_string())?;
    let command = parsed.required("--command")?;
    let result =
        verify_work_item(parsed.root()?, work_item_id, command).map_err(|e| e.to_string())?;
    println!(
        "verification {} {} item_status={}",
        result.verification.id, result.verification.result, result.item_status
    );
    Ok(())
}

fn handoff_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("create") => handoff_create_command(&args[1..]),
        Some("dispatch") => handoff_dispatch_command(&args[1..]),
        Some(command) => Err(format!("unknown handoff command `{command}`")),
        None => Err("handoff command required: create, dispatch".to_string()),
    }
}

fn handoff_create_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "handoff create requires a work item id".to_string())?;
    let from_agent = parsed.required("--from-agent")?;
    let to_agent = parsed.required("--to-agent")?;
    let reason = parsed.required("--reason")?;
    let summary = parsed.optional("--summary").unwrap_or_default();
    let result = create_handoff(
        parsed.root()?,
        work_item_id,
        from_agent,
        to_agent,
        reason,
        summary,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "handoff {} {} -> {}",
        result.handoff.id, result.handoff.from_agent_profile, result.handoff.to_agent_profile
    );
    Ok(())
}

fn handoff_dispatch_command(args: &[String]) -> Result<(), String> {
    run_item_with_nagare_agent(args, AgentRunPurpose::DispatchPreview, "dispatch_agent")
}

fn decision_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("approve") => decision_approve_command(&args[1..]),
        Some(command) => Err(format!("unknown decision command `{command}`")),
        None => Err("decision command required: approve".to_string()),
    }
}

fn decision_approve_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "decision approve requires a work item id".to_string())?;
    let rationale = parsed.optional("--rationale").unwrap_or("");
    let result =
        approve_work_item(parsed.root()?, work_item_id, rationale).map_err(|e| e.to_string())?;
    println!(
        "decision {} approve item_status={}",
        result.decision.id, result.item_status
    );
    Ok(())
}

fn dev_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("scenario") => dev_scenario_command(&args[1..]),
        Some(command) => Err(format!("unknown dev command `{command}`")),
        None => Err("dev command required: scenario".to_string()),
    }
}

fn dev_scenario_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("first") => {
            let parsed = ParsedArgs::parse(&args[1..])?;
            let result = run_first_scenario(parsed.root()?).map_err(|e| e.to_string())?;
            print_scenario_result("scenario first completed", &result);
            Ok(())
        }
        Some("registered-agents") => {
            let parsed = ParsedArgs::parse(&args[1..])?;
            let result =
                run_registered_agent_scenario(parsed.root()?).map_err(|e| e.to_string())?;
            print_scenario_result("scenario registered-agents completed", &result);
            Ok(())
        }
        Some(command) => Err(format!("unknown scenario command `{command}`")),
        None => Err("dev scenario command required: first".to_string()),
    }
}
