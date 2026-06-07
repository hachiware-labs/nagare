use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use nagare_core::{
    AddAgentProfileInput, AddDomainGroupInput, AddDomainProfileInput, AdvanceUntilBlockedInput,
    AdvanceWorkItemInput, AgentModelSelection, AgentRunPurpose, AnswerWorkItemInput,
    ApplyRecoveryPlanInput, ApprovalPolicy, CreateWorkItemInput, DomainWorkflowOverride,
    ExternalAgentBinding, RunWorkItemInput, StaticUiExportInput, UpdateAgentProfileInput,
    UpdateDomainGroupInput, UpdateDomainProfileInput, WorkflowMode, WorkflowSettings,
    accept_dispatch_plan, accept_recovery_plan, add_agent_profile, add_domain_group,
    add_domain_profile, advance_work_item_once, advance_work_item_until_blocked, answer_work_item,
    apply_recovery_plan, approve_work_item, create_recovery_plan, create_work_item_with_input,
    delete_agent_profile, delete_domain_group, delete_domain_profile, delete_work_item,
    export_static_ui, get_nagare_agent_settings, get_work_item_snapshot, logo_png,
    reject_work_item, run_work_item_with_input, set_workflow_settings, update_agent_profile,
    update_domain_group, update_domain_profile,
};

use crate::args::ParsedArgs;
use crate::ui_detail::render_serve_item_detail;
use crate::ui_form::{
    agent_description_from_fields, derive_work_item_title, json, parse_form_urlencoded,
    split_lines, split_list,
};
use crate::ui_pages::{
    render_serve_agent_form, render_serve_domain_form, render_serve_domain_group_form,
    render_serve_home, render_serve_new_item, render_serve_settings,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct UiRunningState {
    pub kind: String,
    pub actor: String,
    pub label: String,
    pub message: String,
    pub related_action: String,
    pub started_at_epoch: u64,
}

pub(crate) fn ui_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("export") => ui_export_command(&args[1..]),
        Some("open") => ui_open_command(&args[1..]),
        Some("serve") => ui_serve_command(&args[1..]),
        Some(command) => Err(format!("unknown ui command `{command}`")),
        None => Err("ui command required: export, open, serve".to_string()),
    }
}

fn ui_export_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let out_dir = parsed
        .optional("--out")
        .or_else(|| parsed.optional("--output"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".nagare/ui"));
    let result = export_static_ui(parsed.root()?, StaticUiExportInput { out_dir })
        .map_err(|error| error.to_string())?;
    println!("ui_export: {}", result.out_dir.display());
    println!("index: {}", result.index_path.display());
    println!("items: {}", result.item_paths.len());
    Ok(())
}

fn ui_open_command(args: &[String]) -> Result<(), String> {
    let parsed = ParsedArgs::parse(args)?;
    let out_dir = parsed
        .optional("--out")
        .or_else(|| parsed.optional("--output"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".nagare/ui"));
    let should_open = parsed
        .optional("--open")
        .map(parse_bool)
        .transpose()?
        .unwrap_or(true);
    let result = export_static_ui(parsed.root()?, StaticUiExportInput { out_dir })
        .map_err(|error| error.to_string())?;
    println!("ui_export: {}", result.out_dir.display());
    println!("index: {}", result.index_path.display());
    println!("items: {}", result.item_paths.len());
    if should_open {
        open_path(&result.index_path)?;
        println!("opened: {}", result.index_path.display());
    } else {
        println!("open: skipped");
    }
    Ok(())
}

fn ui_serve_command(args: &[String]) -> Result<(), String> {
    let normalized_args = normalize_ui_serve_args(args);
    let parsed = ParsedArgs::parse(&normalized_args)?;
    let root = parsed.root()?;
    let host = parsed.optional("--host").unwrap_or("127.0.0.1");
    let port = parsed
        .optional("--port")
        .unwrap_or("4677")
        .parse::<u16>()
        .map_err(|_| "ui serve --port must be a number".to_string())?;
    let should_open = parsed
        .optional("--open")
        .map(parse_bool)
        .transpose()?
        .unwrap_or(true);
    let bind_addr = format!("{host}:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .map_err(|error| format!("failed to bind UI server on {bind_addr}: {error}"))?;
    let url = format!("http://{bind_addr}/");
    println!("ui_serve: {url}");
    if should_open {
        open_url(&url)?;
        println!("opened: {url}");
    } else {
        println!("open: skipped");
    }
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = handle_ui_request(&root, &mut stream) {
                    let _ = write_response(
                        &mut stream,
                        "500 Internal Server Error",
                        "text/plain; charset=utf-8",
                        &error,
                    );
                }
            }
            Err(error) => eprintln!("ui_serve connection error: {error}"),
        }
    }
    Ok(())
}

fn normalize_ui_serve_args(args: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for arg in args {
        if arg == "--browser" {
            normalized.push("--open".to_string());
            normalized.push("true".to_string());
        } else {
            normalized.push(arg.clone());
        }
    }
    normalized
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid bool `{value}`")),
    }
}

fn handle_ui_request(root: &Path, stream: &mut TcpStream) -> Result<(), String> {
    let request = read_http_request(stream)?;
    if request.method == "GET" && request.path == "/" {
        let html = render_serve_home(root)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" && request.path == "/assets/logo.png" {
        return write_binary_response(stream, "200 OK", "image/png", logo_png());
    }
    if request.method == "GET" && request.path == "/settings" {
        let html = render_serve_settings(root)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" && request.path == "/settings/agents/new" {
        let html = render_serve_agent_form(root, None)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" && request.path == "/settings/domain-groups/new" {
        let html = render_serve_domain_group_form(root, None)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" && request.path == "/settings/domains/new" {
        let html = render_serve_domain_form(root, None)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" {
        if let Some(domain_group_id) = request.path.strip_prefix("/settings/domain-groups/") {
            let html = render_serve_domain_group_form(root, Some(domain_group_id))?;
            return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
        }
    }
    if request.method == "GET" {
        if let Some(domain_profile_id) = request.path.strip_prefix("/settings/domains/") {
            let html = render_serve_domain_form(root, Some(domain_profile_id))?;
            return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
        }
    }
    if request.method == "GET" {
        if let Some(agent_profile_id) = request.path.strip_prefix("/settings/agents/") {
            let html = render_serve_agent_form(root, Some(agent_profile_id))?;
            return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
        }
    }
    if request.method == "GET" && request.path == "/new" {
        let html = render_serve_new_item(root)?;
        return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
    }
    if request.method == "GET" {
        if let Some(work_item_id) = request.path.strip_prefix("/items/") {
            let html = render_serve_item_detail(root, work_item_id)?;
            return write_response(stream, "200 OK", "text/html; charset=utf-8", &html);
        }
    }
    if request.method == "POST" && request.path == "/api/items" {
        let fields = parse_form_urlencoded(&request.body);
        let description = fields.get("description").cloned().unwrap_or_default();
        let title = derive_work_item_title(fields.get("title").map(String::as_str), &description);
        if title.is_empty() {
            return write_response(
                stream,
                "400 Bad Request",
                "application/json; charset=utf-8",
                r#"{"error":"title or description is required"}"#,
            );
        }
        let workflow_mode = fields
            .get("workflow_mode")
            .filter(|value| !value.trim().is_empty())
            .map(String::as_str)
            .map(WorkflowMode::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        let approval_policy = fields
            .get("approval_policy")
            .filter(|value| !value.trim().is_empty())
            .map(String::as_str)
            .map(ApprovalPolicy::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        let result = create_work_item_with_input(
            root,
            CreateWorkItemInput {
                title,
                description,
                acceptance_criteria: split_lines(fields.get("acceptance").map(String::as_str)),
                expected_artifacts: split_lines(fields.get("artifacts").map(String::as_str)),
                work_folder: fields
                    .get("work_folder")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                constraints: split_lines(fields.get("constraints").map(String::as_str)),
                domain_group_id: fields
                    .get("domain_group_id")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                domain_id: fields
                    .get("domain_id")
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                workflow_mode,
                approval_policy,
            },
        )
        .map_err(|error| error.to_string())?;
        spawn_background_ui_workflow(root.to_path_buf(), result.item.id.clone(), fields.clone());
        let body = format!(
            r#"{{"id":"{}","title":"{}","status":"{}","background":"started"}}"#,
            json(&result.item.id),
            json(&result.item.title),
            json(&result.item.status.to_string())
        );
        return write_response(
            stream,
            "201 Created",
            "application/json; charset=utf-8",
            &body,
        );
    }
    if request.method == "POST" {
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/delete"))
        {
            let item = delete_work_item(root, work_item_id).map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","title":"{}"}}"#,
                json(&item.id),
                json(&item.title)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/advance"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let prompt = nonempty_field(&fields, "prompt");
            let command = nonempty_field(&fields, "command");
            let dispatch_command = nonempty_field(&fields, "dispatch_command");
            let review_command = nonempty_field(&fields, "review_command");
            let max_steps = fields
                .get("max_steps")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(8);
            let workflow_mode = fields
                .get("workflow_mode")
                .map(String::as_str)
                .map(WorkflowMode::parse)
                .transpose()
                .map_err(|error| error.to_string())?;
            let result = advance_work_item_until_blocked(
                root,
                work_item_id,
                AdvanceUntilBlockedInput {
                    step: AdvanceWorkItemInput {
                        prompt,
                        dev_command: command,
                        dispatch_dev_command: dispatch_command,
                        review_dev_command: review_command,
                        auto_recover: true,
                        workflow_mode,
                        ..AdvanceWorkItemInput::default()
                    },
                    max_steps,
                },
            )
            .map_err(|error| error.to_string())?;
            let final_snapshot =
                get_work_item_snapshot(root, work_item_id).map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"steps":{},"final_status":"{}","stopped_reason":"{}","next_action":"{}"}}"#,
                result.steps.len(),
                json(&result.final_status.to_string()),
                json(&result.stopped_reason),
                json(&final_snapshot.completion.next_action)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" && request.path == "/api/agents" {
        let fields = parse_form_urlencoded(&request.body);
        let id = fields.get("id").map(String::as_str).unwrap_or("").trim();
        let runtime = fields
            .get("runtime")
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        let adapter = fields
            .get("adapter")
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        if id.is_empty() || runtime.is_empty() || adapter.is_empty() {
            return write_response(
                stream,
                "400 Bad Request",
                "application/json; charset=utf-8",
                r#"{"error":"id, runtime, and adapter are required"}"#,
            );
        }
        let description = agent_description_from_fields(&fields);
        let external = agent_external_from_fields(id, &fields);
        let result = add_agent_profile(
            root,
            AddAgentProfileInput {
                id,
                display_name: fields
                    .get("display_name")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(id),
                runtime,
                adapter,
                role: fields.get("role").map(String::as_str).unwrap_or("").trim(),
                working_dir: fields
                    .get("working_dir")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or("."),
                description: &description,
                specialties: split_list(fields.get("specialties").map(String::as_str)),
                domain_group_ids: split_list(fields.get("domain_group_ids").map(String::as_str)),
                domain_ids: split_list(fields.get("domain_ids").map(String::as_str)),
                managed_by: Some("nagare"),
                model: agent_model_from_fields(&fields),
                external,
            },
        )
        .map_err(|error| error.to_string())?;
        let body = format!(
            r#"{{"id":"{}","adapter":"{}","runtime":"{}"}}"#,
            json(&result.profile.id),
            json(&result.profile.adapter),
            json(&result.profile.runtime)
        );
        return write_response(
            stream,
            "201 Created",
            "application/json; charset=utf-8",
            &body,
        );
    }
    if request.method == "POST" && request.path == "/api/domain-groups" {
        let fields = parse_form_urlencoded(&request.body);
        let id = fields.get("id").map(String::as_str).unwrap_or("").trim();
        if id.is_empty() {
            return write_response(
                stream,
                "400 Bad Request",
                "application/json; charset=utf-8",
                r#"{"error":"id is required"}"#,
            );
        }
        let result = add_domain_group(
            root,
            AddDomainGroupInput {
                id,
                display_name: fields
                    .get("display_name")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(id),
                description: fields.get("description").map(String::as_str).unwrap_or(""),
                shared_knowledge: split_lines(fields.get("shared_knowledge").map(String::as_str)),
                common_rubric: split_lines(fields.get("common_rubric").map(String::as_str)),
                dispatch_hints: split_lines(fields.get("dispatch_hints").map(String::as_str)),
                workflow: domain_workflow_override_from_fields(&fields)?,
            },
        )
        .map_err(|error| error.to_string())?;
        let body = format!(
            r#"{{"id":"{}","rubric":{}}}"#,
            json(&result.group.id),
            result.group.common_rubric.len()
        );
        return write_response(
            stream,
            "201 Created",
            "application/json; charset=utf-8",
            &body,
        );
    }
    if request.method == "POST" && request.path == "/api/domains" {
        let fields = parse_form_urlencoded(&request.body);
        let id = fields.get("id").map(String::as_str).unwrap_or("").trim();
        if id.is_empty() {
            return write_response(
                stream,
                "400 Bad Request",
                "application/json; charset=utf-8",
                r#"{"error":"id is required"}"#,
            );
        }
        let result = add_domain_profile(
            root,
            AddDomainProfileInput {
                id,
                group_id: fields
                    .get("group_id")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty()),
                display_name: fields
                    .get("display_name")
                    .map(String::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(id),
                description: fields.get("description").map(String::as_str).unwrap_or(""),
                artifact_types: split_lines(fields.get("artifact_types").map(String::as_str)),
                rubric: split_lines(fields.get("rubric").map(String::as_str)),
                dispatch_hints: split_lines(fields.get("dispatch_hints").map(String::as_str)),
                workflow: domain_workflow_override_from_fields(&fields)?,
            },
        )
        .map_err(|error| error.to_string())?;
        let body = format!(
            r#"{{"id":"{}","rubric":{}}}"#,
            json(&result.domain.id),
            result.domain.rubric.len()
        );
        return write_response(
            stream,
            "201 Created",
            "application/json; charset=utf-8",
            &body,
        );
    }
    if request.method == "POST" {
        if let Some(domain_group_id) = request
            .path
            .strip_prefix("/api/domain-groups/")
            .and_then(|path| path.strip_suffix("/delete"))
        {
            let group =
                delete_domain_group(root, domain_group_id).map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","display_name":"{}"}}"#,
                json(&group.id),
                json(&group.display_name)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(domain_profile_id) = request
            .path
            .strip_prefix("/api/domains/")
            .and_then(|path| path.strip_suffix("/delete"))
        {
            let domain = delete_domain_profile(root, domain_profile_id)
                .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","display_name":"{}"}}"#,
                json(&domain.id),
                json(&domain.display_name)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(domain_group_id) = request.path.strip_prefix("/api/domain-groups/") {
            let fields = parse_form_urlencoded(&request.body);
            let result = update_domain_group(
                root,
                domain_group_id,
                UpdateDomainGroupInput {
                    display_name: fields.get("display_name").map(String::as_str),
                    description: fields.get("description").map(String::as_str),
                    shared_knowledge: Some(split_lines(
                        fields.get("shared_knowledge").map(String::as_str),
                    )),
                    common_rubric: Some(split_lines(
                        fields.get("common_rubric").map(String::as_str),
                    )),
                    dispatch_hints: Some(split_lines(
                        fields.get("dispatch_hints").map(String::as_str),
                    )),
                    workflow: Some(domain_workflow_override_from_fields(&fields)?),
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","rubric":{}}}"#,
                json(&result.group.id),
                result.group.common_rubric.len()
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(domain_profile_id) = request.path.strip_prefix("/api/domains/") {
            let fields = parse_form_urlencoded(&request.body);
            let result = update_domain_profile(
                root,
                domain_profile_id,
                UpdateDomainProfileInput {
                    group_id: Some(
                        fields
                            .get("group_id")
                            .map(String::as_str)
                            .filter(|value| !value.trim().is_empty()),
                    ),
                    display_name: fields.get("display_name").map(String::as_str),
                    description: fields.get("description").map(String::as_str),
                    artifact_types: Some(split_lines(
                        fields.get("artifact_types").map(String::as_str),
                    )),
                    rubric: Some(split_lines(fields.get("rubric").map(String::as_str))),
                    dispatch_hints: Some(split_lines(
                        fields.get("dispatch_hints").map(String::as_str),
                    )),
                    workflow: Some(domain_workflow_override_from_fields(&fields)?),
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","rubric":{}}}"#,
                json(&result.domain.id),
                result.domain.rubric.len()
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" && request.path == "/api/workflow-settings" {
        let fields = parse_form_urlencoded(&request.body);
        let progress_mode = fields
            .get("default_progress_mode")
            .map(String::as_str)
            .map(WorkflowMode::parse)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let approval_policy = fields
            .get("approval_policy")
            .map(String::as_str)
            .map(ApprovalPolicy::parse)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let settings = set_workflow_settings(
            root,
            WorkflowSettings {
                default_progress_mode: progress_mode,
                approval_policy,
            },
        )
        .map_err(|error| error.to_string())?;
        let body = format!(
            r#"{{"default_progress_mode":"{}","approval_policy":"{}"}}"#,
            json(&settings.default_progress_mode.to_string()),
            json(&settings.approval_policy.to_string())
        );
        return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
    }
    if request.method == "POST" {
        if let Some(agent_profile_id) = request
            .path
            .strip_prefix("/api/agents/")
            .and_then(|path| path.strip_suffix("/delete"))
        {
            let profile =
                delete_agent_profile(root, agent_profile_id).map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","display_name":"{}"}}"#,
                json(&profile.id),
                json(&profile.display_name)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(agent_profile_id) = request.path.strip_prefix("/api/agents/") {
            let fields = parse_form_urlencoded(&request.body);
            let description = agent_description_from_fields(&fields);
            let result = update_agent_profile(
                root,
                agent_profile_id,
                UpdateAgentProfileInput {
                    display_name: fields.get("display_name").map(String::as_str),
                    runtime: fields.get("runtime").map(String::as_str),
                    adapter: fields.get("adapter").map(String::as_str),
                    role: fields.get("role").map(String::as_str),
                    working_dir: fields.get("working_dir").map(String::as_str),
                    description: Some(&description),
                    specialties: Some(split_list(fields.get("specialties").map(String::as_str))),
                    domain_group_ids: Some(split_list(
                        fields.get("domain_group_ids").map(String::as_str),
                    )),
                    domain_ids: Some(split_list(fields.get("domain_ids").map(String::as_str))),
                    output_contract: None,
                    managed_by: Some("nagare"),
                    model: Some(agent_model_from_fields(&fields)),
                    external: Some(agent_external_from_fields(agent_profile_id, &fields)),
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"id":"{}","adapter":"{}","runtime":"{}"}}"#,
                json(&result.profile.id),
                json(&result.profile.adapter),
                json(&result.profile.runtime)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    if request.method == "POST" {
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/answer"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let answer = fields
                .get("answer")
                .map(String::as_str)
                .unwrap_or("")
                .trim();
            if answer.is_empty() {
                return write_response(
                    stream,
                    "400 Bad Request",
                    "application/json; charset=utf-8",
                    r#"{"error":"answer is required"}"#,
                );
            }
            let result = answer_work_item(
                root,
                work_item_id,
                AnswerWorkItemInput {
                    question: fields.get("question").map(String::as_str),
                    answer,
                },
            )
            .map_err(|error| error.to_string())?;
            spawn_background_ui_workflow(root.to_path_buf(), work_item_id.to_string(), fields);
            let body = format!(
                r#"{{"id":"{}","item_status":"{}","background":"started"}}"#,
                json(&result.feedback.id),
                json(&result.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/preview"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let prompt = fields
                .get("prompt")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let command = fields
                .get("command")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            if prompt.is_none() && command.is_none() {
                return write_response(
                    stream,
                    "400 Bad Request",
                    "application/json; charset=utf-8",
                    r#"{"error":"prompt or command is required"}"#,
                );
            }
            let defaults = get_nagare_agent_settings(root).map_err(|error| error.to_string())?;
            let result = run_work_item_with_input(
                root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: &defaults.dispatch_agent,
                    dispatch_plan_id: None,
                    path: None,
                    prompt,
                    dev_command: command,
                    purpose: AgentRunPurpose::DispatchPreview,
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"run":"{}","item_status":"{}","dispatch_plan":"{}"}}"#,
                json(&result.run.id),
                json(&result.item_status.to_string()),
                json(result.dispatch_plan_id.as_deref().unwrap_or(""))
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/dispatch/accept"))
        {
            let result = accept_dispatch_plan(root, work_item_id, None)
                .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"dispatch_plan":"{}","target_agent":"{}"}}"#,
                json(&result.plan.id),
                json(&result.plan.target_agent_profile_id)
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/run"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let prompt = fields
                .get("prompt")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let command = fields
                .get("command")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            if prompt.is_none() && command.is_none() {
                return write_response(
                    stream,
                    "400 Bad Request",
                    "application/json; charset=utf-8",
                    r#"{"error":"prompt or command is required"}"#,
                );
            }
            let defaults = get_nagare_agent_settings(root).map_err(|error| error.to_string())?;
            let result = run_work_item_with_input(
                root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: &defaults.work_agent,
                    dispatch_plan_id: None,
                    path: None,
                    prompt,
                    dev_command: command,
                    purpose: AgentRunPurpose::Work,
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"run":"{}","item_status":"{}"}}"#,
                json(&result.run.id),
                json(&result.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/review"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let prompt = fields
                .get("prompt")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let command = fields
                .get("command")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            if prompt.is_none() && command.is_none() {
                return write_response(
                    stream,
                    "400 Bad Request",
                    "application/json; charset=utf-8",
                    r#"{"error":"prompt or command is required"}"#,
                );
            }
            let defaults = get_nagare_agent_settings(root).map_err(|error| error.to_string())?;
            let result = run_work_item_with_input(
                root,
                work_item_id,
                RunWorkItemInput {
                    agent_profile_id: &defaults.review_agent,
                    dispatch_plan_id: None,
                    path: None,
                    prompt,
                    dev_command: command,
                    purpose: AgentRunPurpose::Review,
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"run":"{}","item_status":"{}"}}"#,
                json(&result.run.id),
                json(&result.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/approve"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let rationale = fields.get("rationale").map(String::as_str).unwrap_or("");
            let result = approve_work_item(root, work_item_id, rationale)
                .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"decision":"{}","item_status":"{}"}}"#,
                json(&result.decision.id),
                json(&result.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/reject"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let rationale = fields
                .get("rationale")
                .or_else(|| fields.get("reason"))
                .map(String::as_str)
                .unwrap_or("");
            let result = reject_work_item(root, work_item_id, rationale)
                .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"decision":"{}","item_status":"{}","next_action":"dispatch"}}"#,
                json(&result.decision.id),
                json(&result.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/recover"))
        {
            let result =
                create_recovery_plan(root, work_item_id).map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"recovery_plan":"{}","status":"{}","action":"{}"}}"#,
                json(&result.plan.id),
                json(&result.plan.status.to_string()),
                json(&result.plan.action.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/recover/accept"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let recovery_plan_id = fields
                .get("recovery_plan")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let result = accept_recovery_plan(root, work_item_id, recovery_plan_id)
                .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"recovery_plan":"{}","status":"{}","action":"{}"}}"#,
                json(&result.plan.id),
                json(&result.plan.status.to_string()),
                json(&result.plan.action.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
        if let Some(work_item_id) = request
            .path
            .strip_prefix("/api/items/")
            .and_then(|path| path.strip_suffix("/recover/apply"))
        {
            let fields = parse_form_urlencoded(&request.body);
            let recovery_plan_id = fields
                .get("recovery_plan")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let prompt = fields
                .get("prompt")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            let command = fields
                .get("command")
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty());
            if prompt.is_none() && command.is_none() {
                return write_response(
                    stream,
                    "400 Bad Request",
                    "application/json; charset=utf-8",
                    r#"{"error":"prompt or command is required"}"#,
                );
            }
            let result = apply_recovery_plan(
                root,
                work_item_id,
                ApplyRecoveryPlanInput {
                    recovery_plan_id,
                    prompt,
                    dev_command: command,
                },
            )
            .map_err(|error| error.to_string())?;
            let body = format!(
                r#"{{"recovery_plan":"{}","run":"{}","item_status":"{}"}}"#,
                json(&result.plan.id),
                json(&result.run.run.id),
                json(&result.run.item_status.to_string())
            );
            return write_response(stream, "200 OK", "application/json; charset=utf-8", &body);
        }
    }
    write_response(
        stream,
        "404 Not Found",
        "text/plain; charset=utf-8",
        "not found",
    )
}

struct HttpRequest {
    method: String,
    path: String,
    body: String,
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let read = stream
            .read(&mut chunk)
            .map_err(|error| format!("failed to read request: {error}"))?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            let request_text = String::from_utf8_lossy(&buffer);
            let content_length = content_length(&request_text);
            let header_end = request_text
                .find("\r\n\r\n")
                .map(|index| index + 4)
                .unwrap_or(buffer.len());
            if buffer.len() >= header_end + content_length {
                break;
            }
        }
    }
    let request_text = String::from_utf8_lossy(&buffer);
    let mut lines = request_text.lines();
    let request_line = lines.next().ok_or_else(|| "empty request".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts
        .next()
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();
    let body = request_text
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();
    Ok(HttpRequest { method, path, body })
}

fn content_length(request: &str) -> usize {
    request
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<(), String> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.as_bytes().len()
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("failed to write response: {error}"))
}

fn write_binary_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|error| format!("failed to write response: {error}"))
}

fn nonempty_field<'a>(fields: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    fields
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn agent_model_from_fields(fields: &HashMap<String, String>) -> AgentModelSelection {
    if nonempty_field(fields, "model_mode") == Some("default") {
        return AgentModelSelection::default();
    }
    AgentModelSelection {
        provider: nonempty_field(fields, "model_provider")
            .unwrap_or("")
            .trim()
            .to_string(),
        id: nonempty_field(fields, "model_id")
            .unwrap_or("")
            .trim()
            .to_string(),
        base_url: nonempty_field(fields, "base_url")
            .unwrap_or("")
            .trim()
            .to_string(),
        api_key_env: nonempty_field(fields, "api_key_env")
            .unwrap_or("")
            .trim()
            .to_string(),
    }
}

fn agent_external_from_fields(id: &str, fields: &HashMap<String, String>) -> ExternalAgentBinding {
    let provider = nonempty_field(fields, "external_provider")
        .or_else(|| match nonempty_field(fields, "agent_kind") {
            Some("codex_app_server") => Some("codex"),
            Some("openclaw") => Some("openclaw"),
            _ => Some("codex-cli"),
        })
        .unwrap_or("codex-cli")
        .trim()
        .to_string();
    ExternalAgentBinding {
        provider,
        agent_id: nonempty_field(fields, "external_agent_id")
            .unwrap_or(id)
            .trim()
            .to_string(),
        managed: fields
            .get("external_managed")
            .map(String::as_str)
            .map(parse_bool)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or(true),
        source: nonempty_field(fields, "external_source")
            .unwrap_or("created")
            .trim()
            .to_string(),
    }
}

fn domain_workflow_override_from_fields(
    fields: &HashMap<String, String>,
) -> Result<DomainWorkflowOverride, String> {
    let progress_mode = fields
        .get("workflow_progress_mode")
        .filter(|value| !value.trim().is_empty())
        .map(String::as_str)
        .map(WorkflowMode::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    let approval_policy = fields
        .get("workflow_approval_policy")
        .filter(|value| !value.trim().is_empty())
        .map(String::as_str)
        .map(ApprovalPolicy::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok(DomainWorkflowOverride {
        progress_mode,
        approval_policy,
    })
}

fn advance_ui_workflow(
    root: &Path,
    work_item_id: &str,
    fields: &HashMap<String, String>,
) -> Result<String, String> {
    let max_steps = fields
        .get("max_steps")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8);
    let workflow_mode = fields
        .get("workflow_mode")
        .map(String::as_str)
        .map(WorkflowMode::parse)
        .transpose()
        .map_err(|error| error.to_string())?
        .or(Some(WorkflowMode::FinishFirst));
    let step = AdvanceWorkItemInput {
        prompt: nonempty_field(fields, "prompt"),
        dev_command: nonempty_field(fields, "command"),
        dispatch_dev_command: nonempty_field(fields, "dispatch_command"),
        review_dev_command: nonempty_field(fields, "review_command"),
        auto_recover: true,
        workflow_mode,
        ..AdvanceWorkItemInput::default()
    };
    let mut count = 0;
    let mut stopped_reason = "max_steps_reached".to_string();
    for _ in 0..max_steps.max(1) {
        let snapshot =
            get_work_item_snapshot(root, work_item_id).map_err(|error| error.to_string())?;
        write_ui_running_state(
            root,
            work_item_id,
            &running_state_for_snapshot(root, &snapshot),
        )?;
        let result = advance_work_item_once(root, work_item_id, step.clone())
            .map_err(|error| error.to_string())?;
        count += 1;
        stopped_reason = result.message.clone();
        clear_ui_running_state(root, work_item_id);
        if !result.advanced {
            break;
        }
        let snapshot =
            get_work_item_snapshot(root, work_item_id).map_err(|error| error.to_string())?;
        if matches!(
            snapshot.completion.next_action.as_str(),
            "answer_question" | "approve" | "none" | "wait" | "stop"
        ) {
            break;
        }
    }
    clear_ui_running_state(root, work_item_id);
    Ok(format!("{count} step(s): {stopped_reason}"))
}

fn spawn_background_ui_workflow(
    root: PathBuf,
    work_item_id: String,
    fields: HashMap<String, String>,
) {
    thread::spawn(move || {
        let _ = write_ui_running_state(
            &root,
            &work_item_id,
            &UiRunningState::new(
                "dispatch",
                "dispatcher",
                "dispatcher dispatch_preview",
                "担当 Agent を選定しています。",
                "dispatch",
            ),
        );
        let dispatch = run_initial_ui_dispatch(&root, &work_item_id);
        clear_ui_running_state(&root, &work_item_id);
        if let Err(error) = dispatch {
            eprintln!("ui background dispatch failed for {work_item_id}: {error}");
            return;
        }
        if let Err(error) = advance_ui_workflow(&root, &work_item_id, &fields) {
            eprintln!("ui background workflow failed for {work_item_id}: {error}");
        }
    });
}

fn running_state_for_snapshot(
    root: &Path,
    snapshot: &nagare_core::WorkItemSnapshot,
) -> UiRunningState {
    let settings = get_nagare_agent_settings(root).ok();
    match snapshot.completion.next_action.as_str() {
        "dispatch" => {
            let actor = settings
                .as_ref()
                .map(|settings| settings.dispatch_agent.clone())
                .unwrap_or_else(|| "dispatcher".to_string());
            UiRunningState::new(
                "dispatch",
                &actor,
                &format!("{actor} dispatch_preview"),
                "担当 Agent を選定しています。",
                "dispatch",
            )
        }
        "run_agent" => {
            let actor = snapshot
                .dispatch_plans
                .iter()
                .rev()
                .map(|plan| plan.target_agent_profile_id.clone())
                .next()
                .or_else(|| {
                    settings
                        .as_ref()
                        .map(|settings| settings.work_agent.clone())
                })
                .unwrap_or_else(|| "worker".to_string());
            UiRunningState::new(
                "work",
                &actor,
                &format!("{actor} work"),
                "Work Agent が依頼を処理しています。",
                "run_agent",
            )
        }
        "review" => {
            let actor = settings
                .as_ref()
                .map(|settings| settings.review_agent.clone())
                .unwrap_or_else(|| "reviewer".to_string());
            UiRunningState::new(
                "review",
                &actor,
                &format!("{actor} review"),
                "Review Agent が受入条件を確認しています。",
                "review",
            )
        }
        "recover" | "apply_recovery" => UiRunningState::new(
            "recovery",
            "Workflow",
            "recovery workflow",
            "回復フローを進めています。",
            snapshot.completion.next_action.as_str(),
        ),
        other => UiRunningState::new(
            "workflow",
            "Workflow",
            other,
            "Workflow を進めています。",
            other,
        ),
    }
}

impl UiRunningState {
    fn new(kind: &str, actor: &str, label: &str, message: &str, related_action: &str) -> Self {
        Self {
            kind: kind.to_string(),
            actor: actor.to_string(),
            label: label.to_string(),
            message: message.to_string(),
            related_action: related_action.to_string(),
            started_at_epoch: current_epoch_seconds(),
        }
    }
}

fn ui_running_state_path(root: &Path, work_item_id: &str) -> PathBuf {
    root.join(".nagare")
        .join("state")
        .join(format!("{work_item_id}-ui-running.txt"))
}

fn write_ui_running_state(
    root: &Path,
    work_item_id: &str,
    state: &UiRunningState,
) -> Result<(), String> {
    let path = ui_running_state_path(root, work_item_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create UI state directory: {error}"))?;
    }
    let raw = serde_json::to_string_pretty(state)
        .map_err(|error| format!("failed to encode UI running state: {error}"))?;
    fs::write(path, raw).map_err(|error| format!("failed to write UI running state: {error}"))
}

pub(crate) fn read_ui_running_state(root: &Path, work_item_id: &str) -> Option<String> {
    read_ui_running_status(root, work_item_id).map(|state| state.label)
}

pub(crate) fn read_ui_running_status(root: &Path, work_item_id: &str) -> Option<UiRunningState> {
    let raw = fs::read_to_string(ui_running_state_path(root, work_item_id)).ok()?;
    if let Ok(state) = serde_json::from_str::<UiRunningState>(&raw) {
        return Some(state);
    }
    let mut lines = raw.lines();
    let label = lines
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())?
        .to_string();
    let started_at_epoch = lines
        .next()
        .and_then(|line| line.trim().parse::<u64>().ok())
        .unwrap_or_else(current_epoch_seconds);
    Some(UiRunningState {
        kind: "workflow".to_string(),
        actor: "-".to_string(),
        label,
        message: "Workflow を進めています。".to_string(),
        related_action: "unknown".to_string(),
        started_at_epoch,
    })
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn clear_ui_running_state(root: &Path, work_item_id: &str) {
    fs::remove_file(ui_running_state_path(root, work_item_id)).ok();
}

fn run_initial_ui_dispatch(root: &Path, work_item_id: &str) -> Result<Option<String>, String> {
    let defaults = get_nagare_agent_settings(root).map_err(|error| error.to_string())?;
    let state_dir = root.join(".nagare").join("state");
    fs::create_dir_all(&state_dir)
        .map_err(|error| format!("failed to create UI dispatch state directory: {error}"))?;
    let file_name = format!("{work_item_id}-initial-dispatch.json");
    let output_path = state_dir.join(&file_name);
    let output = format!(
        r#"{{"target_agent_profile_id":"{}","summary":"UI selected the default work agent on item creation.","risks":[],"missing_information":[]}}"#,
        json(&defaults.work_agent)
    );
    fs::write(&output_path, output)
        .map_err(|error| format!("failed to write UI dispatch output: {error}"))?;
    let command = if cfg!(windows) {
        format!("type {}", output_path.display())
    } else {
        format!(r#"cat "{}""#, output_path.display())
    };
    let result = run_work_item_with_input(
        root,
        work_item_id,
        RunWorkItemInput {
            agent_profile_id: &defaults.dispatch_agent,
            dispatch_plan_id: None,
            path: None,
            prompt: Some("Select the initial execution agent for this work item."),
            dev_command: Some(&command),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(result.dispatch_plan_id)
}

fn open_path(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(path)
        .status();

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(path).status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(path).status();

    let status = status.map_err(|error| format!("failed to open UI: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to open UI with status {status}; open manually: {}",
            path.display()
        ))
    }
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd").args(["/C", "start", "", url]).status();

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(url).status();

    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(url).status();

    let status = status.map_err(|error| format!("failed to open UI: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to open UI with status {status}; open manually: {url}"
        ))
    }
}
