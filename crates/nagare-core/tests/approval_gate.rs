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

#[test]
fn approval_gate_ready_after_current_review() {
    let root = test_root("approval-gate-ready");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- criteria implemented\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- criteria satisfied\ncriteria:\n- evidence recorded: pass\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Approval gate ready".to_string(),
            acceptance_criteria: vec!["evidence recorded".to_string()],
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
            dev_command: Some(cat_command("result.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work should run");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should run");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert!(snapshot.approval_gate.ready);
    assert_eq!(snapshot.approval_gate.state, "ready");
    assert_eq!(snapshot.approval_gate.criteria_passed, 1);
    assert_eq!(snapshot.approval_gate.criteria_total, 1);
    assert!(snapshot.approval_gate.latest_review_id.is_some());
    assert!(snapshot.approval_gate.blockers.is_empty());
    let approve_hint = format!("nagare decision approve {}", item.id);
    assert_eq!(
        snapshot.approval_gate.command_hint.as_deref(),
        Some(approve_hint.as_str())
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn approval_reject_records_reason_and_returns_to_dispatch() {
    let root = test_root("approval-reject-dispatch");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use worker.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch should write");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- implementation completed\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- result passes\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item(&root, "Reject approved-looking result", "")
        .expect("item should create")
        .item;

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "dispatcher",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("dispatch.json").as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch should run");
    accept_dispatch_plan(&root, &item.id, None).expect("dispatch should accept");
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
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should run");

    let result = reject_work_item(&root, &item.id, "方向性が違うため再選定する")
        .expect("reject should record decision");
    assert_eq!(result.item_status, WorkItemStatus::Ready);

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Ready);
    assert_eq!(snapshot.completion.next_action, "dispatch");
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_type, "reject");
    assert_eq!(
        snapshot.decisions[0].rationale,
        "方向性が違うため再選定する"
    );
    assert!(
        snapshot
            .dispatch_plans
            .iter()
            .all(|plan| plan.status == DispatchPlanStatus::Superseded)
    );

    let decision = create_workflow_decision(&root, &item.id)
        .expect("workflow decision should dispatch")
        .decision;
    assert_eq!(decision.action, WorkflowDecisionAction::Dispatch);
    fs::remove_dir_all(root).ok();
}

#[test]
fn approval_gate_blocks_stale_review_after_new_work() {
    let root = test_root("approval-gate-stale-review");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("first.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- first implementation\nnext_action: review\n",
    )
    .expect("first result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- first implementation reviewed\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    fs::write(
        root.join("second.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- second implementation changed the work\nnext_action: review\n",
    )
    .expect("second result should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Stale review gate".to_string(),
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
            dev_command: Some(cat_command("first.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("first work should run");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("review.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("first review should run");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("second.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("second work should run");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert!(!snapshot.approval_gate.ready);
    assert_eq!(snapshot.approval_gate.state, "blocked");
    assert!(
        snapshot
            .approval_gate
            .blockers
            .contains(&"review_not_passed".to_string())
    );
    assert!(approve_work_item(&root, &item.id, "stale approval").is_err());
    fs::remove_dir_all(root).ok();
}
