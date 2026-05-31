use std::env;
use std::fs;
use std::process::Command;
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

fn run_git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .expect("git should run");
    assert!(status.success(), "git command failed: {args:?}");
}

fn event_count(snapshot: &WorkItemSnapshot, event_type: &str) -> usize {
    snapshot
        .timeline
        .iter()
        .filter(|event| event.event_type == event_type)
        .count()
}

fn modify_tracked_file_and_emit_result_command() -> String {
    if cfg!(windows) {
        "echo changed> tracked.txt && echo ## Nagare Result && echo status: succeeded && echo summary: && echo - changed tracked file && echo next_action: approve".to_string()
    } else {
        "printf 'changed\n' > tracked.txt; printf '## Nagare Result\nstatus: succeeded\nsummary:\n- changed tracked file\nnext_action: approve\n'".to_string()
    }
}

#[test]
fn nagare_result_questions_set_work_item_needs_input() {
    let root = test_root("result-question-test");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: blocked\nquestions:\n- release note URLを追加してよいですか？\nnext_action: answer_question\n",
    )
    .expect("result should write");
    let item = create_work_item(&root, "Ask human", "").expect("item").item;
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
    .expect("run should parse result");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::NeedsInput);
    assert_eq!(snapshot.completion.state, "blocked");
    assert_eq!(snapshot.completion.next_action, "answer_question");
    assert_eq!(
        snapshot.completion.blocking_reason.as_deref(),
        Some("release note URLを追加してよいですか？")
    );
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "question")
    );
    assert_eq!(
        snapshot.agent_outputs[0].next_action.as_deref(),
        Some("answer_question")
    );
    assert_eq!(snapshot.agent_outputs[0].questions.len(), 1);

    let answered = answer_work_item(
        &root,
        &item.id,
        AnswerWorkItemInput {
            question: None,
            answer: "追加してよいです。",
        },
    )
    .expect("answer should record");
    assert_eq!(answered.item_status, WorkItemStatus::Ready);

    fs::write(
        root.join("done.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- updated after human feedback\nnext_action: approve\n",
    )
    .expect("second result should write");
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
    .expect("next run should include human feedback context");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.human_feedback.len(), 1);
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "human_feedback")
    );
    assert!(
        snapshot
            .resolved_run_packets
            .last()
            .expect("run packet should exist")
            .constraints
            .contains(&"human_feedback_context_applied".to_string())
    );
    fs::write(
        root.join("review_after_answer.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- answered output is acceptable\ncompleted:\n- reviewed answered output\nfindings:\n- none\nquestions:\nnext_notes:\n- ready for approval\nnext_action: approve\n",
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
            dev_command: Some(cat_command("review_after_answer.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should pass");
    approve_work_item(&root, &item.id, "contract output flow completed")
        .expect("approval should complete item");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "review")
    );
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "decision")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn nagare_result_none_question_does_not_require_input() {
    let root = test_root("result-no-question-test");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- 日本の5番目に高い山は槍ヶ岳です。\ncompleted:\n- 回答した\nquestions:\nなし\nnext_notes:\n- 追加作業なし\nnext_action: review\n",
    )
    .expect("result should write");
    let item = create_work_item(&root, "No human input needed", "")
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
    .expect("run should parse result");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert_eq!(snapshot.completion.next_action, "review");
    assert!(snapshot.agent_outputs[0].questions.is_empty());
    assert_eq!(event_count(&snapshot, "question"), 0);
}

#[test]
fn missing_required_nagare_result_records_unparsed_warning() {
    let root = test_root("missing-result-contract");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("plain.txt"),
        "The agent completed the work but forgot the Nagare Result block.\n",
    )
    .expect("plain output should write");
    let item = create_work_item(&root, "Missing contract", "")
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
            dev_command: Some(cat_command("plain.txt").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("run should preserve unparsed output");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert_eq!(snapshot.completion.next_action, "review");
    assert_eq!(snapshot.agent_outputs.len(), 1);
    let output = &snapshot.agent_outputs[0];
    assert_eq!(output.parse_status, AgentOutputParseStatus::Unparsed);
    assert!(
        output
            .warnings
            .contains(&"output_contract_unparsed".to_string())
    );
    assert!(!output.execution_record_id.is_empty());
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| { event.event_type == "agent_output" && event.status == "unparsed" })
    );
    let recovery = create_recovery_plan(&root, &item.id).expect("recovery should create");
    assert_eq!(
        recovery.plan.action,
        RecoveryAction::RerunWithContractReminder
    );
    assert_eq!(recovery.plan.reason, "output_contract_missing");
    assert_eq!(recovery.plan.failure_class, "contract_violation");
    let accepted = accept_recovery_plan(&root, &item.id, Some(&recovery.plan.id))
        .expect("recovery should accept");
    assert_eq!(accepted.plan.status, RecoveryPlanStatus::Accepted);
    assert!(accepted.plan.prompt_hint.is_some());

    fs::write(
        root.join("fixed.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- restated with the required contract\nnext_action: approve\n",
    )
    .expect("fixed output should write");
    let applied = apply_recovery_plan(
        &root,
        &item.id,
        ApplyRecoveryPlanInput {
            recovery_plan_id: Some(&accepted.plan.id),
            prompt: None,
            dev_command: Some(cat_command("fixed.md").as_str()),
        },
    )
    .expect("contract recovery should apply");
    assert_eq!(applied.run.item_status, WorkItemStatus::ReadyForReview);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(
        snapshot
            .agent_outputs
            .last()
            .expect("latest output")
            .parse_status,
        AgentOutputParseStatus::Parsed
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn unparsed_review_moves_to_recovery_instead_of_repeating_review() {
    let root = test_root("unparsed-review-recovery");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("work.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work finished\ncompleted:\n- produced an answer\nnext_notes:\n- review the answer\nnext_action: review\n",
    )
    .expect("work output should write");
    fs::write(
        root.join("bad-review.txt"),
        "Review looks good, but this forgot the Nagare Review block.\n",
    )
    .expect("review output should write");
    let item = create_work_item(&root, "Review contract failure", "")
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
            dev_command: Some(cat_command("work.md").as_str()),
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
            dev_command: Some(cat_command("bad-review.txt").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should preserve contract failure");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ChangesRequested);
    assert_eq!(snapshot.completion.next_action, "recover");
    let output = snapshot
        .agent_outputs
        .iter()
        .find(|output| output.purpose == AgentRunPurpose::Review)
        .expect("review output");
    assert_eq!(output.parse_status, AgentOutputParseStatus::Unparsed);
    assert!(
        output
            .warnings
            .contains(&"output_contract_unparsed".to_string())
    );
    let decision = create_workflow_decision(&root, &item.id)
        .expect("decision")
        .decision;
    assert_eq!(decision.action, WorkflowDecisionAction::CreateRecoveryPlan);

    let recovery = create_recovery_plan(&root, &item.id).expect("recovery");
    assert_eq!(recovery.plan.failure_class, "contract_violation");
    let accepted =
        accept_recovery_plan(&root, &item.id, Some(&recovery.plan.id)).expect("accept recovery");
    fs::write(
        root.join("fixed-review.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- recovered review contract\ncompleted:\n- restated review with the required contract\nfindings:\n- no blocker\nquestions:\nnext_notes:\n- ready for approval\nnext_action: approve\n",
    )
    .expect("fixed review output should write");
    let applied = apply_recovery_plan(
        &root,
        &item.id,
        ApplyRecoveryPlanInput {
            recovery_plan_id: Some(&accepted.plan.id),
            prompt: None,
            dev_command: Some(cat_command("fixed-review.md").as_str()),
        },
    )
    .expect("review contract recovery should apply");
    assert_eq!(applied.run.run.purpose, AgentRunPurpose::Review);
    assert_eq!(applied.run.item_status, WorkItemStatus::ReadyForReview);
}

#[test]
fn answer_question_without_question_does_not_require_input() {
    let root = test_root("answer-action-without-question-test");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("result.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- answered the request\ncompleted:\n- answered the request\nquestions:\n- none\nnext_notes:\n- no follow up\nnext_action: answer_question\n",
    )
    .expect("result should write");
    let item = create_work_item(&root, "Answer only", "")
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
    .expect("run should parse result");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert_eq!(snapshot.completion.next_action, "review");
    let output = snapshot.agent_outputs.last().expect("output");
    assert!(output.questions.is_empty());
    assert_eq!(output.next_action.as_deref(), Some("answer_question"));
    assert!(
        output
            .warnings
            .contains(&"next_action_without_question".to_string())
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn nagare_result_handoff_next_action_sets_needs_handoff() {
    let root = test_root("handoff-next-action");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("handoff.md"),
        "## Nagare Result\nstatus: blocked\nsummary:\n- Specialist review is required.\nnext_action: create_handoff\n",
    )
    .expect("handoff output should write");
    let item = create_work_item(&root, "Need handoff", "")
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
            dev_command: Some(cat_command("handoff.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("run should request handoff");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::NeedsHandoff);
    assert_eq!(snapshot.completion.next_action, "create_handoff");
    assert_eq!(
        snapshot.agent_outputs[0].next_action.as_deref(),
        Some("create_handoff")
    );
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| { event.event_type == "agent_output" && event.title == "create_handoff" })
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn nagare_review_questions_set_work_item_needs_input() {
    let root = test_root("review-question");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: blocked\nfindings:\n- Evidence is incomplete.\nquestions:\n- 追加の検証ログはありますか？\nnext_action: answer_question\n",
    )
    .expect("review output should write");
    let item = create_work_item(&root, "Review asks", "")
        .expect("item")
        .item;

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
    .expect("review should parse question");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::NeedsInput);
    assert_eq!(snapshot.completion.next_action, "answer_question");
    assert_eq!(snapshot.agent_outputs.len(), 1);
    assert_eq!(snapshot.review_results.len(), 1);
    assert_eq!(snapshot.review_results[0].verdict, ReviewVerdict::Blocked);
    assert_eq!(snapshot.agent_outputs[0].purpose, AgentRunPurpose::Review);
    assert_eq!(snapshot.agent_outputs[0].contract, "nagare.review.v1");
    assert_eq!(
        snapshot.agent_outputs[0].questions,
        vec!["追加の検証ログはありますか？".to_string()]
    );
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "question")
    );
    fs::remove_dir_all(root).ok();
}

#[test]
fn review_pass_and_request_changes_drive_work_item_status() {
    let root = test_root("review-status-transition");
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Review transition", "")
        .expect("item")
        .item;

    fs::write(
        root.join("changes.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- Implementation needs one more fix.\nfindings:\n- Missing test evidence.\nrequested_changes:\n- Add review evidence before approval.\nquestions:\nnext_action: run_agent\n",
    )
    .expect("request changes output should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("changes.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("request changes review should run");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ChangesRequested);
    assert_eq!(snapshot.completion.next_action, "run_agent");
    assert_eq!(
        snapshot.completion.blocking_reason.as_deref(),
        Some("Add review evidence before approval.")
    );
    assert_eq!(
        snapshot.review_results[0].verdict,
        ReviewVerdict::RequestChanges
    );

    fs::write(
        root.join("pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- Review passed.\nfindings:\n- No blocker.\nquestions:\nnext_action: approve\n",
    )
    .expect("pass output should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("pass.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("pass review should run");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ReadyForReview);
    assert_eq!(snapshot.completion.next_action, "approve");
    assert_eq!(snapshot.review_results.len(), 2);
    assert_eq!(snapshot.review_results[1].verdict, ReviewVerdict::Pass);
    assert_eq!(event_count(&snapshot, "review"), 2);
    fs::remove_dir_all(root).ok();
}

#[test]
fn dispatch_preview_selects_registered_agent_and_records_timeline() {
    let root = test_root("dispatch-preview-route");
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "researcher",
            working_dir: ".",
            description: "調査を担当するagent",
            specialties: vec!["research".to_string()],
            domain_group_ids: Vec::new(),
            domain_ids: Vec::new(),
        },
    )
    .expect("research profile should be added");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"research-agent","summary":"Research agent should handle source gathering.","risks":["source quality"],"missing_information":["source list"]}"#,
    )
    .expect("dispatch output should write");
    let item = create_work_item(&root, "Route research", "")
        .expect("item")
        .item;

    let preview = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: Some("docs/topic.md"),
            prompt: None,
            dev_command: Some(cat_command("dispatch.json").as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch preview should create plan");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    let plan = &snapshot.dispatch_plans[0];
    assert_eq!(snapshot.completion.next_action, "run_agent");
    assert_eq!(preview.dispatch_plan_id.as_deref(), Some(plan.id.as_str()));
    assert_eq!(plan.target_agent_profile_id, "research-agent");
    assert_eq!(plan.risks, vec!["source quality".to_string()]);
    assert_eq!(plan.missing_information, vec!["source list".to_string()]);
    assert!(snapshot.timeline.iter().any(|event| {
        event.event_type == "dispatch" && event.actor.as_deref() == Some("worker")
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn multi_agent_question_handoff_review_and_approval_workflow_completes() {
    let root = test_root("multi-agent-complex-flow");
    init_project(&root).expect("project should init");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "research-agent",
            display_name: "Research Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "researcher",
            working_dir: ".",
            description: "調査と不足情報の確認を担当するagent",
            specialties: vec!["research".to_string(), "source-check".to_string()],
            domain_group_ids: Vec::new(),
            domain_ids: Vec::new(),
        },
    )
    .expect("research profile should be added");
    add_agent_profile(
        &root,
        AddAgentProfileInput {
            id: "repair-agent",
            display_name: "Repair Agent",
            runtime: "codex-local",
            adapter: "process.codex-cli",
            role: "implementer",
            working_dir: ".",
            description: "調査結果を受けて実装と検証準備を担当するagent",
            specialties: vec!["implementation".to_string(), "review".to_string()],
            domain_group_ids: Vec::new(),
            domain_ids: Vec::new(),
        },
    )
    .expect("repair profile should be added");

    let item = create_work_item(
        &root,
        "Complex workflow",
        "Route to research, ask for human input, hand off to repair, review, and approve.",
    )
    .expect("item should create")
    .item;

    fs::write(
        root.join("dispatch_research.json"),
        r#"{"target_agent_profile_id":"research-agent","summary":"Start with research because the request lacks source context.","risks":["ambiguous source"],"missing_information":["source URL"]}"#,
    )
    .expect("dispatch output should write");
    let first_dispatch = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("dispatch_research.json").as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("first dispatch should create plan");
    let first_plan_id = first_dispatch.dispatch_plan_id.expect("plan should exist");
    accept_dispatch_plan(&root, &item.id, Some(&first_plan_id))
        .expect("first dispatch should be accepted");

    let first_selection = select_agent_for_work_item_run(
        &root,
        &item.id,
        SelectRunAgentInput {
            explicit_agent_profile_id: None,
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
        },
    )
    .expect("accepted dispatch should select research agent");
    assert_eq!(first_selection.agent_profile_id, "research-agent");

    fs::write(
        root.join("research_question.md"),
        "## Nagare Result\nstatus: blocked\nsummary:\n- Source context is missing.\nquestions:\n- 参照すべき一次情報URLはどれですか？\nnext_action: answer_question\n",
    )
    .expect("research question should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: first_selection.agent_profile_id.as_str(),
            dispatch_plan_id: first_selection.dispatch_plan_id.as_deref(),
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("research_question.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("research run should ask a question");
    assert_eq!(
        get_work_item_snapshot(&root, &item.id)
            .expect("snapshot")
            .item
            .status,
        WorkItemStatus::NeedsInput
    );

    answer_work_item(
        &root,
        &item.id,
        AnswerWorkItemInput {
            question: None,
            answer: "docs/source-a.md を一次情報として扱ってください。",
        },
    )
    .expect("human answer should record");

    fs::write(
        root.join("research_handoff.md"),
        "## Nagare Result\nstatus: blocked\nsummary:\n- Source scope is clear, but implementation should move to repair-agent.\nevidence:\n- docs/source-a.md was selected by the user.\nnext_action: create_handoff\n",
    )
    .expect("research handoff should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "research-agent",
            dispatch_plan_id: first_selection.dispatch_plan_id.as_deref(),
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("research_handoff.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("research run should request handoff");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::NeedsHandoff);
    assert!(
        snapshot
            .resolved_run_packets
            .last()
            .expect("run packet should exist")
            .constraints
            .contains(&"human_feedback_context_applied".to_string())
    );

    let handoff = create_handoff(
        &root,
        &item.id,
        "research-agent",
        "repair-agent",
        "Research found the source, implementation should continue elsewhere.",
        "Use docs/source-a.md and produce a verifiable implementation summary.",
    )
    .expect("handoff should be created");
    assert_eq!(handoff.handoff.current_state, "needs_handoff");
    assert!(handoff.handoff.artifact_ids.is_empty());
    assert!(handoff.handoff.next_request.contains("docs/source-a.md"));

    fs::write(
        root.join("dispatch_repair.json"),
        r#"{"target_agent_profile_id":"repair-agent","summary":"Continue with repair-agent after research handoff.","risks":["implementation may need review"],"missing_information":[]}"#,
    )
    .expect("handoff dispatch output should write");
    let handoff_dispatch = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("dispatch_repair.json").as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("handoff dispatch should create repair plan");
    let repair_plan_id = handoff_dispatch
        .dispatch_plan_id
        .expect("repair plan should exist");
    accept_dispatch_plan(&root, &item.id, Some(&repair_plan_id))
        .expect("repair dispatch should be accepted");

    let repair_selection = select_agent_for_work_item_run(
        &root,
        &item.id,
        SelectRunAgentInput {
            explicit_agent_profile_id: None,
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
        },
    )
    .expect("accepted repair dispatch should select repair agent");
    assert_eq!(repair_selection.agent_profile_id, "repair-agent");
    assert_eq!(
        repair_selection.dispatch_plan_id.as_deref(),
        Some(repair_plan_id.as_str())
    );

    fs::write(
        root.join("repair_done.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- Implemented the requested workflow using the selected source.\nartifacts:\n- docs/feature.md\nevidence:\n- Source docs/source-a.md was applied.\nnext_action: review\n",
    )
    .expect("repair output should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: repair_selection.agent_profile_id.as_str(),
            dispatch_plan_id: repair_selection.dispatch_plan_id.as_deref(),
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("repair_done.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("repair run should succeed");
    assert_eq!(
        get_work_item_snapshot(&root, &item.id)
            .expect("snapshot")
            .item
            .status,
        WorkItemStatus::ReadyForReview
    );

    fs::write(
        root.join("review_question.md"),
        "## Nagare Review\nverdict: blocked\nsummary:\n- The work is plausible but review evidence is not explicit.\nfindings:\n- Missing test log reference.\nquestions:\n- Review内でどの確認を実行しましたか？\nnext_action: answer_question\n",
    )
    .expect("review question should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("review_question.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review should ask a question");
    assert_eq!(
        get_work_item_snapshot(&root, &item.id)
            .expect("snapshot")
            .item
            .status,
        WorkItemStatus::NeedsInput
    );

    answer_work_item(
        &root,
        &item.id,
        AnswerWorkItemInput {
            question: None,
            answer: "Review内でcargo test --workspace相当の確認を実行済みです。",
        },
    )
    .expect("review answer should record");

    fs::write(
        root.join("review_pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- Review evidence is sufficient.\nfindings:\n- No blocking issue.\nquestions:\nnext_action: approve\n",
    )
    .expect("review pass should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "reviewer",
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("review_pass.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review pass should parse");

    approve_work_item(&root, &item.id, "complex workflow completed")
        .expect("approval should complete item");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert_eq!(snapshot.review_results.len(), 2);
    assert_eq!(snapshot.completion.state, "done");
    assert_eq!(snapshot.completion.next_action, "done");
    assert_eq!(snapshot.dispatch_plans.len(), 2);
    assert_eq!(
        snapshot.dispatch_plans[0].status,
        DispatchPlanStatus::Superseded
    );
    assert_eq!(
        snapshot.dispatch_plans[1].status,
        DispatchPlanStatus::Accepted
    );
    assert_eq!(snapshot.runs.len(), 7);
    assert_eq!(snapshot.agent_outputs.len(), 5);
    assert_eq!(snapshot.human_feedback.len(), 2);
    assert_eq!(snapshot.handoffs.len(), 1);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(event_count(&snapshot, "dispatch"), 2);
    assert_eq!(event_count(&snapshot, "question"), 2);
    assert_eq!(event_count(&snapshot, "human_feedback"), 2);
    assert_eq!(event_count(&snapshot, "handoff"), 1);
    assert!(snapshot.resolved_run_packets.iter().any(|packet| {
        packet.dispatch_plan_id.as_deref() == Some(repair_plan_id.as_str())
            && packet.agent_profile_id == "repair-agent"
            && packet
                .constraints
                .contains(&"human_feedback_context_applied".to_string())
            && packet
                .constraints
                .contains(&"handoff_context_applied".to_string())
    }));
    assert!(snapshot.timeline.iter().any(|event| {
        event.event_type == "decision"
            && event.status == "approve"
            && event.title == "complex workflow completed"
    }));
    fs::remove_dir_all(root).ok();
}

#[test]
fn work_run_collects_changed_files_and_diff_execution_records() {
    let root = test_root("git-execution-record-collection");
    fs::create_dir_all(&root).expect("root should create");
    Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("init")
        .status()
        .expect("git init should run");
    fs::write(root.join("tracked.txt"), "initial\n").expect("tracked file should write");
    run_git(&root, &["add", "tracked.txt"]);
    run_git(
        &root,
        &[
            "-c",
            "user.email=nagare@example.invalid",
            "-c",
            "user.name=Nagare Test",
            "commit",
            "-m",
            "initial",
        ],
    );
    init_project(&root).expect("project should init");
    let item = create_work_item(&root, "Collect execution records", "")
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
            dev_command: Some(modify_tracked_file_and_emit_result_command().as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work run should collect git artifacts");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert!(snapshot.execution_records.iter().any(|record| {
        record.record_type == "changed_files" && record.title.contains("changed files")
    }));
    let diff_record = snapshot
        .execution_records
        .iter()
        .find(|record| record.record_type == "diff_patch")
        .expect("diff execution record should exist");
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "execution_record" && event.id == diff_record.id)
    );
    assert!(snapshot.artifacts.is_empty());
    let diff_path = diff_record
        .uri
        .strip_prefix("file://")
        .expect("execution record uri should be file");
    let diff = fs::read_to_string(diff_path).expect("diff execution record should read");
    assert!(diff.contains("tracked.txt"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn completion_state_points_to_work_after_review_changes() {
    let root = test_root("completion-review-changes");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("done.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- work finished\nnext_action: review\n",
    )
    .expect("done output should write");
    let item = create_work_item(&root, "Needs recovery", "")
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
            dev_command: Some(cat_command("done.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("work should run");
    fs::write(
        root.join("review_changes.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- review requested changes\nrequested_changes:\n- address review feedback\nquestions:\nnext_action: run_agent\n",
    )
    .expect("review changes should write");
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
    .expect("review changes should record");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::ChangesRequested);
    assert_eq!(snapshot.completion.state, "blocked");
    assert_eq!(snapshot.completion.next_action, "run_agent");
    assert!(
        snapshot
            .completion
            .blocking_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("address review feedback"))
    );
    let first = create_recovery_plan(&root, &item.id).expect("first recovery should create");
    assert_eq!(first.plan.action, RecoveryAction::RerunSameAgent);
    assert_eq!(first.plan.reason, "changes_requested");
    assert_eq!(first.plan.failure_class, "review_changes");
    let second = create_recovery_plan(&root, &item.id).expect("second recovery should create");
    assert_eq!(second.plan.status, RecoveryPlanStatus::Draft);
    let accepted =
        accept_recovery_plan(&root, &item.id, None).expect("latest draft recovery should accept");
    assert_eq!(accepted.plan.id, second.plan.id);
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.recovery_plans.len(), 2);
    assert_eq!(
        snapshot.recovery_plans[0].status,
        RecoveryPlanStatus::Superseded
    );
    assert_eq!(
        snapshot.recovery_plans[1].status,
        RecoveryPlanStatus::Accepted
    );
    assert_eq!(event_count(&snapshot, "recovery"), 2);
    fs::remove_dir_all(root).ok();
}
