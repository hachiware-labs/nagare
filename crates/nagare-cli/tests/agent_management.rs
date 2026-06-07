use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_root(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    env::temp_dir().join(format!("nagare-cli-{label}-{}-{now}", std::process::id()))
}

fn nagare(root: &Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nagare"));
    command.args(args).arg("--root").arg(root);
    command.output().expect("nagare command should run")
}

fn assert_success(output: std::process::Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn agent_add_delete_targets_codex_cli_profile() {
    let root = test_root("codex-cli");
    assert_success(nagare(&root, &["init"]));
    assert_success(nagare(
        &root,
        &[
            "agent",
            "add",
            "--id",
            "target-codex-cli",
            "--provider",
            "codex-cli",
            "--model-provider",
            "openai",
            "--model",
            "gpt-5.3-codex",
        ],
    ));
    let show = nagare(&root, &["agent", "show", "target-codex-cli"]);
    assert_success(show);
    let stdout =
        String::from_utf8_lossy(&nagare(&root, &["agent", "show", "target-codex-cli"]).stdout)
            .to_string();
    assert!(stdout.contains("adapter: process.codex-cli"));
    assert!(stdout.contains("model.id: gpt-5.3-codex"));
    assert!(stdout.contains("external.provider: codex-cli"));
    assert_success(nagare(&root, &["agent", "delete", "target-codex-cli"]));
    let missing = nagare(&root, &["agent", "show", "target-codex-cli"]);
    assert!(!missing.status.success());
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_add_can_attach_installed_skill_sets() {
    let root = test_root("agent-skills");
    assert_success(nagare(&root, &["init"]));
    let config_path = root.join(".nagare").join("project.toml");
    let mut config = fs::read_to_string(&config_path).expect("config should read");
    config.push_str(
        r#"

[skill_sets.react-review]
paths = ["skills/react-review"]
required_capabilities = ["repo_read"]
optional_capabilities = []

[skill_sets.test-runner]
paths = ["skills/test-runner"]
required_capabilities = ["repo_read"]
optional_capabilities = ["shell_command"]
"#,
    );
    fs::write(&config_path, config).expect("config should write");

    assert_success(nagare(
        &root,
        &[
            "agent",
            "add",
            "--id",
            "frontend-worker",
            "--provider",
            "codex-cli",
            "--model-provider",
            "openai",
            "--model",
            "gpt-5.3-codex",
            "--skills",
            "react-review,test-runner",
        ],
    ));

    let show = nagare(&root, &["agent", "show", "frontend-worker"]);
    assert_success(show);
    let stdout =
        String::from_utf8_lossy(&nagare(&root, &["agent", "show", "frontend-worker"]).stdout)
            .to_string();
    assert!(stdout.contains("tool_kind: codex_cli"));
    assert!(stdout.contains("skills: react-review,test-runner"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn skill_add_from_skill_creator_folder_infers_skill_name() {
    let root = test_root("skill-creator");
    assert_success(nagare(&root, &["init"]));
    let skill_dir = root.join("skills").join("react-review");
    fs::create_dir_all(&skill_dir).expect("skill dir should create");
    fs::write(
        skill_dir.join("SKILL.md"),
        r#"---
name: react-review
description: Review React code for maintainability and performance.
---

# React Review
"#,
    )
    .expect("skill should write");

    assert_success(nagare(
        &root,
        &[
            "skill",
            "add",
            "--from",
            "skill-creator",
            "--path",
            skill_dir.to_str().expect("path should be utf-8"),
            "--requires",
            "repo_read",
            "--optional",
            "shell_command",
        ],
    ));

    let config =
        fs::read_to_string(root.join(".nagare").join("project.toml")).expect("config should read");
    assert!(config.contains("[skill_packages.react-review]"));
    assert!(config.contains("source_kind = \"skill_creator\""));
    assert!(config.contains("[skill_sets.react-review]"));
    assert!(config.contains("required_capabilities = [\"repo_read\"]"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn skill_add_records_clawhub_and_vercel_sources() {
    let root = test_root("skill-sources");
    assert_success(nagare(&root, &["init"]));

    assert_success(nagare(
        &root,
        &[
            "skill",
            "add",
            "--from",
            "clawhub",
            "--id",
            "skill-provenance",
            "--source",
            "skill-provenance",
            "--ref",
            "1.0.0",
            "--checksum",
            "sha256:example",
            "--requires",
            "repo_read",
        ],
    ));
    assert_success(nagare(
        &root,
        &[
            "skill",
            "add",
            "--from",
            "vercel",
            "--id",
            "vercel-react",
            "--source",
            "vercel-labs/agent-skills",
            "--skill-id",
            "vercel-react",
            "--paths",
            "src",
            "--requires",
            "repo_read",
        ],
    ));

    let list = nagare(&root, &["skill", "list"]);
    assert_success(list);
    let stdout = String::from_utf8_lossy(&nagare(&root, &["skill", "list"]).stdout).to_string();
    assert!(stdout.contains("skill-provenance source_kind=clawhub"));
    assert!(stdout.contains("vercel-react source_kind=vercel"));
    assert!(stdout.contains("skill_sets:"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_add_delete_targets_codex_app_server_profile() {
    let root = test_root("codex-app");
    assert_success(nagare(&root, &["init"]));
    assert_success(nagare(
        &root,
        &[
            "agent",
            "add",
            "--id",
            "target-codex-app",
            "--provider",
            "codex",
            "--model-provider",
            "openai-codex",
            "--model",
            "gpt-5.3-codex",
        ],
    ));
    let show = nagare(&root, &["agent", "show", "target-codex-app"]);
    assert_success(show);
    let stdout =
        String::from_utf8_lossy(&nagare(&root, &["agent", "show", "target-codex-app"]).stdout)
            .to_string();
    assert!(stdout.contains("adapter: stdio.codex-app-server"));
    assert!(stdout.contains("model.provider: openai-codex"));
    assert!(stdout.contains("external.provider: codex"));
    assert_success(nagare(&root, &["agent", "delete", "target-codex-app"]));
    let missing = nagare(&root, &["agent", "show", "target-codex-app"]);
    assert!(!missing.status.success());
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_add_delete_targets_openclaw_external_agent() {
    let root = test_root("openclaw");
    fs::create_dir_all(&root).expect("root should create");
    let log_path = root.join("openclaw.log");
    let fake_openclaw = fake_openclaw_command(&root, &log_path);
    assert_success(nagare(&root, &["init"]));

    let mut add = Command::new(env!("CARGO_BIN_EXE_nagare"));
    add.env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args([
            "add",
            "--id",
            "target-openclaw",
            "--provider",
            "openclaw",
            "--external-agent-id",
            "openclaw-target-1",
            "--model-provider",
            "anthropic",
            "--model",
            "claude-sonnet-4",
            "--base-url",
            "https://api.example.test/v1",
            "--api-key-env",
            "ANTHROPIC_API_KEY",
        ])
        .arg("--root")
        .arg(&root);
    assert_success(add.output().expect("add should run"));

    let mut rename = Command::new(env!("CARGO_BIN_EXE_nagare"));
    rename
        .env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args([
            "update",
            "target-openclaw",
            "--display-name",
            "Target OpenClaw Renamed",
        ])
        .arg("--root")
        .arg(&root);
    assert_success(rename.output().expect("rename should run"));

    let mut remodel = Command::new(env!("CARGO_BIN_EXE_nagare"));
    remodel
        .env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args(["update", "target-openclaw", "--model", "claude-opus-4"])
        .arg("--root")
        .arg(&root);
    assert_success(remodel.output().expect("model update should run"));

    let mut delete = Command::new(env!("CARGO_BIN_EXE_nagare"));
    delete
        .env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args(["delete", "target-openclaw"])
        .arg("--root")
        .arg(&root);
    assert_success(delete.output().expect("delete should run"));

    let log = fs::read_to_string(log_path).expect("fake openclaw log should exist");
    assert!(log.contains("config set models.providers.anthropic"));
    assert!(log.contains("agents add openclaw-target-1"));
    assert!(log.contains("--model anthropic/claude-sonnet-4"));
    assert!(log.contains("agents set-identity --agent openclaw-target-1 --workspace"));
    assert!(log.contains("Target OpenClaw Renamed"));
    assert!(log.contains("agents delete openclaw-target-1 --force --json"));
    assert!(log.contains("--model anthropic/claude-opus-4"));
    assert!(log.contains("agents delete openclaw-target-1 --force --json"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_add_openclaw_can_use_default_model() {
    let root = test_root("openclaw-default-model");
    fs::create_dir_all(&root).expect("root should create");
    let log_path = root.join("openclaw.log");
    let fake_openclaw = fake_openclaw_command(&root, &log_path);
    assert_success(nagare(&root, &["init"]));

    let mut add = Command::new(env!("CARGO_BIN_EXE_nagare"));
    add.env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args([
            "add",
            "--id",
            "default-model-openclaw",
            "--provider",
            "openclaw",
            "--external-agent-id",
            "openclaw-default-model",
        ])
        .arg("--root")
        .arg(&root);
    assert_success(add.output().expect("add should run"));

    let log = fs::read_to_string(log_path).expect("fake openclaw log should exist");
    assert!(log.contains("agents add openclaw-default-model --workspace"));
    assert!(!log.contains("--model"));
    fs::remove_dir_all(root).ok();
}

#[test]
fn agent_add_openclaw_ollama_requires_base_url() {
    let root = test_root("openclaw-ollama-base-url");
    fs::create_dir_all(&root).expect("root should create");
    let log_path = root.join("openclaw.log");
    let fake_openclaw = fake_openclaw_command(&root, &log_path);
    assert_success(nagare(&root, &["init"]));

    let mut add = Command::new(env!("CARGO_BIN_EXE_nagare"));
    add.env("NAGARE_OPENCLAW_COMMAND", &fake_openclaw)
        .arg("agent")
        .args([
            "add",
            "--id",
            "ollama-openclaw",
            "--provider",
            "openclaw",
            "--model-provider",
            "ollama",
            "--model",
            "qwen2.5-coder:32b",
        ])
        .arg("--root")
        .arg(&root);
    let output = add.output().expect("add should run");
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires model.base_url"));
    fs::remove_dir_all(root).ok();
}

fn fake_openclaw_command(root: &Path, log_path: &Path) -> PathBuf {
    if cfg!(windows) {
        let path = root.join("fake-openclaw.cmd");
        fs::write(
            &path,
            format!(
                "@echo off\r\necho %*>>\"{}\"\r\nexit /B 0\r\n",
                log_path.display()
            ),
        )
        .expect("fake openclaw should write");
        path
    } else {
        let path = root.join("fake-openclaw.sh");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
                log_path.display()
            ),
        )
        .expect("fake openclaw should write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }
        path
    }
}
