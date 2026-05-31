use std::fs;
use std::path::Path;
use std::process::Command;

use crate::*;

pub(crate) fn collect_git_execution_records(
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    work_item_id: &str,
    run_id: &str,
    locale: &str,
    created_at: &str,
) -> Result<Vec<ExecutionRecord>, NagareError> {
    if !is_git_work_tree(layout) {
        return Ok(Vec::new());
    }
    let changed_files = git_changed_files(layout)?;
    if changed_files.is_empty() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let changed_files_id = ledger.next_id("exec");
    let changed_files_path = layout.logs_dir.join(format!("{run_id}_changed_files.txt"));
    fs::write(&changed_files_path, changed_files.join("\n"))?;
    records.push(ExecutionRecord {
        id: changed_files_id,
        work_item_id: work_item_id.to_string(),
        agent_run_id: Some(run_id.to_string()),
        record_type: "changed_files".to_string(),
        uri: path_uri(&changed_files_path),
        title: format!("{run_id} changed files"),
        locale: locale.to_string(),
        created_at: created_at.to_string(),
    });

    let diff = git_diff(layout)?;
    if !diff.trim().is_empty() {
        let diff_id = ledger.next_id("exec");
        let diff_path = layout.logs_dir.join(format!("{run_id}_diff.patch"));
        fs::write(&diff_path, diff)?;
        records.push(ExecutionRecord {
            id: diff_id,
            work_item_id: work_item_id.to_string(),
            agent_run_id: Some(run_id.to_string()),
            record_type: "diff_patch".to_string(),
            uri: path_uri(&diff_path),
            title: format!("{run_id} git diff"),
            locale: locale.to_string(),
            created_at: created_at.to_string(),
        });
    }
    Ok(records)
}

pub(crate) fn collect_expected_artifacts(
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    item: &WorkItem,
    run_id: &str,
    locale: &str,
    created_at: &str,
) -> Vec<Artifact> {
    item.expected_artifacts
        .iter()
        .filter_map(|expected| {
            expected_artifact_path(&layout.root, expected).map(|path| (expected, path))
        })
        .filter(|(_, path)| path.exists())
        .map(|(expected, path)| Artifact {
            id: ledger.next_id("art"),
            work_item_id: item.id.clone(),
            agent_run_id: Some(run_id.to_string()),
            artifact_type: "deliverable_file".to_string(),
            uri: path_uri(&path),
            title: expected.trim().to_string(),
            locale: locale.to_string(),
            created_at: created_at.to_string(),
        })
        .collect()
}

fn expected_artifact_path(root: &Path, expected: &str) -> Option<std::path::PathBuf> {
    let expected = expected.trim();
    if expected.is_empty() {
        return None;
    }
    let path = Path::new(expected);
    Some(if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    })
}

fn is_git_work_tree(layout: &ProjectLayout) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(&layout.root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn git_changed_files(layout: &ProjectLayout) -> Result<Vec<String>, NagareError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(&layout.root)
        .args(["status", "--porcelain", "--untracked-files=all"])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(status_path)
        .filter(|path| !path.starts_with(".nagare/") && path != ".nagare")
        .collect())
}

fn status_path(line: &str) -> Option<String> {
    let value = line.get(3..)?.trim();
    if value.is_empty() {
        return None;
    }
    Some(
        value
            .rsplit_once(" -> ")
            .map(|(_, to)| to)
            .unwrap_or(value)
            .replace('\\', "/"),
    )
}

fn git_diff(layout: &ProjectLayout) -> Result<String, NagareError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(&layout.root)
        .args(["diff", "--binary"])
        .output()?;
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
