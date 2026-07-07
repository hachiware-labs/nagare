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
        "## Nagare Review\nverdict: pass\nsummary:\n- trace criterion satisfied\ncriteria:\n- trace criterion: pass\nfindings:\n- no blockers\nquestions:\nnext_action: approve\n",
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
            ..CreateWorkItemInput::default()
        },
    )
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
            agent_profile_id: "supervisor",
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
    assert_eq!(organizer.payload["agent"]["id"], "dispatcher");
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
            agent_profile_id: "supervisor",
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
    assert_eq!(summary.payload["agent"]["id"], "supervisor");
    assert_eq!(summary.payload["agent"]["role"], "organizer");
    assert_eq!(
        summary.payload["actions_summary"],
        "integrated final answer for the requester"
    );
}
