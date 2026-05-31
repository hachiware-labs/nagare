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
fn work_and_review_output_notes_are_parsed() {
    let root = test_root("agent-output-notes");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- implemented note capture\ncompleted:\n- added completed notes to the result\nnext_notes:\n- reviewer should check note visibility\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- notes are visible\ncompleted:\n- reviewed completed and next notes\nfindings:\n- no blocker\nquestions:\nnext_notes:\n- reviewer found the result ready for approval\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item(&root, "Agent output notes", "")
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
    .expect("work should parse notes");
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
    .expect("review should parse notes");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    let work_output = snapshot
        .agent_outputs
        .iter()
        .find(|output| output.purpose == AgentRunPurpose::Work)
        .expect("work output");
    assert_eq!(
        work_output.fields.get("completed").expect("completed"),
        &vec!["added completed notes to the result".to_string()]
    );
    assert_eq!(
        work_output.fields.get("next_notes").expect("next_notes"),
        &vec!["reviewer should check note visibility".to_string()]
    );
    let review_output = snapshot
        .agent_outputs
        .iter()
        .find(|output| output.purpose == AgentRunPurpose::Review)
        .expect("review output");
    assert_eq!(
        review_output.fields.get("completed").expect("completed"),
        &vec!["reviewed completed and next notes".to_string()]
    );
    assert_eq!(
        review_output.fields.get("next_notes").expect("next_notes"),
        &vec!["reviewer found the result ready for approval".to_string()]
    );
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert_eq!(snapshot.completion.next_action, "approve");
    fs::remove_dir_all(root).ok();
}

#[test]
fn missing_output_notes_create_warning_and_recovery_plan() {
    let root = test_root("missing-agent-output-notes");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work completed without notes\nnext_action: review\n",
    )
    .expect("result should write");
    let item = create_work_item(&root, "Missing agent output notes", "")
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

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    let output = snapshot.agent_outputs.last().expect("output");
    assert_eq!(output.parse_status, AgentOutputParseStatus::Parsed);
    assert!(output.warnings.contains(&"missing_completed".to_string()));
    assert!(output.warnings.contains(&"missing_next_notes".to_string()));

    let recovery = create_recovery_plan(&root, &item.id).expect("recovery should create");
    assert_eq!(recovery.plan.failure_class, "output_notes_missing");
    assert_eq!(recovery.plan.reason, "output_notes_missing");
    assert_eq!(
        recovery.plan.action,
        RecoveryAction::RerunWithContractReminder
    );
    assert!(
        recovery
            .plan
            .prompt_hint
            .as_deref()
            .expect("prompt hint")
            .contains("completed, next_notes")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn missing_review_notes_keep_review_transition_but_warn() {
    let root = test_root("missing-review-output-notes");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work completed\ncompleted:\n- implemented work\nnext_notes:\n- review the work\nnext_action: review\n",
    )
    .expect("result should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- review passed\nfindings:\n- no blocker\nquestions:\nnext_action: approve\n",
    )
    .expect("review should write");
    let item = create_work_item(&root, "Missing review output notes", "")
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
    assert_eq!(snapshot.completion.next_action, "approve");
    let output = snapshot
        .agent_outputs
        .iter()
        .rev()
        .find(|output| output.purpose == AgentRunPurpose::Review)
        .expect("review output");
    assert!(output.warnings.contains(&"missing_completed".to_string()));
    assert!(output.warnings.contains(&"missing_next_notes".to_string()));
    fs::remove_dir_all(root).ok();
}

#[test]
fn newer_complete_notes_clear_missing_notes_recovery() {
    let root = test_root("fixed-agent-output-notes");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("missing.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- first output missed notes\nnext_action: review\n",
    )
    .expect("missing result should write");
    fs::write(
        root.join("fixed.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- fixed output includes notes\ncompleted:\n- restated the output with notes\nnext_notes:\n- proceed to review\nnext_action: review\n",
    )
    .expect("fixed result should write");
    let item = create_work_item(&root, "Fixed agent output notes", "")
        .expect("item")
        .item;

    for file in ["missing.md", "fixed.md"] {
        run_work_item_with_input(
            &root,
            &item.id,
            RunWorkItemInput {
                agent_profile_id: "worker",
                dispatch_plan_id: None,
                path: None,
                prompt: None,
                dev_command: Some(cat_command(file).as_str()),
                purpose: AgentRunPurpose::Work,
            },
        )
        .expect("work should run");
    }

    let recovery = create_recovery_plan(&root, &item.id).expect("recovery should create");
    assert_ne!(recovery.plan.failure_class, "output_notes_missing");
    fs::remove_dir_all(root).ok();
}

#[test]
fn nested_contract_keys_inside_summary_create_warning() {
    let root = test_root("nested-contract-fields");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("nested.md"),
        "## Nagare Result\nsummary: status: completed\nsummary: answered request\ncompleted:\n1. implemented answer\nnext_notes: follow up\nnext_action: review\n",
    )
    .expect("nested output should write");
    let item = create_work_item(&root, "Nested contract fields", "")
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
            dev_command: Some(cat_command("nested.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("run should preserve warning");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    let output = snapshot.agent_outputs.last().expect("agent output");
    assert_eq!(output.parse_status, AgentOutputParseStatus::Parsed);
    assert!(
        output
            .warnings
            .contains(&"nested_contract_fields".to_string())
    );
}
