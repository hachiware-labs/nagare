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

fn pass_command() -> &'static str {
    if cfg!(windows) {
        "cmd /C exit 0"
    } else {
        "true"
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
            agent_profile_id: "codex-cli",
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
        "## Nagare Result\nstatus: succeeded\nsummary:\n- updated after human feedback\nnext_action: verify\n",
    )
    .expect("second result should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
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
    verify_work_item(
        &root,
        &item.id,
        if cfg!(windows) {
            "cmd /C exit 0"
        } else {
            "true"
        },
    )
    .expect("verification should pass");
    approve_work_item(&root, &item.id, "contract output flow completed")
        .expect("approval should complete item");
    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| event.event_type == "verification")
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
            agent_profile_id: "codex-cli",
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
    assert_eq!(snapshot.agent_outputs.len(), 1);
    let output = &snapshot.agent_outputs[0];
    assert_eq!(output.parse_status, AgentOutputParseStatus::Unparsed);
    assert!(
        output
            .warnings
            .contains(&"output_contract_unparsed".to_string())
    );
    assert!(output.artifact_id.is_some());
    assert!(
        snapshot
            .timeline
            .iter()
            .any(|event| { event.event_type == "agent_output" && event.status == "unparsed" })
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
            agent_profile_id: "codex-cli",
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
            agent_profile_id: "codex-app-server",
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
    assert_eq!(snapshot.agent_outputs.len(), 1);
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
            agent_profile_id: "codex-cli",
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
    assert_eq!(preview.dispatch_plan_id.as_deref(), Some(plan.id.as_str()));
    assert_eq!(plan.target_agent_profile_id, "research-agent");
    assert_eq!(plan.risks, vec!["source quality".to_string()]);
    assert_eq!(plan.missing_information, vec!["source list".to_string()]);
    assert!(snapshot.timeline.iter().any(|event| {
        event.event_type == "dispatch" && event.actor.as_deref() == Some("codex-cli")
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
            specialties: vec!["implementation".to_string(), "verification".to_string()],
        },
    )
    .expect("repair profile should be added");

    let item = create_work_item(
        &root,
        "Complex workflow",
        "Route to research, ask for human input, hand off to repair, review, verify, and approve.",
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
            agent_profile_id: "codex-cli",
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

    create_handoff(
        &root,
        &item.id,
        "research-agent",
        "repair-agent",
        "Research found the source, implementation should continue elsewhere.",
        "Use docs/source-a.md and produce a verifiable implementation summary.",
    )
    .expect("handoff should be created");

    fs::write(
        root.join("dispatch_repair.json"),
        r#"{"target_agent_profile_id":"repair-agent","summary":"Continue with repair-agent after research handoff.","risks":["implementation may need verification"],"missing_information":[]}"#,
    )
    .expect("handoff dispatch output should write");
    let handoff_dispatch = run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-cli",
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
        "## Nagare Result\nstatus: succeeded\nsummary:\n- Implemented the requested workflow using the selected source.\nartifacts:\n- docs/feature.md\nevidence:\n- Source docs/source-a.md was applied.\nverification:\n- run verification command\nnext_action: review\n",
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
        "## Nagare Review\nverdict: blocked\nsummary:\n- The work is plausible but verification evidence is not explicit.\nfindings:\n- Missing verification log reference.\nquestions:\n- 検証コマンドは何を実行しましたか？\nnext_action: answer_question\n",
    )
    .expect("review question should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-app-server",
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
            answer: "cargo test --workspace 相当の検証を実行済みです。",
        },
    )
    .expect("review answer should record");

    fs::write(
        root.join("review_pass.md"),
        "## Nagare Review\nverdict: pass\nsummary:\n- Verification evidence is sufficient.\nfindings:\n- No blocking issue.\nquestions:\nnext_action: verify\n",
    )
    .expect("review pass should write");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "codex-app-server",
            dispatch_plan_id: None,
            path: Some("docs/feature.md"),
            prompt: None,
            dev_command: Some(cat_command("review_pass.md").as_str()),
            purpose: AgentRunPurpose::Review,
        },
    )
    .expect("review pass should parse");

    verify_work_item(&root, &item.id, pass_command()).expect("verification should pass");
    approve_work_item(&root, &item.id, "complex workflow completed")
        .expect("approval should complete item");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot");
    assert_eq!(snapshot.item.status, WorkItemStatus::Done);
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
    assert_eq!(snapshot.verification_results.len(), 1);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(event_count(&snapshot, "dispatch"), 2);
    assert_eq!(event_count(&snapshot, "question"), 2);
    assert_eq!(event_count(&snapshot, "human_feedback"), 2);
    assert_eq!(event_count(&snapshot, "handoff"), 1);
    assert_eq!(event_count(&snapshot, "verification"), 1);
    assert!(snapshot.resolved_run_packets.iter().any(|packet| {
        packet.dispatch_plan_id.as_deref() == Some(repair_plan_id.as_str())
            && packet.agent_profile_id == "repair-agent"
            && packet
                .constraints
                .contains(&"human_feedback_context_applied".to_string())
    }));
    assert!(snapshot.timeline.iter().any(|event| {
        event.event_type == "decision"
            && event.status == "approve"
            && event.title == "complex workflow completed"
    }));
    fs::remove_dir_all(root).ok();
}
