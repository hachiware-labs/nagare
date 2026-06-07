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
