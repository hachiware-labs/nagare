use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use serde_json::{Value, json};

use crate::*;

pub(crate) struct ProcessCodexCliAdapter;

impl AgentAdapter for ProcessCodexCliAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError> {
        if let Some(command) = request.dev_command {
            return run_dev_command(command, request.working_dir);
        }

        let output = run_codex_cli_exec(request.working_dir, request.prompt)?;
        Ok(AdapterRunOutput {
            command: format!("codex exec --cd {} <prompt>", request.working_dir.display()),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        })
    }
}

fn run_codex_cli_exec(working_dir: &Path, prompt: &str) -> Result<Output, NagareError> {
    let cd = working_dir.display().to_string();
    let args = ["exec", "--cd", cd.as_str(), prompt];
    if cfg!(windows) {
        if let Some(script) = find_windows_codex_js() {
            return Ok(Command::new("node").arg(script).args(args).output()?);
        }
    }
    Ok(run_tool("codex", &args)?)
}

fn find_windows_codex_js() -> Option<PathBuf> {
    let output = Command::new("where").arg("codex").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let path = PathBuf::from(line.trim());
            let dir = path.parent()?;
            let script = dir
                .join("node_modules")
                .join("@openai")
                .join("codex")
                .join("bin")
                .join("codex.js");
            script.exists().then_some(script)
        })
        .next()
}

pub(crate) struct StdioCodexAppServerAdapter;

impl AgentAdapter for StdioCodexAppServerAdapter {
    fn run(&self, request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError> {
        if let Some(command) = request.dev_command {
            return run_dev_command(command, request.working_dir);
        }

        run_codex_app_server(request)
    }
}

fn run_codex_app_server(request: &AdapterRunRequest<'_>) -> Result<AdapterRunOutput, NagareError> {
    let mut session = CodexAppServerSession::spawn(request.working_dir)?;
    let cwd = absolute_path_string(request.working_dir)?;

    session.send_request(
        1,
        "initialize",
        json!({
            "clientInfo": {
                "name": "nagare",
                "title": "Nagare",
                "version": VERSION,
            },
            "capabilities": {
                "experimentalApi": true,
            },
        }),
    )?;
    let initialize = session.read_response(1)?;
    let mut transcript = Vec::new();
    transcript.push(format!("initialize: {}", compact_json(&initialize)));

    session.send_request(
        2,
        "thread/start",
        json!({
            "cwd": cwd,
            "ephemeral": true,
            "approvalPolicy": "never",
            "sandbox": "workspace-write",
            "threadSource": "user",
            "developerInstructions": format!(
                "You are executing Nagare run packet {} for Work Item {}. Keep the final response concise and describe the work result.",
                request.run_packet.id, request.run_packet.work_item_id
            ),
        }),
    )?;
    let thread_start = session.read_response(2)?;
    transcript.push(format!("thread/start: {}", compact_json(&thread_start)));
    let thread_id = thread_start
        .pointer("/result/thread/id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            NagareError::InvalidState("codex app-server response missing thread id".to_string())
        })?
        .to_string();

    session.send_request(
        3,
        "turn/start",
        json!({
            "threadId": thread_id,
            "cwd": cwd,
            "approvalPolicy": "never",
            "input": [{
                "type": "text",
                "text": request.prompt,
            }],
        }),
    )?;
    let turn_start = session.read_response(3)?;
    transcript.push(format!("turn/start: {}", compact_json(&turn_start)));

    let completed = session.read_until_turn_completed(&mut transcript)?;
    session.shutdown();

    Ok(AdapterRunOutput {
        command: format!(
            "codex app-server --listen stdio:// thread/start + turn/start [cwd={}]",
            request.working_dir.display()
        ),
        stdout: transcript.join("\n"),
        stderr: session.stderr.clone(),
        exit_code: if completed { Some(0) } else { Some(1) },
    })
}

struct CodexAppServerSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr: String,
}

impl CodexAppServerSession {
    fn spawn(cwd: &Path) -> Result<Self, NagareError> {
        let mut child = spawn_codex_app_server(cwd)?;
        let stdin = child.stdin.take().ok_or_else(|| {
            NagareError::InvalidState("failed to open codex app-server stdin".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            NagareError::InvalidState("failed to open codex app-server stdout".to_string())
        })?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            stderr: String::new(),
        })
    }

    fn send_request(&mut self, id: u64, method: &str, params: Value) -> Result<(), NagareError> {
        let request = json!({
            "id": id,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{request}")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self, expected_id: u64) -> Result<Value, NagareError> {
        loop {
            let line = self.read_line()?;
            if line.trim().is_empty() {
                continue;
            }
            let message: Value = serde_json::from_str(&line)?;
            if is_response_id(&message, expected_id) {
                if let Some(error) = message.get("error") {
                    return Err(NagareError::InvalidState(format!(
                        "codex app-server request {expected_id} failed: {}",
                        compact_json(error)
                    )));
                }
                return Ok(message);
            }
        }
    }

    fn read_until_turn_completed(
        &mut self,
        transcript: &mut Vec<String>,
    ) -> Result<bool, NagareError> {
        loop {
            let line = self.read_line()?;
            if line.trim().is_empty() {
                continue;
            }
            let message: Value = serde_json::from_str(&line)?;
            let Some(method) = message.get("method").and_then(Value::as_str) else {
                transcript.push(format!("response: {}", compact_json(&message)));
                continue;
            };
            match method {
                "item/agentMessage/delta" => {
                    if let Some(delta) = message.pointer("/params/delta").and_then(Value::as_str) {
                        transcript.push(format!("agent.delta: {delta}"));
                    }
                }
                "turn/completed" => {
                    transcript.push(format!("turn/completed: {}", compact_json(&message)));
                    let status = message
                        .pointer("/params/turn/status")
                        .and_then(Value::as_str);
                    return Ok(status == Some("completed"));
                }
                "error" => {
                    transcript.push(format!("error: {}", compact_json(&message)));
                    return Ok(false);
                }
                _ => transcript.push(format!("{method}: {}", compact_json(&message))),
            }
        }
    }

    fn read_line(&mut self) -> Result<String, NagareError> {
        let mut line = String::new();
        let count = self.stdout.read_line(&mut line)?;
        if count == 0 {
            return Err(NagareError::InvalidState(
                "codex app-server closed stdout before run completed".to_string(),
            ));
        }
        Ok(line)
    }

    fn shutdown(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for CodexAppServerSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn spawn_codex_app_server(cwd: &Path) -> Result<Child, NagareError> {
    match Command::new("codex")
        .args(["app-server", "--listen", "stdio://"])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => Ok(child),
        Err(error) if cfg!(windows) && error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Command::new("codex.cmd")
                .args(["app-server", "--listen", "stdio://"])
                .current_dir(cwd)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?)
        }
        Err(error) => Err(error.into()),
    }
}

fn absolute_path_string(path: &Path) -> Result<String, NagareError> {
    Ok(path.canonicalize()?.display().to_string())
}

fn is_response_id(message: &Value, expected_id: u64) -> bool {
    message
        .get("id")
        .and_then(Value::as_u64)
        .is_some_and(|id| id == expected_id)
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<invalid json>".to_string())
}
