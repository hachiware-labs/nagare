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
fn trace_jsonl_records_nf2_decision_flow() {
    let root = test_root("trace-contract");
    init_project(&root).expect("project should init");
    add_artifact_type(
        &root,
        AddArtifactTypeInput {
            id: "trace-report",
            domain_id: Some("general"),
            display_name: "Trace Report",
            description: "A report backed by execution trace evidence.",
            artifact_types: vec!["markdown".to_string()],
            rubric: vec![
                "## Correctness (60)".to_string(),
                "- The report matches the recorded execution.".to_string(),
                "## Clarity (40)".to_string(),
                "- The report explains the decision trail.".to_string(),
            ],
            dispatch_hints: Vec::new(),
            workflow: DomainWorkflowOverride::default(),
        },
    )
    .expect("artifact type should be added");
    fs::write(
        root.join("dispatch.json"),
        r#"{"target_agent_profile_id":"worker","summary":"Use the worker because it matches the general implementation request.","risks":[],"missing_information":[]}"#,
    )
    .expect("dispatch fixture should write");
    fs::write(
        root.join("work.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- trace-backed work completed\ncompleted:\n- produced the requested result\nnext_notes:\n- ready for review\nnext_action: review\n",
    )
    .expect("work fixture should write");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: pass\noverall_score: 85\nsummary:\n- trace criterion satisfied\ncompleted:\n- reviewed the trace report\ncriteria:\n- trace criterion: passed - trace evidence is present\nrubric_scores:\n- Correctness | points=50 | max_points=60 | verdict=partial | evidence=One edge case is not explained.\n- Clarity | points=35 | max_points=40 | verdict=pass | evidence=The decision trail is readable.\nfindings:\n- no blockers\nquestions:\nnext_notes:\n- ready for approval\nnext_action: approve\n",
    )
    .expect("review fixture should write");
    fs::write(
        root.join("synthesis.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- trace-backed final answer prepared\ncompleted:\n- summarized worker output and review\nnext_action: approve\n",
    )
    .expect("synthesis fixture should write");

    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Trace-backed flow".to_string(),
            description: "Record the human-readable decision trail.".to_string(),
            acceptance_criteria: vec!["trace criterion".to_string()],
            expected_artifacts: vec!["work.md".to_string()],
            artifact_type_id: Some("trace-report".to_string()),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("dispatch.json").as_str()),
            purpose: AgentRunPurpose::DispatchPreview,
        },
    )
    .expect("dispatch should run");
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
    .expect("worker should run");
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
    .expect("reviewer should run");
    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("synthesis.md").as_str()),
            purpose: AgentRunPurpose::Synthesis,
        },
    )
    .expect("synthesis should run");
    approve_work_item(&root, &item.id, "trace review passed").expect("approval should pass");

    let trace = list_work_trace(&root, &item.id).expect("trace should read");
    let records = trace
        .iter()
        .map(|record| record.record.as_str())
        .collect::<Vec<_>>();

    assert_eq!(records[0], "work_header");
    assert!(records.contains(&"organizer_decision"));
    assert!(records.contains(&"worker_output"));
    assert!(records.contains(&"reviewer_verdict"));
    assert!(records.contains(&"organizer_summary"));
    assert!(records.contains(&"human_decision"));
    assert_eq!(
        trace.iter().map(|record| record.seq).collect::<Vec<_>>(),
        (1..=trace.len() as u64).collect::<Vec<_>>()
    );

    let organizer = trace
        .iter()
        .find(|record| record.record == "organizer_decision")
        .expect("organizer decision should be traced");
    assert_eq!(organizer.payload["agent"]["id"], "organizer");
    assert_eq!(organizer.payload["agent"]["role"], "organizer");
    assert_eq!(
        organizer.payload["interpreted_request"],
        "Use the worker because it matches the general implementation request."
    );
    assert_eq!(organizer.payload["assignments"][0]["agent_id"], "worker");
    assert_eq!(
        organizer.payload["assignments"][0]["rationale"],
        "Use the worker because it matches the general implementation request."
    );
    assert_eq!(organizer.payload["plan"][0]["step_kind"], "create");
    assert!(
        organizer.payload["diagnostics"]["session_ref"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "organizer diagnostics should keep a session reference behind the visible UI"
    );

    let worker = trace
        .iter()
        .find(|record| record.record == "worker_output")
        .expect("worker output should be traced");
    assert_eq!(worker.payload["agent"]["id"], "worker");
    assert_eq!(worker.payload["agent"]["role"], "worker");
    let worker_input = worker.payload["inputs"]["summary"]
        .as_str()
        .expect("worker input summary should be a string");
    assert!(worker_input.contains("Record the human-readable decision trail."));
    assert!(worker_input.contains("trace criterion"));
    assert!(worker_input.contains("work.md"));
    assert_eq!(
        worker.payload["actions_summary"],
        "trace-backed work completed"
    );
    assert_eq!(worker.payload["answer"], "trace-backed work completed");
    assert!(
        worker.payload["artifacts"][0]["path"]
            .as_str()
            .is_some_and(|value| value.ends_with("work.md")),
        "worker trace should include concrete artifact file paths for the result section"
    );
    assert_eq!(worker.payload["artifacts"][0]["change"], "modified");
    assert!(worker.payload["effective_capabilities"]["skills"].is_array());
    assert!(worker.payload["effective_capabilities"]["mcp_connections"].is_array());
    assert!(worker.payload["context_refs"].is_array());

    let reviewer = trace
        .iter()
        .find(|record| record.record == "reviewer_verdict")
        .expect("reviewer verdict should be traced");
    assert_eq!(reviewer.payload["agent"]["id"], "reviewer");
    assert_eq!(reviewer.payload["agent"]["role"], "reviewer");
    assert_eq!(
        reviewer.payload["item_verdicts"][0]["item"],
        "trace criterion"
    );
    assert_eq!(reviewer.payload["item_verdicts"][0]["verdict"], "pass");
    assert_eq!(reviewer.payload["item_verdicts"][0]["points"], 1);
    assert_eq!(reviewer.payload["item_verdicts"][0]["max_points"], 1);
    assert_eq!(reviewer.payload["rubric_ref"]["id"], "trace-report");
    assert_eq!(reviewer.payload["rubric_ref"]["version"], 1);
    assert_eq!(
        reviewer.payload["rubric_item_verdicts"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        reviewer.payload["rubric_item_verdicts"][0]["item"],
        "Correctness"
    );
    assert_eq!(reviewer.payload["rubric_item_verdicts"][0]["points"], 50);
    assert_eq!(reviewer.payload["rubric_total_score"], 85);
    assert_eq!(reviewer.payload["rubric_max_score"], 100);
    assert_eq!(reviewer.payload["rubric_items_expected"], 2);
    assert_eq!(reviewer.payload["rubric_items_recorded"], 2);
    assert_eq!(reviewer.payload["rubric_complete"], true);
    assert_eq!(reviewer.payload["diagnostics"]["prompt_version"], "v1");
    assert_eq!(reviewer.payload["knowledge_refs"][0]["id"], "general");
    assert_eq!(reviewer.payload["knowledge_refs"][0]["version"], 1);
    assert_eq!(
        reviewer.payload["knowledge_refs"][0]["kind"],
        "domain_knowledge"
    );
    assert_eq!(reviewer.payload["knowledge_refs"][1]["id"], "trace-report");
    assert_eq!(reviewer.payload["knowledge_refs"][1]["version"], 1);
    assert_eq!(
        reviewer.payload["knowledge_refs"][1]["kind"],
        "artifact_definition"
    );
    assert!(
        reviewer.payload["target_artifacts"][0]
            .as_str()
            .is_some_and(|value| value.ends_with("work.md")),
        "review trace should reference the concrete artifact being evaluated"
    );
    assert_eq!(reviewer.payload["total_score"], 1);
    assert_eq!(reviewer.payload["max_score"], 1);
    assert_eq!(reviewer.payload["recommendation"], "approve");
    assert_eq!(reviewer.payload["summary"], "trace criterion satisfied");

    let snapshot = get_work_item_snapshot(&root, &item.id).expect("snapshot should load");
    assert_eq!(snapshot.review_results[0].rubric_expected_count, 2);
    assert_eq!(
        snapshot.review_results[0].rubric_results[0].points,
        Some(50)
    );
    let review_packet = snapshot
        .resolved_run_packets
        .iter()
        .find(|packet| packet.purpose == AgentRunPurpose::Review)
        .expect("review run packet should exist");
    assert_eq!(review_packet.prompt_version, "v1");
    assert_eq!(review_packet.rubric_id.as_deref(), Some("trace-report"));
    assert_eq!(review_packet.rubric_version, Some(1));
    assert_eq!(
        review_packet.domain_knowledge_id.as_deref(),
        Some("general")
    );
    assert_eq!(review_packet.domain_knowledge_version, Some(1));
    assert_eq!(
        review_packet.artifact_definition_id.as_deref(),
        Some("trace-report")
    );
    assert_eq!(review_packet.artifact_definition_version, Some(1));

    let trace_path = root
        .join(".nagare")
        .join("works")
        .join(&item.id)
        .join("trace.jsonl");
    assert!(trace_path.exists());
}

#[test]
fn synthesis_trace_is_recorded_as_organizer_summary() {
    let root = test_root("trace-synthesis");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("synthesis.md"),
        "## Nagare Result\nstatus: succeeded\nsummary:\n- integrated final answer for the requester\ncompleted:\n- summarized implementation and research worker outputs\nnext_action: approve\n",
    )
    .expect("synthesis fixture should write");

    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Synthesize final answer".to_string(),
            description: "Summarize multiple workers.".to_string(),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;

    run_work_item_with_input(
        &root,
        &item.id,
        RunWorkItemInput {
            agent_profile_id: "organizer",
            dispatch_plan_id: None,
            path: None,
            prompt: Some("Summarize the reviewed worker results."),
            dev_command: Some(cat_command("synthesis.md").as_str()),
            purpose: AgentRunPurpose::Synthesis,
        },
    )
    .expect("synthesis should run");

    let trace = list_work_trace(&root, &item.id).expect("trace should read");
    let summary = trace
        .iter()
        .find(|record| record.record == "organizer_summary")
        .expect("organizer summary should be traced");
    assert_eq!(summary.payload["step_kind"], "synthesis");
    assert_eq!(summary.payload["agent"]["id"], "organizer");
    assert_eq!(summary.payload["agent"]["role"], "organizer");
    assert_eq!(
        summary.payload["actions_summary"],
        "integrated final answer for the requester"
    );
}
