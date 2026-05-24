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
  DispatchPlan
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
description = "Codex CLI implementation profile"
specialties = ["implementation", "verification"]
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
description = "Codex app-server implementation and planning profile"
specialties = ["planning", "review"]
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
description = "コード実装と検証を担当する Codex CLI agent"
specialties = ["implementation", "verification"]
```

These files are created by `nagare agent add`. They override same-id profiles
from `.nagare/project.toml`. `working_dir` is the directory where the agent run
starts. It must be a relative path inside the project; the default is `"."`.
`description` and `specialties` are compact routing hints for the Nagare
dispatch agent. They are not treated as observed capability; actual availability
still comes from CapabilityProbe.

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
  It receives only a small Agent Profile candidate list, then returns a selected
  `target_agent_profile_id` in DispatchPlan-oriented JSON.

## Ledger JSON Shapes

### CapabilityProbe

```json
{
  "id": "probe_0001",
  "agent_profile_id": "codex-impl",
  "runtime_id": "codex-local",
  "adapter_id": "process.codex-cli",
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

Run / Preview reuses the latest CapabilityProbe only when all of the following
are true:

- `agent_profile_id` matches the selected Agent Profile.
- `runtime_id` matches the current Agent Profile runtime.
- `adapter_id` matches the normalized current Agent Profile adapter.
- `runtime_version` matches the current runtime healthcheck detail.
- `probed_at` is within the current TTL. MVP default is 24 hours.

If any condition fails, Nagare records a new CapabilityProbe before resolving
Skill Sets and Run Packet.

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
  "adapter_id": "process.codex-cli",
  "purpose": "work",
  "working_dir": "file://./crates/nagare-core",
  "goal": "Refactor core run orchestration",
  "path": "crates/nagare-core/src/lib.rs",
  "permission_policy_id": "medium-code-task",
  "workspace_policy_id": "worktree-per-item",
  "resolved_skill_context_id": "skillctx_0001",
  "project_rule_ids": ["nagare-core"],
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

### DispatchPlan

```json
{
  "id": "dispatch_0001",
  "work_item_id": "work_0001",
  "status": "accepted",
  "agent_run_id": "run_0001",
  "dispatch_agent_profile_id": "codex-dispatch",
  "target_agent_profile_id": "codex-impl",
  "resolved_run_packet_id": "runpkt_0001",
  "raw_output_artifact_id": "art_0001",
  "path": "crates/nagare-core/src/lib.rs",
  "summary": "Use codex-impl with the rust-core rule and run cargo test --workspace.",
  "risks": ["core usecase file is approaching the 800-line split threshold"],
  "missing_information": [],
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:01:20+09:00"
}
```

`target_agent_profile_id` is selected from the compact candidate list returned
to the dispatch agent. Nagare accepts the selected ID only when it matches a
registered Agent Profile; otherwise it falls back to the Project Rule or default
target.

`status` controls the execution lifecycle:

- `draft`: dispatch agent proposal recorded by preview or handoff dispatch.
- `accepted`: the plan was selected by `nagare item dispatch accept` and can
  route `nagare item run` when `--agent` is omitted.
- `superseded`: replaced by a newer draft or accepted plan for the same Work
  Item.

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
10. Run or reuse a fresh capability probe.
11. Compare required capabilities and record skill sets as applied or skipped.
12. Create `ResolvedSkillContext`.
13. Create `ResolvedRunPacket`.
14. Start an Agent Run through the adapter.
15. For dispatch preview, give the dispatch agent a compact candidate list.
16. Parse dispatch output JSON and create draft `DispatchPlan`.
17. Optionally accept the DispatchPlan.
18. For `item run`, resolve agent in this order: explicit `--agent`,
    explicit accepted `--dispatch-plan`, latest accepted DispatchPlan,
    Project Rule path, then `work_agent`.

If a required skill set is unavailable, the current implementation records it
in `skipped_skill_set_ids` and adds the reason to Run Packet constraints. Later
policy enforcement can decide whether this should block execution or require
human approval.

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
records and as JSON artifacts under `.nagare/artifacts/`. Dispatch preview also
stores `DispatchPlan` in the ledger and links it to the run log artifact.
