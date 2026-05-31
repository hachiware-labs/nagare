use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use nagare_core::*;

fn test_root(label: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    env::temp_dir().join(format!("nagare-{label}-{}-{now}", std::process::id()))
}

fn cat_command(path: &str) -> String {
    if cfg!(windows) {
        format!("type {path}")
    } else {
        format!("cat {path}")
    }
}

fn event_count(snapshot: &WorkItemSnapshot, event_type: &str) -> usize {
    snapshot
        .timeline
        .iter()
        .filter(|event| event.event_type == event_type)
        .count()
}

#[test]
fn work_item_definition_of_done_flows_into_run_packet() {
    let root = test_root("work-item-dod");
    init_project(&root).expect("project should init");
    fs::create_dir_all(root.join("docs")).expect("docs dir should exist");
    fs::write(
        root.join("done.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- documented acceptance criteria\nartifacts:\n- docs/feature.md\nnext_action: review\n",
    )
    .expect("result should write");

    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Define done".to_string(),
            description: "Capture success conditions before running.".to_string(),
            acceptance_criteria: vec!["criteria are visible".to_string()],
            expected_artifacts: vec!["docs/feature.md".to_string()],
            work_folder: Some("docs".to_string()),
            constraints: vec!["keep docs concise".to_string()],
            workflow_mode: Some(WorkflowMode::ConfirmFirst),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("done.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("run should use item definition");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(
        snapshot.item.acceptance_criteria,
        vec!["criteria are visible".to_string()]
    );
    assert_eq!(snapshot.item.work_folder.as_deref(), Some("docs"));
    let packet = snapshot
        .resolved_run_packets
        .last()
        .expect("run packet should exist");
    assert_eq!(packet.path.as_deref(), Some("docs"));
    assert_eq!(packet.work_folder.as_deref(), Some("docs"));
    assert!(packet.goal.contains("## Acceptance Criteria"));
    assert!(
        packet
            .constraints
            .contains(&"acceptance_criteria_context_applied".to_string())
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn workflow_decision_records_next_structured_action() {
    let root = test_root("workflow-decision");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Decide next action", "")
        .expect("item")
        .item;

    let decision = create_workflow_decision(&root, &item.id)
        .expect("decision should record")
        .decision;
    assert_eq!(decision.action, WorkflowDecisionAction::Dispatch);
    assert_eq!(decision.source, WorkflowDecisionSource::Deterministic);
    assert!(!decision.requires_human);
    assert_eq!(
        decision.target_agent_profile_id.as_deref(),
        Some("dispatcher")
    );

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.workflow_decisions.len(), 1);
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| { event.event_type == "workflow_decision" && event.status == "dispatch" })
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn advance_once_records_and_executes_one_workflow_step() {
    let root = test_root("advance-once");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use the default work agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch output should write");
    let item = create_work_item(&root, "Advance one step", "")
        .expect("item")
        .item;

    let dispatched = advance_work_item_once(
        &root,
        &item.id,
        AdvanceWorkItemInput {
            dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
            ..AdvanceWorkItemInput::default()
        },
    )
    .expect("advance should dispatch");
    assert_eq!(dispatched.decision.action, WorkflowDecisionAction::Dispatch);
    assert!(dispatched.advanced);
    assert!(dispatched.dispatch_plan_id.is_some());

    let accepted = advance_work_item_once(&root, &item.id, AdvanceWorkItemInput::default())
        .expect("advance should accept draft dispatch");
    assert_eq!(
        accepted.decision.action,
        WorkflowDecisionAction::AcceptDispatch
    );
    assert!(accepted.advanced);

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.workflow_decisions.len(), 2);
    assert_eq!(
        snapshot.dispatch_plans[0].status,
        DispatchPlanStatus::Accepted
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn supervisor_agent_can_record_workflow_decision_for_advance() {
    let root = test_root("supervisor-decision");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("supervisor.md"),
        "## Nagare Workflow Decision\naction: dispatch\nreason: supervisor selected dispatch first\ntarget_agent_profile_id: worker\nrequires_human: false\nconfidence: 0.91\ncommand_hint: nagare item preview\n",
    )
    .expect("supervisor output should write");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Supervisor chose default dispatch.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch output should write");
    let item = create_work_item(&root, "Supervisor advances", "")
        .expect("item")
        .item;

    let result = advance_work_item_once(
        &root,
        &item.id,
        AdvanceWorkItemInput {
            use_supervisor: true,
            supervisor_dev_command: Some(cat_command("supervisor.md").as_str()),
            dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
            ..AdvanceWorkItemInput::default()
        },
    )
    .expect("advance should use supervisor decision");

    assert_eq!(
        result.decision.source,
        WorkflowDecisionSource::SupervisorAgent
    );
    assert_eq!(result.decision.action, WorkflowDecisionAction::Dispatch);
    assert_eq!(result.decision.reason, "supervisor selected dispatch first");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert!(
        snapshot
            .runs
            .iter()
            .any(|run| run.purpose == AgentRunPurpose::WorkflowSupervision)
    );
    assert!(
        snapshot
            .runs
            .iter()
            .any(|run| run.purpose == AgentRunPurpose::DispatchPreview)
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn advance_until_blocked_runs_to_human_approval_gate() {
    let root = test_root("advance-until-blocked");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use default work agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch output should write");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work done\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- criteria satisfied\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Advance until blocked".to_string(),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;

    let result = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
                dev_command: Some(cat_command("result.md").as_str()),
                review_dev_command: Some(cat_command("review.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 8,
        },
    )
    .expect("workflow should advance until approval");

    assert_eq!(result.final_status, WorkItemStatus::ReadyForReview);
    assert_eq!(
        result.steps.last().expect("last step").decision.action,
        WorkflowDecisionAction::Approve
    );
    assert!(!result.steps.last().expect("last step").advanced);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(
        event_count(&snapshot, "workflow_decision"),
        result.steps.len()
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn auto_complete_policy_finishes_after_passing_review() {
    let root = test_root("auto-complete-policy");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use default work agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch output should write");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work done\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- criteria satisfied\nfindings:\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Auto complete".to_string(),
            approval_policy: Some(ApprovalPolicy::AutoCompleteOnReviewPass),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;

    let result = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
                dev_command: Some(cat_command("result.md").as_str()),
                review_dev_command: Some(cat_command("review.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 8,
        },
    )
    .expect("workflow should auto complete");

    assert_eq!(result.final_status, WorkItemStatus::Done);
    assert_eq!(
        result.steps.last().expect("last step").decision.action,
        WorkflowDecisionAction::Done
    );
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert!(snapshot.decisions.iter().any(|decision| {
        decision.decision_type == "approve"
            && decision.rationale.contains("auto_complete_on_review_pass")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn recovery_classifies_missing_artifact_and_no_diff_candidates() {
    let root = test_root("recovery-intelligence");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work claimed done\nnext_action: review\n",
    )
    .expect("result should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Needs concrete artifact".to_string(),
            expected_artifacts: vec!["docs/output.md".to_string()],
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("result.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work should run");

    create_recovery_plan(&root, &item.id).expect("recovery should create candidates");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    let classes = snapshot
        .recovery_plans
        .iter()
        .map(|plan| plan.failure_class.as_str())
        .collect::<Vec<_>>();
    assert!(classes.contains(&"missing_artifact"));
    assert!(classes.contains(&"no_diff"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn criteria_aware_review_blocks_and_then_allows_approval() {
    let root = test_root("criteria-review");
    init_project(&root).expect("project should init");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Criteria gated approval".to_string(),
            acceptance_criteria: vec!["docs mention locale".to_string()],
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;
    fs::write(
        root.join("review_missing.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- Looks good.\nfindings:\n- Criteria not checked.\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review_missing.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should run");
    assert_eq!(
        get_work_item_snapshot(&root, &item.id)
            .expect("snapshot")
            .item
            .status,
        WorkItemStatus::ChangesRequested
    );

    fs::write(
        root.join("review_pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- Criteria covered.\ncriteria:\n- docs mention locale: pass\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review_pass.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("criteria review should run");
    approve_work_item(&root, &item.id, "criteria passed").expect("approval should pass");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert_eq!(
        snapshot
            .review_results
            .last()
            .expect("review")
            .criteria_results[0]
            .status,
        CriteriaReviewStatus::Passed
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn complex_workflow_recovers_from_review_changes_to_approval() {
    let root = test_root("complex-advance-recovery");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use implementation agent.","risks":["criteria may be missed"],"missing_information":[]}"#,
    )
    .expect("dispatch should write");
    fs::write(
        root.join("initial_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- initial work completed\nartifacts:\n- docs/final.md\nnext_action: review\n",
    )
    .expect("initial result should write");
    fs::write(
        root.join("review_changes.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- criteria not proven\nrequested_changes:\n- Add explicit criteria evidence.\nquestions:\nnext_action: run_agent\n",
    )
    .expect("review changes should write");
    fs::write(
        root.join("fixed_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- criteria evidence added\nartifacts:\n- docs/final.md\nevidence:\n- final artifact recorded\nnext_action: review\n",
    )
    .expect("fixed result should write");
    fs::write(
        root.join("review_pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- criteria covered\ncriteria:\n- final artifact recorded: pass\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review pass should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Complex recovery workflow".to_string(),
            acceptance_criteria: vec!["final artifact recorded".to_string()],
            expected_artifacts: vec!["docs/final.md".to_string()],
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;

    let first = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
                dev_command: Some(cat_command("initial_result.md").as_str()),
                review_dev_command: Some(cat_command("review_changes.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 5,
        },
    )
    .expect("workflow should stop at recovery plan");
    assert_eq!(first.final_status, WorkItemStatus::ChangesRequested);
    assert_eq!(
        first.steps.last().expect("last").decision.action,
        WorkflowDecisionAction::CreateRecoveryPlan
    );

    let accept_gate = advance_work_item_once(&root, &item.id, AdvanceWorkItemInput::default())
        .expect("default advance should require recovery acceptance");
    assert_eq!(
        accept_gate.decision.action,
        WorkflowDecisionAction::AcceptRecoveryPlan
    );
    assert!(!accept_gate.advanced);
    assert_eq!(accept_gate.message, "recovery plan acceptance required");

    accept_recovery_plan(&root, &item.id, None).expect("recovery should accept");
    apply_recovery_plan(
        &root,
        &item.id,
        ApplyRecoveryPlanInput {
            recovery_plan_id: None,
            prompt: None,
            dev_command: Some(cat_command("fixed_result.md").as_str()),
        },
    )
    .expect("recovery should rerun work");

    let final_gate = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                review_dev_command: Some(cat_command("review_pass.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 5,
        },
    )
    .expect("workflow should reach approval gate");
    assert_eq!(final_gate.final_status, WorkItemStatus::ReadyForReview);
    assert_eq!(
        final_gate.steps.last().expect("last").decision.action,
        WorkflowDecisionAction::Approve
    );
    approve_work_item(&root, &item.id, "complex recovered workflow complete")
        .expect("approval should complete");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert!(
        snapshot
            .recovery_plans
            .iter()
            .any(|plan| plan.failure_class == "review_changes")
    );
    assert!(snapshot.workflow_decisions.len() >= 6);
    fs::remove_dir_all(root).ok();
}

#[test]
fn finish_first_workflow_auto_recovers_to_approval_gate() {
    let root = test_root("finish-first-auto-recovers");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use implementation agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch should write");
    fs::write(
        root.join("initial_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- initial work completed\nnext_action: review\n",
    )
    .expect("initial result should write");
    fs::write(
        root.join("fixed_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- evidence corrected\nnext_action: review\n",
    )
    .expect("fixed result should write");
    fs::write(
        root.join("review_changes.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- criteria not proven\nrequested_changes:\n- Add explicit criteria evidence.\nquestions:\nnext_action: run_agent\n",
    )
    .expect("review changes should write");
    fs::write(
        root.join("review_pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- criteria covered\ncriteria:\n- evidence recorded: pass\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review pass should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Auto recovery workflow".to_string(),
            acceptance_criteria: vec!["evidence recorded".to_string()],
            workflow_mode: Some(WorkflowMode::FinishFirst),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item")
    .item;

    let review_changes = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                dispatch_dev_command: Some(cat_command("dispatch.json").as_str()),
                dev_command: Some(cat_command("initial_result.md").as_str()),
                review_dev_command: Some(cat_command("review_changes.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 7,
        },
    )
    .expect("auto recovery should apply review-change recovery");

    assert_eq!(review_changes.final_status, WorkItemStatus::ReadyForReview);
    assert!(
        review_changes
            .steps
            .iter()
            .any(|step| step.decision.action == WorkflowDecisionAction::AcceptRecoveryPlan)
    );
    assert!(
        review_changes
            .steps
            .iter()
            .any(|step| step.decision.action == WorkflowDecisionAction::ApplyRecoveryPlan)
    );

    let approval_gate = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                review_dev_command: Some(cat_command("review_pass.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 8,
        },
    )
    .expect("workflow should stop at approval gate");

    assert_eq!(approval_gate.final_status, WorkItemStatus::ReadyForReview);
    assert_eq!(
        approval_gate.steps.last().expect("last").decision.action,
        WorkflowDecisionAction::Approve
    );
    assert!(!approval_gate.steps.last().expect("last").advanced);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.workflow_mode, WorkflowMode::FinishFirst);
    assert!(snapshot.review_results.len() >= 2);
    assert!(
        snapshot
            .recovery_plans
            .iter()
            .any(|plan| plan.failure_class == "review_changes")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn advance_applies_accepted_recovery_plan() {
    let root = test_root("advance-applies-recovery");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("initial_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- initial work\nnext_action: review\n",
    )
    .expect("initial result should write");
    fs::write(
        root.join("review_changes.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- change requested\nrequested_changes:\n- Add evidence.\nquestions:\nnext_action: run_agent\n",
    )
    .expect("review changes should write");
    fs::write(
        root.join("fixed_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- evidence added\nnext_action: review\n",
    )
    .expect("fixed result should write");
    let item = create_work_item(&root, "Apply accepted recovery", "")
        .expect("item")
        .item;

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("initial_result.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("initial work should run");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review_changes.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should request changes");
    create_recovery_plan(&root, &item.id).expect("recovery should create");
    accept_recovery_plan(&root, &item.id, None).expect("recovery should accept");

    let result = advance_work_item_once(
        &root,
        &item.id,
        AdvanceWorkItemInput {
            dev_command: Some(cat_command("fixed_result.md").as_str()),
            ..AdvanceWorkItemInput::default()
        },
    )
    .expect("advance should apply accepted recovery");

    assert_eq!(
        result.decision.action,
        WorkflowDecisionAction::ApplyRecoveryPlan
    );
    assert!(result.advanced);
    assert!(result.recovery_plan_id.is_some());
    assert!(result.run_id.is_some());
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert!(
        snapshot
            .workflow_decisions
            .iter()
            .any(|decision| decision.action == WorkflowDecisionAction::ApplyRecoveryPlan)
    );
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    fs::remove_dir_all(root).ok();
}
