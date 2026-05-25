use std::path::PathBuf;

use crate::*;

pub fn run_first_scenario(root: impl Into<PathBuf>) -> Result<ScenarioResult, NagareError> {
    let root = root.into();
    init_project(&root)?;
    let item = create_work_item(
        &root,
        "Repair failing agent run",
        "Demonstrate Codex CLI failure, Codex App Server handoff, verification, and approval.",
    )?
    .item;
    let codex_run = run_work_item(
        &root,
        &item.id,
        "codex-cli",
        scenario_command("codex run failed", false).as_str(),
    )?
    .run;
    let handoff = create_handoff(
        &root,
        &item.id,
        "codex-cli",
        "codex-app-server",
        "Codex agent profile produced a failing run",
        "Retry with Codex App Server agent profile using the captured run log as evidence.",
    )?
    .handoff;
    let codex_app_run = run_work_item(
        &root,
        &item.id,
        "codex-app-server",
        scenario_command("codex app server retry fixed the task", true).as_str(),
    )?
    .run;
    let verification = verify_work_item(
        &root,
        &item.id,
        scenario_command("verification passed", true).as_str(),
    )?
    .verification;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required verification passed after cross-agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        verification_id: verification.id,
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
            specialties: vec!["implementation".to_string(), "verification".to_string()],
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
        },
    )?;

    let item = create_work_item(
        &root,
        "Repair failing registered agent run",
        "Demonstrate registered Agent Profiles, handoff, verification, and approval.",
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
    let verification = verify_work_item(
        &root,
        &item.id,
        scenario_command("registered verification passed", true).as_str(),
    )?
    .verification;
    let decision = approve_work_item(
        &root,
        &item.id,
        "Required verification passed after registered agent handoff.",
    )?
    .decision;
    let final_status = get_work_item_snapshot(&root, &item.id)?.item.status;

    Ok(ScenarioResult {
        work_item_id: item.id,
        codex_run_id: codex_run.id,
        handoff_id: handoff.id,
        codex_app_run_id: codex_app_run.id,
        verification_id: verification.id,
        decision_id: decision.id,
        final_status,
    })
}
