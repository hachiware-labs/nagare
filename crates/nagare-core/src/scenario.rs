use std::path::PathBuf;

use crate::*;

pub fn run_first_scenario(root: impl Into<PathBuf>) -> Result<ScenarioResult, NagareError> {
    let root = root.into();
    init_project(&root)?;
    let item = create_work_item(
        &root,
        "Repair failing agent run",
        "Demonstrate Codex CLI failure, Codex App Server handoff, review, and approval.",
    )?
    .item;
    let codex_run = run_work_item(
        &root,
        &item.id,
        "worker",
        scenario_command("codex run failed", false).as_str(),
    )?
    .run;
    let handoff = create_handoff(
        &root,
        &item.id,
        "worker",
        "reviewer",
        "Codex agent profile produced a failing run",
        "Retry with Codex App Server agent profile using the captured run log as evidence.",
    )?
    .handoff;
    let codex_app_run = run_work_item(
        &root,
        &item.id,
        "reviewer",
        scenario_command("codex app server retry fixed the task", true).as_str(),
    )?
    .run;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(scenario_review_command("review passed").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )?;
    let review = get_work_item_snapshot(&root, &item.id)?
        .review_results
        .last()
        .cloned()
        .ok_or_else(|| NagareError::InvalidState("scenario review should record".to_string()))?;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required review passed after cross-agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        review_id: review.id,
        decision_id: decision.id,
        final_status,
    })
}

pub fn run_registered_agent_scenario(
    root: impl Into<PathBuf>,
) -> Result<ScenarioResult, NagareError> {
    let root = root.into();
    init_project(&root)?;
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-impl-smoke",
            display_name: "Codex CLI Smoke Implementer",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
            description: "Codex CLI profile used for smoke-test work execution.",
            specialties: vec!["implementation".to_string(), "review-checks".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding {
                provider: "codex-cli".to_string(),
                agent_id: "codex-impl-smoke".to_string(),
                managed: true,
                source: "created".to_string(),
            },
        },
    )?;
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "codex-app-smoke",
            display_name: "Codex App Server Smoke Implementer",
            runtime: "codex-app-local",
            adapter: "stdio.codex-app-server",
            role: "implementer",
            working_dir: ".",
            description: "Codex app-server profile used for smoke-test review and planning.",
            specialties: vec!["review".to_string(), "planning".to_string()],
            skill_set_ids: Vec::new(),
            domain_ids: Vec::new(),
            artifact_type_ids: Vec::new(),
            managed_by: Some("nagare"),
            model: AgentModelSelection::default(),
            external: ExternalAgentBinding {
                provider: "codex".to_string(),
                agent_id: "codex-app-smoke".to_string(),
                managed: true,
                source: "created".to_string(),
            },
        },
    )?;

    let item = create_work_item(
        &root,
        "Repair failing registered agent run",
        "Demonstrate registered Agent Profiles, handoff, review, and approval.",
    )?
    .item;
    let codex_run = run_work_item(
        &root,
        &item.id,
        "codex-impl-smoke",
        scenario_command("registered codex run failed", false).as_str(),
    )?
    .run;
    let handoff = create_handoff(
        &root,
        &item.id,
        "codex-impl-smoke",
        "codex-app-smoke",
        "Registered Codex agent profile produced a failing run",
        "Retry with the registered Codex App Server profile using the captured run log as evidence.",
    )?
    .handoff;
    let codex_app_run = run_work_item(
        &root,
        &item.id,
        "codex-app-smoke",
        scenario_command("registered codex app server retry fixed the task", true).as_str(),
    )?
    .run;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-app-smoke",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(scenario_review_command("registered review passed").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )?;
    let review = get_work_item_snapshot(&root, &item.id)?
        .review_results
        .last()
        .cloned()
        .ok_or_else(|| {
            NagareError::InvalidState("registered scenario review should record".to_string())
        })?;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required review passed after registered agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        review_id: review.id,
        decision_id: decision.id,
        final_status,
    })
}
