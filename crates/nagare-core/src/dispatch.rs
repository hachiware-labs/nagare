use std::path::PathBuf;

use crate::*;

pub fn accept_dispatch_plan(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    dispatch_plan_id: Option<&str>,
) -> Result<AcceptDispatchPlanResult, NagareError> {
    let layout = ensure_project(root)?;
    let mut ledger = load_ledger(&layout)?;
    ledger.work_item(work_item_id)?;

    let selected_index = match dispatch_plan_id {
        Some(id) => ledger
            .dispatch_plans
            .iter()
            .position(|plan| plan.work_item_id == work_item_id && plan.id == id),
        None => ledger.dispatch_plans.iter().rposition(|plan| {
            plan.work_item_id == work_item_id && plan.status == DispatchPlanStatus::Draft
        }),
    }
    .ok_or_else(|| {
        let target = dispatch_plan_id.unwrap_or("latest draft");
        NagareError::NotFound(format!("dispatch plan `{target}` for `{work_item_id}`"))
    })?;

    let target_agent_profile_id = ledger.dispatch_plans[selected_index]
        .target_agent_profile_id
        .clone();
    validate_existing_agent_profile(&layout, &target_agent_profile_id)?;
    let selected_id = ledger.dispatch_plans[selected_index].id.clone();
    for plan in &mut ledger.dispatch_plans {
        if plan.work_item_id == work_item_id && plan.status != DispatchPlanStatus::Superseded {
            plan.status = if plan.id == selected_id {
                DispatchPlanStatus::Accepted
            } else {
                DispatchPlanStatus::Superseded
            };
        }
    }
    let plan = ledger.dispatch_plans[selected_index].clone();
    save_ledger(&layout, &ledger)?;
    Ok(AcceptDispatchPlanResult { plan })
}

pub fn select_agent_for_work_item_run(
    root: impl Into<PathBuf>,
    work_item_id: &str,
    input: SelectRunAgentInput<'_>,
) -> Result<SelectRunAgentResult, NagareError> {
    let layout = ensure_project(root)?;
    let ledger = load_ledger(&layout)?;
    ledger.work_item(work_item_id)?;

    if let Some(agent_profile_id) = input.explicit_agent_profile_id {
        validate_existing_agent_profile(&layout, agent_profile_id)?;
        return Ok(SelectRunAgentResult {
            agent_profile_id: agent_profile_id.to_string(),
            source: RunAgentSelectionSource::Explicit,
            dispatch_plan_id: None,
            rule_resolution: None,
        });
    }

    if let Some(dispatch_plan_id) = input.dispatch_plan_id {
        let plan = dispatch_plan_for_run(&ledger, work_item_id, dispatch_plan_id)?;
        validate_existing_agent_profile(&layout, &plan.target_agent_profile_id)?;
        return Ok(SelectRunAgentResult {
            agent_profile_id: plan.target_agent_profile_id.clone(),
            source: RunAgentSelectionSource::DispatchPlan,
            dispatch_plan_id: Some(plan.id.clone()),
            rule_resolution: None,
        });
    }

    if let Some(plan) = latest_accepted_dispatch_plan(&ledger, work_item_id) {
        validate_existing_agent_profile(&layout, &plan.target_agent_profile_id)?;
        return Ok(SelectRunAgentResult {
            agent_profile_id: plan.target_agent_profile_id.clone(),
            source: RunAgentSelectionSource::DispatchPlan,
            dispatch_plan_id: Some(plan.id.clone()),
            rule_resolution: None,
        });
    }

    if let Some(path) = input.path {
        let resolution = resolve_rule_for_path_from_layout(&layout, Some(path), None)?;
        return Ok(SelectRunAgentResult {
            agent_profile_id: resolution.agent_profile_id.clone(),
            source: RunAgentSelectionSource::ProjectRule,
            dispatch_plan_id: None,
            rule_resolution: Some(resolution),
        });
    }

    let settings = load_project_config(&layout)?.nagare_agents;
    validate_existing_agent_profile(&layout, &settings.work_agent)?;
    Ok(SelectRunAgentResult {
        agent_profile_id: settings.work_agent,
        source: RunAgentSelectionSource::Default,
        dispatch_plan_id: None,
        rule_resolution: None,
    })
}

fn dispatch_plan_for_run<'a>(
    ledger: &'a Ledger,
    work_item_id: &str,
    dispatch_plan_id: &str,
) -> Result<&'a DispatchPlan, NagareError> {
    let plan = ledger
        .dispatch_plans
        .iter()
        .find(|plan| plan.work_item_id == work_item_id && plan.id == dispatch_plan_id)
        .ok_or_else(|| {
            NagareError::NotFound(format!(
                "dispatch plan `{dispatch_plan_id}` for `{work_item_id}`"
            ))
        })?;
    if plan.status != DispatchPlanStatus::Accepted {
        return Err(NagareError::InvalidState(format!(
            "dispatch plan `{dispatch_plan_id}` is {}, not accepted",
            plan.status
        )));
    }
    Ok(plan)
}

fn latest_accepted_dispatch_plan<'a>(
    ledger: &'a Ledger,
    work_item_id: &str,
) -> Option<&'a DispatchPlan> {
    ledger.dispatch_plans.iter().rev().find(|plan| {
        plan.work_item_id == work_item_id && plan.status == DispatchPlanStatus::Accepted
    })
}
