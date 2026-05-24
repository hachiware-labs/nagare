use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use crate::adapters::{ProcessCodexCliAdapter, StdioCodexAppServerAdapter};
use crate::model::AgentAdapter;
use crate::*;

pub(crate) fn is_ja(locale: &str) -> bool {
    locale.to_ascii_lowercase().starts_with("ja")
}

pub(crate) fn localized_text<'a>(locale: &str, en: &'a str, ja: &'a str) -> &'a str {
    if is_ja(locale) { ja } else { en }
}

pub(crate) fn agent_run_claim(
    locale: &str,
    purpose: AgentRunPurpose,
    status: AgentRunStatus,
    agent_profile_id: &str,
) -> String {
    match (is_ja(locale), purpose, status) {
        (true, AgentRunPurpose::Work, AgentRunStatus::Succeeded) => {
            format!("Agent Profile `{agent_profile_id}` の実行が成功した")
        }
        (true, AgentRunPurpose::Work, AgentRunStatus::Failed) => {
            format!("Agent Profile `{agent_profile_id}` の実行が失敗した")
        }
        (true, AgentRunPurpose::DispatchPreview, AgentRunStatus::Succeeded) => {
            format!("Dispatch Agent `{agent_profile_id}` の実行前確認が成功した")
        }
        (true, AgentRunPurpose::DispatchPreview, AgentRunStatus::Failed) => {
            format!("Dispatch Agent `{agent_profile_id}` の実行前確認が失敗した")
        }
        (true, AgentRunPurpose::Review, AgentRunStatus::Succeeded) => {
            format!("Review Agent `{agent_profile_id}` の評価が成功した")
        }
        (true, AgentRunPurpose::Review, AgentRunStatus::Failed) => {
            format!("Review Agent `{agent_profile_id}` の評価が失敗した")
        }
        (false, AgentRunPurpose::Work, AgentRunStatus::Succeeded) => {
            format!("Agent run succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Work, AgentRunStatus::Failed) => {
            format!("Agent run failed with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::DispatchPreview, AgentRunStatus::Succeeded) => {
            format!("Dispatch preview succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::DispatchPreview, AgentRunStatus::Failed) => {
            format!("Dispatch preview failed with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Review, AgentRunStatus::Succeeded) => {
            format!("Review succeeded with profile `{agent_profile_id}`")
        }
        (false, AgentRunPurpose::Review, AgentRunStatus::Failed) => {
            format!("Review failed with profile `{agent_profile_id}`")
        }
    }
}

pub(crate) fn command_exit_basis(locale: &str, exit_code: Option<i32>) -> String {
    if is_ja(locale) {
        format!("command の exit code は {exit_code:?}")
    } else {
        format!("command exit code {exit_code:?}")
    }
}

pub(crate) fn verification_claim(locale: &str, result: VerificationStatus) -> String {
    if is_ja(locale) {
        match result {
            VerificationStatus::Passed => "検証に成功した".to_string(),
            VerificationStatus::Failed => "検証に失敗した".to_string(),
        }
    } else {
        format!("Verification {result}")
    }
}

pub(crate) fn verification_basis(locale: &str, command: &str, exit_code: Option<i32>) -> String {
    if is_ja(locale) {
        format!("command `{command}` の exit code は {exit_code:?}")
    } else {
        format!("command `{command}` exit code {exit_code:?}")
    }
}

pub(crate) fn default_approval_rationale(locale: &str) -> &'static str {
    localized_text(
        locale,
        "Human approved after required verification",
        "必要な検証を確認したため、人間が承認した",
    )
}

pub(crate) fn run_shell(command: &str) -> io::Result<CommandRunOutput> {
    run_shell_in(command, None)
}

pub(crate) fn run_shell_in(command: &str, cwd: Option<&Path>) -> io::Result<CommandRunOutput> {
    let mut process = if cfg!(windows) {
        let mut command_builder = Command::new("cmd");
        command_builder.args(["/C", command]);
        command_builder
    } else {
        let mut command_builder = Command::new("sh");
        command_builder.args(["-lc", command]);
        command_builder
    };
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    let output = process.output()?;
    Ok(CommandRunOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
    })
}

pub(crate) fn write_command_log(
    path: &Path,
    command: &str,
    output: &CommandRunOutput,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "command: {command}\nexit_code: {:?}\n\n[stdout]\n{}\n[stderr]\n{}\n",
            output.exit_code, output.stdout, output.stderr
        ),
    )
}

pub(crate) fn write_adapter_log(
    path: &Path,
    run_packet: &ResolvedRunPacket,
    output: &AdapterRunOutput,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        format!(
            "run_packet: {}\nwork_item: {}\nagent_profile: {}\nadapter: {}\nworking_dir: {}\ngoal: {}\ncommand: {}\nexit_code: {:?}\n\n[stdout]\n{}\n[stderr]\n{}\n",
            run_packet.id,
            run_packet.work_item_id,
            run_packet.agent_profile_id,
            run_packet.adapter_id,
            run_packet.working_dir,
            run_packet.goal,
            output.command,
            output.exit_code,
            output.stdout,
            output.stderr
        ),
    )
}

pub(crate) fn write_json_artifact<T: Serialize>(
    layout: &ProjectLayout,
    filename: &str,
    value: &T,
) -> Result<(), NagareError> {
    fs::create_dir_all(&layout.artifacts_dir)?;
    let path = layout.artifacts_dir.join(filename);
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{raw}\n"))?;
    Ok(())
}

pub(crate) fn summarize_dispatch_output(output: &str) -> String {
    let deltas = output
        .lines()
        .filter_map(|line| line.trim().strip_prefix("agent.delta: "))
        .collect::<Vec<_>>();
    if !deltas.is_empty() {
        return deltas.join("").trim().to_string();
    }

    for line in output.lines() {
        let Some(raw) = line.trim().strip_prefix("item/completed: ") else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(raw) else {
            continue;
        };
        if let Some(text) = value.pointer("/params/item/text").and_then(Value::as_str) {
            if !text.trim().is_empty() {
                return text.trim().to_string();
            }
        }
    }

    output
        .lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.to_ascii_lowercase().starts_with("risk:")
                && !line.to_ascii_lowercase().starts_with("missing:")
        })
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            first_nonempty_line(output).unwrap_or_else(|| "(no dispatch output)".to_string())
        })
}

pub(crate) fn extract_prefixed_lines(output: &str, prefix: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .to_ascii_lowercase()
                .strip_prefix(prefix)
                .map(|_| trimmed[prefix.len()..].trim().to_string())
        })
        .filter(|line| !line.is_empty())
        .collect()
}

pub(crate) fn run_dev_command(command: &str, cwd: &Path) -> Result<AdapterRunOutput, NagareError> {
    let output = run_shell_in(command, Some(cwd))?;
    Ok(AdapterRunOutput {
        command: format!("{command} [cwd={}]", cwd.display()),
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.exit_code,
    })
}

pub(crate) fn normalize_adapter_id(adapter_id: &str) -> Result<&'static str, NagareError> {
    match adapter_id {
        "process.codex-cli" | "process-codex-cli" => Ok("process.codex-cli"),
        "stdio.codex-app-server" | "stdio-codex-app-server" => Ok("stdio.codex-app-server"),
        _ => Err(NagareError::InvalidState(format!(
            "unsupported adapter `{adapter_id}`"
        ))),
    }
}

pub(crate) fn adapter_for_id(adapter_id: &str) -> Result<Box<dyn AgentAdapter>, NagareError> {
    match adapter_id {
        "process.codex-cli" => Ok(Box::new(ProcessCodexCliAdapter)),
        "stdio.codex-app-server" => Ok(Box::new(StdioCodexAppServerAdapter)),
        _ => Err(NagareError::InvalidState(format!(
            "unsupported adapter `{adapter_id}`"
        ))),
    }
}

pub(crate) fn path_uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

pub(crate) fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("{seconds}")
}

pub(crate) fn timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub(crate) fn scenario_command(message: &str, success: bool) -> String {
    if cfg!(windows) {
        if success {
            format!("echo {message} && exit /B 0")
        } else {
            format!("echo {message} && exit /B 1")
        }
    } else if success {
        format!("echo {message}; exit 0")
    } else {
        format!("echo {message}; exit 1")
    }
}
