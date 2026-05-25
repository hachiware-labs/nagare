# Agent Management Model

See [architecture](architecture.md) for the whole system design and
[Agent Profile and Skill Data Model](agent_data_model.md) for concrete TOML and
JSON shapes.

Nagare must manage agent usage before it manages individual adapters. Agent
tools can be invoked as local processes, stdio app servers, hosted jobs, or CI
workers. Projects and folders can also require different working directories,
permissions, and verification expectations.

The management model separates five concerns:

```text
Runtime
  What executable, server, cloud job, or protocol endpoint exists?

Adapter
  How does Nagare talk to that runtime?

Agent Profile
  Which runtime + adapter + role + working_dir + limits should be used for a
  class of work?

Capability Probe
  What does the agent tool actually support in this project right now?

Resolved Skill Context
  Which discovered capabilities and instruction sources were actually used for
  a Work Item preview or Agent Run?
```

## Configuration Layers

Nagare resolves configuration from broad to narrow scope:

```text
1. Built-in defaults
2. User config
   ~/.config/nagare/config.toml
3. Project config
   .nagare/project.toml
4. Work Item overrides
```

Narrower layers can add or override agent profiles, policies, and
verification defaults. They should not mutate global runtime discovery results.

## Storage Split

Nagare stores configuration and runtime evidence separately.

```text
Config files:
  Declared intent.
  Runtimes, agent profiles, policies.

Ledger:
  Historical facts.
  Agent runs, probe results, resolved run packets, artifact/evidence records.

Filesystem artifacts:
  Large or append-only material.
  Raw logs, screenshots, transcripts, copied skill bundles, generated run packets.
```

Initial local paths:

```text
.nagare/project.toml
  project-level runtime, agent profile, locale, Nagare defaults, and policy declarations

.nagare/agents/*.toml
  project-local agent profile declarations created by `nagare agent add`

.nagare/state/ledger.json
  MVP JSON ledger for work items, runs, evidence, probe snapshots, and resolved run packets

.nagare/artifacts/
  run packets, probe outputs, run logs
```

SQLite can replace `ledger.json` later, but the object boundaries should stay
the same.

## Runtime Registry

A runtime is a concrete tool or endpoint that can execute work.

```toml
[[runtimes]]
id = "codex-local"
kind = "process"
command = "codex"
args = ["exec"]
healthcheck = ["codex", "--version"]

[[runtimes]]
id = "codex-app-local"
kind = "stdio"
command = "codex"
args = ["app-server", "--listen", "stdio://"]
healthcheck = ["codex", "app-server", "--help"]

```

Runtime records answer:

- Is the tool available?
- What version is it?
- Is it local, remote, persistent, or job-based?
- Which adapter can use it?

Runtime declaration is config. Runtime version and availability are probe
results and belong in the ledger.

## Adapter Registry

An adapter is the protocol implementation behind a runtime.

```toml
[[adapters]]
id = "process.codex-cli"
runtime_kind = "process"
capabilities = ["code_edit", "test_run", "repo_analysis"]

[[adapters]]
id = "stdio.codex-app-server"
runtime_kind = "stdio"
capabilities = ["code_edit", "test_run", "repo_analysis", "thread_state", "approval_flow", "event_stream"]

```

Adapters normalize:

- run packet input
- process or job lifecycle
- run events
- stdout/stderr or event streams
- artifact collection
- cancellation behavior
- healthcheck output

## Agent Profiles

An agent profile is what users run Work Items with. It binds a runtime and
adapter to a role, workspace policy, permission policy, skill set, and limits.
It also declares `working_dir`, the project-relative directory where the agent
process starts. This is separate from the directory where the profile TOML file
is stored. `description` and `specialties` are compact routing hints that the
Nagare dispatch agent can use when selecting from a small candidate list.
`output_contracts` declare the Nagare-managed final-output contracts to inject
for work, review, and dispatch runs.

```toml
[agent_profiles.codex-impl]
id = "codex-impl"
display_name = "Codex Implementer"
runtime = "codex-local"
adapter = "process.codex-cli"
role = "implementer"
working_dir = "."
description = "Implementation-focused Codex CLI profile"
specialties = ["implementation", "verification"]
workspace_policy = "worktree-per-item"
permission_policy = "medium-code-task"
max_parallel_runs = 2
timeout_minutes = 60
probe_before_run = true

[agent_profiles.codex-impl.output_contracts.work]
contract = "nagare.result.v1"
instruction_pack = "nagare-result-writer.v1"
required = true
injection = "prompt_suffix"

[agent_profiles.codex-impl.output_contracts.review]
contract = "nagare.review.v1"
instruction_pack = "nagare-review-writer.v1"
required = true
injection = "prompt_suffix"

[agent_profiles.codex-impl.output_contracts.dispatch]
contract = "nagare.dispatch.v1"
instruction_pack = "nagare-dispatch-writer.v1"
required = true
injection = "prompt_suffix"

[agent_profiles.codex-app-review]
id = "codex-app-review"
display_name = "Codex App Server Reviewer"
runtime = "codex-app-local"
adapter = "stdio.codex-app-server"
role = "reviewer"
working_dir = "apps/web"
description = "Review-focused Codex app-server profile"
specialties = ["review", "planning"]
workspace_policy = "read-only-worktree"
permission_policy = "review-only"
```

The user should choose `agent profile` IDs, not raw commands. Raw commands are a
temporary development fallback for the earliest CLI slice.

Agent profile records are declarations. They should not claim that a tool
actually supports a skill or capability. That is discovered through probes.
Dispatch uses the declarations as a compact shortlist only; large instruction
source bodies such as AGENTS.md or SOUL.md are not expanded into the dispatch
prompt.

Output contracts are declarations controlled by Nagare. They are not evidence
that an agent truly complied. Nagare always stores raw output as an artifact;
contract parsing failures should become warnings or review inputs, not automatic
Run failures.

## Nagare Agent Defaults

Nagare also needs to know which registered Agent Profiles it should use for its
own routing decisions.

```toml
[nagare_agents]
work_agent = "codex-impl"
review_agent = "codex-app-review"
dispatch_agent = "codex-impl"
```

- `work_agent`: default Work Item execution target when `--agent` is omitted.
- `review_agent`: default reviewer profile for later review flows.
- `dispatch_agent`: default profile that will prepare or resolve dispatch
  proposals before execution.

These are references to Agent Profile IDs. They do not create new profiles.

## Capability Probes

Agent tools differ widely. Nagare should ask or inspect before assuming skill
support.

Probe sources can include:

```text
process:
  command --version
  command help
  known config files
  instruction files such as AGENTS.md

stdio app server:
  initialize handshake
  protocol version
  event stream support
  approval flow support

ci/cloud:
  provider API
  workflow metadata
  permission scope
```

Probe results are ledger facts:

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
  "probed_at": "2026-05-24T15:00:00+09:00"
}
```

The probe cache must be invalidated when:

- runtime version changes
- agent profile declaration changes
- project instruction files change
- working_dir changes

## Resolved Skill Context

Resolved skill context is produced when a Work Item is previewed or run. It is
the exact capability and instruction-source context given to an Agent Run.

```json
{
  "id": "skillctx_0001",
  "work_item_id": "work_0001",
  "agent_profile_id": "codex-impl",
  "discovered_capability_ids": ["repo_read", "file_edit", "shell_command"],
  "instruction_sources": ["AGENTS.md"],
  "artifact_uri": "file://.nagare/artifacts/work_0001/skill_context.json",
  "content_hash": "sha256:...",
  "resolved_at": "2026-05-24T15:01:00+09:00"
}
```

The resolved context belongs in run provenance. Reviewers need to know which
Agent Profile, working_dir, capabilities, and instruction sources shaped the run.

## Policy Model

Policies are separate from Agent Profile attributes. Agent Profile and Probe
describe what the agent can do; policies define what execution is allowed to do.

```toml
[[permission_policies]]
id = "medium-code-task"
allowed_actions = ["repo_read", "worktree_write", "test_run"]
disallowed_actions = ["main_push", "production_access", "secrets_read"]
approval_required = ["network_access", "dependency_install"]
```

Policy decisions should be included in run provenance:

- selected policy id
- allowed/disallowed action summary
- human approvals
- detected violations

## Run Packet Preview and Resolution

Before a Work Item is run, Nagare should resolve:

```text
Work Item
  + agent profile
  + request work_folder
  + agent working_dir
  + runtime
  + adapter
  + workspace policy
  + permission policy
  + latest valid capability probe
  + resolved skill context
  + verification defaults
  = Run Packet
```

The Run Packet should record hashes or versions of all resolved inputs:

- agent profile id/version
- request work_folder
- agent working_dir
- runtime id/version
- adapter id/version
- capability probe id/hash
- resolved skill context id/hash
- permission policy id/version

This is necessary for evidence-first review: a reviewer must know not just what
the agent did, but what instructions, policies, and project scope shaped the
run.

## CLI Shape

Implemented management commands:

```text
nagare agent list
nagare agent add --id codex-impl --runtime codex-local --adapter process.codex-cli --working-dir crates/nagare-core
nagare agent show codex-impl
nagare agent defaults
nagare agent use --work-agent codex-impl --review-agent codex-app-review --dispatch-agent codex-impl
nagare agent doctor codex-impl
nagare agent probe codex-impl
nagare item preview work_0001
nagare item review work_0001
nagare handoff dispatch work_0001
```

Planned management commands:

```text
nagare runtime list
nagare runtime doctor
nagare runtime add process --id codex-local --command codex

nagare adapter list

nagare item preview work_0001 --work-folder apps/web
nagare item run work_0001 --work-folder apps/web
```

The `item preview` command is the pre-run confirmation step. It runs the
configured `dispatch_agent`, resolves a compact work scope from the requested
`work_folder`, registered Agent Profile attributes, Profile `working_dir`,
fresh Capability Probe, policy, and verification context, records Resolved Skill
Context and Resolved Run Packet, and stores an Agent Run with purpose
`dispatch_preview` without advancing the Work Item status. A successful preview
also stores a DispatchPlan that links the dispatch AgentRun, target Agent
Profile, ResolvedRunPacket, and raw output Artifact.

The `item review` command runs the configured `review_agent` and records an
Agent Run with purpose `review` without advancing the Work Item status.
The `handoff dispatch` command uses the same `dispatch_agent` path after a
handoff has been created, so the next target can be assessed again before
another `item run`.

The `item run` command can use a real agent through `--prompt`. `--command`
remains a smoke-test fallback and should not be treated as an Agent adapter.

## MVP Implication

Current implementation connects the project-local Agent Profile registry,
probe snapshots, Agent Profile attribute-based scope resolution, Resolved Skill
Context, Resolved Run Packet, DispatchPlan, and adapter execution. The next
slice should deepen policy enforcement and work-scope debugging before
broadening agent support.
