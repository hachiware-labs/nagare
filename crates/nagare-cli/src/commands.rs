use nagare_core::{
    AddAgentProfileInput, AgentRunPurpose, RunWorkItemInput, SetLocaleInput,
    SetNagareAgentSettingsInput, VERSION, add_agent_profile, agent_doctor, agent_probe,
    approve_work_item, create_handoff, create_work_item, doctor, get_agent_profile,
    get_locale_settings, get_nagare_agent_settings, get_work_item_snapshot, init_project,
    list_agent_profiles, list_work_items, resolve_rule_for_path, run_first_scenario,
    run_registered_agent_scenario, run_work_item_with_input, set_locale_settings,
    set_nagare_agent_settings, verify_work_item,
};

use crate::args::ParsedArgs;
use crate::output::*;

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
        Some("rule") => rule_command(&args[1..]),
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
        Some("list") => agent_list_command(&args[1..]),
        Some("show") => agent_show_command(&args[1..]),
        Some("defaults") => agent_defaults_command(&args[1..]),
        Some("use") => agent_use_command(&args[1..]),
        Some("doctor") => agent_doctor_command(&args[1..]),
        Some("probe") => agent_probe_command(&args[1..]),
        Some(command) => Err(format!("unknown agent command `{command}`")),
        None => {
            Err("agent command required: add, list, show, defaults, use, doctor, probe".to_string())
        }
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
    let result = add_agent_profile(
        parsed.root()?,
        AddAgentProfileInput {
            id,
            display_name,
            runtime,
            adapter,
            role,
            working_dir,
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
    println!("source: {}", profile.source);
    Ok(())
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

fn rule_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("check") => rule_check_command(&args[1..]),
        Some(command) => Err(format!("unknown rule command `{command}`")),
        None => Err("rule command required: check".to_string()),
    }
}

fn rule_check_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let path = parsed
        .positionals
        .first()
        .ok_or_else(|| "rule check requires a path".to_string())?;
    let resolution = resolve_rule_for_path(parsed.root()?, Some(path), parsed.optional("--agent"))
        .map_err(|e| e.to_string())?;
    print_rule_resolution(&resolution);
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
        Some("run") => item_run_command(&args[1..]),
        Some("review") => item_review_command(&args[1..]),
        Some(command) => Err(format!("unknown item command `{command}`")),
        None => Err("item command required: create, list, show, preview, run, review".to_string()),
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

fn item_run_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let root = parsed.root()?;
    let work_item_id = parsed
        .positionals
        .first()
        .ok_or_else(|| "item run requires a work item id".to_string())?;
    let agent;
    let path = parsed.optional("--path");
    if let Some(explicit_agent) = parsed.optional("--agent") {
        agent = explicit_agent.to_string();
    } else if let Some(path) = path {
        let resolution =
            resolve_rule_for_path(&root, Some(path), None).map_err(|e| e.to_string())?;
        print_rule_resolution(&resolution);
        agent = resolution.agent_profile_id;
    } else {
        agent = get_nagare_agent_settings(&root)
            .map_err(|e| e.to_string())?
            .work_agent;
    };
    let command = parsed.optional("--command");
    let prompt = parsed.optional("--prompt");
    if command.is_none() && prompt.is_none() {
        return Err("item run requires --prompt or --command".to_string());
    }
    let result = run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: agent.as_str(),
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
        "dispatch_agent" => defaults.dispatch_agent,
        "review_agent" => defaults.review_agent,
        _ => defaults.work_agent,
    };
    let agent = parsed.optional("--agent").unwrap_or(default_agent.as_str());
    let command = parsed.optional("--command");
    let path = parsed.optional("--path");
    let resolution = if purpose == AgentRunPurpose::DispatchPreview {
        match path {
            Some(path) => {
                let resolution =
                    resolve_rule_for_path(&root, Some(path), None).map_err(|e| e.to_string())?;
                print_rule_resolution(&resolution);
                Some(resolution)
            }
            None => None,
        }
    } else {
        None
    };
    let generated_prompt = if parsed.optional("--prompt").is_none() {
        resolution.as_ref().map(dispatch_prompt)
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
