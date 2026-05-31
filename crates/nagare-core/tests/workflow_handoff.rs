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
fn finish_first_resumes_from_created_handoff_to_approval_gate() {
    let root = test_root("finish-first-handoff");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("needs_handoff.md"),
        "## Nagare Result\nstatus: blocked\nsummary:\n- specialist handoff required\nnext_action: create_handoff\n",
    )
    .expect("handoff request should write");
    fs::write(
        root.join("handoff_dispatch.json"),
        r#"{"target_agent_profile_id":"reviewer","summary":"Continue after handoff with app server agent.","risks":[],"missing_information":[]}"#,
    )
    .expect("handoff dispatch should write");
    fs::write(
        root.join("handoff_result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- handoff work completed\nnext_action: review\n",
    )
    .expect("handoff result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- handoff result is ready\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Resume handoff workflow".to_string(),
            workflow_mode: Some(WorkflowMode::FinishFirst),
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
            dev_command: Some(cat_command("needs_handoff.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work should request handoff");
    let handoff_gate = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput::default(),
            max_steps: 3,
        },
    )
    .expect("missing handoff should stop");
    assert_eq!(handoff_gate.final_status, WorkItemStatus::NeedsHandoff);
    assert_eq!(
        handoff_gate.steps.last().expect("last").decision.action,
        WorkflowDecisionAction::CreateHandoff
    );
    assert!(!handoff_gate.steps.last().expect("last").advanced);

    create_handoff(
        &root,
        &item.id,
        "worker",
        "reviewer",
        "Specialist context is required.",
        "Continue the handoff work and return a verifiable result.",
    )
    .expect("handoff should create");
    let approval_gate = advance_work_item_until_blocked(
        &root,
        &item.id,
        AdvanceUntilBlockedInput {
            step: AdvanceWorkItemInput {
                dispatch_dev_command: Some(cat_command("handoff_dispatch.json").as_str()),
                dev_command: Some(cat_command("handoff_result.md").as_str()),
                review_dev_command: Some(cat_command("review.md").as_str()),
                ..AdvanceWorkItemInput::default()
            },
            max_steps: 8,
        },
    )
    .expect("created handoff should resume to approval");

    assert_eq!(approval_gate.final_status, WorkItemStatus::ReadyForReview);
    assert_eq!(
        approval_gate.steps.last().expect("last").decision.action,
        WorkflowDecisionAction::Approve
    );
    assert!(!approval_gate.steps.last().expect("last").advanced);
    for action in [
        WorkflowDecisionAction::Dispatch,
        WorkflowDecisionAction::AcceptDispatch,
        WorkflowDecisionAction::RunAgent,
        WorkflowDecisionAction::RunReview,
    ] {
        assert!(
            approval_gate
                .steps
                .iter()
                .any(|step| step.decision.action == action)
        );
    }

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.workflow_mode, WorkflowMode::FinishFirst);
    assert_eq!(snapshot.handoffs.len(), 1);
    assert_eq!(snapshot.dispatch_plans.len(), 1);
    assert_eq!(
        snapshot.dispatch_plans[0].status,
        DispatchPlanStatus::Accepted
    );
    assert_eq!(
        snapshot.dispatch_plans[0].target_agent_profile_id,
        "reviewer"
    );
    assert!(snapshot.decisions.is_empty());
    fs::remove_dir_all(root).ok();
}
