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
fn static_ui_export_writes_board_and_item_detail() {
    let root = test_root("static-ui-export");
    init_project(&root).expect("project should init");
    fs::write(
        root.join("review.md"),
        "## Nagare Review\nverdict: request_changes\nsummary:\n- source missing\ncompleted:\n- checked source criteria\ncriteria:\n- source listed: failed\nfindings:\n- missing source\nrequested_changes:\n- Add the source.\nquestions:\nnext_notes:\n- next agent should attach docs/source.md\nnext_action: run_agent\n",
    )
    .expect("review should write");
    fs::write(
        root.join("question.md"),
        "## Nagare Result\nstatus: blocked\ncompleted:\n- checked requirements\nquestions:\n- 追加の方針は？\nnext_notes:\n- waiting for user direction\nnext_action: answer_question\n",
    )
    .expect("question should write");
    let item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Export UI detail".to_string(),
            acceptance_criteria: vec!["source listed".to_string()],
            expected_artifacts: vec!["docs/source.md".to_string()],
            work_folder: Some("docs".to_string()),
            workflow_mode: Some(WorkflowMode::FinishFirst),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("item should create")
    .item;
    create_workflow_decision(&root, &item.id).expect("decision should record");
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
    create_recovery_plan(&root, &item.id).expect("recovery should create");
    let question_item = create_work_item_with_input(
        &root,
        CreateWorkItemInput {
            title: "Needs user answer".to_string(),
            workflow_mode: Some(WorkflowMode::ConfirmFirst),
            ..CreateWorkItemInput::default()
        },
    )
    .expect("question item should create")
    .item;
    run_work_item_with_input(
        &root,
        &question_item.id,
        RunWorkItemInput {
            agent_profile_id: "worker",
            dispatch_plan_id: None,
            path: None,
            prompt: None,
            dev_command: Some(cat_command("question.md").as_str()),
            purpose: AgentRunPurpose::Work,
        },
    )
    .expect("question run should run");

    let out_dir = root.join("ui");
    let result = export_static_ui(
        &root,
        StaticUiExportInput {
            out_dir: out_dir.clone(),
        },
    )
    .expect("ui should export");

    assert!(result.index_path.exists());
    assert_eq!(result.item_paths.len(), 2);
    let index = fs::read_to_string(result.index_path).expect("index should read");
    assert!(index.contains("Work Queue"));
    assert!(index.contains("確認キュー"));
    assert!(index.contains("attention-row"));
    assert!(index.contains("criteria"));
    assert!(index.contains("回復案"));
    assert!(index.contains("Needs user answer"));
    assert!(index.contains("追加の方針は？"));
    let detail = fs::read_to_string(out_dir.join("items").join(format!("{}.html", item.id)))
        .expect("detail should read");
    assert!(detail.contains("Next Action Panel"));
    assert!(detail.contains("Human Input Panel"));
    assert!(detail.contains("textarea"));
    assert!(detail.contains("Copy"));
    assert!(detail.contains(&format!("nagare item run {} --prompt", item.id)));
    assert!(detail.contains(&format!("nagare item recover accept {}", item.id)));
    assert!(detail.contains("missing_artifact / request_changes"));
    assert!(detail.contains("Workflow Decision"));
    assert!(detail.contains("Approval Gate"));
    assert!(detail.contains("Agent Output Notes"));
    assert!(detail.contains("Review"));
    assert!(detail.contains("Recovery"));
    assert!(detail.contains("checked source criteria"));
    assert!(detail.contains("next agent should attach docs/source.md"));
    assert!(detail.contains("source listed"));
    assert!(detail.contains("finish_first"));
    let question_detail = fs::read_to_string(
        out_dir
            .join("items")
            .join(format!("{}.html", question_item.id)),
    )
    .expect("question detail should read");
    assert!(question_detail.contains("Human Input Panel"));
    assert!(question_detail.contains("追加の方針は？"));
    assert!(question_detail.contains(&format!("nagare item answer {} --answer", question_item.id)));
    answer_work_item(
        &root,
        &question_item.id,
        AnswerWorkItemInput {
            question: None,
            answer: "続行してください。",
        },
    )
    .expect("answer should record");
    export_static_ui(
        &root,
        StaticUiExportInput {
            out_dir: out_dir.clone(),
        },
    )
    .expect("ui should re-export");
    let ready_detail = fs::read_to_string(
        out_dir
            .join("items")
            .join(format!("{}.html", question_item.id)),
    )
    .expect("ready detail should read");
    assert!(ready_detail.contains("run_agent"));
    assert!(ready_detail.contains(&format!("nagare item run {} --prompt", question_item.id)));
    fs::remove_dir_all(root).ok();
}
