use std::fs;
use std::process::Command;

use crate::*;

pub(crate) fn collect_git_run_artifacts(
    layout: &ProjectLayout,
    ledger: &mut Ledger,
    work_item_id: &str,
    run_id: &str,
    locale: &str,
    created_at: &str,
) -> Result<Vec<Artifact>, NagareError> {
    if !is_git_work_tree(layout) {
        return Ok(Vec::new());
    }
    let changed_files = git_changed_files(layout)?;
    if changed_files.is_empty() {
        return Ok(Vec::new());
    }

    let mut artifacts = Vec::new();
    let changed_files_id = ledger.next_id("art");
    let changed_files_path = layout
        .artifacts_dir
        .join(format!("{run_id}_changed_files.txt"));
    fs::write(&changed_files_path, changed_files.join("\n"))?;
    artifacts.push(Artifact {
        id: changed_files_id,
        work_item_id: work_item_id.to_string(),
        agent_run_id: Some(run_id.to_string()),
        artifact_type: "changed_files".to_string(),
        uri: path_uri(&changed_files_path),
        title: format!("{run_id} changed files"),
        locale: locale.to_string(),
        created_at: created_at.to_string(),
    });

    let diff = git_diff(layout)?;
    if !diff.trim().is_empty() {
        let diff_id = ledger.next_id("art");
        let diff_path = layout.artifacts_dir.join(format!("{run_id}_diff.patch"));
        fs::write(&diff_path, diff)?;
        artifacts.push(Artifact {
            id: diff_id,
            work_item_id: work_item_id.to_string(),
            agent_run_id: Some(run_id.to_string()),
            artifact_type: "diff_patch".to_string(),
            uri: path_uri(&diff_path),
            title: format!("{run_id} git diff"),
            locale: locale.to_string(),
            created_at: created_at.to_string(),
        });
    }
    Ok(artifacts)
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
