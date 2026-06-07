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
    pub domain_groups_dir: PathBuf,
    pub domains_dir: PathBuf,
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
            domain_groups_dir: nagare_dir.join("domain-groups"),
            domains_dir: nagare_dir.join("domains"),
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
    fs::create_dir_all(&layout.domain_groups_dir)?;
    fs::create_dir_all(&layout.domains_dir)?;
    seed_default_domains(&layout)?;

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

pub(crate) fn seed_default_domains(layout: &ProjectLayout) -> io::Result<()> {
    let i18n = default_seed_i18n(layout);
    write_default_seed_file(
        &layout.domain_groups_dir.join("general.toml"),
        &i18n.general_domain_group_toml(),
        &[
            "General-purpose work that does not need a specialized domain.",
            "display_name = \"General\"",
        ],
    )?;
    write_default_seed_file(
        &layout.domains_dir.join("general.toml"),
        &i18n.general_domain_profile_toml(),
        &[
            "General implementation, review, documentation, and maintenance work.",
            "display_name = \"General\"",
        ],
    )
}

fn write_default_seed_file(path: &Path, contents: &str, legacy_markers: &[&str]) -> io::Result<()> {
    if !path.exists() {
        return fs::write(path, contents);
    }
    let raw = fs::read_to_string(path)?;
    if legacy_markers.iter().any(|marker| raw.contains(marker)) {
        return fs::write(path, contents);
    }
    Ok(())
}

fn default_seed_i18n(layout: &ProjectLayout) -> I18n {
    if let Ok(raw) = fs::read_to_string(&layout.config_path) {
        if let Ok(value) = raw.parse::<toml::Value>() {
            if let Some(language) = value
                .get("locale")
                .and_then(|locale| locale.get("language"))
                .and_then(toml::Value::as_str)
            {
                return I18n::new(language);
            }
        }
    }
    I18n::environment()
}

pub(crate) fn default_config() -> String {
    let i18n = I18n::environment();
    i18n.default_config_toml(&detect_environment_timezone())
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
