# Agent Profile and Skill Data Model

This document defines the first data shape for agent tool management. It is a
design target for the Agent Management Kernel.

## Entity Ownership

```text
Config-owned:
  RuntimeDeclaration
  AdapterDeclaration
  AgentProfileDeclaration
  PermissionPolicy
  WorkspacePolicy

Ledger-owned:
  CapabilityProbe
  ResolvedSkillContext
  ResolvedRunPacket
  DispatchPlan
  AgentRun
  Artifact
  ExecutionRecord
  Evidence
  ReviewResult
  RecoveryPlan
  WorkflowDecision
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
supervisor_agent = "codex-impl"

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
specialties = ["implementation", "review"]
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
permission_policy = "medium-code-task"
workspace_policy = "worktree-per-item"
probe_before_run = true
timeout_minutes = 60
max_parallel_runs = 2

[permission_policies.medium-code-task]
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]

[workspace_policies.worktree-per-item]
kind = "git_worktree"
isolate_per_work_item = true
cleanup = "keep"

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
tool_kind = "codex_cli"
runtime = "codex-local"
adapter = "process.codex-cli"
role = "implementer"
working_dir = "packages/app"
description = "コード実装と検証を担当する Codex CLI agent"
specialties = ["implementation", "review"]
skill_set_ids = ["rust-core", "test-runner"]

[agent_profile.prompt]
instructions = "小さく実装し、検証結果を最後に書く。"
version = "v1"

[agent_profile.output_contracts.work]
contract = "nagare.result.v1"
instruction_pack = "nagare-result-writer.v1"
required = true
injection = "prompt_suffix"

[agent_profile.output_contracts.review]
contract = "nagare.review.v1"
instruction_pack = "nagare-review-writer.v1"
required = true
injection = "prompt_suffix"

[agent_profile.output_contracts.dispatch]
contract = "nagare.dispatch.v1"
instruction_pack = "nagare-dispatch-writer.v1"
required = true
injection = "prompt_suffix"
```

These files are created by `nagare agent add`. They override same-id profiles
from `.nagare/project.toml`. `working_dir` is the directory where the agent run
starts. It must be a relative path inside the project; the default is `"."`.
`description` and `specialties` are compact routing hints for the Nagare
dispatch agent. They are not treated as observed capability; actual availability
still comes from CapabilityProbe.

`tool_kind` is the user-facing agent tool category. Existing profiles without
`tool_kind` are inferred from `runtime` and `adapter`; saved profiles write the
field explicitly. `model` selects the model inside the chosen tool. Codex and
Codex CLI accept OpenAI/Codex providers. OpenClaw can use OpenAI, Ollama, or LM
Studio style providers; local providers require `base_url`.

`skill_set_ids` are agent-specific skills. At run time Nagare merges
ProjectRule skill sets with Agent Profile skill sets and records the resolved
applied/skipped result in `ResolvedSkillContext`.

Skill packages record where a skill came from. The package entry is separate
from the skill set so ClawHub, Vercel Skills, git, local folders, and
skill-creator output can all provide the same Agent-facing `skill_set_id`.

```toml
[skill_packages.react-review]
source_kind = "skill_creator"
source = "./skills/react-review"
installed_path = "./skills/react-review"
provided_skill_sets = ["react-review"]

[skill_sets.react-review]
paths = ["./skills/react-review"]
required_capabilities = ["repo_read"]
optional_capabilities = ["shell_command"]
```

`prompt.instructions` is the execution instruction for this Agent. `description`
remains a compact display and dispatch hint. During migration, if
`prompt.instructions` is empty, Nagare uses `description` as the fallback
instruction.

`output_contracts` are Nagare-managed instruction packs for stable final
outputs. They are configured per Agent Profile and per purpose:

- `work`: final user-facing result, deliverable artifacts when requested, evidence, questions, next action.
- `review`: verdict, findings, referenced artifacts, requested changes, questions, next action.
- `dispatch`: selected target Agent Profile, summary, risks, missing information.

Nagare distinguishes four record classes:

- `Artifact`: a user-requested deliverable file, or a file required to create that deliverable. If the Work Item does not ask for a file or supporting file, an Agent Run should not create artifacts merely because it produced logs or stdout.
- `ExecutionRecord`: run/review traces such as adapter logs, stdout/stderr captures, transcripts, changed-file lists, and diff patches. These are evidence records, not deliverables.
- `AgentOutputRecord`: the agent's generated result parsed from the output contract. For a weather question, the weather answer belongs here; no artifact is created unless the user requested a file.
- `next_notes`: handoff notes for the next dispatch or agent run. It is not the final answer.

The contract is a Nagare data contract; the instruction pack is the way Nagare
asks an agent to follow it. MVP supports `prompt_suffix` injection for
`process.codex-cli` and `stdio.codex-app-server`.

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
  "supported_instruction_modes": [
    "prompt_injection",
    "file_reference"
  ],
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
the Run Packet.

### ResolvedSkillContext

```json
{
  "id": "skillctx_0001",
  "work_item_id": "work_0001",
  "agent_profile_id": "codex-impl",
  "capability_probe_id": "probe_0001",
  "instruction_sources": [
    "AGENTS.md"
  ],
  "capabilities_in_force": [
    "repo_read",
    "file_edit",
    "shell_command"
  ],
  "execution_record_uri": "file://.nagare/logs/skillctx_0001.json",
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
  "work_folder": "crates/nagare-core",
  "dispatch_plan_id": "dispatch_0001",
  "permission_policy_id": "medium-code-task",
  "workspace_policy_id": "worktree-per-item",
  "resolved_skill_context_id": "skillctx_0001",
  "output_contract": {
    "contract": "nagare.result.v1",
    "instruction_pack": "nagare-result-writer.v1",
    "required": true,
    "injection": "prompt_suffix"
  },
  "project_rule_ids": ["nagare-core"],
  "review_checks": ["cargo test --workspace"],
  "constraints": [
    "Do not push to main",
    "Do not access production credentials"
  ],
  "execution_record_uri": "file://.nagare/logs/runpkt_0001.json",
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
  "raw_output_execution_record_id": "exec_0001",
  "path": "crates/nagare-core/src/lib.rs",
  "summary": "Use codex-impl because its working_dir matches the requested folder.",
  "risks": ["core usecase file is approaching the 800-line split threshold"],
  "missing_information": [],
  "selection_warnings": [],
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:01:20+09:00"
}
```

`target_agent_profile_id` is selected from the compact candidate list returned
to the dispatch agent. Nagare accepts the selected ID only when it matches a
registered Agent Profile; otherwise it falls back to the default target.
Contract violations are recorded in `selection_warnings`.

Dispatch output contract:

```json
{
  "target_agent_profile_id": "research-agent",
  "summary": "Research is required before writing.",
  "risks": ["source quality"],
  "missing_information": ["source list"]
}
```

`target_agent_profile_id` and `summary` are required. `risks` and
`missing_information` are optional arrays of strings. If JSON parsing fails,
the target is missing, or the target does not match a registered Agent Profile,
Nagare uses the default fallback target and records the reason in
`selection_warnings`.

### ExecutionRecord

```json
{
  "id": "exec_0001",
  "work_item_id": "work_0001",
  "agent_run_id": "run_0001",
  "record_type": "run_log",
  "uri": "file://.nagare/logs/run_0001.log",
  "title": "codex-impl work log",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:03:00+09:00"
}
```

`ExecutionRecord` stores reproducibility and audit traces. It may point to a log,
raw output capture, transcript, changed-file list, diff patch, or review evidence
log. These files are not shown as deliverable artifacts and do not satisfy
`expected_artifacts`.

### AgentOutputRecord

```json
{
  "id": "out_0001",
  "work_item_id": "work_0001",
  "agent_run_id": "run_0001",
  "agent_profile_id": "codex-impl",
  "purpose": "work",
  "contract": "nagare.result.v1",
  "instruction_pack": "nagare-result-writer.v1",
  "parse_status": "parsed",
  "fields": {
    "status": ["blocked"],
    "questions": ["release note URLを追加してよいですか？"],
    "next_action": ["answer_question"]
  },
  "questions": ["release note URLを追加してよいですか？"],
  "next_action": "answer_question",
  "warnings": [],
  "execution_record_id": "exec_0001",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:03:00+09:00"
}
```

`AgentOutputRecord` is created for `work` and `review` runs. MVP parsing reads
Markdown sections named `## Nagare Result` and `## Nagare Review`. If a required
contract block is missing, Nagare records `parse_status: "unparsed"` and
`output_contract_unparsed` in warnings while keeping the raw run execution
record. Questions set the Work Item status to `needs_input`.

### ReviewResult

```json
{
  "id": "review_0001",
  "work_item_id": "work_0001",
  "agent_run_id": "run_0002",
  "agent_profile_id": "codex-review",
  "verdict": "request_changes",
  "summary": ["Review evidence is incomplete."],
  "findings": ["No test log was referenced."],
  "requested_changes": ["Add review evidence before approval."],
  "referenced_artifacts": ["art_0003"],
  "criteria_results": [
    {
      "criterion": "cargo test --workspace passes",
      "status": "passed",
      "note": "cargo test --workspace passes: pass"
    }
  ],
  "questions": [],
  "next_action": "run_agent",
  "execution_record_id": "exec_0002",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:08:00+09:00"
}
```

`ReviewResult` is derived from a parsed `## Nagare Review` block. Verdicts are
`pass`, `request_changes`, `blocked`, and `unknown`. `pass` moves the Work Item
to `ready_for_review` only when all Work Item acceptance criteria are
covered as `passed`; otherwise it moves to `changes_requested`.
`request_changes` moves it to `changes_requested`; questions or `blocked` move
it to `needs_input`.

### HumanFeedback

```json
{
  "id": "feedback_0001",
  "work_item_id": "work_0001",
  "source_agent_output_id": "out_0001",
  "question": "release note URLを追加してよいですか？",
  "answer": "追加してよいです。",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:05:00+09:00"
}
```

Human feedback is recorded by `nagare item answer`. When feedback exists for a
Work Item, the next Agent Run receives it as a `## Nagare Human Feedback`
prompt section, and the Run Packet records `human_feedback_context_applied` in
constraints.

### WorkItemHistoryStep

```json
{
  "id": "step_run_0001",
  "kind": "work",
  "title": "作業実行",
  "state": "succeeded",
  "actor": "codex-impl",
  "started_at": "2026-05-24T15:04:00+09:00",
  "ended_at": "2026-05-24T15:06:00+09:00",
  "summary": "README diff を作成した。",
  "facts": [
    { "label": "Agent", "value": "codex-impl" },
    { "label": "Artifacts", "value": "2" }
  ],
  "links": [
    { "label": "Run", "record_id": "run_0001", "record_type": "run" },
    { "label": "Run log", "record_id": "art_0001", "record_type": "artifact" }
  ],
  "source_record_ids": ["run_0001", "out_0001"],
  "next_action": "review"
}
```

`WorkItemHistoryStep` is the primary UI read model generated from the ledger.
It does not replace source records. It groups low-level records into the step
shape users need to understand the work: `request`, `dispatch`, `work`,
`review`, `input`, `handoff`, `recovery`, and `approval`.

Every step has the same display contract:

- `kind`: stable workflow category.
- `state`: normalized state such as `recorded`, `draft`, `accepted`,
  `succeeded`, `pass`, `passed`, `needs_input`, `failed`, or `approve`.
- `actor`: user, Agent Profile, Workflow, or Review Agent.
- `started_at` / `ended_at`: timing when source records expose it. Instant
  records use the same timestamp for both.
- `summary`: one user-readable sentence.
- `facts`: compact key-value facts shown directly on the history card.
- `links`: source records opened by the inspector.
- `source_record_ids`: audit trail back to ledger records.
- `next_action`: the likely continuation after this step.

`WorkItemTimelineEvent` remains as a lower-level compatibility read model. New
UI surfaces should prefer `WorkItemHistoryStep` for Processing History.

### UiRunningState

```json
{
  "kind": "work",
  "actor": "codex-impl",
  "label": "codex-impl work",
  "message": "Work Agent が依頼を処理しています。",
  "related_action": "run_agent",
  "started_at_epoch": 1780053600
}
```

`UiRunningState` is an ephemeral local UI state file stored under
`.nagare/state/<work_id>-ui-running.txt` while `nagare ui serve` advances a Work
Item in the background. It is not a ledger record. It uses the same `kind` /
`actor` vocabulary as `WorkItemHistoryStep` so running UI cards can be displayed
without inventing separate status semantics.

### WorkItemCompletion

```json
{
  "state": "blocked",
  "blocking_reason": "review_failed: cargo test --workspace",
  "next_action": "recover",
  "next_command_hint": "nagare item recover work_0001"
}
```

`WorkItemCompletion` is a read model on `WorkItemSnapshot`. It is calculated
from ledger records and tells CLI/UI what should happen next.

### RecoveryPlan

```json
{
  "id": "recovery_0001",
  "work_item_id": "work_0001",
  "status": "draft",
  "action": "rerun_with_contract_reminder",
  "target_agent_profile_id": "codex-impl",
  "failure_class": "contract_violation",
  "reason": "output_contract_missing",
  "summary": "Ask `codex-impl` to restate the final output using `nagare.result.v1`.",
  "source_event_id": "out_0001",
  "command_hint": "nagare item recover apply work_0001",
  "prompt_hint": "Restate the previous run output using the required contract.",
  "warnings": [],
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:09:00+09:00"
}
```

RecoveryPlan actions are `rerun_same_agent`, `rerun_with_contract_reminder`,
`handoff`, `ask_human`, `request_changes`, and `redispatch`.
The lifecycle is `draft`, `accepted`, `superseded`.

`status` controls the execution lifecycle:

- `draft`: recovery candidate recorded by `nagare item recover`.
- `accepted`: the plan was selected by `nagare item recover accept` and can be
  applied when the action supports agent rerun.
- `superseded`: replaced by a newer draft or accepted plan for the same Work
  Item.

`failure_class` is the machine-readable recovery cause. Current values include
`contract_violation`, `review_changes`,
`missing_artifact`, `no_diff`, `missing_input`, `needs_handoff`,
`continue_workflow`. A single recovery request may
create multiple draft candidates when secondary risks such as missing artifacts
or missing diff evidence are detected.

### WorkflowDecision

```json
{
  "id": "wfd_0001",
  "work_item_id": "work_0001",
  "action": "run_review",
  "source": "deterministic",
  "reason": "ready_for_review",
  "requires_human": false,
  "target_agent_profile_id": "codex-review",
  "agent_run_id": null,
  "confidence": 0.7,
  "command_hint": "nagare item review work_0001",
  "warnings": [],
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:10:00+09:00"
}
```

`WorkflowDecision` is recorded by `item advance` and by explicit decision
creation. It is the audit record for why Nagare selected dispatch, accept,
run, review, recovery, human input, handoff, approval, or done as
the next step. When `item advance --supervisor true` is used, Nagare records a
`workflow_supervision` AgentRun and derives the decision from the supervisor
agent's `## Nagare Workflow Decision` output contract.

### HandoffPacket

```json
{
  "id": "handoff_0001",
  "work_item_id": "work_0001",
  "from_agent_profile": "research-agent",
  "to_agent_profile": "implementation-agent",
  "reason": "Research is complete and implementation should continue.",
  "summary": "Use docs/source-a.md and produce a verifiable implementation summary.",
  "current_state": "needs_handoff",
  "open_questions": [],
  "artifact_ids": ["art_0001", "art_0002"],
  "execution_record_ids": ["exec_0001", "exec_0002"],

  "review_result_ids": ["review_0001"],
  "next_request": "Use docs/source-a.md and produce a verifiable implementation summary.",
  "locale": "ja-JP",
  "created_at": "2026-05-24T15:11:00+09:00"
}
```

The latest HandoffPacket is injected into the next Agent Run as
`## Nagare Handoff Context`; the Run Packet records
`handoff_context_applied` in constraints. This keeps handoff context compact
without expanding full instruction sources.

## Resolution Rules

1. Load built-in defaults.
2. Merge user config.
3. Merge `.nagare/project.toml`.
4. Merge `.nagare/agents/*.toml`.
5. Resolve requested `work_folder` / `path`.
6. Select explicit `--agent`, accepted DispatchPlan target, or `work_agent`.
7. Load the agent profile.
8. Load runtime and adapter declarations.
9. Run or reuse a fresh capability probe.
10. Create `ResolvedSkillContext`.
11. Create `ResolvedRunPacket`.
12. Start an Agent Run through the adapter.
13. For dispatch preview, give the dispatch agent a compact candidate list.
14. Parse dispatch output JSON and create draft `DispatchPlan`.
15. Optionally accept the DispatchPlan.
16. For `item run`, resolve agent in this order: explicit `--agent`,
    explicit accepted `--dispatch-plan`, latest accepted DispatchPlan,
    then `work_agent`.
17. If a DispatchPlan selected the run agent, persist `dispatch_plan_id` in
    `ResolvedRunPacket`.

The compact candidate list is fixed to at most 5 Agent Profiles in the initial
MVP. This is intentionally not project-configurable yet; the goal is to keep
dispatch context small until the full UI and operating limits are clear.

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

nagare item preview work_0001 --work-folder crates/nagare-core --agent codex-impl
nagare item review work_0001 --agent codex-app-impl
nagare item run work_0001 --work-folder crates/nagare-core --agent codex-impl
```

`item preview` uses `dispatch_agent` by default, includes work_folder and
Agent Profile working_dir context when provided, and records a
`dispatch_preview` Agent Run. `item review` uses
`review_agent` by default and records a `review` Agent Run. These runs do not
advance the Work Item status.

The first implementation can store this in `ledger.json`. Once the shape is
stable, move the ledger-owned entities to SQLite tables with the same names.

Current implementation persists `ResolvedSkillContext` and `ResolvedRunPacket`
for every `item preview` / `item run`. They are stored both as ledger-owned
records and as JSON execution records under `.nagare/logs/`. Dispatch preview
also stores `DispatchPlan` in the ledger and links it to the run log execution
record.
