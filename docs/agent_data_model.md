# Agent Profile and Skill Data Model

This document defines the first data shape for agent tool management. It is a
design target for the Agent Management Kernel.

## Entity Ownership

```text
Config-owned:
  RuntimeDeclaration
  AdapterDeclaration
  AgentProfileDeclaration
  DeclaredSkillSet
  ProjectRule
  PermissionPolicy
  WorkspacePolicy

Ledger-owned:
  CapabilityProbe
  ResolvedSkillContext
  ResolvedRunPacket
  AgentRun
  Artifact
  Evidence
  VerificationResult
  HandoffPacket
  HumanDecision
```

Config-owned entities describe intent. Ledger-owned entities describe what was
actually used or observed.

## Project Config TOML

Initial project file:

```text
.nagare/project.toml
```

Example:

```toml
[project]
id = "nagare-local"
name = "Nagare Local"

[locale]
language = "ja-JP"
timezone = "Asia/Tokyo"

[nagare_agents]
work_agent = "codex-impl"
review_agent = "codex-app-impl"
dispatch_agent = "codex-impl"

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

[agent_profiles.codex-impl]
display_name = "Codex Implementer"
runtime = "codex-local"
adapter = "process-codex-cli"
role = "implementer"
working_dir = "."
declared_skill_sets = ["nagare-core", "repo-default"]
permission_policy = "medium-code-task"
workspace_policy = "worktree-per-item"
probe_before_run = true
timeout_minutes = 60
max_parallel_runs = 2

[agent_profiles.codex-app-impl]
display_name = "Codex App Server Implementer"
runtime = "codex-app-local"
adapter = "stdio-codex-app-server"
role = "implementer"
working_dir = "."
declared_skill_sets = ["nagare-core", "repo-default"]
permission_policy = "medium-code-task"
workspace_policy = "worktree-per-item"
probe_before_run = true
timeout_minutes = 60
max_parallel_runs = 2

[skill_sets.nagare-core]
paths = ["skills/nagare-core"]
required_capabilities = ["repo_read"]
optional_capabilities = ["file_edit", "shell_command"]

[skill_sets.repo-default]
paths = ["AGENTS.md"]
required_capabilities = ["repo_read"]
optional_capabilities = []

[permission_policies.medium-code-task]
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]

[workspace_policies.worktree-per-item]
kind = "git_worktree"
isolate_per_work_item = true
cleanup = "keep"

[[project_rules]]
id = "rust-core"
match = ["crates/**"]
default_agent = "codex-impl"
review_agent = "codex-app-impl"
skill_sets = ["nagare-core", "repo-default"]
permission_policy = "medium-code-task"
workspace_policy = "worktree-per-item"
verification = ["cargo test --workspace"]
```

## Project Agent Directory

Project-local Agent Profiles can also be stored as one file per profile:

```text
.nagare/agents/<agent_profile_id>.toml
```

Example:

```toml
[agent_profile]
id = "codex-impl-smoke"
display_name = "Codex CLI Smoke Implementer"
runtime = "codex-local"
adapter = "process.codex-cli"
role = "implementer"
working_dir = "packages/app"
```

These files are created by `nagare agent add`. They override same-id profiles
from `.nagare/project.toml`. `working_dir` is the directory where the agent run
starts. It must be a relative path inside the project; the default is `"."`.

## Nagare Agent Defaults

`[nagare_agents]` selects the Agent Profiles Nagare itself uses when the user
does not explicitly specify one.

```toml
[nagare_agents]
work_agent = "codex-impl"
review_agent = "codex-app-impl"
dispatch_agent = "codex-impl"
```

- `work_agent`: default target for `nagare item run` when `--agent` is omitted.
- `review_agent`: default profile for review-oriented flows.
- `dispatch_agent`: default profile that will propose or resolve dispatch plans.

## Ledger JSON Shapes

### CapabilityProbe

```json
{
  "id": "probe_0001",
  "agent_profile_id": "codex-impl",
  "runtime_id": "codex-local",
  "adapter_id": "process-codex-cli",
  "runtime_version": "codex-cli 0.130.0",
  "available": true,
  "discovered_capabilities": [
    "repo_read",
    "file_edit",
    "shell_command",
    "stdin_prompt"
  ],
  "instruction_sources": [
    "AGENTS.md",
    ".codex/config.toml"
  ],
  "supported_skill_modes": [
    "prompt_injection",
    "file_reference"
  ],
  "unsupported_declared_skill_sets": [],
  "warnings": [],
  "locale": "ja-JP",
  "source_hashes": {
    "AGENTS.md": "sha256:..."
  },
  "probed_at": "2026-05-24T15:00:00+09:00"
}
```

### ResolvedSkillContext

```json
{
  "id": "skillctx_0001",
  "work_item_id": "work_0001",
  "agent_profile_id": "codex-impl",
  "capability_probe_id": "probe_0001",
  "project_rule_ids": ["rust-core"],
  "declared_skill_set_ids": ["nagare-core", "repo-default"],
  "applied_skill_set_ids": ["nagare-core", "repo-default"],
  "skipped_skill_set_ids": [],
  "instruction_sources": [
    "skills/nagare-core/SKILL.md",
    "AGENTS.md"
  ],
  "capabilities_in_force": [
    "repo_read",
    "file_edit",
    "shell_command"
  ],
  "artifact_uri": "file://.nagare/artifacts/work_0001/skill_context.json",
  "content_hash": "sha256:...",
  "locale": "ja-JP",
  "resolved_at": "2026-05-24T15:01:00+09:00"
}
```

### ResolvedRunPacket

```json
{
  "id": "runpkt_0001",
  "work_item_id": "work_0001",
  "agent_profile_id": "codex-impl",
  "runtime_id": "codex-local",
  "adapter_id": "process-codex-cli",
  "permission_policy_id": "medium-code-task",
  "workspace_policy_id": "worktree-per-item",
  "resolved_skill_context_id": "skillctx_0001",
  "verification": ["cargo test --workspace"],
  "constraints": [
    "Do not push to main",
    "Do not access production credentials"
  ],
  "artifact_uri": "file://.nagare/artifacts/work_0001/run_packet.json",
  "content_hash": "sha256:...",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:01:10+09:00"
}
```

## Resolution Rules

1. Load built-in defaults.
2. Merge user config.
3. Merge `.nagare/project.toml`.
4. Merge `.nagare/agents/*.toml`.
5. Find matching project rules for requested paths.
6. Select `default_agent` or explicit `--agent`.
7. Load the agent profile.
8. Load runtime and adapter declarations.
9. Load declared skill sets from agent profile + project rule + work item overrides.
10. Run or reuse a valid capability probe.
11. Drop skill sets whose required capabilities are unavailable.
12. Create `ResolvedSkillContext`.
13. Create `ResolvedRunPacket`.
14. Start an Agent Run through the adapter.

If a required skill set is dropped, Nagare should mark the Work Item
`needs_human` unless the user explicitly allows degraded execution.

## Minimum CLI Contract

```text
nagare locale show
nagare locale use --language ja-JP --timezone Asia/Tokyo

nagare agent list
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir crates/nagare-core
nagare agent show codex-impl
nagare agent defaults
nagare agent use --work-agent codex-impl --review-agent codex-app-impl --dispatch-agent codex-impl
nagare agent doctor codex-impl
nagare agent probe codex-impl

nagare skill list
nagare skill show nagare-core

nagare rule check crates/nagare-core/src/lib.rs

nagare item preview work_0001 --path crates/nagare-core/src/lib.rs --agent codex-impl
nagare item review work_0001 --agent codex-app-impl
nagare item run work_0001 --path crates/nagare-core/src/lib.rs --agent codex-impl
```

`rule check` resolves the project rule for a path. `item preview` uses
`dispatch_agent` by default, includes the resolved rule context when `--path` is
provided, and records a `dispatch_preview` Agent Run. `item review` uses
`review_agent` by default and records a `review` Agent Run. These runs do not
advance the Work Item status.

The first implementation can store this in `ledger.json`. Once the shape is
stable, move the ledger-owned entities to SQLite tables with the same names.

Current implementation persists `ResolvedSkillContext` and `ResolvedRunPacket`
for every `item preview` / `item run`. They are stored both as ledger-owned
records and as JSON artifacts under `.nagare/artifacts/`.
