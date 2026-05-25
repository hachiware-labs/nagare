use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::*;

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub nagare_dir: PathBuf,
    pub config_path: PathBuf,
    pub agents_dir: PathBuf,
    pub state_dir: PathBuf,
    pub ledger_path: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl ProjectLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let nagare_dir = root.join(".nagare");
        let state_dir = nagare_dir.join("state");
        Self {
            config_path: nagare_dir.join("project.toml"),
            agents_dir: nagare_dir.join("agents"),
            ledger_path: state_dir.join("ledger.json"),
            state_dir,
            artifacts_dir: nagare_dir.join("artifacts"),
            logs_dir: nagare_dir.join("logs"),
            nagare_dir,
            root,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InitResult {
    pub layout: ProjectLayout,
    pub created_config: bool,
    pub created_ledger: bool,
}

pub fn init_project(root: impl Into<PathBuf>) -> io::Result<InitResult> {
    let layout = ProjectLayout::new(root);
    fs::create_dir_all(&layout.state_dir)?;
    fs::create_dir_all(&layout.artifacts_dir)?;
    fs::create_dir_all(&layout.logs_dir)?;
    fs::create_dir_all(&layout.agents_dir)?;

    let created_config = if layout.config_path.exists() {
        false
    } else {
        fs::write(&layout.config_path, default_config())?;
        true
    };

    let created_ledger = if layout.ledger_path.exists() {
        false
    } else {
        save_ledger(&layout, &Ledger::default())?;
        true
    };

    Ok(InitResult {
        layout,
        created_config,
        created_ledger,
    })
}

pub(crate) fn default_config() -> &'static str {
    r#"# Nagare local project configuration.

[project]
name = "nagare-local"

[storage]
kind = "json-ledger"
path = ".nagare/state/ledger.json"
sqlite_future_path = ".nagare/state/nagare.db"

[locale]
language = "ja-JP"
timezone = "Asia/Tokyo"

[nagare_agents]
work_agent = "codex-cli"
review_agent = "codex-app-server"
dispatch_agent = "codex-cli"

[runtimes.codex-local]
kind = "process"
command = "codex"
args = ["exec"]
healthcheck = ["codex", "--version"]

[runtimes.codex-app-local]
kind = "stdio"
command = "codex"
args = ["app-server", "--listen", "stdio://"]
healthcheck = ["codex", "app-server", "--help"]

[adapters.process-codex-cli]
kind = "process.codex-cli"
runtime_kind = "process"
known_capabilities = ["repo_read", "file_edit", "shell_command", "stdin_prompt"]

[adapters.stdio-codex-app-server]
kind = "stdio.codex-app-server"
runtime_kind = "stdio"
known_capabilities = ["repo_read", "file_edit", "shell_command", "thread_state", "approval_flow", "event_stream"]

[agent_profiles.codex-cli]
display_name = "Codex CLI Implementer"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "implementer"
working_dir = "."

[agent_profiles.codex-app-server]
display_name = "Codex App Server Implementer"
runtime = "codex-app-local"
adapter = "stdio-codex-app-server"
role = "implementer"
working_dir = "."

[permission_policies.medium-code-task]
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]

[workspace_policies.project-root]
kind = "project_root"
isolate_per_work_item = false
cleanup = "keep"
"#
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub root: PathBuf,
    pub has_git: bool,
    pub has_config: bool,
    pub has_ledger: bool,
    pub tools: Vec<ToolStatus>,
}

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub detail: String,
}

impl fmt::Display for ToolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.available { "ok" } else { "missing" };
        write!(f, "{}: {} ({})", self.name, marker, self.detail)
    }
}

pub fn doctor(root: impl Into<PathBuf>) -> DoctorReport {
    let root = root.into();
    let layout = ProjectLayout::new(&root);
    DoctorReport {
        has_git: root.join(".git").exists(),
        has_config: layout.config_path.exists(),
        has_ledger: layout.ledger_path.exists(),
        tools: vec![
            check_tool("git", &["--version"]),
            check_tool("node", &["--version"]),
            check_tool("npm", &["--version"]),
            check_tool("codex", &["--version"]),
            check_tool("codex", &["app-server", "--help"]),
        ],
        root,
    }
}

pub(crate) fn check_tool(name: &str, args: &[&str]) -> ToolStatus {
    match run_tool(name, args) {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = first_nonempty_line(&stdout)
                .or_else(|| first_nonempty_line(&stderr))
                .unwrap_or_else(|| "available".to_string());
            ToolStatus {
                name: name.to_string(),
                available: true,
                detail,
            }
        }
        Ok(output) => ToolStatus {
            name: name.to_string(),
            available: false,
            detail: format!("exit status {}", output.status),
        },
        Err(error) => ToolStatus {
            name: name.to_string(),
            available: false,
            detail: error.to_string(),
        },
    }
}

pub(crate) fn check_command(command: &str, args: &[String]) -> ToolStatus {
    let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
    check_tool(command, &borrowed)
}

pub(crate) fn run_tool(name: &str, args: &[&str]) -> io::Result<std::process::Output> {
    match Command::new(name).args(args).output() {
        Ok(output) => Ok(output),
        Err(error) if cfg!(windows) && error.kind() == io::ErrorKind::NotFound => {
            Command::new(format!("{name}.cmd")).args(args).output()
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

pub fn resolve_root(root_arg: Option<&str>) -> io::Result<PathBuf> {
    match root_arg {
        Some(root) => Ok(Path::new(root).to_path_buf()),
        None if env::var_os("NAGARE_ROOT").is_some() => Ok(PathBuf::from(
            env::var_os("NAGARE_ROOT").expect("checked above"),
        )),
        None => env::current_dir(),
    }
}
